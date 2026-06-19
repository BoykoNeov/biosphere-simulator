"""Phase-1 Step-11 tests: the assembled single-producer season (4b).

``domains.biosphere.season`` wires the seven processes into one winter-wheat season.
This is the **integration gate** — the tightest Phase-1 constraint:

* **Conservation + ``rationed == 0`` + no extinction** over the full season — the hard
  deliverable. Conservation is asserted every step by the always-on gate (a violation
  would raise mid-run); ``rationed == 0`` holds *by construction* (every clamped
  withdrawal self-limits — organ pools, ``soil_water``, ``soil_n``); the well-fed plot
  triggers no extinction.
* **Liveness** — conservation and ``rationed == 0`` both pass *trivially on a dead/null
  trajectory* (a plant that never grows conserves and never rations) — liveness is what
  makes "green" meaningful: the plant must actually fix carbon, build a canopy, develop,
  and fill grain. Bounds are **peak-based** (robust to the senescence tail).
* **Determinism + flow-order independence** — bit-identical re-runs; the registry's
  canonical id-order makes the result independent of flow registration order (#7/#15).
* **Potential-production setup** — ``f_water = f_N = 1`` all season (mirrors the
  ``Wofost72_PP`` oracle); ``carbon_fraction`` parity between the canopy and nitrogen
  param files (both fold it — divergence would be a silently inconsistent plant).

**Documented finding (NOT a validated match).** With uncalibrated ``TODO(cite)``
placeholder params and a no-vernalization phenology (DVS reaches maturity mid-season —
the documented overrun), the trajectory runs ~2 orders of magnitude below the oracle
(peak LAI ≈ 0.09 vs ≈ 6). That gap is the **deferred quantitative gate** (a user
decision); this test gates the *machinery* (conservation/rationed/liveness/determinism),
not behavioral fidelity. Quantifying the gap is the qualitative oracle smoke check.

PCSE-free: ``runner.load_weather`` reads the committed JSON weather fixture (no PCSE).
"""

import json
import math
from pathlib import Path

import pytest
import yaml

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import (
    CANOPY_PARAMS_PATH,
    NITROGEN_PARAMS_PATH,
    load_canopy_params,
    load_nitrogen_params,
)
from domains.biosphere.nitrogen import nitrogen_stress_factor
from domains.biosphere.phenology import development_stage
from domains.biosphere.season import (
    CO2_RESP,
    LEAF_C,
    PLANT_N,
    ROOT_C,
    SOIL_WATER,
    STEM_C,
    STORAGE_C,
    SeasonScenario,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.transpiration import water_stress_factor
from simcore.integrator import EulerIntegrator
from simcore.registry import Registry
from simcore.state import State

# The committed raw-weather fixture (NASAPower facts) lives beside the runner that
# regenerates it. Read it as plain JSON by path (no PCSE, no ``tests.oracle`` import) so
# this stays a default-suite test.
_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_SCENARIO = SeasonScenario()
_ORGANS = (LEAF_C, STEM_C, ROOT_C, STORAGE_C)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(scenario: SeasonScenario = _SCENARIO) -> tuple[list[State], int, tuple]:
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, len(_weather()))


@pytest.fixture(scope="module")
def season() -> tuple[list[State], int, tuple]:
    return _run()


def _organ_carbon(s: State) -> float:
    return sum(s.stocks[o].amount for o in _ORGANS)


def _dvs(s: State) -> float:
    return development_stage(
        s.aux["thermal_time"], tsum_anthesis=1100.0, tsum_maturity=750.0
    )


def _lai(s: State) -> float:
    cp = load_canopy_params()
    return leaf_area_index(
        s.stocks[LEAF_C].amount, sla_per_mol_c=cp.sla_per_mol_c, ground_area=1.0
    )


# --- the hard deliverable: conservation + rationed == 0 + no extinction ------
def test_season_runs_full_length(season: tuple[list[State], int, tuple]) -> None:
    states, _, _ = season
    assert len(states) == len(_weather()) + 1  # initial + one per day


def test_season_never_rations(season: tuple[list[State], int, tuple]) -> None:
    # rationed == 0 by construction (self-limiting kinetics; the backstop is the rare
    # numerical guard, never the mechanism). A nonzero total is a failing gate.
    _, total_rationed, _ = season
    assert total_rationed == 0


def test_season_has_no_extinction_events(
    season: tuple[list[State], int, tuple],
) -> None:
    _, _, events = season
    assert events == ()


