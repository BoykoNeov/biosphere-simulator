"""Convergence / drift gate: a mass-conserving Lotka–Volterra oscillator (step 11).

This is the suite's **numerical convergence/drift** exit gate (see the test-suite
table in ``docs/plans/phase-0-engine-skeleton.md``). It exercises the integrators on
an *oscillatory* problem — the regime where an explicit scheme's order of accuracy
actually shows, and where forward Euler famously misbehaves.

**The scenario — Lotka–Volterra, made mass-conserving.** Predator–prey is the
canonical autonomous oscillator (its small oscillations about the fixed point are
harmonic), and it is the textbook trap for forward Euler, whose trajectory spirals
*outward* — spurious amplitude growth — no matter how small ``dt`` is. The raw LV
equations are not mass-conserving (prey appear from nothing; predators vanish), so
to run them through the always-on conservation gate every transfer is balanced
against **one unclamped BOUNDARY carbon reservoir** ``R``:

    prey ``x`` and predator ``y`` are CARBON ``POOL`` stocks; ``R`` is the "outside".

    * **birth**      ``R -> x``                ``α·x·dt``   (prey grow by drawing C)
    * **predation**  ``x -> y`` (+ ``x -> R``) ``β·x·y·dt`` eaten, ``δ·x·y·dt`` to
                     the predator, the ``(β−δ)·x·y·dt`` conversion inefficiency to R
    * **death**      ``y -> R``                ``γ·y·dt``   (predators return C to R)

Every flow has ``Σ legs == 0`` in carbon, so ``x + y + R`` is exactly constant and
the per-step conservation gate (which runs inside ``step_report``) passes for free;
the dynamics of ``(x, y)`` are nonetheless exactly classical LV
(``ẋ = αx − βxy``, ``ẏ = δxy − γy``). Each flow is first-order and ``dt``-linear
(``leg == dt·rate``), so RK4 keeps its 4th order (the increment-form contract).

**Why POOL, not POPULATION.** ``x``/``y`` are ``POOL`` deliberately: a ``POPULATION``
stock would invite the extinction machinery (snap-to-0 + loss-sink + event) if an
Euler trough dipped to/below 0, masking the growth signal. As ``POOL`` stocks their
only protection against going negative is arbitration — which is exactly the
``rationed == 0`` guarantee the non-arbitrating gate below asserts.

**The conserved invariant.** The continuous LV flow conserves
``V(x, y) = δx − γ·ln x + β·y − α·ln y`` (closed orbits are its level sets). The
integrator's *drift* in ``V`` is therefore a clean, trajectory-spanning measure of
numerical error: ~0 for the exact flow, O(dt⁴) for RK4, and a one-signed O(dt)
*growth* for Euler (the spiral). Drift is measured as ``max_t |V − V₀|`` over the
run — not the endpoint, which lands anywhere in the per-orbit wobble.

Bands below were set from measured values (RK4 halving-dt ratios 16.3–18.1; Euler vs
RK4 drift separation ~10⁶×) with generous margin. Test-local flow dataclasses,
mirroring the ``_DecayFlow`` precedent in ``test_integrator`` — ``simcore`` and the
biosphere domain stay free of this synthetic oscillator.
"""

import dataclasses
import math

from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- ids -------------------------------------------------------------------
OSC = DomainId("osc")
PREY = StockId("osc.prey")
PRED = StockId("osc.pred")
RESERVOIR = StockId("boundary.osc_reservoir")
RESERVOIR0 = 1000.0  # arbitrary: R is never *read* by a flow, only bookkept.


@dataclasses.dataclass(frozen=True)
class LvParams:
    """Lotka–Volterra coefficients. ``delta <= beta`` (conversion efficiency ≤ 1)."""

    alpha: float  # prey birth
    beta: float  # predation pressure on prey
    delta: float  # predator growth per unit predation (carbon actually assimilated)
    gamma: float  # predator death


# --- test-local flows (all CARBON, all dt-linear, all balanced) ------------
@dataclasses.dataclass(frozen=True)
class PreyBirth:
    """``reservoir -> prey`` at ``α·prey·dt`` — prey grow by drawing boundary carbon."""

    id: FlowId
    priority: int
    prey: StockId
    reservoir: StockId
    alpha: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        grown = self.alpha * snapshot.stocks[self.prey].amount * dt
        return FlowResult(legs=(Leg(self.reservoir, -grown), Leg(self.prey, grown)))


