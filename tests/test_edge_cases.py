"""Step-5 edge-case suite (Phase 0.5) — degenerate boundary values, pinned explicitly.

Realizes the roadmap's *"zero stocks, depleted resources, negative-flow attempts,
overflow protection."* Much of the *mechanism* already exists — arbitration handles
depletion (``simcore.arbitration``), and ``Stock.__post_init__`` rejects a non-finite
amount — so this suite is deliberately scoped to the **degenerate boundary values**
those mechanisms must survive: an empty model, a stock at exactly 0, the exact-fit
seam, a strict over-draw, and an overflow. The *proportional multi-flow sag*, the
``unclamped`` skip, and registration-order independence are already covered in
``tests/test_arbitration.py`` and are **not** re-tested here.

The four pinned behaviors (Phase-0.5 plan, Step 5):

* **Empty / zero** — an empty registry over empty *or* unchanged stocks steps cleanly
  (``n -> n+1``, no-op, conserves trivially); a stock at exactly 0 with a withdrawal
  flow has the Euler backstop scale the draw to 0 (``scale_f == 0``), stays ``>= 0``,
  and conserves.
* **Depletion** — a POOL drawn to exactly empty stays at 0 (never negative) under
  Euler; the same over-draw under RK4 raises ``ArbitrationError`` (the integrator
  contract's Euler-scale / RK4-hard-error asymmetry). Plus the exact-fit seam: a
  withdrawal of *exactly* the available amount lands at 0 with **no** arbitration.
* **Negative-flow attempt** — a flow withdrawing more than available is throttled
  (Euler) / hard-errors (RK4); this is the depletion contract, re-pinned at the
  boundary (the strict-over-draw tests below).
* **Overflow protection** — a flow driving an amount past the largest finite double
  produces ``+inf``, which ``Stock.__post_init__``'s ``math.isfinite`` guard rejects
  with a loud ``ValueError`` rather than silently poisoning the ledger with ``inf``.

**No new guard was needed (test-only change).** Every reduction path
(``_reduce`` / ``_combine`` / ``_scale_factors``) feeds ``Stock`` construction
(``_shifted_stocks`` / ``_perturb``) *before* the conservation gate runs, and
``math.isfinite`` rejects ``inf`` **and** ``nan`` there — so a non-finite value from
any reduction is caught at the producing step regardless of which reduction overflowed.
Nothing in ``simcore/`` moves, so the purity / frozen-API / determinism gates stay
green by construction.

The level-*independent* (constant) ``_ConstantDrain`` below is load-bearing: a
proportional ``rate·amount`` flow tapers to 0 at an empty stock (``demand_s == 0 ->
scale_s == 1``), so the backstop would never engage and the edge would silently not be
exercised — the constant withdrawal is the synthetic over-draw the backstop exists to
catch (mirrors ``tests/test_arbitration.py``). Test-local flow dataclass per repo
convention; pure stdlib + the simcore engine; this file is under ``tests/``, outside
the purity gate.
"""

import dataclasses
import sys

import pytest

from simcore import boundary
from simcore.arbitration import ArbitrationError, min_scaling
from simcore.environment import Environment, SourceResolver
from simcore.flow import FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

# --- ids -------------------------------------------------------------------
BIO = DomainId("bio")
S = StockId("bio.s")  # the POOL under test
DST = StockId("bio.dst")  # a POOL deposit target (overflow case)
K = StockId("boundary.k")  # a boundary disposal sink
SRC = StockId("boundary.src")  # an unclamped boundary supply (never throttled)


