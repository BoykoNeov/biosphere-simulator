"""Validation: Power → biosphere lighting (P6.5) — energy enters biology via the lamp.

Step 5's payload. A battery-powered grow lamp (``station.flows.Lamp``, ``battery →
light_used + waste_heat``) both **drains ENERGY** from ``power.battery`` and **sets the
biosphere's PAR forcing** — the phase's one NON-shared-stock coupling (finding #3 /
#16): Power and the biosphere share no stock, only the lamp-draw schedule, which feeds
the ENERGY ledger (this flow) and the PAR the frozen biosphere reads (``lamp_par``). The
seam is stepped by the two-rate master-step driver (biosphere once/day at ``dt = 1``,
Power ``substep`` ×24/day at ``dt = 3600`` s).

The non-vacuous demonstration:

* **Lamp flow closes ENERGY** — the 3-leg split (``−D`` battery, ``+η_lamp·D`` light,
  ``+(1−η_lamp)·D`` heat) balances ENERGY and touches nothing else; ``η_lamp`` is
  *derived* from the one ``photon_efficacy`` param via the biosphere's own McCree PAR
  constant.
* **PAR + daylength are reconstructed from the lamp** — ``PAR =
photon_efficacy·lamp_power_w
  /ground_area``, ``daylength_s = photoperiod_hours·3600`` (the "the factor actually
  bit"
  check; both must come from the lamp — the daily photon dose is PAR × daylength).
* **The signed "it bit" gate** — with the lamp on the seedling net-assimilates
  (``bio_organic_C`` grows); with the lamp **off** (PAR = 0) it only respires
  (``bio_organic_C`` declines). The lamp genuinely carries the energy driving carbon
  fixation.
* **The battery drains by exactly the lamp's daily energy** — ``lamp_power_w·photoperiod
  ·3600·days`` (well-fed, ``rationed == 0``); ``light_used`` / ``waste_heat`` accumulate
  the η-split of it. Lamp off ⇒ the battery is flat.
* **The biosphere internal loops still close** (Step 5 does not couple the chamber's
  CARBON / OXYGEN / WATER / NITROGEN to Power — the water ring stays independently
  closed).

The biosphere is **Euler-locked by its freeze**, so this is an Euler run (no RK4
cross-check — the frozen biosphere's numerics are fixed at ``dt = 1`` Euler; the Lamp is
forced, so it would agree bit-for-bit on the battery anyway).

Pure-stdlib spine; the crop params load from the biosphere YAMLs, the lamp photon
efficacy from ``station/params/lamp.yaml``.
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.stocks import (
    CONDENSATE,
    DAYLENGTH_VAR,
    LEAF_C,
    LITTER_CARBON,
    MICROBIAL_CARBON,
    PAR_VAR,
    ROOT_C,
    SOIL_WATER,
    STEM_C,
    STORAGE_C,
    WATER_VAPOR,
)
from domains.power.stocks import BATTERY, WASTE_HEAT, battery_stock
from simcore import boundary
from simcore.conservation import compute_ledger
from simcore.environment import BoundEnvironment, SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.flows import (
    LAMP_POWER_VAR,
    PAR_PHOTON_ENERGY_J_PER_UMOL,
    Lamp,
    LampParams,
    lamp_energy_split,
)
from station.lighting import (
    LIGHT_USED,
    build_lighting,
    lamp_average_power,
    lamp_par,
    lighting_bio_resolver,
    lighting_power_resolver,
    run_lighting,
)
from station.loader import load_lamp_params
from station.scenario import LIGHTING_SCENARIO

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_LP = load_lamp_params()
_SC = LIGHTING_SCENARIO

# The biosphere organic-carbon pools (the plant's cumulative sink lives here).
_BIO_C = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, LITTER_CARBON, MICROBIAL_CARBON)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(*, with_lamp: bool = True) -> tuple[list[State], int, tuple[object, ...]]:
    state, bio_reg, power_reg = build_lighting(_LP, _SC, with_lamp=with_lamp)
    return run_lighting(
        EulerIntegrator(bio_reg),
        EulerIntegrator(power_reg),
        state,
        lighting_bio_resolver(_weather(), _LP, _SC, with_lamp=with_lamp),
        lighting_power_resolver(_SC),
        _SC,
    )


def _amt(state: State, sid: StockId) -> float:
    return state.stocks[sid].amount


def _bio_organic_c(state: State) -> float:
    return sum(_amt(state, s) for s in _BIO_C)


# --- shared flow fixtures ----------------------------------------------------
def _lamp_state(*, battery: float = 1.0e8) -> State:
    """A State with the three lighting Power stocks (battery POOL + the two sinks)."""
    stocks = {
        s.id: s
        for s in (
            battery_stock(battery),
            boundary.sink(LIGHT_USED, Quantity.ENERGY, 0.0),
            boundary.sink(WASTE_HEAT, Quantity.ENERGY, 0.0),
        )
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _lamp_env(state: State, dt: float, *, lamp_power: float) -> BoundEnvironment:
    return SourceResolver(forcings={LAMP_POWER_VAR: constant(lamp_power)}).bind(
        state, dt
    )


def _lamp(efficacy: float = 2.5) -> Lamp:
    return Lamp(
        FlowId("station.lamp"),
        0,
        battery=BATTERY,
        light_used=LIGHT_USED,
        waste_heat=WASTE_HEAT,
        params=LampParams(photon_efficacy=efficacy),
    )


# --- rate law: lamp_energy_split ---------------------------------------------
def test_lamp_energy_split_is_the_radiant_partition() -> None:
    # radiant = η_lamp·D, heat = (1−η_lamp)·D, η_lamp = efficacy·E_photon.
    radiant, heat = lamp_energy_split(100.0, photon_efficacy=2.5)
    eta = 2.5 * PAR_PHOTON_ENERGY_J_PER_UMOL
    assert math.isclose(radiant, eta * 100.0, rel_tol=1e-12)
    assert math.isclose(heat, (1.0 - eta) * 100.0, rel_tol=1e-12)


def test_lamp_energy_split_sums_to_draw() -> None:
    # The two fractions sum back to the input (every joule named, not lost).
    radiant, heat = lamp_energy_split(137.0, photon_efficacy=2.1)
    assert math.isclose(radiant + heat, 137.0, rel_tol=1e-12)


def test_lamp_energy_split_zero_draw() -> None:
    # Lamp off ⇒ nothing radiated, nothing to heat.
    assert lamp_energy_split(0.0, photon_efficacy=2.5) == (0.0, 0.0)


# --- Lamp flow ---------------------------------------------------------------
def test_lamp_splits_draw_into_light_and_heat() -> None:
    # The draw leaves the battery (−D) and lands as radiant PAR (+η·D) + heat
    # (+(1−η)·D).
    state = _lamp_state()
    legs = {
        leg.stock: leg.amount
        for leg in _lamp(2.5)
        .evaluate(state, _lamp_env(state, 1.0, lamp_power=100.0), 1.0)
        .legs
    }
    eta = 2.5 * PAR_PHOTON_ENERGY_J_PER_UMOL
    assert math.isclose(legs[BATTERY], -100.0, rel_tol=1e-12)  # withdrawn
    assert math.isclose(legs[LIGHT_USED], eta * 100.0, rel_tol=1e-12)  # PAR light
    assert math.isclose(legs[WASTE_HEAT], (1.0 - eta) * 100.0, rel_tol=1e-12)  # heat


def test_lamp_balances_energy_only() -> None:
    # The 3-leg lossy flow balances ENERGY and touches no other quantity.
    state = _lamp_state()
    result = _lamp(2.5).evaluate(state, _lamp_env(state, 1.0, lamp_power=100.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    assert set(per_quantity_residual(result, state.stocks)) == {Quantity.ENERGY}


def test_lamp_always_three_legs() -> None:
    # The structural-legs convention (SolarCharge's "emit even at zero"): three legs at
    # the ceiling efficacy (heat leg 0) and lamp-off (all 0), never a variable count.
    state = _lamp_state()
    ceiling = 1.0 / PAR_PHOTON_ENERGY_J_PER_UMOL  # η_lamp = 1
    for eff, power in ((2.5, 100.0), (ceiling, 100.0), (2.5, 0.0)):
        legs = (
            _lamp(eff)
            .evaluate(state, _lamp_env(state, 1.0, lamp_power=power), 1.0)
            .legs
        )
        assert len(legs) == 3


def test_lamp_lossless_at_ceiling_efficacy() -> None:
    # At the physical ceiling (all input → PAR photons) the heat leg is exactly 0.
    state = _lamp_state()
    ceiling = 1.0 / PAR_PHOTON_ENERGY_J_PER_UMOL
    legs = {
        leg.stock: leg.amount
        for leg in _lamp(ceiling)
        .evaluate(state, _lamp_env(state, 1.0, lamp_power=100.0), 1.0)
        .legs
    }
    assert legs[WASTE_HEAT] == pytest.approx(0.0, abs=1e-9)
    assert math.isclose(legs[LIGHT_USED], 100.0, rel_tol=1e-12)


def test_lamp_off_is_noop() -> None:
    # No draw ⇒ three zero-amount legs (a clean no-op step, dt-independent).
    state = _lamp_state()
    legs = _lamp(2.5).evaluate(state, _lamp_env(state, 1.0, lamp_power=0.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


def test_lamp_is_dt_linear() -> None:
    # flux = rate·dt — the increment-form contract (RK4 order; multi-rate-safe).
    state = _lamp_state()
    half = {
        leg.stock: leg.amount
        for leg in _lamp(2.5)
        .evaluate(state, _lamp_env(state, 0.5, lamp_power=100.0), 0.5)
        .legs
    }
    full = {
        leg.stock: leg.amount
        for leg in _lamp(2.5)
        .evaluate(state, _lamp_env(state, 1.0, lamp_power=100.0), 1.0)
        .legs
    }
    assert math.isclose(full[LIGHT_USED], 2.0 * half[LIGHT_USED], rel_tol=1e-12)
    assert math.isclose(full[WASTE_HEAT], 2.0 * half[WASTE_HEAT], rel_tol=1e-12)


# --- loader ------------------------------------------------------------------
def test_load_lamp_params_value_and_unit() -> None:
    # The committed lamp.yaml loads to the expected efficacy (µmol/J).
    assert _LP.photon_efficacy == 2.5


def test_lamp_loader_rejects_bad_unit(tmp_path: Path) -> None:
    bad = tmp_path / "lamp.yaml"
    bad.write_text(
        "name: lamp\nprocess: grow_lamp\nparameters:\n"
        '  photon_efficacy: {value: 2.5, unit: "W", source: t}\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="photon_efficacy must be declared"):
        load_lamp_params(bad)


def test_lamp_loader_rejects_over_ceiling(tmp_path: Path) -> None:
    # Above PAR_UMOL_PER_J (≈4.57): an over-unity lamp (radiant PAR > input) — rejected.
    bad = tmp_path / "lamp.yaml"
    bad.write_text(
        "name: lamp\nprocess: grow_lamp\nparameters:\n"
        '  photon_efficacy: {value: 5.0, unit: "umol/J", source: t}\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="physical ceiling"):
        load_lamp_params(bad)


def test_lamp_loader_rejects_nonpositive(tmp_path: Path) -> None:
    bad = tmp_path / "lamp.yaml"
    bad.write_text(
        "name: lamp\nprocess: grow_lamp\nparameters:\n"
        '  photon_efficacy: {value: 0.0, unit: "umol/J", source: t}\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="photon_efficacy must be in"):
        load_lamp_params(bad)


# --- the reconstruct-the-factor gate -----------------------------------------
def test_par_reconstructed_from_lamp_draw() -> None:
    # The biosphere's PAR forcing IS photon_efficacy·lamp_power_w/ground_area (the "it
    # bit" factor check), and daylength IS photoperiod_hours·3600 — both from the lamp.
    resolver = lighting_bio_resolver(_weather(), _LP, _SC, with_lamp=True)
    bound = resolver.bind(State(n=0, stocks={}, rng_seed=0), _SC.bio_dt)
    expected_par = _LP.photon_efficacy * _SC.lamp_power_w / _SC.bio.ground_area
    assert bound.get(PAR_VAR) == expected_par
    assert bound.get(DAYLENGTH_VAR) == _SC.photoperiod_hours * 3600.0
    assert lamp_par(_LP, _SC) == expected_par


def test_par_zero_when_unlit() -> None:
    # Lamp off ⇒ PAR forcing is 0 (dark), but daylength stays a valid positive window.
    resolver = lighting_bio_resolver(_weather(), _LP, _SC, with_lamp=False)
    bound = resolver.bind(State(n=0, stocks={}, rng_seed=0), _SC.bio_dt)
    assert bound.get(PAR_VAR) == 0.0
    assert bound.get(DAYLENGTH_VAR) > 0.0


# --- the run -----------------------------------------------------------------
def test_rationed_zero_and_event_free() -> None:
    # Well-fed both lit AND dark (the battery is sized to survive the lamp draw; the
    # plant has no POPULATION stock to go extinct).
    for with_lamp in (True, False):
        _, rationed, events = _run(with_lamp=with_lamp)
        assert rationed == 0, f"lighting must be well-fed (with_lamp={with_lamp})"
        assert events == (), f"lighting must be event-free (with_lamp={with_lamp})"


def test_every_day_boundary_conserves() -> None:
    # Independent re-check of the driver's per-sub-step gate: over each master day every
    # conserved quantity balances across the combined biosphere+Power ledger.
    states, _, _ = _run()
    for before, after in zip(states, states[1:], strict=False):
        for ql in compute_ledger(before, after):
            assert abs(ql.residual) <= 1e-6, (
                f"{ql.quantity} must close across each lighting day (residual "
                f"{ql.residual:.2e})"
            )


def test_plant_grows_under_lamp() -> None:
    # The signed sink check: the seedling fixes net carbon under the lamp (biosphere
    # organic carbon increases over the growth-phase window, DVS 0 start).
    states, _, _ = _run(with_lamp=True)
    gained = _bio_organic_c(states[-1]) - _bio_organic_c(states[0])
    assert gained > 0.0, (
        "the lit plant must fix net carbon (photosynthesis under the lamp)"
    )


def test_plant_declines_without_lamp() -> None:
    # The other arm of the "it bit" gate: with PAR = 0 gross assimilation is 0, so the
    # plant only respires — biosphere organic carbon DECLINES. The lamp is what makes
    # the difference (same biosphere, same weather temp/VPD — only PAR changed).
    lit = _run(with_lamp=True)[0]
    dark = _run(with_lamp=False)[0]
    assert _bio_organic_c(dark[-1]) < _bio_organic_c(dark[0]), (
        "the unlit plant must lose carbon (respiration only, no assimilation)"
    )
    assert _bio_organic_c(lit[-1]) > _bio_organic_c(dark[-1]), (
        "the lamp must make the plant a net sink relative to the dark baseline"
    )


def test_battery_drains_by_the_lamp_daily_energy() -> None:
    # ENERGY closure, quantitatively: the battery loses exactly the lamp's daily energy
    # ×days, and light_used / waste_heat accumulate the η-split of it.
    states, _, _ = _run(with_lamp=True)
    initial, final = states[0], states[-1]
    daily = _SC.lamp_power_w * _SC.photoperiod_hours * 3600.0  # = avg_power·86400
    drawn = daily * _SC.days
    assert lamp_average_power(_SC) * 86400.0 == pytest.approx(daily, rel=1e-12)
    assert _amt(initial, BATTERY) - _amt(final, BATTERY) == pytest.approx(
        drawn, rel=1e-9
    )
    eta = _LP.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL
    assert _amt(final, LIGHT_USED) == pytest.approx(eta * drawn, rel=1e-9)
    assert _amt(final, WASTE_HEAT) == pytest.approx((1.0 - eta) * drawn, rel=1e-9)
    # Well-fed: the battery stayed far above 0 (rationed == 0 is structural proof, this
    # is the magnitude).
    assert _amt(final, BATTERY) > 0.5 * _amt(initial, BATTERY)


def test_battery_flat_without_lamp() -> None:
    # Lamp off ⇒ the Power registry is empty, so the battery never moves (the seam's
    # only driver is the lamp).
    states, _, _ = _run(with_lamp=False)
    assert _amt(states[-1], BATTERY) == _amt(states[0], BATTERY)


def test_biosphere_internal_water_loop_closed() -> None:
    # Step 5 does NOT couple the biosphere water cycle to Power, so the internal ring
    # soil_water → water_vapor → condensate → soil_water stays closed (its total is
    # conserved to round-off across the whole run) — the biosphere's own closure
    # survives the lighting coupling untouched.
    states, _, _ = _run()
    loop = (SOIL_WATER, WATER_VAPOR, CONDENSATE)
    total0 = sum(_amt(states[0], s) for s in loop)
    totalf = sum(_amt(states[-1], s) for s in loop)
    assert abs(totalf - total0) <= 1e-9, (
        f"biosphere internal water loop must stay closed (drift {totalf - total0:.2e})"
    )


def test_power_and_biosphere_share_no_stock() -> None:
    # The finding-#3 statement, asserted structurally: the biosphere's own stock ids and
    # the three Power ENERGY stock ids are DISJOINT — the only coupling is the lamp-draw
    # schedule (a forcing, #16), never a shared stock.
    from domains.biosphere.season import build_season

    bio_state, _ = build_season(_SC.bio)
    bio_ids = set(bio_state.stocks)
    power_ids = {BATTERY, LIGHT_USED, WASTE_HEAT}
    assert power_ids.isdisjoint(bio_ids), (
        "Power and the biosphere must share NO stock — the only coupling is the "
        "lamp-draw schedule (a forcing, #16)"
    )


def test_determinism() -> None:
    # Bit-identical re-run (decision #7): the lighting station is deterministic.
    assert _run()[0][-1] == _run()[0][-1]
