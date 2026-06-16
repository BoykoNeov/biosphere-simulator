"""Step-8 tests: the conservation ledger + the every-step balance gate.

The augmented (modeled + boundary) system is closed, so per asserted quantity the
total mass across all stocks is unchanged each step (decision #13). ``compute_ledger``
decomposes the per-step change into boundary (Input/Output) and stored (ΔStored)
deltas; ``assert_conserved`` is the every-step engine gate, wired into the integrator
so a violation raises ``ConservationError``. ``ENERGY`` is balance-exempt (#8):
reported as a diagnostic, never asserted.

Built test-first against the "Step 8 design" section of the plan.
"""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore import boundary
from simcore.conservation import QuantityLedger, assert_conserved, compute_ledger
from simcore.environment import Environment, SourceResolver
from simcore.flow import ConservationError, FlowResult, Leg
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

A = StockId("bio.a")
B = StockId("bio.b")
C = StockId("bio.c")
D = StockId("bio.d")
S = StockId("bio.s")
P = StockId("bio.pop")
E = StockId("bio.e")
SINK = StockId("boundary.sink")
K1 = StockId("boundary.k1")
K2 = StockId("boundary.k2")


def _pool(sid: StockId, amount: float, quantity: Quantity = Quantity.CARBON) -> Stock:
    return Stock(
        id=sid,
        domain=DomainId("bio"),
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
    )


def _population(amount: float, threshold: float) -> Stock:
    return Stock(
        id=P,
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=threshold,
    )


@dataclasses.dataclass(frozen=True)
class _PairFlow:
    """Withdraws ``withdraw·dt`` from ``src`` and deposits ``deposit·dt`` into ``dst``.

    A *constant* (level-independent) transfer — dt-linear, so valid under RK4. When
    ``withdraw == deposit`` it is balanced (the same quantity in/out); set them unequal
    to synthesize an unbalanced "engine bug" for the gate to catch.
    """

    id: FlowId
    priority: int
    src: StockId
    dst: StockId
    withdraw: float
    deposit: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        return FlowResult(
            legs=(Leg(self.src, -self.withdraw * dt), Leg(self.dst, self.deposit * dt))
        )


def _by_quantity(before: State, after: State) -> dict[Quantity, QuantityLedger]:
    return {ql.quantity: ql for ql in compute_ledger(before, after)}


# --- balanced step conserves (both schemes) --------------------------------
@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_balanced_step_conserves_and_residual_is_zero(integrator_cls: type) -> None:
    stocks = {
        A: _pool(A, 100.0),
        B: _pool(B, 10.0),
        SINK: boundary.sink(SINK, Quantity.CARBON),
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry(
        [
            _PairFlow(FlowId("transfer"), 0, A, B, 5.0, 5.0),
            _PairFlow(FlowId("harvest"), 0, B, SINK, 2.0, 2.0),
        ],
        stocks,
    )

    # The gate is always-on inside the integrator: a non-raising step is itself the
    # assertion that the step conserved.
    nxt = integrator_cls(reg).step(state, SourceResolver(), 0.5)

    carbon = _by_quantity(state, nxt)[Quantity.CARBON]
    assert carbon.residual == pytest.approx(0.0, abs=1e-12)
    # the decomposition is exact by construction.
    assert carbon.boundary_delta + carbon.stored_delta == carbon.residual


# --- function-level pass / fail --------------------------------------------
def test_assert_conserved_passes_on_balanced_states() -> None:
    before = State(0, {A: _pool(A, 10.0), B: _pool(B, 5.0)}, 0)
    after = State(1, {A: _pool(A, 7.0), B: _pool(B, 8.0)}, 0)  # −3 / +3, balanced
    assert_conserved(before, after)  # must not raise


def test_assert_conserved_raises_on_carbon_imbalance() -> None:
    before = State(0, {A: _pool(A, 10.0), B: _pool(B, 5.0)}, 0)
    after = State(1, {A: _pool(A, 7.0), B: _pool(B, 5.0)}, 0)  # −3 vanishes
    with pytest.raises(ConservationError, match="CARBON"):
        assert_conserved(before, after)


# --- the gate catches a bug, isolated from arbitration/extinction/ref-int ---
@pytest.mark.parametrize("integrator_cls", [EulerIntegrator, Rk4Integrator])
def test_integrator_step_raises_on_unbalanced_flow(integrator_cls: type) -> None:
    # −5 from A, +3 to B: both AMPLE carbon POOLs, so no over-draw (arbitration can't
    # pre-empt), no POPULATION (extinction can't), real stocks (referential integrity
    # can't) — the only thing that can fire is the every-step conservation gate.
    stocks = {A: _pool(A, 1000.0), B: _pool(B, 1000.0)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_PairFlow(FlowId("leak"), 0, A, B, 5.0, 3.0)], stocks)
    with pytest.raises(ConservationError):
        integrator_cls(reg).step(state, SourceResolver(), 1.0)


# --- ENERGY is exempt from the assertion but reported as a diagnostic -------
def test_energy_only_imbalance_is_not_asserted_but_reported() -> None:
    before = State(0, {E: _pool(E, 10.0, Quantity.ENERGY)}, 0)
    after = State(1, {E: _pool(E, 25.0, Quantity.ENERGY)}, 0)  # +15 from nowhere
    assert_conserved(before, after)  # ENERGY exempt (#8) → must not raise
    energy = _by_quantity(before, after)[Quantity.ENERGY]
    assert energy.residual == pytest.approx(15.0)


