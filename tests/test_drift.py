"""Phase-4 Step-1 (P4.1): the drift INSTRUMENT, validated on synthetic traces.

These tests pin :mod:`domains.biosphere.drift` against hand-built traces — independent
of the biology — so a regression in the *instrument* is caught separately from a
regression in the *simulation* (which ``test_decade_stability.py`` owns). Three things
are proved:

* **slope recovery** — a known linear-drift trace recovers its slope;
* **discrimination (the teeth)** — a deliberate leak at real-bug scale (~1e-9/step)
  **fails** the axis-(a) detector, while round-off-scale jitter **passes** it: the
  derived bounds sit *between* the round-off floor and a real-bug leak, not merely
  "slope-of-a-line works";
* **the stationarity / collapse split (advisor-confirmed, mandatory)** —
  ``is_stationary`` catches *amplifying* drift (diff-detectable) but is **blind** to
  creeping decay toward extinction (geometric decay shrinks the same-phase diffs
  identically to a converging
  cycle); only ``non_collapsing`` (a level/floor check) catches decay. Each synthetic
  case asserts *which* detector owns it.

Pure stdlib; no weather fixture, no run.
"""

import math

import pytest

from domains.biosphere.drift import (
    MASS_DRIFT_ABS_BOUND,
    MASS_DRIFT_SLOPE_BOUND,
    drift_slope,
    is_period_2,
    is_stationary,
    least_squares_slope,
    mass_drift_trace,
    max_abs,
    non_collapsing,
    same_phase_diffs,
    total_quantity,
    year_summaries,
)
from simcore.ids import DomainId, StockId, UnitLabel
from simcore.quantities import Quantity, StockKind
from simcore.state import State, Stock

_N = 4575  # the 15-yr decade-probe horizon, so the synthetic scales match the real one


# --- builders for the State-shaped instrument inputs --------------------------


def _stock(sid: str, quantity: Quantity, amount: float, comp=None) -> Stock:
    return Stock(
        id=StockId(sid),
        domain=DomainId("test"),
        quantity=quantity,
        unit=UnitLabel("mol"),
        amount=amount,
        kind=StockKind.POOL,
        composition=comp or {quantity: 1.0},
    )


def _state(n: int, *stocks: Stock) -> State:
    return State(n=n, stocks={s.id: s for s in stocks}, rng_seed=0)


# --- total_quantity: the promoted fold ---------------------------------------


