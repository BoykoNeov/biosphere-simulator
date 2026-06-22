"""Phase-3 Step-4 (P3.4) tests: closure-preserving mortality + annual reset.

The perennial sealed chamber (``PERENNIAL_CHAMBER_SCENARIO``, the season weather tiled
``PERENNIAL_CHAMBER_YEARS×``, Euler-daily) with an **annual phenology reset / re-sow**
applied by :func:`season.run_perennial` at each year boundary. Validates the two P3.4
deliverables — with **zero** ``simcore`` changes (``annual_reset`` is a driver helper,
the scenario is ``scenario.py``, the golden is ``test_regression_perennial_season.py``):

1. **Closure-preserving mortality.** Death routes the old plant's organ + grain carbon
   to the in-system ``litter_carbon`` POOL, never to the BOUNDARY loss-sink: the carbon
   loss-sink stays **exactly 0.0** and ``events == ()`` over the whole run. The
   loss-sink is the numerical round-off guard only (decision #6), not the death pathway,
   so the sealed chamber stays genuinely closed.
2. **Sustained multi-year oscillation.** With the reset, DVS reaches maturity (2.0)
   **every year** (not the one-shot "plant dies after year 1" baseline) and every year's
   biomass peak clears a floor — a stable emergent period-2 limit cycle
   (overcompensation, no control code). Throughout: ``rationed == 0`` and all four
   quantities conserved, including across the discrete resets (the driver re-asserts the
   gate at each reset).

The ``annual_reset`` carbon redistribution is also unit-tested directly: it conserves
CARBON exactly, moves no other quantity, leaves every amount non-negative, zeroes
``thermal_time``, and refuses to re-sow when the seed bank cannot cover a seedling (the
closure caveat).

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.loader import load_nitrogen_params, load_phenology_params
from domains.biosphere.nitrogen import nitrogen_stress_factor
from domains.biosphere.phenology import development_stage
from domains.biosphere.season import (
    LEAF_C,
    LITTER_CARBON,
    PERENNIAL_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_YEARS,
    PLANT_N,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    THERMAL_TIME,
    annual_reset,
    build_season,
    run_perennial,
    weather_resolver,
)
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


_YEAR = len(_weather())  # season length in steps (the tiling + reset period)
_PHENO = load_phenology_params()


def _run_canonical() -> tuple[list[State], int, tuple]:
    """Run the canonical perennial multi-year season (the single source of truth)."""
    weather = _weather() * PERENNIAL_CHAMBER_YEARS
    steps = len(weather)
    state, registry = build_season(PERENNIAL_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, PERENNIAL_CHAMBER_SCENARIO)
    return run_perennial(
        EulerIntegrator(registry),
        state,
        PERENNIAL_CHAMBER_SCENARIO,
        resolver,
        1.0,
        steps,
        year=_YEAR,
    )


@pytest.fixture(scope="module")
def perennial() -> tuple[list[State], int, tuple]:
    return _run_canonical()


def _dvs(state: State) -> float:
    return development_stage(
        state.aux[THERMAL_TIME],
        tsum_anthesis=_PHENO.tsum_anthesis,
        tsum_maturity=_PHENO.tsum_maturity,
    )


def _total(state: State, quantity: Quantity) -> float:
    return sum(
        st.amount * st.composition.get(quantity, 0.0) for st in state.stocks.values()
    )


def _vegetative(state: State) -> float:
    return (
        state.stocks[LEAF_C].amount
        + state.stocks[STEM_C].amount
        + state.stocks[ROOT_C].amount
    )


# --- closure-preserving mortality: the loss-sink stays the numerical guard only ----
def test_perennial_genuinely_closed(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # The P3.4 closure headline: the old plant's carbon routes to the in-system
    # litter_carbon POOL at each reset, never to the BOUNDARY loss-sink — so the sealed
    # chamber stays genuinely closed. The carbon loss-sink is empty on every step and no
    # extinction ever fires (the reset catches the organs before they reach threshold).
    states, _, events = perennial
    assert events == ()
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states)


# --- sustained oscillation: DVS=2 every year + a biomass floor (not period-matching) -
def test_perennial_sustained_oscillation(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # "Sustained, not damped": every year reaches maturity (DVS = 2.0) — regrowth, not a
    # one-shot — AND every year's vegetative-biomass peak clears a floor. Deliberately
    # NOT an N-vs-N+2 period assertion: the cycle is a period-2 attractor still in its
    # transient over this horizon (odd years rising ~0.74, even ~1.00 falling), so
    # equality/convergence would be flaky; the floor is the robust "alive every year".
    states, _, _ = perennial
    for y in range(PERENNIAL_CHAMBER_YEARS):
        segment = states[y * _YEAR : (y + 1) * _YEAR + 1]
        assert max(_dvs(s) for s in segment) == pytest.approx(2.0)
        assert max(_vegetative(s) for s in segment) > 0.5


def test_perennial_dvs_resets_each_year(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # The reset is real: DVS is pinned at maturity (2.0) at each year-end (pre-reset,
    # the stored boundary state) yet has fallen near 0 one step into the next year (the
    # thermal_time zeroing + one re-accumulation day): DVS genuinely cycles 2 -> 0 -> 2.
    states, _, _ = perennial
    for y in range(1, PERENNIAL_CHAMBER_YEARS):
        assert _dvs(states[y * _YEAR]) == pytest.approx(2.0)  # year-end, pre-reset
        assert _dvs(states[y * _YEAR + 1]) < 0.1  # one step after the reset


# --- stability through the discrete resets -----------------------------------------
def test_perennial_never_rations(perennial: tuple[list[State], int, tuple]) -> None:
    # rationed == 0 survives the discrete annual reset (structural positivity: the
    # redistribution sets amounts directly and the kinetics self-limit off zero).
    _, rationed, _ = perennial
    assert rationed == 0


@pytest.mark.parametrize(
    ("quantity", "abs_tol"),
    [
        (Quantity.CARBON, 1e-12),
        (Quantity.OXYGEN, 1e-11),
        (Quantity.WATER, 1e-7),
        (Quantity.NITROGEN, 1e-9),
    ],
)
def test_perennial_conserves_every_quantity(
    perennial: tuple[list[State], int, tuple], quantity: Quantity, abs_tol: float
) -> None:
    # All four quantities conserved on every stored step — INCLUDING across the resets
    # (annual_reset only redistributes CARBON between in-system stocks; the driver
    # re-asserts the gate at each reset, and the stored pre-reset / post-step states
    # both balance). The reset's only non-conserved write is thermal_time, off the gate.
    states, _, _ = perennial
    q0 = _total(states[0], quantity)
    for s in states:
        assert math.isclose(_total(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol)


def test_perennial_is_deterministic(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # Bit-identical on a re-run (the golden's premise; the reset closure is pure).
    states, rationed, events = perennial
    states2, rationed2, events2 = _run_canonical()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


# --- the carbon-only reset is justified: f_N ≡ 1 (verified, not assumed) -------------
def test_perennial_f_n_stays_one(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # annual_reset is CARBON-only: plant_n persists across the death, so the small reset
    # seedling inherits the full standing N (an N *windfall*). That is harmless ONLY if
    # f_N stays == 1 — otherwise the concentration spike would feed limitation =
    # f_water·f_N into gross assimilation and perturb the carbon trajectory (and the
    # golden would silently encode a different model than documented). Pinned here (the
    # test_sealed_f_n_stays_one precedent: verified, not assumed): the reset shrinks
    # biomass to the seedling, which RAISES N concentration, so f_N == 1 is in fact more
    # robust than the sealed run. Biomass is leaf+stem+root (storage excluded).
    states, _, _ = perennial
    nitro = load_nitrogen_params()
    for s in states:
        f_n = nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            _vegetative(s),
            n_residual_per_mol_c=nitro.n_residual_per_mol_c,
            n_critical_per_mol_c=nitro.n_critical_per_mol_c,
        )
        assert f_n == 1.0


# --- annual_reset (the pure transform) unit tests -----------------------------------
def test_annual_reset_conserves_carbon_and_moves_nothing_else(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # On a real pre-reset state (year-1 boundary), the redistribution conserves CARBON
    # exactly and moves NO other quantity (O2/N/water totals unchanged): it shuffles
    # carbon only between organs / grain / litter.
    states, _, _ = perennial
    before = states[_YEAR]  # end of year 1, pre-reset (the stored boundary state)
    after = annual_reset(before, PERENNIAL_CHAMBER_SCENARIO)
    assert math.isclose(
        _total(after, Quantity.CARBON),
        _total(before, Quantity.CARBON),
        rel_tol=0.0,
        abs_tol=1e-12,
    )
    for q in (Quantity.OXYGEN, Quantity.WATER, Quantity.NITROGEN):
        assert _total(after, q) == _total(before, q)


def test_annual_reset_state_shape(
    perennial: tuple[list[State], int, tuple],
) -> None:
    # The post-reset state: organs == the scenario seedling, grain drained to 0,
    # thermal_time zeroed, every amount non-negative, and n preserved (the clock).
    states, _, _ = perennial
    before = states[_YEAR]
    after = annual_reset(before, PERENNIAL_CHAMBER_SCENARIO)
    sc = PERENNIAL_CHAMBER_SCENARIO
    assert after.stocks[LEAF_C].amount == sc.leaf_c0
    assert after.stocks[STEM_C].amount == sc.stem_c0
    assert after.stocks[ROOT_C].amount == sc.root_c0
    assert after.stocks[STORAGE_C].amount == 0.0
    assert after.aux[THERMAL_TIME] == 0.0
    assert after.n == before.n
    assert all(st.amount >= 0.0 for st in after.stocks.values())
    # litter grew by exactly the residual old_veg + grain − seedling_total
    seedling_total = sc.leaf_c0 + sc.stem_c0 + sc.root_c0
    expected_litter = (
        before.stocks[LITTER_CARBON].amount
        + _vegetative(before)
        + before.stocks[STORAGE_C].amount
        - seedling_total
    )
    assert math.isclose(
        after.stocks[LITTER_CARBON].amount, expected_litter, rel_tol=0.0, abs_tol=1e-12
    )


def test_annual_reset_refuses_empty_seed_bank() -> None:
    # The closure caveat: re-sow must draw the seedling from the in-system grain. With
    # an empty seed bank (initial storage_c == 0 < seedling), conjuring a seedling would
    # create carbon from nothing / drive storage_c negative — so it raises.
    state, _ = build_season(PERENNIAL_CHAMBER_SCENARIO)
    assert state.stocks[STORAGE_C].amount == 0.0
    with pytest.raises(ValueError, match="seed bank too small"):
        annual_reset(state, PERENNIAL_CHAMBER_SCENARIO)
