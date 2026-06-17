"""Step-2 gate (Phase 0.5): the adaptive Dormand–Prince RK45 validation oracle.

Exercises ``lab.rk45`` per the Step-2 test plan (N1): the oracle honors its requested
``atol``/``rtol`` on analytic decay, its step size *adapts*, it conserves mass, and —
used as a reference — fixed-step RK4 converges *toward the RK45 trajectory* on a
non-analytic (Lotka–Volterra) scenario, enriching Step 1 beyond the analytic cases.

Two **discriminating controls** guard the thing most likely to be wrong — a transcribed
Butcher-tableau coefficient — because tolerance-honoring alone would not catch it (a
subtly-wrong method can still honor a tolerance by taking more steps), mirroring how
Step 1 pinned ``fit_order`` on synthetic data:
  * a *static* consistency check (each ``A`` row sums to its node ``c``; ``ΣB == 1``;
    ``ΣB_STAR == 1``), and
  * an *empirical* one — the embedded error estimate is **5th-order** in ``dt``, fit
    via the Step-1 ``fit_order`` harness over a single-step ``dt`` ladder.

Per N1 the oracle breaks the integer clock (#14) by design and is **not** added to the
determinism / bit-identical-across-ports gates. Flows are defined test-locally (the
repo convention — mirrors ``_DecayFlow`` in ``test_integrator`` and the LV flows in
``test_oscillator``).
"""

import dataclasses
import math

import pytest

from lab.convergence import fit_order
from lab.rk45 import _A, _B, _B_STAR, _C, _rk45_step, rk45_trajectory
from simcore import boundary
from simcore.conservation import assert_conserved
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- analytic decay scenario (src -> boundary sink, exact y0·e^{-λt}) --------
SRC = StockId("decay.src")
SINK = StockId("boundary.decay_sink")


@dataclasses.dataclass(frozen=True)
class _DecayFlow:
    """``src -> sink`` (boundary) at first-order ``rate`` — dt-linear, balanced."""

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.sink, moved)))


def _decay_scenario(a0: float, lam: float) -> tuple[Registry, State, SourceResolver]:
    src = Stock(
        id=SRC,
        domain=DomainId("decay"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=a0,
        kind=StockKind.POOL,
    )
    stocks = {SRC: src, SINK: boundary.sink(SINK, Quantity.CARBON)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_DecayFlow(FlowId("decay"), 0, SRC, SINK, lam)], stocks)
    return reg, state, SourceResolver()


# --- non-analytic Lotka–Volterra scenario (mass-conserving, per test_oscillator) ---
PREY = StockId("osc.prey")
PRED = StockId("osc.pred")
RES = StockId("boundary.osc_reservoir")
_RES0 = 1000.0


@dataclasses.dataclass(frozen=True)
class _PreyBirth:
    """``reservoir -> prey`` at ``α·prey·dt`` — prey grow by drawing boundary carbon."""

    id: FlowId
    priority: int
    alpha: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        grown = self.alpha * snapshot.stocks[PREY].amount * dt
        return FlowResult(legs=(Leg(RES, -grown), Leg(PREY, grown)))


@dataclasses.dataclass(frozen=True)
class _Predation:
    """``prey -> predator`` (+ inefficiency ``prey -> reservoir``), rate ``β·x·y``."""

    id: FlowId
    priority: int
    beta: float
    delta: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        xy = snapshot.stocks[PREY].amount * snapshot.stocks[PRED].amount * dt
        eaten = self.beta * xy
        assimilated = self.delta * xy
        return FlowResult(
            legs=(
                Leg(PREY, -eaten),
                Leg(PRED, assimilated),
                Leg(RES, eaten - assimilated),  # (β−δ)·x·y·dt conversion inefficiency
            )
        )


@dataclasses.dataclass(frozen=True)
class _PredatorDeath:
    """``predator -> reservoir`` at ``γ·predator·dt`` — predators return carbon to R."""

    id: FlowId
    priority: int
    gamma: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        died = self.gamma * snapshot.stocks[PRED].amount * dt
        return FlowResult(legs=(Leg(PRED, -died), Leg(RES, died)))


# Fixed point (1,1), started off-orbit at (1, 0.5) — the test_oscillator scenario.
_ALPHA, _BETA, _DELTA, _GAMMA = 1.0, 1.0, 0.5, 0.5
_X0, _Y0 = 1.0, 0.5