@dataclasses.dataclass(frozen=True)
class Predation:
    """``prey -> predator`` (+ inefficiency ``prey -> reservoir``), rate ``β·x·y``.

    ``β·x·y·dt`` is removed from prey; ``δ·x·y·dt`` is assimilated into the predator;
    the ``(β−δ)·x·y·dt`` remainder (conversion inefficiency) returns to the boundary
    reservoir. Carbon-balanced for any ``δ ≤ β``.
    """

    id: FlowId
    priority: int
    prey: StockId
    pred: StockId
    reservoir: StockId
    beta: float
    delta: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        xy = snapshot.stocks[self.prey].amount * snapshot.stocks[self.pred].amount * dt
        eaten = self.beta * xy
        assimilated = self.delta * xy
        lost = eaten - assimilated  # (β−δ)·x·y·dt ≥ 0
        return FlowResult(
            legs=(
                Leg(self.prey, -eaten),
                Leg(self.pred, assimilated),
                Leg(self.reservoir, lost),
            )
        )


@dataclasses.dataclass(frozen=True)
class PredatorDeath:
    """``predator -> reservoir`` at ``γ·predator·dt`` — predators return carbon to R."""

    id: FlowId
    priority: int
    pred: StockId
    reservoir: StockId
    gamma: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        died = self.gamma * snapshot.stocks[self.pred].amount * dt
        return FlowResult(legs=(Leg(self.pred, -died), Leg(self.reservoir, died)))


# --- scenario builder + metrics --------------------------------------------
def _build(p: LvParams, x0: float, y0: float) -> tuple[State, Registry]:
    carbon = canonical_unit(Quantity.CARBON)
    prey = Stock(
        id=PREY,
        domain=OSC,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=x0,
        kind=StockKind.POOL,
    )
    pred = Stock(
        id=PRED,
        domain=OSC,
        quantity=Quantity.CARBON,
        unit=carbon,
        amount=y0,
        kind=StockKind.POOL,
    )
    # An unclamped BOUNDARY carbon reservoir — birth withdraws, death + predation
    # inefficiency deposit. unclamped so min-scaling never throttles the supply
    # (and it may go negative by design — it is pure bookkeeping; no flow reads it).
    reservoir = boundary.source(RESERVOIR, Quantity.CARBON, RESERVOIR0)
    stocks = {s.id: s for s in (prey, pred, reservoir)}
    flows = [
        PreyBirth(FlowId("osc.birth"), 0, PREY, RESERVOIR, p.alpha),
        Predation(FlowId("osc.predation"), 0, PREY, PRED, RESERVOIR, p.beta, p.delta),
        PredatorDeath(FlowId("osc.death"), 0, PRED, RESERVOIR, p.gamma),
    ]
    return State(n=0, stocks=stocks, rng_seed=0), Registry(flows, stocks)


def _invariant(p: LvParams, x: float, y: float) -> float:
    """The conserved LV quantity ``V = δx − γ·ln x + β·y − α·ln y``."""
    return p.delta * x - p.gamma * math.log(x) + p.beta * y - p.alpha * math.log(y)


@dataclasses.dataclass(frozen=True)
class _RunStats:
    max_v_drift: float  # max_t |V − V₀| over the run (the convergence/drift metric)
    v_growth: float  # V_end − V₀ (one-signed and positive ⇒ the Euler spiral)
    rationed: int  # total backstop firings (must be 0 — non-arbitrating)
    x_min: float  # smallest prey value seen (>0 ⇒ never went negative)
    y_min: float  # smallest predator value seen
    prey_peak_early: float  # max prey over the first half of the run
    prey_peak_late: float  # max prey over the second half (Euler: > early ⇒ growth)
    max_total_drift: float  # max_t |x + y + R − (x₀ + y₀ + R₀)| (mass closure)


def _run(
    cls: type, p: LvParams, x0: float, y0: float, dt: float, steps: int
) -> _RunStats:
    """Step ``cls`` ``steps`` times, tracking the convergence/drift/closure metrics."""
    state, registry = _build(p, x0, y0)
    integrator = cls(registry)
    env = SourceResolver()  # the LV flows are autonomous — they read no env var
    v0 = _invariant(p, x0, y0)
    total0 = x0 + y0 + RESERVOIR0
    max_v_drift = 0.0
    max_total_drift = 0.0
    rationed = 0
    x_min, y_min = x0, y0
    v_end = v0
    peak_early = peak_late = 0.0
    half = steps / 2.0
    for i in range(steps):
        report = integrator.step_report(state, env, dt)
        state = report.state
        rationed += report.rationed
        x = state.stocks[PREY].amount
        y = state.stocks[PRED].amount
        r = state.stocks[RESERVOIR].amount
        v_end = _invariant(p, x, y)
        max_v_drift = max(max_v_drift, abs(v_end - v0))
        max_total_drift = max(max_total_drift, abs(x + y + r - total0))
        x_min, y_min = min(x_min, x), min(y_min, y)
        if i < half:
            peak_early = max(peak_early, x)
        else:
            peak_late = max(peak_late, x)
    return _RunStats(
        max_v_drift=max_v_drift,
        v_growth=v_end - v0,
        rationed=rationed,
        x_min=x_min,
        y_min=y_min,
        prey_peak_early=peak_early,
        prey_peak_late=peak_late,
        max_total_drift=max_total_drift,
    )


