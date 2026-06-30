"""Drift instrumentation (Phase-4 Step-1, P4.1) — decade-scale stability metrics.

A **measurement instrument** over a closed-biosphere trajectory (``list[State]``), used
to decide *operationally* whether the emergent period-2 limit cycle is
**conservation-stable over decade-scale runs** (roadmap line 211, "100k+ steps, no
drift"). "No drift" is the subtle one — the cycle is *meant* to oscillate, so
drift must be defined for a limit cycle along three axes (the P4.1 decision):

* **(a) Mass-conservation drift** — does ``total_q`` walk away from its step-0
  value? :func:`mass_drift_trace` / :func:`drift_slope` / :func:`max_abs`. Two
  tiers: a structural **ceiling** (``N * BALANCE_ATOL``, the triangle-inequality
  worst case, asserted by the caller) and a **detector** — the round-off-scale
  bounds, the real test that a systematic leak (linear in ``n``) is absent. The
  detector bounds are *derived* from the measured round-off accumulation, not
  hand-tuned (see the constants below).
* **(b) Limit-cycle stationarity** — per-year scalar summaries
  (:func:`year_summaries`) reach an attractor and **hold** it. The catch: for a
  period-2 cycle ``summary(k) - summary(k-1)`` does *not* vanish — it is the cycle
  amplitude — so stationarity is tested on **same-phase** differences
  :func:`same_phase_diffs` (``summary(k) - summary(k-2)``). The split
  (advisor-confirmed, mandatory not stylistic):
    - :func:`is_stationary` = **bounded + non-amplifying**. Catches *amplifying*
      drift, which is diff-detectable (``|d[k]|`` grows). It passes a
      still-converging cycle (amplitude shrinking toward a finite attractor) — the
      lock does NOT require a reached attractor.
    - :func:`non_collapsing` = the **extinction detector** (a level/floor check).
      This is *mandatory*, not stylistic: creeping decay toward extinction is
      **diff-blind** — geometric decay ``s[k] = C * r**k`` shrinks ``|d[k]|`` toward
      zero *identically* to a cycle converging to a finite attractor; the only
      difference is the *limit* (zero vs nonzero), a property of the summary
      **level**, not its diffs. So decay can only be caught by checking the
      summaries stay above a floor.
  :func:`is_period_2` is the separate **discrete** structural check (the cycle
  remains period-2) — kept apart from the scalar vector (a phase index is not a
  scalar you can call "non-increasing").

Pure stdlib; imports ``simcore.state`` / ``simcore.quantities`` **read-only** — a
domain module, not core. ``git diff src/simcore/`` stays empty. It is generic over
``State``: the per-year ``summary_fn`` (which references domain stock ids) is
supplied by the caller, so this module never imports a stock-id catalog.
"""

from collections.abc import Callable, Sequence

from simcore.quantities import Quantity
from simcore.state import State

# --- the promoted fold -------------------------------------------------------


def total_quantity(state: State, quantity: Quantity) -> float:
    """Folded total of ``quantity`` over ALL stocks (boundaries/leak-sink incl.).

    Promotes the ``_total`` fold duplicated across the Phase-3 chamber tests. The
    ``composition.get(quantity, 0.0)`` default-zero is load-bearing: it folds correctly
    over a heterogeneous ``State`` (water/N/O2 pools, boundary + loss + leak sinks all
    contribute ``0.0`` for an absent quantity), so a *vented* leak still conserves the
    total because the boundary sink carries the vented mass.
    """
    return sum(
        st.amount * st.composition.get(quantity, 0.0) for st in state.stocks.values()
    )


# --- shared primitive --------------------------------------------------------


def least_squares_slope(values: Sequence[float]) -> float:
    """Ordinary-least-squares slope of ``values`` vs index ``0..len-1`` (pure stdlib).

    The systematic-trend signature: a leak is linear in ``n`` (nonzero slope); bounded
    round-off jitter is not (slope ~ machine-eps noise). Returns ``0.0`` for fewer than
    two points or a degenerate (zero-variance) abscissa.
    """
    n = len(values)
    if n < 2:
        return 0.0
    mean_x = (n - 1) / 2.0
    mean_y = sum(values) / n
    numerator = sum((i - mean_x) * (v - mean_y) for i, v in enumerate(values))
    denominator = sum((i - mean_x) ** 2 for i in range(n))
    return numerator / denominator if denominator != 0.0 else 0.0


# --- axis (a): mass-conservation drift ---------------------------------------