def test_total_quantity_folds_composition_over_heterogeneous_stocks() -> None:
    # A CO2 stock carries {CARBON:1, OXYGEN:2}; O2 carries {OXYGEN:2}; biomass
    # {CARBON:1}; a water pool {WATER:1}. total_quantity must weight each stock's amount
    # by its coeff for the asked quantity and contribute 0 for an absent one (the
    # load-bearing default).
    co2 = _stock(
        "co2", Quantity.CARBON, 5.0, {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
    )
    o2 = _stock("o2", Quantity.OXYGEN, 3.0, {Quantity.OXYGEN: 2.0})
    bio = _stock("bio", Quantity.CARBON, 7.0)
    water = _stock("water", Quantity.WATER, 4.0)
    state = _state(0, co2, o2, bio, water)
    assert total_quantity(state, Quantity.CARBON) == 5.0 + 7.0  # co2 + biomass
    assert total_quantity(state, Quantity.OXYGEN) == 5.0 * 2 + 3.0 * 2  # co2 + o2
    assert total_quantity(state, Quantity.WATER) == 4.0
    assert total_quantity(state, Quantity.NITROGEN) == 0.0  # absent everywhere


def test_total_quantity_matches_the_inline_fold() -> None:
    # Byte-equivalent to the ``_total`` one-liner it promotes (the four chamber tests).
    co2 = _stock(
        "co2", Quantity.CARBON, 1.25, {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
    )
    bio = _stock("bio", Quantity.CARBON, 2.5)
    state = _state(0, co2, bio)
    for q in Quantity:
        inline = sum(
            st.amount * st.composition.get(q, 0.0) for st in state.stocks.values()
        )
        assert total_quantity(state, q) == inline


# --- axis (a): slope recovery + discrimination -------------------------------


def test_least_squares_slope_recovers_a_known_slope() -> None:
    m = 1e-9
    trace = [m * n for n in range(_N)]
    assert math.isclose(drift_slope(trace), m, rel_tol=1e-9)
    assert math.isclose(least_squares_slope(trace), m, rel_tol=1e-9)


def test_least_squares_slope_degenerate_inputs() -> None:
    assert least_squares_slope([]) == 0.0
    assert least_squares_slope([42.0]) == 0.0
    assert least_squares_slope([3.0, 3.0, 3.0]) == 0.0  # flat → zero slope


def test_max_abs() -> None:
    assert max_abs([]) == 0.0
    assert max_abs([1.0, -5.0, 3.0]) == 5.0


def test_detector_discriminates_leak_from_roundoff() -> None:
    # THE TEETH: the derived bounds sit BETWEEN the round-off floor and a real-bug leak.
    # A systematic leak ~1e-9/step must breach BOTH the slope and the abs bound; a
    # bounded round-off jitter at ~1e-11 scale (well above the measured ~3e-12 floor,
    # well below the leak) must pass BOTH. This proves the bound is not merely
    # "slope-of-a-line works".
    leak = [1e-9 * n for n in range(_N)]
    assert drift_slope(leak) > MASS_DRIFT_SLOPE_BOUND  # leak fails the slope detector
    assert max_abs(leak) > MASS_DRIFT_ABS_BOUND  # ...and the abs detector

    # Deterministic mean-zero round-off jitter (a fixed pattern, like the real trace).
    jitter = [1e-11 * ((n % 7) - 3) for n in range(_N)]
    assert (
        abs(drift_slope(jitter)) < MASS_DRIFT_SLOPE_BOUND
    )  # round-off passes the slope
    assert max_abs(jitter) < MASS_DRIFT_ABS_BOUND  # ...and the abs bound


def test_mass_drift_trace_is_relative_to_step_zero() -> None:
    # Clean deltas (avoid catastrophic cancellation): the point is the step-0 baseline.
    s0 = _state(0, _stock("c", Quantity.CARBON, 10.0))
    s1 = _state(1, _stock("c", Quantity.CARBON, 12.0))
    s2 = _state(2, _stock("c", Quantity.CARBON, 9.0))
    trace = mass_drift_trace([s0, s1, s2], Quantity.CARBON)
    assert trace == [0.0, 2.0, -1.0]


# --- axis (b): segmentation + same-phase diffs -------------------------------


def test_year_summaries_segments_like_the_perennial_tests() -> None:
    # year=3: 7 states (amounts 0..6) → 2 years; segment y spans [y*3 : (y+1)*3 + 1],
    # i.e. a full year PLUS the next year-boundary state (the perennial slice).
    states = [_state(i, _stock("x", Quantity.CARBON, float(i))) for i in range(7)]
    peak = year_summaries(
        states, 3, lambda seg: max(s.stocks[StockId("x")].amount for s in seg)
    )
    assert peak == [3.0, 6.0]  # seg0 = states[0:4] → 3 ; seg1 = states[3:7] → 6


def test_same_phase_diffs() -> None:
    assert same_phase_diffs([1.0, 2.0, 3.0, 4.0, 5.0], period=2) == [2.0, 2.0, 2.0]
    assert (
        same_phase_diffs([10.0, 20.0], period=2) == []
    )  # too short for a period-2 diff


# --- axis (b): the stationarity / collapse split -----------------------------


def test_is_stationary_passes_a_settled_period_2_cycle() -> None:
    settled = [1.0, 2.0, 1.0, 2.0, 1.0, 2.0]  # same-phase diffs all 0
    diffs = same_phase_diffs(settled, period=2)
    assert is_stationary(diffs, bound=0.1, slope_tol=1e-9)


def test_is_stationary_passes_a_still_converging_cycle() -> None:
    # The lock does NOT require a reached attractor: a cycle whose amplitude is still
    # shrinking toward a finite attractor is bounded + non-amplifying → stationary.
    converging = [0.5, 2.5, 0.8, 2.3, 0.95, 2.1, 1.0, 2.0, 1.0, 2.0]
    diffs = same_phase_diffs(converging, period=2)
    assert is_stationary(diffs, bound=0.5, slope_tol=1e-9)
    assert non_collapsing(converging, floor=0.1)  # ...and alive throughout


def test_is_stationary_fails_an_amplifying_cycle() -> None:
    # Amplitude GROWS (geometric: 0.1, 0.2, 0.4) → |same-phase diff| grows → the
    # non-amplifying slope check trips. This is the drift is_stationary DOES own.
    amplifying = [1.4, 1.6, 1.3, 1.7, 1.1, 1.9, 0.7, 2.3]
    diffs = same_phase_diffs(amplifying, period=2)
    assert not is_stationary(
        diffs, bound=10.0, slope_tol=0.0
    )  # fails on non-amplifying
    assert is_period_2(amplifying)  # ...still structurally period-2 while it diverges


def test_decay_is_diff_blind_and_only_the_floor_catches_it() -> None:
    # THE MANDATORY SPLIT: a cycle creeping toward extinction (geometric decay) has
    # SHRINKING same-phase diffs — mathematically indistinguishable from a converging
    # cycle by the diffs alone — so it PASSES is_stationary. Only the level/floor
    # check (non_collapsing) catches it. Assert BOTH so the test documents which
    # detector owns extinction (advisor: plan line 250's "decay fails is_stationary" is
    # wrong).
    decaying = [2.0, 1.0, 0.5, 0.25, 0.125, 0.0625, 0.03, 0.015]
    diffs = same_phase_diffs(decaying, period=2)
    assert is_stationary(diffs, bound=2.0, slope_tol=0.0)  # diff-BLIND: passes
    assert not non_collapsing(decaying, floor=0.1)  # the floor is the only detector


def test_non_collapsing_floor() -> None:
    assert non_collapsing([0.5, 0.6, 0.55, 0.6], floor=0.1)
    assert not non_collapsing(
        [0.5, 0.2, 0.05, 0.6], floor=0.1
    )  # one dip below the floor


# --- axis (b-discrete): the period-2 structural check ------------------------


def test_is_period_2_structural() -> None:
    assert is_period_2([1.0, 2.0, 1.0, 2.0, 1.0])  # clean alternation
    assert is_period_2([0.18, 0.26, 0.18, 0.25, 0.18])  # the real perennial signature
    assert not is_period_2([1.0, 2.0, 3.0, 4.0])  # monotone → period-1, not period-2
    assert not is_period_2([5.0, 5.0, 5.0])  # flat → no phase
    assert not is_period_2([1.0, 2.0])  # too short to establish alternation


def test_is_period_2_respects_the_transient() -> None:
    # A big leading transient then a clean alternation: dropping it reveals period-2.
    series = [0.0, 9.0, 1.0, 2.0, 1.0, 2.0, 1.0]
    assert is_period_2(series, transient=2)


@pytest.mark.parametrize("bad", [[], [1.0]])
def test_year_summaries_handles_short_trajectories(bad: list[float]) -> None:
    # (len(states) - 1) // year == 0 when there is not even one full year: no summaries.
    states = [_state(i, _stock("x", Quantity.CARBON, v)) for i, v in enumerate(bad)]
    assert year_summaries(states, 3, lambda seg: 0.0) == []