# A single shared scenario: fixed point (1, 1), started off-orbit at (1, 0.5);
# small-oscillation period ≈ 2π/√(αγ) ≈ 8.886, so T_end = 20 spans ~2.25 periods.
# Horizon kept short enough that even Euler's outward spiral leaves x,y comfortably
# above 0 (measured x_min ≈ 0.27) — `_invariant` takes math.log(x)/log(y) live during
# the run, so a much longer horizon (orbit nearing an axis) would need x,y > 0 ensured
# before the log, not just asserted afterwards.
_P = LvParams(alpha=1.0, beta=1.0, delta=0.5, gamma=0.5)
_X0, _Y0 = 1.0, 0.5
_DT_COARSE, _DT_FINE, _T_END = 0.1, 0.05, 20.0
_STEPS_COARSE = round(_T_END / _DT_COARSE)  # 200
_STEPS_FINE = round(_T_END / _DT_FINE)  # 400


# --- the gates -------------------------------------------------------------
def test_rk4_invariant_drift_converges_at_fourth_order() -> None:
    """RK4 conserves V, and its drift falls ~16× per dt-halving (4th order → dt→0).

    The exact flow conserves ``V``; RK4's residual drift is O(dt⁴), so halving ``dt``
    must shrink ``max_t|V−V₀|`` toward 16×. Measured ratios 16.3–18.1 → band [12, 20].
    """
    coarse = _run(Rk4Integrator, _P, _X0, _Y0, _DT_COARSE, _STEPS_COARSE)
    fine = _run(Rk4Integrator, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)

    # RK4 actually conserves the invariant tightly (well above the round-off floor).
    assert fine.max_v_drift < 1e-6
    # ...and the error is genuinely 4th-order: ~16× drop per dt-halving.
    ratio = coarse.max_v_drift / fine.max_v_drift
    assert 12.0 < ratio < 20.0


def test_euler_exhibits_spurious_amplitude_growth() -> None:
    """The L-V / Euler trap: forward Euler spirals outward — detected, not tolerated.

    On the *same* oscillator and ``dt`` where RK4 holds ``V`` to ~1e-8, Euler's
    invariant drifts by O(1e-1), the drift is **one-signed growth** (``V_end > V₀``),
    and the prey amplitude in the run's second half exceeds the first — the spurious
    growth the gate exists to catch.
    """
    euler = _run(EulerIntegrator, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)
    rk4 = _run(Rk4Integrator, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)

    assert euler.max_v_drift > 1e-2  # measured ≈ 0.117
    assert euler.v_growth > 1e-2  # one-signed: the invariant *grows* (spiral out)
    assert euler.max_v_drift > 1000.0 * rk4.max_v_drift  # measured ≈ 6e6×
    assert euler.prey_peak_late > euler.prey_peak_early  # amplitude grows over time


def test_oscillator_run_is_non_arbitrating_and_stays_positive() -> None:
    """A "non-arbitrating" oscillator: the backstop never fires, stocks stay ≥ 0.

    For ``POOL`` prey/predator, ``rationed == 0`` over the Euler run is simultaneously
    the non-arbitrating gate *and* the proof that neither stock went negative
    (min-scaling fires exactly when a withdrawal would exceed a stock). RK4 completing
    without an ``ArbitrationError`` is the higher-order arm of the same guarantee.
    """
    euler = _run(EulerIntegrator, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)
    assert euler.rationed == 0
    assert euler.x_min > 0.0
    assert euler.y_min > 0.0

    # RK4 hard-errors on any over-draw; reaching here means it never needed to.
    rk4 = _run(Rk4Integrator, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)
    assert rk4.rationed == 0
    assert rk4.x_min > 0.0
    assert rk4.y_min > 0.0


def test_oscillator_conserves_total_carbon() -> None:
    """``x + y + R`` is exactly constant — the always-on gate enforces it every step.

    The per-step conservation gate runs inside ``step_report`` (a violation would have
    raised ``ConservationError`` mid-run), so completing the run is itself the
    assertion; this pins the closure explicitly: total carbon holds to the float floor
    for *both* schemes.
    """
    for cls in (EulerIntegrator, Rk4Integrator):
        stats = _run(cls, _P, _X0, _Y0, _DT_FINE, _STEPS_FINE)
        assert stats.max_total_drift < 1e-8  # measured ≈ 1e-12