def mass_drift_trace(states: Sequence[State], quantity: Quantity) -> list[float]:
    """``[total_q(n) - total_q(0)]`` over the trajectory — the accumulation trace.

    For a structurally-balanced flow set ``total_q`` is conserved up to float round-off,
    so this trace is expected to be bounded-oscillating / sqrt(N) noise, NOT linearly
    trending; a systematic leak shows up as a nonzero :func:`drift_slope`.
    """
    base = total_quantity(states[0], quantity)
    return [total_quantity(s, quantity) - base for s in states]


def drift_slope(trace: Sequence[float]) -> float:
    """Least-squares slope of a mass-drift trace vs step index (the leak signature)."""
    return least_squares_slope(trace)


def max_abs(trace: Sequence[float]) -> float:
    """``max|trace|`` — the interpretable conservation bound (vs the ceiling)."""
    return max((abs(x) for x in trace), default=0.0)


# Detector bounds for axis (a) — DERIVED from the Step-1 probe measurement, NOT
# hand-tuned (the "parameters are data, tolerances are derived" discipline). A
# regression-guard threshold derived from round-off is part of the *test*, not a model
# coefficient, so it lives here as a documented constant with provenance — not in
# config YAML (a pydantic schema for one diagnostic number is the speculative generality
# this codebase rejects; cf. ``BALANCE_ATOL`` being a ``simcore`` constant).
#
# The trace JITTERS, it does not trend (measured below): for a structurally-balanced
# flow set ``total_q`` is conserved to bounded sqrt(N) round-off, so ``max|d_q|`` is the
# directly-interpretable primary statistic and ``drift_slope`` is the secondary
# systematic-leak signature (a leak is linear in ``n``; round-off is not). Both are
# asserted — the abs bound catches accumulation, the slope catches a slow leak.
#
# PROVENANCE (re-derivable — the goldens' regeneration discipline):
#   Procedure : run PERENNIAL_CHAMBER_SCENARIO and CONSUMER_CHAMBER_SCENARIO under BOTH
#               EulerIntegrator and Rk4Integrator, dt = 1.0, to a 15-year horizon
#               (steps = 15 * len(weather) = 4575), and measure per quantity
#               q in {CARBON, OXYGEN, NITROGEN, WATER}:
#                 max|d_q| = max_abs(mass_drift_trace(states, q))
#                 slope_q  = drift_slope(mass_drift_trace(states, q))
#   Observed  : worst-case over all (scenario x integrator x q) at 15 yr —
#                 max|d_q|  ~ 3.3e-12  (WATER; CARBON ~1e-14, the tightest)
#                 |slope_q| ~ 7.3e-16  (WATER) — i.e. machine-eps noise, no trend
#               vs the structural ceiling N * BALANCE_ATOL ~ 4575 * 1e-9 ~ 4.6e-6 (~6-9
#               orders looser) and a real-bug leak ~1e-9 / step (max|d| ~ 4.6e-6, slope
#               ~ 1e-9 at this horizon).
#   Bounds    : set BETWEEN the measured noise floor and the real-bug leak, so the
#               detector has teeth (test_drift.py::test_detector_discriminates) yet
#               never trips on round-off.
#   Re-confirmed at 328 yr (the Step-3 100k stress, test_biosphere_stress.py — steps =
#               328 * len(weather) = 100,040, Euler, both closed scenarios):
#                 max|d_q|  ~ 7.4e-11  (WATER; CARBON ~2e-14, still the tightest)
#                 |slope_q| ~ 7.5e-16  (WATER) — STILL ~3x machine-eps, no growth
#               Over the 22x-longer run the SLOPE stayed flat (no systematic leak
#               emerged), while max|d_q| grew ~linearly AT that machine-eps slope
#               (slope * N ~ 7.5e-16 * 1e5 ~ 7.4e-11 ~ max|d_q|) — i.e. deterministic
#               round-off in this fold's summation order, NOT a leak. Both bounds hold
#               with margin at 100k (abs ~13x, slope ~4.5 orders), so they span both
#               horizons; no re-derivation needed.
#
# ABS bound: ~300x the 15-yr round-off floor (3.3e-12), ~13x the 328-yr floor (7.4e-11),
# always >> a 1e-9/step leak (which puts max|d| at the ceiling ~1e-4 within ~328 yr).
MASS_DRIFT_ABS_BOUND: float = 1e-9
# SLOPE bound: ~4 orders above the measured round-off slope (~7.3e-16 at 15 yr, ~7.5e-16
# at 328 yr — flat), ~2 orders below a 1e-9/step leak's slope (1e-9).
MASS_DRIFT_SLOPE_BOUND: float = 1e-11


# --- axis (b): limit-cycle stationarity --------------------------------------


