"""Step-4 100k-step stability gate (Phase 0.5) — long-run boundedness, per N6.

Runs a **dissipative relaxation to a stable fixed point** for 100k+ steps under
Euler, RK4, and one multi-rate config, and asserts what is genuinely *new* at this
length (everything else is already covered by the order/conservation tests):

* **finite** — no `NaN`/`Inf`/overflow. *Not* re-instrumented per step: every step
  constructs fresh `Stock`s and `Stock.__post_init__` rejects a non-finite `amount`
  at the step that produces it, so a non-finite value raises *during* the run. The
  genuinely new failure mode this gate adds is **finite-but-diverging** — caught by
  the upper bound below, which `Stock` does not catch.
* **bounded** — the state relaxes, it does not blow up. The scenario is closed
  (total carbon `T` conserved), so every amount must stay in `[−atol, T+atol]`; we
  track the running `max|amount|` over the whole run and assert it never exceeds `T`.
* **settled / no secular drift** — the per-step change over the *final* step is below
  a tiny epsilon: the system reached its steady fixed point and stays there (this is
  why N6 demands a dissipative system, not an oscillator — an oscillator drifts
  secularly under RK4 over 100k and would conflate drift with the gate).
* **non-arbitrating** — `rationed == 0` summed over the whole run. This is the real
  check for the **Euler** config (its backstop could silently scale an over-draw);
  RK4 and RK4-multi-rate *hard-error* on an over-draw (`ArbitrationError`), so for
  them simply completing the run is the non-arbitration assertion.
* **conservation is length-independent** — the always-on conservation gate
  (`simcore.conservation`, decision #13) runs *inside every step* (single-rate
  `_finalize`; the multi-rate composite boundary). Completing 100k steps without a
  `ConservationError` is therefore the assertion — it makes the tolerance's
  length-independence (argued analytically in `conservation.assert_conserved`'s
  docstring) **empirical**.

**The scenario — a closed reversible exchange A ⇌ B.** Two CARBON `POOL` stocks; a
*forward* flow `A → B` at rate `k_f·A` and a *backward* flow `B → A` at rate `k_b·B`.
The system is closed (`A + B = T` constant), so it conserves carbon with **no**
boundary reservoir, and it is linear with a single real-negative eigenvalue
`−(k_f + k_b)` — a monotone, non-oscillating decay to the fixed point
`A* = k_b·T/(k_f + k_b)` (decision N6: relaxation to a *stable* fixed point, no
secular drift). With `k_f=0.7, k_b=0.3, dt=0.1` the Euler amplification factor is
`1 + λ·dt = 0.9` (`|0.9| < 1` → stable), `A* = 0.3`, `B* = 0.7`. The amounts never
approach an over-draw (the kinetics are self-limiting), so no config ever arbitrates.

**Runtime / gating.** The whole module is marked `slow` (the 100k multi-rate run is
the cost). The mark is an opt-*out* handle only — a plain `uv run pytest` still runs
the gate (so it is not theater); a fast iteration loop deselects it with
`-m "not slow"`, or shrinks it with `STATION_STABILITY_STEPS=2000 uv run pytest`. The
multi-rate config uses `n_sub=2` (split order is pinned by `test_multirate.py`, so
`n_sub` does not change what *this* gate checks — it only trims runtime).

Test-local flow dataclass, mirroring the `_Cascade` / oscillator precedent. Pure
stdlib + the simcore engine; this file is under `tests/`, outside the purity gate.
"""

import dataclasses
import os

import pytest

from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator, StepReport
from simcore.multirate import multirate_step
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# The 100k gate is the cost; mark the whole module slow so `-m "not slow"` skips it
# while a bare `uv run pytest` still runs it (the mark is opt-OUT, not deselect-by-
# default — the latter would make the gate theater with no CI running `-m slow`).
pytestmark = pytest.mark.slow

# --- ids + exchange parameters ---------------------------------------------
XCHG = DomainId("xchg")
A = StockId("xchg.a")
B = StockId("xchg.b")

A0, B0 = 1.0, 0.0
K_F, K_B = 0.7, 0.3  # forward A→B, backward B→A; λ = −(k_f+k_b) = −1 (Euler factor 0.9)
DT = 0.1
TOTAL = A0 + B0  # 1.0 — conserved (closed system, no boundary reservoir)
A_STAR = K_B * TOTAL / (K_F + K_B)  # 0.3 — the analytic stable fixed point
N_SUB = 2  # multi-rate sub-steps; order is pinned elsewhere, so this only trims runtime

# Default 100k (the exit-criterion length); env-overridable for a fast iteration loop.
STEPS = int(os.environ.get("STATION_STABILITY_STEPS", "100000"))


