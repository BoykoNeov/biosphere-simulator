"""Step-1 (P2.1) tests: the stock element-composition core change.

A ``Stock`` carries a ``composition`` map — moles of each conserved quantity per
canonical unit — and the conservation gate folds it at **two** sites: the leg side
(``flow.per_quantity_residual`` / ``assert_flow_balanced``) and the state-delta side
(``conservation.compute_ledger`` / ``assert_conserved``). This is the deferred
genuine multi-quantity stoichiometric flow (one atomic flow balancing CARBON *and*
OXYGEN), built as pure infra with synthetic stocks before any Phase-2 biology.

Built test-first against the "Step 1 design" section of
``docs/plans/phase-2-closed-chamber.md``. The load-bearing correctness points:

* the 1:1 default reproduces Phase-1 exactly (also pinned by the regenerated
  goldens — here we assert the in-memory map directly);
* a ``CO2 -> biomass + O2`` flow balances **both** quantities on **both** gate
  paths (the state-delta path is the one the earlier "one change" claim missed —
  a regression there is the false-OXYGEN-trip bug);
* a deliberately mis-stoichiometric oxygen coeff trips the gate (element
  accounting doing its job over a silent factor).
"""

import dataclasses

import pytest
from hypothesis import given
from hypothesis import strategies as st

from simcore.conservation import assert_conserved, compute_ledger
from simcore.environment import Environment, SourceResolver
from simcore.flow import (
    ConservationError,
    FlowResult,
    Leg,
    assert_flow_balanced,
    per_quantity_residual,
)
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

BIO = DomainId("bio")
CO2 = StockId("bio.co2")
O2 = StockId("bio.o2")
BIOMASS = StockId("bio.biomass")