# --- conserves through arbitration (throttled Euler) ------------------------
def test_conserves_under_euler_arbitration() -> None:
    # Two flows over-draw S (demand 16 > 10); min-scaling throttles, and the always-on
    # gate still passes — whole-flow scaling preserves balance (the suite table's
    # "incl. arbitration events").
    stocks = {
        S: _pool(S, 10.0),
        K1: boundary.sink(K1, Quantity.CARBON),
        K2: boundary.sink(K2, Quantity.CARBON),
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry(
        [
            _PairFlow(FlowId("drain_1"), 0, S, K1, 8.0, 8.0),
            _PairFlow(FlowId("drain_2"), 0, S, K2, 8.0, 8.0),
        ],
        stocks,
    )

    report = EulerIntegrator(reg).step_report(state, SourceResolver(), 1.0)

    assert report.rationed == 2  # arbitration fired ...
    carbon = _by_quantity(state, report.state)[Quantity.CARBON]
    assert carbon.residual == pytest.approx(0.0, abs=1e-12)  # ... and mass conserved


# --- conserves across extinction (residual lands in the loss-sink) ----------
def test_conserves_across_extinction() -> None:
    ls = boundary.loss_sink(Quantity.CARBON)
    stocks = {P: _population(0.3, threshold=0.5), ls.id: ls}
    state = State(n=0, stocks=stocks, rng_seed=0)

    report = EulerIntegrator(Registry([], stocks)).step_report(
        state, SourceResolver(), 1.0
    )

    assert len(report.events) == 1  # extinction happened ...
    carbon = _by_quantity(state, report.state)[Quantity.CARBON]
    assert carbon.residual == pytest.approx(0.0, abs=1e-12)  # ... and conserved
    assert carbon.boundary_delta == pytest.approx(0.3)  # loss-sink gained the residual
    assert carbon.stored_delta == pytest.approx(-0.3)  # the population lost it


# --- ledger decomposition separates boundary (I/O) from stored (ΔStored) ----
def test_ledger_decomposition_separates_boundary_and_stored() -> None:
    # A modeled POOL → boundary sink (a Harvest-shaped flow): stored loses, boundary
    # gains, residual ~0.
    stocks = {A: _pool(A, 100.0), SINK: boundary.sink(SINK, Quantity.CARBON)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_PairFlow(FlowId("harvest"), 0, A, SINK, 10.0, 10.0)], stocks)

    nxt = EulerIntegrator(reg).step(state, SourceResolver(), 1.0)

    carbon = _by_quantity(state, nxt)[Quantity.CARBON]
    assert carbon.stored_delta == pytest.approx(-10.0)
    assert carbon.boundary_delta == pytest.approx(10.0)
    assert carbon.residual == pytest.approx(0.0, abs=1e-12)


# --- the relative tolerance term is exercised, not just atol ----------------
def test_tolerance_relative_term_is_applied() -> None:
    # Residual 5e-4 against scale ~1e6 is within atol + rtol·scale (~1e-3); on atol
    # alone (1e-9) it would fail — so the relative term is doing real work.
    before = State(0, {A: _pool(A, 1e6), B: _pool(B, 0.0)}, 0)
    passing = State(1, {A: _pool(A, 0.0), B: _pool(B, 1e6 + 5e-4)}, 0)
    assert_conserved(before, passing)  # within the relative tolerance

    failing = State(1, {A: _pool(A, 0.0), B: _pool(B, 1e6 + 2e-3)}, 0)
    with pytest.raises(ConservationError):
        assert_conserved(before, failing)


# --- registration-/insertion-order independence of the reduction (#15) ------
@given(perm=st.permutations(range(4)))
def test_compute_ledger_residual_is_stock_insertion_order_independent(
    perm: list[int],
) -> None:
    # Mixed large + small amounts so a naive (unsorted) sum would differ by ULPs under
    # reordering; compute_ledger sorts by stock id, so the residual is bit-identical.
    ids = [A, B, C, D]
    before_amt = [1e8, 3.0, 1e8, 7.0]
    after_amt = [1e8 + 0.1, 2.0, 1e8 - 0.1, 8.0]  # deltas +0.1 / −1 / −0.1 / +1 = 0

    def state(n: int, amt: list[float], order: list[int]) -> State:
        return State(n, {ids[i]: _pool(ids[i], amt[i]) for i in order}, 0)

    shuffled = compute_ledger(state(0, before_amt, perm), state(1, after_amt, perm))
    canonical = compute_ledger(
        state(0, before_amt, list(range(4))), state(1, after_amt, list(range(4)))
    )
    s = {ql.quantity: ql for ql in shuffled}[Quantity.CARBON]
    c = {ql.quantity: ql for ql in canonical}[Quantity.CARBON]
    assert s.residual == c.residual  # bit-identical (exact ==)


# --- engine-bug guard: stock-id key sets must match -------------------------
def test_compute_ledger_rejects_mismatched_stock_keys() -> None:
    before = State(0, {A: _pool(A, 1.0), B: _pool(B, 1.0)}, 0)
    after = State(1, {A: _pool(A, 1.0)}, 0)  # B dropped — never happens in Phase 0
    with pytest.raises(ValueError, match="stock"):
        compute_ledger(before, after)