# --- test-local flow -------------------------------------------------------
@dataclasses.dataclass(frozen=True)
class _Exchange:
    """One direction of A⇌B: ``src → dst`` at first-order ``rate``.

    ``leg == dt·rate·src``, balanced (``Σ legs == 0``). Two of these — A→B at ``k_f``
    and B→A at ``k_b`` — make a closed, dissipative linear system that relaxes
    monotonically to the fixed point, with no oscillation or secular drift (N6).
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.dst, moved)))


# --- builders --------------------------------------------------------------
def _pool(sid: StockId, amount: float) -> Stock:
    return Stock(
        id=sid,
        domain=XCHG,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _stocks() -> dict[StockId, Stock]:
    return {A: _pool(A, A0), B: _pool(B, B0)}


def _forward() -> _Exchange:
    return _Exchange(FlowId("xchg.fwd"), 0, A, B, K_F)


def _backward() -> _Exchange:
    return _Exchange(FlowId("xchg.bwd"), 0, B, A, K_B)


# --- per-scheme steppers ----------------------------------------------------
# Each factory takes the run's stock dict (so its Registry's domain index is built
# from the same stocks) and returns a `state -> StepReport` callable, giving the
# runner one uniform shape across single-rate and multi-rate.
def _euler(stocks: dict[StockId, Stock]):
    integ = EulerIntegrator(Registry([_forward(), _backward()], stocks))
    env = SourceResolver()
    return lambda s: integ.step_report(s, env, DT)


def _rk4(stocks: dict[StockId, Stock]):
    integ = Rk4Integrator(Registry([_forward(), _backward()], stocks))
    env = SourceResolver()
    return lambda s: integ.step_report(s, env, DT)


def _multirate(stocks: dict[StockId, Stock]):
    # Nominal partition (k_f, k_b are not stiff-separated): the point is to exercise
    # the Strang+RK4 driver over a long run, not a real efficiency scenario.
    slow = Rk4Integrator(Registry([_backward()], stocks))
    fast = Rk4Integrator(Registry([_forward()], stocks))
    env = SourceResolver()
    return lambda s: multirate_step(slow, fast, s, env, DT, N_SUB)


def _run(make_stepper) -> tuple[State, float, int, float]:
    """Drive `make_stepper`'s stepper for ``STEPS`` steps from a fresh A⇌B state.

    Returns ``(final_state, max_abs, total_rationed, last_delta)``:
    ``max_abs`` is ``max|amount|`` over every stock across the whole run (incl. the
    initial state), ``total_rationed`` sums the Euler-backstop firing count, and
    ``last_delta`` is the max per-stock ``|Δamount|`` over the **final** step (the
    settled-ness probe). The conservation gate runs inside each step, so completing
    this loop *is* the per-step conservation assertion (decision #13).
    """
    stocks = _stocks()
    state = State(n=0, stocks=stocks, rng_seed=0)
    step = make_stepper(stocks)
    max_abs = max(abs(s.amount) for s in state.stocks.values())
    total_rationed = 0
    last_delta = 0.0
    for _ in range(STEPS):
        report: StepReport = step(state)
        nxt = report.state
        total_rationed += report.rationed
        last_delta = max(
            abs(nxt.stocks[sid].amount - state.stocks[sid].amount) for sid in nxt.stocks
        )
        max_abs = max(max_abs, max(abs(s.amount) for s in nxt.stocks.values()))
        state = nxt
    return state, max_abs, total_rationed, last_delta


@pytest.mark.parametrize(
    "make_stepper",
    [
        pytest.param(_euler, id="euler"),
        pytest.param(_rk4, id="rk4"),
        pytest.param(_multirate, id="multirate-strang-rk4"),
    ],
)
def test_100k_step_stability(make_stepper) -> None:
    """100k+ steps stay finite, bounded, settled, non-arbitrating, and conserving.

    The conservation gate fires inside every step (single-rate `_finalize`, or the
    multi-rate composite boundary), so reaching this line at all means it held for
    every one of `STEPS` steps — the length-independence claim made empirical.
    """
    state, max_abs, total_rationed, last_delta = _run(make_stepper)

    # Bounded: no finite-but-diverging amount. (NaN/Inf/overflow would already have
    # raised in `Stock.__post_init__` at the producing step; this catches divergence,
    # which `Stock` does not.) Closed system ⇒ every amount lies in [−atol, T+atol].
    assert max_abs <= TOTAL + 1e-9

    # Settled to a stable fixed point: the final step barely moved — no secular drift.
    assert last_delta <= 1e-9

    # Relaxed to the neighborhood of the analytic fixed point A*=0.3 (loose band: the
    # multi-rate Strang split sits at its own O(dt²)-shifted fixed point, not exactly
    # A*; single-rate Euler/RK4 reach A* to machine precision — far inside this band).
    assert abs(state.stocks[A].amount - A_STAR) <= 1e-2

    # Non-arbitrating over the whole run. For Euler this is the live check (its
    # backstop could scale a draw); for RK4 / RK4-multi-rate an over-draw would have
    # hard-errored, so completing already implies it — asserting 0 documents intent.
    assert total_rationed == 0

    # Conservation, pinned explicitly at the end (belt-and-suspenders beyond the
    # per-step gate): total carbon equals the closed-system total T.
    total = sum(s.amount for s in state.stocks.values())
    assert abs(total - TOTAL) <= 1e-9