def _lv_scenario() -> tuple[Registry, State, SourceResolver]:
    carbon = canonical_unit(Quantity.CARBON)

    def _pool(sid: StockId, amount: float) -> Stock:
        return Stock(
            id=sid,
            domain=DomainId("osc"),
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=amount,
            kind=StockKind.POOL,
        )

    stocks = {
        PREY: _pool(PREY, _X0),
        PRED: _pool(PRED, _Y0),
        RES: boundary.source(RES, Quantity.CARBON, _RES0),
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows = [
        _PreyBirth(FlowId("osc.birth"), 0, _ALPHA),
        _Predation(FlowId("osc.predation"), 0, _BETA, _DELTA),
        _PredatorDeath(FlowId("osc.death"), 0, _GAMMA),
    ]
    return Registry(flows, stocks), state, SourceResolver()


# === discriminating control #1: the Butcher tableau is internally consistent ===
def test_dopri5_tableau_is_consistent() -> None:
    """Each A-row sums to its node ``c``; both weight rows sum to 1 (order condition).

    A transcribed coefficient is this module's likeliest defect; these are the cheap
    static invariants that catch most typos before any integration runs.
    """
    for i, row in enumerate(_A):
        assert sum(row) == pytest.approx(_C[i], abs=1e-14), f"A row {i} ≠ c[{i}]"
    assert sum(_B) == pytest.approx(1.0, abs=1e-14)
    assert sum(_B_STAR) == pytest.approx(1.0, abs=1e-14)
    # The 5th-order weights are exactly the last A-row (the FSAL structure of DOPRI5).
    assert _B[:-1] == _A[-1]


# === discriminating control #2: the embedded error estimate is 5th-order =======
def test_embedded_error_estimate_is_fifth_order() -> None:
    """A single RK45 step's error estimate scales as ``dt⁵`` (pins the tableau order).

    Mirrors Step-1's synthetic ``fit_order`` pin: take *one* DOPRI step on decay from a
    fixed state over a geometric ``dt`` ladder and fit the slope of ``|error|`` vs
    ``dt``. The embedded 4(5) estimate is O(dt⁵), so the slope must be ≈5 — independent
    of the adaptive driver. (The ladder stays above the round-off floor: the finest
    estimate is ~1e-11 ≫ 1e-16.)
    """
    reg, state, env = _decay_scenario(a0=2.0, lam=1.0)
    y = {sid: state.stocks[sid].amount for sid in state.stocks}
    dts = (0.2, 0.1, 0.05, 0.025)
    errors = []
    for h in dts:
        _, error = _rk45_step(reg, state, env, y, h)
        errors.append(max(abs(e) for e in error.values()))
    print(f"\nembedded-error ladder: {list(zip(dts, errors, strict=True))}")
    # Strictly decreasing as dt → 0 (above the floor → finer is genuinely better).
    assert all(b < a for a, b in zip(errors[:-1], errors[1:], strict=True))
    p = fit_order(dts, errors)
    print(f"embedded-error order p = {p}")
    assert 4.6 < p < 5.4


# === tolerance honoring on analytic decay ======================================
def _decay_endpoint_error(a0: float, lam: float, t_end: float, tol: float) -> float:
    reg, state, env = _decay_scenario(a0, lam)
    traj = rk45_trajectory(reg, state, env, t_end, atol=tol, rtol=tol, dt0=0.1)
    exact = a0 * math.exp(-lam * t_end)
    return abs(traj.final()[SRC] - exact)


def test_rk45_honors_tolerance_on_analytic_decay() -> None:
    """Endpoint error matches ``y0·e^{-λt}`` within the requested tolerance, and a
    tighter tolerance yields a strictly smaller error (the control actually controls).
    """
    a0, lam, t_end = 2.0, 1.0, 2.0
    loose = _decay_endpoint_error(a0, lam, t_end, 1e-6)
    tight = _decay_endpoint_error(a0, lam, t_end, 1e-9)
    print(f"\ndecay endpoint error: tol=1e-6 -> {loose}, tol=1e-9 -> {tight}")
    # Honors the tolerance (generous multiple — global vs the per-step local control —
    # but still fails a method that ignores its tolerance).
    assert loose < 1e-6 * 1000.0
    assert tight < 1e-9 * 1000.0
    # Tightening the tolerance tightens the error — the discriminating direction.
    assert tight < loose


# === step-size adaptation (demonstrated on the LV orbit) =======================
def test_rk45_step_size_adapts_over_the_lv_orbit() -> None:
    """The controller varies ``dt`` widely over an LV orbit (fast spikes, slow troughs).

    Adaptation is shown on Lotka–Volterra rather than decay: decay *does* adapt (a few
    ×), but an LV orbit drives an order-of-magnitude swing in the admissible step,
    making the gate robust. (The plan groups adaptation under the decay case; this is
    the noted regrouping — same property, a sharper demonstrator.)
    """
    reg, state, env = _lv_scenario()
    traj = rk45_trajectory(reg, state, env, 12.0, atol=1e-8, rtol=1e-8, dt0=0.1)
    sizes = traj.step_sizes()
    ratio = max(sizes) / min(sizes)
    print(
        f"\nLV step sizes: n_accepted={traj.n_accepted}, n_rejected={traj.n_rejected}, "
        f"min={min(sizes)}, max={max(sizes)}, max/min={ratio}"
    )
    assert traj.n_accepted > 10  # actually integrated, not one giant step
    assert ratio > 3.0  # the step genuinely adapts (measured ≫ this)


# === mass conservation (verified at the call site, dt-independent) =============
def test_rk45_conserves_mass_over_the_lv_run() -> None:
    """Total carbon is conserved across the whole reference trajectory.

    RK45 advances a linear combination of *balanced* derivatives, so every sample
    conserves mass to round-off. Verified by rebuilding each sample as a ``State`` and
    running the core's ``assert_conserved`` between consecutive samples (the gate is
    ``dt``-independent — it compares total mass before/after), keeping the assertion out
    of the oracle itself.
    """
    reg, state, env = _lv_scenario()
    traj = rk45_trajectory(reg, state, env, 12.0, atol=1e-9, rtol=1e-9, dt0=0.1)
    total0 = _X0 + _Y0 + _RES0
    max_drift = 0.0
    for i in range(len(traj.samples) - 1):
        # A violation raises ConservationError mid-loop; completing IS the assertion.
        assert_conserved(traj.state_at(i), traj.state_at(i + 1))
        s = traj.samples[i + 1]
        max_drift = max(max_drift, abs(s[PREY] + s[PRED] + s[RES] - total0))
    print(f"\nLV reference max total-carbon drift = {max_drift}")
    assert max_drift < 1e-9


# === enrichment: fixed-step RK4 converges toward the RK45 reference (non-analytic) ===
def test_rk4_converges_toward_rk45_reference_on_lv() -> None:
    """On LV (no closed form), RK4's error vs the RK45 reference falls at ~4th order.

    This extends Step 1's convergence study to a *non-analytic* scenario: the RK45
    oracle (very tight tolerance) is the stand-in for the unknown exact solution. Two
    references at adjacent tolerances bound the reference's own accuracy; the finest RK4
    rung is kept ≳1e3× above that floor so the fitted order measures RK4 truncation
    error, not reference noise (printed for inspection).
    """
    t_end = 10.0
    reg, state, env = _lv_scenario()
    ref = rk45_trajectory(reg, state, env, t_end, atol=1e-12, rtol=1e-12, dt0=0.05)
    ref_tighter = rk45_trajectory(
        reg, state, env, t_end, atol=1e-13, rtol=1e-13, dt0=0.05
    )
    ref_x, ref_y = ref.final()[PREY], ref.final()[PRED]
    # The reference's self-consistency floor: how much it moves under a 10× tighter tol.
    ref_floor = math.dist(
        (ref_x, ref_y), (ref_tighter.final()[PREY], ref_tighter.final()[PRED])
    )

    def _rk4_error(dt: float) -> float:
        r, s, e = _lv_scenario()
        integ = Rk4Integrator(r)
        steps = int(round(t_end / dt))
        for _ in range(steps):
            s = integ.step(s, e, dt)
        return math.dist((s.stocks[PREY].amount, s.stocks[PRED].amount), (ref_x, ref_y))

    # Coarse ladder on purpose: it keeps the finest RK4 error ≫ the reference floor
    # (~1e3×+), where the fit measures truncation error rather than reference noise —
    # finer rungs would dip toward the reference's own ~1e-12 accuracy.
    dts = (0.2, 0.1, 0.05, 0.025)
    errors = [_rk4_error(dt) for dt in dts]
    print(f"\nRK4-vs-RK45 errors: {list(zip(dts, errors, strict=True))}")
    print(
        f"reference self-consistency floor = {ref_floor}, finest RK4 err = {errors[-1]}"
    )
    # Monotone convergence, and the finest rung well above the reference floor.
    assert all(b < a for a, b in zip(errors[:-1], errors[1:], strict=True))
    assert errors[-1] > 1000.0 * ref_floor
    p = fit_order(dts, errors)
    print(f"RK4-vs-RK45 fitted order p = {p}")
    assert 3.5 < p < 4.5


# === input validation ==========================================================
def test_rk45_rejects_bad_input() -> None:
    """Non-positive ``t_end`` / ``atol`` / ``rtol`` / ``dt0`` each raise ValueError."""
    reg, state, env = _decay_scenario(1.0, 1.0)

    with pytest.raises(ValueError, match="t_end must be"):
        rk45_trajectory(reg, state, env, 0.0, atol=1e-9, rtol=1e-9, dt0=0.1)
    with pytest.raises(ValueError, match="atol and rtol"):
        rk45_trajectory(reg, state, env, 1.0, atol=0.0, rtol=1e-9, dt0=0.1)
    with pytest.raises(ValueError, match="atol and rtol"):
        rk45_trajectory(reg, state, env, 1.0, atol=1e-9, rtol=0.0, dt0=0.1)
    with pytest.raises(ValueError, match="dt0 must be"):
        rk45_trajectory(reg, state, env, 1.0, atol=1e-9, rtol=1e-9, dt0=0.0)