def year_summaries(
    states: Sequence[State],
    year: int,
    summary_fn: Callable[[Sequence[State]], float],
) -> list[float]:
    """One scalar per year via ``summary_fn`` over each year's segment.

    Reuses the perennial segmentation ``states[y*year : (y+1)*year + 1]`` (each segment
    spans a full year and includes the next year-boundary state, exactly as the Phase-3
    perennial / ledger tests slice). ``summary_fn`` maps a segment to a scalar (e.g.
    peak ``leaf_c``, min ``carbon_pool``, year-end ``consumer_carbon``) — it
    references the domain stock ids, keeping this module generic over ``State``.
    """
    n_years = (len(states) - 1) // year
    return [summary_fn(states[y * year : (y + 1) * year + 1]) for y in range(n_years)]


def same_phase_diffs(summaries: Sequence[float], period: int = 2) -> list[float]:
    """``[summary(k) - summary(k-period)]`` — the same-branch difference of the cycle.

    For a period-2 cycle the *adjacent* difference is the cycle amplitude (it does not
    vanish); the same-phase difference *does* vanish once the branch settles, so it is
    the right stationarity signal.
    """
    return [summaries[k] - summaries[k - period] for k in range(period, len(summaries))]


def is_stationary(
    diffs: Sequence[float],
    *,
    bound: float,
    slope_tol: float,
    transient: int = 0,
) -> bool:
    """Bounded + non-amplifying past the ``transient`` (the amplitude-drift detector).

    ``bounded`` = ``max|diff| <= bound`` (vs the summary scale). ``non_amplifying`` =
    least-squares slope of ``|diff|`` <= ``slope_tol`` — a *trend* test, not strict
    pairwise-monotone, which with ~4-8 diffs and float noise would be the flakiness P4.1
    warns against.

    Catches *amplifying* drift (``|diff|`` grows — diff-detectable) and passes a
    settled or still-converging cycle (``|diff|`` flat or shrinking). It is **blind**
    to creeping decay toward extinction (``|diff|`` shrinks too — see
    :func:`non_collapsing`).
    """
    tail = list(diffs)[transient:]
    if not tail:
        return True
    bounded = max(abs(d) for d in tail) <= bound
    non_amplifying = least_squares_slope([abs(d) for d in tail]) <= slope_tol
    return bounded and non_amplifying


def non_collapsing(summaries: Sequence[float], *, floor: float) -> bool:
    """The extinction detector: every per-year summary stays above ``floor``.

    Mandatory companion to :func:`is_stationary`: a cycle decaying toward extinction has
    shrinking same-phase diffs (diff-blind), so only a *level* check on the summaries
    themselves catches it. Reuses the perennial ``max(_vegetative) > 0.5`` floor idiom.
    """
    return all(s >= floor for s in summaries)


# --- axis (b-discrete): the period-2 structural check ------------------------


def is_period_2(
    summaries: Sequence[float], *, transient: int = 0, min_rel_gap: float = 1e-3
) -> bool:
    """Discrete structural check: a *sustained* period-2 cycle past ``transient``.

    A genuine period-2 limit cycle has the odd/even years on **opposite branches**, so
    every adjacent difference ``summary(k+1) - summary(k)`` is (1) a real jump and (2)
    alternating in sign. Both conditions must hold across the whole post-``transient``
    tail:

    * **alternation** — the structural period-2 signature;
    * **a sustained branch gap** — EVERY adjacent ``|diff|`` exceeds ``min_rel_gap *
      scale`` (``scale`` = ``max|summary|`` over the tail). This is the load-bearing
      guard: a *damped* oscillation **converging to a fixed point (period-1)** rings —
      its adjacent diffs alternate during the transient — but the gap collapses to ~0.
      Without the gap floor that ringing is misread as period-2 (the consumer chamber is
      period-1: the herbivore damps the producer cycle to a fixed point). So pick a
      ``transient`` that reaches the settled tail before calling this.

    Kept SEPARATE from the scalar stationarity vector: a phase index is not a scalar you
    can call "non-increasing".
    """
    tail = list(summaries)[transient:]
    if len(tail) < 3:
        return False
    adjacent = [tail[k + 1] - tail[k] for k in range(len(tail) - 1)]
    scale = max((abs(s) for s in tail), default=0.0)
    floor = min_rel_gap * scale
    # A collapsed adjacent gap means the branches merged → a fixed point, not a cycle.
    if any(abs(d) <= floor for d in adjacent):
        return False
    rises = [d > 0.0 for d in adjacent]
    return all(rises[i] != rises[i + 1] for i in range(len(rises) - 1))