# Per-mol element compositions (P2.1). Pure-carbon biomass + PQ=1: photosynthesis
# CO2 -> organ_C + O2 closes OXYGEN through CO2<->O2 directly, no water term.
_CO2_COMP = {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
_O2_COMP = {Quantity.OXYGEN: 2.0}


def _stock(
    sid: StockId,
    amount: float,
    quantity: Quantity,
    *,
    kind: StockKind = StockKind.POOL,
    composition: dict[Quantity, float] | None = None,
) -> Stock:
    # An empty map is the Stock "not supplied" sentinel → fills to the 1:1 default.
    return Stock(
        id=sid,
        domain=BIO,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=kind,
        composition=composition or {},
    )


def _co2(amount: float) -> Stock:
    return _stock(CO2, amount, Quantity.CARBON, composition=_CO2_COMP)


def _o2(amount: float) -> Stock:
    return _stock(O2, amount, Quantity.OXYGEN, composition=_O2_COMP)


def _biomass(amount: float) -> Stock:
    # Pure carbon — default 1:1 composition (unchanged from Phase 1).
    return _stock(BIOMASS, amount, Quantity.CARBON)


# --- 1:1 default reproduces Phase-1 exactly --------------------------------
def test_default_composition_is_one_to_one() -> None:
    s = _biomass(5.0)
    assert dict(s.composition) == {Quantity.CARBON: 1.0}


def test_explicit_composition_is_preserved_and_readonly() -> None:
    s = _co2(3.0)
    assert dict(s.composition) == {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
    with pytest.raises(TypeError):  # MappingProxyType — frozen post-construction
        s.composition[Quantity.CARBON] = 9.0  # type: ignore[index]


def test_default_composition_stock_stays_hashable() -> None:
    # ``hash=False`` on the composition field keeps Stock hashable over its other
    # fields; only the (unhashable Mapping) composition is excluded from the hash.
    hash(_biomass(1.0))  # must not raise


# --- leg side: a CO2 -> biomass + O2 flow balances CARBON and OXYGEN -------
def _photosynthesis_legs(rate: float) -> FlowResult:
    # rate mol CO2 fixed -> rate mol biomass C + rate mol O2 released (PQ=1).
    return FlowResult(
        legs=(Leg(CO2, -rate), Leg(BIOMASS, rate), Leg(O2, rate)),
    )


def test_photosynthesis_flow_balances_both_quantities() -> None:
    stocks = {CO2: _co2(10.0), O2: _o2(5.0), BIOMASS: _biomass(0.0)}
    result = _photosynthesis_legs(1.0)

    residual = per_quantity_residual(result, stocks)
    # CARBON: -1*1 (CO2) + 1*1 (biomass) = 0 ; OXYGEN: -1*2 (CO2) + 1*2 (O2) = 0.
    assert residual[Quantity.CARBON] == pytest.approx(0.0, abs=1e-12)
    assert residual[Quantity.OXYGEN] == pytest.approx(0.0, abs=1e-12)
    assert_flow_balanced(result, stocks)  # must not raise


def test_respiration_flow_reverses_and_balances() -> None:
    stocks = {CO2: _co2(10.0), O2: _o2(5.0), BIOMASS: _biomass(2.0)}
    # biomass + O2 -> CO2 (the reverse of photosynthesis).
    result = FlowResult(legs=(Leg(BIOMASS, -1.0), Leg(O2, -1.0), Leg(CO2, 1.0)))
    assert_flow_balanced(result, stocks)  # must not raise


def test_leg_side_mis_stoichiometric_oxygen_trips_gate() -> None:
    # Only 0.5 mol O2 released per mol CO2 fixed: CARBON still balances, but OXYGEN
    # nets -1*2 + 0.5*2 = -1 != 0 — the gate must catch the silent factor.
    stocks = {CO2: _co2(10.0), O2: _o2(5.0), BIOMASS: _biomass(0.0)}
    result = FlowResult(legs=(Leg(CO2, -1.0), Leg(BIOMASS, 1.0), Leg(O2, 0.5)))

    residual = per_quantity_residual(result, stocks)
    assert residual[Quantity.CARBON] == pytest.approx(0.0, abs=1e-12)
    assert residual[Quantity.OXYGEN] == pytest.approx(-1.0)
    with pytest.raises(ConservationError, match="OXYGEN"):
        assert_flow_balanced(result, stocks)


# --- state-delta side: the fold the "one change" claim missed --------------
def test_state_delta_co2_to_o2_conserves_both_quantities() -> None:
    # 1 mol CO2 -> 1 mol biomass + 1 mol O2 over the step (PQ=1).
    before = State(0, {CO2: _co2(10.0), O2: _o2(5.0), BIOMASS: _biomass(0.0)}, 0)
    after = State(1, {CO2: _co2(9.0), O2: _o2(6.0), BIOMASS: _biomass(1.0)}, 0)

    ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
    # CARBON: CO2 -1*1 + biomass +1*1 = 0 ; OXYGEN: CO2 -1*2 + O2 +1*2 = 0.
    assert ledger[Quantity.CARBON].residual == pytest.approx(0.0, abs=1e-12)
    assert ledger[Quantity.OXYGEN].residual == pytest.approx(0.0, abs=1e-12)
    assert_conserved(before, after)  # the fold-site-2 pin: must not raise


def test_state_delta_off_stoichiometry_trips_oxygen_gate() -> None:
    # O2 rises 1.5 instead of 1.0: OXYGEN residual = -1*2 + 1.5*2 = +1 != 0.
    before = State(0, {CO2: _co2(10.0), O2: _o2(5.0), BIOMASS: _biomass(0.0)}, 0)
    after = State(1, {CO2: _co2(9.0), O2: _o2(6.5), BIOMASS: _biomass(1.0)}, 0)

    ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
    assert ledger[Quantity.CARBON].residual == pytest.approx(0.0, abs=1e-12)
    assert ledger[Quantity.OXYGEN].residual == pytest.approx(1.0)
    with pytest.raises(ConservationError, match="OXYGEN"):
        assert_conserved(before, after)


# --- determinism: the composition fold is order-independent (#15) -----------
@given(perm=st.permutations(range(3)))
def test_composition_fold_is_stock_order_independent(perm: list[int]) -> None:
    # Multi-quantity stocks with mixed magnitudes; compute_ledger sorts by stock id,
    # so the per-quantity residual is bit-identical under registration shuffle.
    builders = [lambda: _co2(1e8), lambda: _o2(1e8 + 0.5), lambda: _biomass(3.0)]
    ids = [CO2, O2, BIOMASS]

    def make(n: int, build: list, order: list[int]) -> State:
        return State(n, {ids[i]: build[i]() for i in order}, 0)

    after_builders = [
        lambda: _co2(1e8 - 1.0),
        lambda: _o2(1e8 + 0.5 + 2.0),
        lambda: _biomass(4.0),
    ]
    shuffled = compute_ledger(make(0, builders, perm), make(1, after_builders, perm))
    canonical = compute_ledger(
        make(0, builders, [0, 1, 2]), make(1, after_builders, [0, 1, 2])
    )
    s = {ql.quantity: ql for ql in shuffled}
    c = {ql.quantity: ql for ql in canonical}
    for q in (Quantity.CARBON, Quantity.OXYGEN):
        assert s[q].residual == c[q].residual  # bit-identical (exact ==)


# --- construction-time validation of the composition map -------------------
def test_population_must_be_single_quantity() -> None:
    # A multi-quantity POPULATION would leak its non-nominal quantities at
    # extinction (loss-sink routing is single-quantity) — rejected at construction.
    with pytest.raises(ValueError, match="POPULATION"):
        _stock(
            CO2, 1.0, Quantity.CARBON, kind=StockKind.POPULATION, composition=_CO2_COMP
        )


def test_composition_must_include_own_quantity_positive() -> None:
    with pytest.raises(ValueError, match="own quantity"):
        # nominal CARBON absent from the map
        _stock(O2, 1.0, Quantity.CARBON, composition={Quantity.OXYGEN: 2.0})
    with pytest.raises(ValueError, match="own quantity"):
        # present but non-positive
        _stock(BIOMASS, 1.0, Quantity.CARBON, composition={Quantity.CARBON: 0.0})


def test_composition_rejects_non_finite_coeff() -> None:
    with pytest.raises(ValueError, match="finite"):
        _stock(
            CO2,
            1.0,
            Quantity.CARBON,
            composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: float("inf")},
        )


def test_composition_rejects_non_quantity_key() -> None:
    with pytest.raises(ValueError, match="not a Quantity"):
        _stock(BIOMASS, 1.0, Quantity.CARBON, composition={"carbon": 1.0})  # type: ignore[dict-item]


# --- a multi-quantity flow conserves through the whole integrator step ------
@dataclasses.dataclass(frozen=True)
class _PhotosynthesisFlow:
    """A constant (dt-linear) CO2 -> biomass + O2 transfer, valid under RK4."""

    id: FlowId
    priority: int
    rate: float

    def evaluate(self, snapshot: State, env: Environment, dt: float) -> FlowResult:
        r = self.rate * dt
        return FlowResult(legs=(Leg(CO2, -r), Leg(BIOMASS, r), Leg(O2, r)))


def test_multiquantity_flow_conserves_through_integrator() -> None:
    stocks = {CO2: _co2(100.0), O2: _o2(10.0), BIOMASS: _biomass(0.0)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    reg = Registry([_PhotosynthesisFlow(FlowId("photo"), 0, 1.0)], stocks)

    # The always-on gate inside the integrator asserts CARBON and OXYGEN balance;
    # a non-raising step is the assertion.
    nxt = EulerIntegrator(reg).step(state, SourceResolver(), 1.0)
    ledger = {ql.quantity: ql for ql in compute_ledger(state, nxt)}
    assert ledger[Quantity.CARBON].residual == pytest.approx(0.0, abs=1e-12)
    assert ledger[Quantity.OXYGEN].residual == pytest.approx(0.0, abs=1e-12)
