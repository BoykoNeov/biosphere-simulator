"""Step-1 convergence / timestep-sensitivity gate (Phase 0.5).

Generalizes the ad-hoc halving-ratio checks already in the suite (``test_integrator``'s
decay order test; ``test_oscillator``'s V-drift order) into a *systematic order-
measurement harness*: run a scheme over a geometric ``dt`` ladder, fit the observed
order ``p`` in ``error ≈ C·dt**p`` (``lab.convergence.fit_order`` — a log-log least-
squares slope over the whole ladder), and gate ``p`` against the scheme's theoretical
order. This is the reusable harness the RK45 (Step 2) and multi-rate (Step 3) order
checks plug into; here it is exercised on **two schemes** (Euler→1, RK4→4) to prove
reuse across schemes.

The reference is analytic exponential decay (``dy/dt = -λy``, exact ``y0·e^{-λt}``) —
the same first-order, ``dt``-linear, conservation-clean scenario ``test_integrator``
uses, run as a POOL ``src`` → boundary ``sink`` so it is free of extinction/over-draw.
Flow defined test-locally (the repo's convention — mirrors ``_DecayFlow`` in
``test_integrator`` and the test-local flows in ``test_oscillator``).
"""

import dataclasses
import math

import pytest

from lab.convergence import convergence_order, fit_order
from simcore import boundary
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

SRC = StockId("decay.src")
SINK = StockId("boundary.decay_sink")


@dataclasses.dataclass(frozen=True)
class _DecayFlow:
    """``src -> sink`` (boundary) at first-order ``rate`` — ``dt``-linear, balanced."""

    id: FlowId
    priority: int
    src: StockId
    sink: StockId
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        moved = self.rate * snapshot.stocks[self.src].amount * dt
        return FlowResult(legs=(Leg(self.src, -moved), Leg(self.sink, moved)))


def _decay_endpoint_error(
    integrator_cls: type, dt: float, lam: float, t_end: float
) -> float:
    """Run decay to ``t_end`` at step ``dt``; return ``|y_N − y0·e^{-λt}|``."""
    a0 = 1.0
    steps = int(round(t_end / dt))
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
    integrator = integrator_cls(reg)
    env = SourceResolver()
    for _ in range(steps):
        state = integrator.step(state, env, dt)
    exact = a0 * math.exp(-lam * t_end)
    return abs(state.stocks[SRC].amount - exact)


# Geometric (halving) dt ladder. ``t_end/dt`` is an integer at every rung, and the
# finest RK4 error stays well above the f64 round-off floor (~1e-10 ≫ 1e-16), so the
# fitted slope measures truncation error, not numerical noise.
_LAM, _T_END = 1.0, 1.0
_DTS = (0.1, 0.05, 0.025, 0.0125)


@pytest.mark.parametrize(
    "integrator_cls,lo,hi",
    [
        (EulerIntegrator, 0.8, 1.3),  # forward Euler is 1st-order
        (Rk4Integrator, 3.6, 4.3),  # classical RK4 is 4th-order
    ],
)
def test_observed_order_matches_scheme(
    integrator_cls: type, lo: float, hi: float
) -> None:
    """The fitted order over the dt ladder matches the scheme (Euler→1, RK4→4)."""
    errors = [_decay_endpoint_error(integrator_cls, dt, _LAM, _T_END) for dt in _DTS]
    # Error decreases monotonically as dt → 0 (the ladder stays above the round-off
    # floor, so finer is always strictly better — the "results converge" property).
    assert all(b < a for a, b in zip(errors[:-1], errors[1:], strict=True))
    p = fit_order(_DTS, errors)
    assert lo < p < hi


def test_convergence_order_helper_matches_fit_order() -> None:
    """``convergence_order`` (the convenience wrapper) equals ``fit_order`` exactly."""
    errors = [_decay_endpoint_error(Rk4Integrator, dt, _LAM, _T_END) for dt in _DTS]
    p_helper = convergence_order(
        lambda dt: _decay_endpoint_error(Rk4Integrator, dt, _LAM, _T_END), _DTS
    )
    assert p_helper == fit_order(_DTS, errors)


# --- the fitter itself, pinned on synthetic data (discriminating control) ----
def test_fit_order_recovers_known_power_law() -> None:
    """On exact ``error = C·dt**p`` data the fit recovers ``p`` to the float floor.

    This pins the estimator independently of any integrator: if the integrator order
    tests above fail, this isolates whether the fault is the scheme or the fitter.
    """
    dts = (0.1, 0.05, 0.025, 0.0125)
    for p_true in (1.0, 2.0, 4.0):
        errors = [3.0 * dt**p_true for dt in dts]
        assert fit_order(dts, errors) == pytest.approx(p_true)


def test_fit_order_rejects_bad_input() -> None:
    """Length mismatch, too few rungs, and non-positive dt/error all raise."""
    with pytest.raises(ValueError, match="same length"):
        fit_order((0.1, 0.05), (1.0,))
    with pytest.raises(ValueError, match="at least two"):
        fit_order((0.1,), (1.0,))
    with pytest.raises(ValueError, match="strictly positive"):
        fit_order((0.1, 0.0), (1.0, 1.0))
    with pytest.raises(ValueError, match="round-off floor"):
        fit_order((0.1, 0.05), (1.0, 0.0))