# --- builders --------------------------------------------------------------
def _pool(sid: StockId, amount: float, quantity: Quantity = Quantity.CARBON) -> Stock:
    return Stock(
        id=sid,
        domain=BIO,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


@dataclasses.dataclass(frozen=True)
class _ConstantDrain:
    """Withdraws a fixed ``amount`` per ``dt`` from ``src`` into ``dst`` (balanced).

    A *constant* withdrawal — independent of the stock level, so it does **not**
    taper as the source empties. That is exactly the synthetic over-draw the backstop
    exists to catch (real saturating kinetics would taper to 0). ``leg == amount·dt``;
    ``Σ legs == 0`` so it conserves whatever quantity ``src``/``dst`` share.
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    amount: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        x = self.amount * dt
        return FlowResult(legs=(Leg(self.src, -x), Leg(self.dst, x)))


def _env() -> SourceResolver:
    return SourceResolver()  # empty wiring — the edge flows read no env vars


# --- empty / zero: no-op stepping ------------------------------------------
@pytest.mark.parametrize(
    "integ_cls", [EulerIntegrator, Rk4Integrator], ids=["euler", "rk4"]
)
def test_empty_registry_and_stocks_steps_cleanly(integ_cls) -> None:
    """An empty model steps cleanly: ``n -> n+1``, no events, conserves trivially.

    Reaching the assertions at all means the every-step conservation gate ran and
    passed (vacuously — no quantities present). Pinned for both schemes: RK4's
    union-of-stage-keys combine over four empty maps is also a no-op.
    """
    stocks: dict[StockId, Stock] = {}
    state = State(n=0, stocks=stocks, rng_seed=0)
    report = integ_cls(Registry([], stocks)).step_report(state, _env(), 1.0)

    assert report.state.n == 1
    assert dict(report.state.stocks) == {}
    assert report.rationed == 0
    assert report.events == ()


@pytest.mark.parametrize(
    "integ_cls", [EulerIntegrator, Rk4Integrator], ids=["euler", "rk4"]
)
def test_empty_registry_passes_stocks_through_unchanged(integ_cls) -> None:
    """No flows ⇒ stocks pass through byte-for-byte; only ``n`` advances."""
    stocks = {S: _pool(S, 3.0), K: boundary.sink(K, Quantity.CARBON, 2.0)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    report = integ_cls(Registry([], stocks)).step_report(state, _env(), 1.0)

    assert report.state.n == 1
    assert {sid: s.amount for sid, s in report.state.stocks.items()} == {S: 3.0, K: 2.0}
    assert report.rationed == 0
    assert report.events == ()


# --- zero availability: scale_f == 0 (the 0/x ratio edge) ------------------
def test_min_scaling_scales_draw_from_empty_stock_to_zero() -> None:
    """A POOL at exactly 0: ``scale_s = min(1, 0/x) = 0`` ⇒ every leg scales to 0.

    The degenerate ``available_s == 0`` edge, distinct from
    ``test_arbitration``'s nonzero-availability proportional sag. Pinned at the pure
    ``min_scaling`` level so ``scale_f == 0`` is observable directly (the integrator
    only exposes the firing count).
    """
    stocks = {S: _pool(S, 0.0), K: boundary.sink(K, Quantity.CARBON)}
    results = [FlowResult(legs=(Leg(S, -5.0), Leg(K, 5.0)))]

    scaled, fired = min_scaling(results, stocks)

    assert fired == 1
    assert all(leg.amount == 0.0 for leg in scaled[0].legs)  # scale_f == 0


def test_euler_withdrawal_from_empty_stock_stays_nonnegative_and_conserves() -> None:
    """Euler over an empty POOL: the draw scales to 0; the stock stays 0, conserved."""
    stocks = {S: _pool(S, 0.0), K: boundary.sink(K, Quantity.CARBON)}
    flows = [_ConstantDrain(FlowId("drain"), 0, S, K, 5.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry(flows, stocks)).step_report(state, _env(), 1.0)

    assert report.rationed == 1  # the backstop fired (scale_f == 0 < 1)
    assert report.state.stocks[S].amount == 0.0  # >= 0, exact — nothing moved
    assert report.state.stocks[K].amount == 0.0
    after = sum(s.amount for s in report.state.stocks.values())
    assert after == sum(s.amount for s in state.stocks.values())  # total conserved


def test_rk4_withdrawal_from_empty_stock_hard_errors() -> None:
    """RK4 over an empty POOL hard-errors (``scale_f < 1`` is not silently clamped)."""
    stocks = {S: _pool(S, 0.0), K: boundary.sink(K, Quantity.CARBON)}
    flows = [_ConstantDrain(FlowId("drain"), 0, S, K, 5.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    with pytest.raises(ArbitrationError):
        Rk4Integrator(Registry(flows, stocks)).step(state, _env(), 1.0)


# --- depletion / strict over-draw: the Euler/RK4 asymmetry -----------------
def test_euler_overdraw_depletes_pool_to_zero_without_going_negative() -> None:
    """Euler throttles a strict over-draw so the POOL lands at exactly 0, never below.

    A single constant drain of 16/dt from a POOL holding 10: demand 16 > 10, so the
    backstop scales by ``10/16 = 0.625`` (binary-exact) and the realized draw is
    exactly 10 → the pool lands at exactly 0. (The *proportional multi-flow* sag is
    covered in ``test_arbitration``; this re-pins the contract at the depletion
    boundary with a single flow.)
    """
    stocks = {S: _pool(S, 10.0), K: boundary.sink(K, Quantity.CARBON)}
    flows = [_ConstantDrain(FlowId("drain"), 0, S, K, 16.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry(flows, stocks)).step_report(state, _env(), 1.0)

    assert report.rationed == 1
    assert report.state.stocks[S].amount == 0.0
    assert report.state.stocks[S].amount >= 0.0  # the non-negativity invariant
    assert report.state.stocks[K].amount == 10.0  # the 10 from S landed in the sink
    after = sum(s.amount for s in report.state.stocks.values())
    assert after == sum(s.amount for s in state.stocks.values())  # total conserved


def test_rk4_overdraw_hard_errors_the_asymmetry() -> None:
    """The same strict over-draw under RK4 raises ``ArbitrationError`` (the asymmetry).

    Min-scaling's conservation-safety proof is single-evaluation (Euler-only); under
    RK4 a needed ``scale_f < 1`` is a hard error — positivity must come from the
    kinetics, not the backstop (the integrator contract).
    """
    stocks = {S: _pool(S, 10.0), K: boundary.sink(K, Quantity.CARBON)}
    flows = [_ConstantDrain(FlowId("drain"), 0, S, K, 16.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    with pytest.raises(ArbitrationError):
        Rk4Integrator(Registry(flows, stocks)).step(state, _env(), 1.0)


def test_euler_exact_fit_withdrawal_lands_at_zero_without_arbitrating() -> None:
    """Exact-fit seam: withdraw *exactly* the available 10 → lands at 0, no throttle.

    ``demand == available`` ⇒ ``scale_s = min(1, 10/10) = 1.0``, so the backstop does
    **not** fire (``rationed == 0``) yet the pool still lands at exactly 0 — the
    throttle / no-throttle boundary. Deliberately **Euler-only**: under RK4 a constant
    exact-fit flow hard-errors (stage 2 perturbs the pool to 5 while demand stays 10,
    so ``scale_f = 0.5 < 1``); the strict-over-draw test above carries the RK4 side of
    the asymmetry.
    """
    stocks = {S: _pool(S, 10.0), K: boundary.sink(K, Quantity.CARBON)}
    flows = [_ConstantDrain(FlowId("drain"), 0, S, K, 10.0)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry(flows, stocks)).step_report(state, _env(), 1.0)

    assert report.rationed == 0  # no arbitration
    assert report.state.stocks[S].amount == 0.0  # but still lands exactly empty


# --- overflow protection ---------------------------------------------------
def test_overflow_deposit_is_rejected_by_the_stock_finiteness_guard() -> None:
    """An amount driven past the largest finite double is rejected, not silent ``inf``.

    An *unclamped* boundary source (min-scaling skips it, so the withdrawal is never
    throttled) feeds a POOL already sitting at ``sys.float_info.max``; depositing
    another ``float_info.max`` overflows the destination to ``+inf``. The apply path
    constructs the new ``Stock``, and ``Stock.__post_init__``'s ``math.isfinite`` guard
    raises ``ValueError`` — a loud failure rather than an ``inf`` poisoning the ledger.
    The ``match`` ties the assertion to *that* guard (not an unrelated ``ValueError``).

    No new guard is needed: the would-be gap ("overflow inside a reduction before
    ``Stock`` construction") does not bite — the ``inf`` (or a ``nan`` from a
    contrived ``inf − inf``) from any reduction flows into ``Stock`` construction,
    which rejects it before the conservation gate runs.
    """
    big = sys.float_info.max
    src = boundary.source(
        SRC, Quantity.CARBON, amount=0.0
    )  # unclamped: may go negative
    stocks = {SRC: src, DST: _pool(DST, big)}
    flows = [_ConstantDrain(FlowId("flood"), 0, SRC, DST, big)]
    state = State(n=0, stocks=stocks, rng_seed=0)

    with pytest.raises(ValueError, match="not finite"):
        EulerIntegrator(Registry(flows, stocks)).step(state, _env(), 1.0)