def test_season_conserves_every_step() -> None:
    # The always-on conservation gate runs in step_report; reaching the end without a
    # ConservationError IS the per-step assertion. (Re-run here so the assertion is
    # explicit at this test's name, independent of the module fixture.)
    states, _, _ = _run()
    assert len(states) == len(_weather()) + 1


# --- liveness (peak-based — the plant actually grew, not a null trajectory) --
def test_season_liveness(season: tuple[list[State], int, tuple]) -> None:
    states, _, _ = season
    initial_organ = _organ_carbon(states[0])
    peak_organ = max(_organ_carbon(s) for s in states)
    peak_lai = max(_lai(s) for s in states)
    final_dvs = _dvs(states[-1])
    final_co2_resp = states[-1].stocks[CO2_RESP].amount
    final_storage = states[-1].stocks[STORAGE_C].amount

    assert peak_organ > 1.5 * initial_organ  # net carbon fixed into biomass
    assert peak_lai > 0.05  # a canopy formed (catches "never grew")
    assert final_dvs > 1.0  # development advanced past anthesis
    assert final_co2_resp > 0.0  # respiration happened (co2_resp only accumulates)
    assert final_storage > 0.0  # grain filled


def test_season_storage_fills_only_after_anthesis(
    season: tuple[list[State], int, tuple],
) -> None:
    # Grain (storage_c) must fill in the reproductive window (DVS ≥ 1, fo > 0) — never
    # before. A storage leg before anthesis would mean a broken partition table.
    states, _, _ = season
    for s in states:
        if s.stocks[STORAGE_C].amount > 1e-12:
            assert _dvs(s) >= 1.0


def test_season_is_potential_production(season: tuple[list[State], int, tuple]) -> None:
    # PP: f_water = f_N = 1 every step (the soil pools stay above their bands), so the
    # carbon trajectory is light/temperature-limited only — comparable to Wofost72_PP.
    states, _, _ = season
    np_ = load_nitrogen_params()
    for s in states:
        f_water = water_stress_factor(
            s.stocks[SOIL_WATER].amount,
            sw_wilting=_SCENARIO.sw_wilting,
            sw_critical=_SCENARIO.sw_critical,
        )
        biomass = (
            s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount
        )
        f_n = nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            biomass,
            n_residual_per_mol_c=np_.n_residual_per_mol_c,
            n_critical_per_mol_c=np_.n_critical_per_mol_c,
        )
        assert f_water == 1.0
        assert f_n == 1.0


# --- determinism + flow-order independence (#7/#15) -------------------------
def test_season_is_deterministic() -> None:
    a, _, _ = _run()
    b, _, _ = _run()
    final_a, final_b = a[-1], b[-1]
    assert final_a.aux == final_b.aux
    for sid, stock in final_a.stocks.items():
        assert stock.amount == final_b.stocks[sid].amount  # bit-identical


def test_season_is_flow_registration_order_independent() -> None:
    # The registry sorts flows by canonical id, so a shuffled registration order gives a
    # bit-identical trajectory (the determinism invariant the engine guarantees).
    state, registry = build_season(_SCENARIO)
    resolver = weather_resolver(_weather(), _SCENARIO)
    forward, _, _ = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, len(_weather())
    )
    shuffled = Registry(
        list(reversed(registry.flows)),
        state.stocks,
        aux_processes=registry.aux_processes,
    )
    state2, _ = build_season(_SCENARIO)
    reversed_run, _, _ = run_season(
        EulerIntegrator(shuffled), state2, resolver, 1.0, len(_weather())
    )
    for sid, stock in forward[-1].stocks.items():
        assert stock.amount == reversed_run[-1].stocks[sid].amount


# --- carbon_fraction parity (the Step-11 consistency assertion) --------------
def test_carbon_fraction_parity_between_canopy_and_nitrogen() -> None:
    # Both fold carbon_fraction (sla → m²/mol C; N-thresholds → kg N/mol C); divergent
    # values would be a silently inconsistent plant. Read the raw yamls (utf-8).
    canopy = yaml.safe_load(CANOPY_PARAMS_PATH.read_text(encoding="utf-8"))
    nitro = yaml.safe_load(NITROGEN_PARAMS_PATH.read_text(encoding="utf-8"))
    cf_canopy = canopy["parameters"]["carbon_fraction"]["value"]
    cf_nitro = nitro["parameters"]["carbon_fraction"]["value"]
    assert math.isclose(cf_canopy, cf_nitro, rel_tol=1e-12)
