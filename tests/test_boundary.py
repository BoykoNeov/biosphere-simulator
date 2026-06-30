"""Step-4 tests: Boundary reservoir constructors + numerical-loss-sink identity.

Construction-level only — arbitration's "unclamped source is not throttled" half
of the Boundary-exchange gate needs the arbitrator (step 7) and the every-step
ledger needs step 8. The *conservation* half ("an otherwise-unbalanced flow
balances once the boundary reservoir is counted") is exercised here by reusing
the step-3 ``assert_flow_balanced`` helper, without dragging the later machinery
forward.
"""

from simcore.boundary import (
    BOUNDARY_DOMAIN,
    is_loss_sink,
    loss_sink,
    loss_sink_id,
    loss_sinks,
    sink,
    source,
)
from simcore.flow import FlowResult, Leg, assert_flow_balanced
from simcore.ids import DomainId, StockId
from simcore.quantities import (
    ASSERTED_QUANTITIES,
    Quantity,
    StockKind,
    canonical_unit,
)
from simcore.state import Stock

PLANT_C = StockId("bio.plant_c")


def _plant_c(amount: float = 10.0) -> Stock:
    """A biosphere carbon biomass stock — the modeled side of a boundary exchange."""
    return Stock(
        id=PLANT_C,
        domain=DomainId("bio"),
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
    )


# --- loss-sink identity ----------------------------------------------------
def test_loss_sink_id_is_deterministic_and_quantity_specific() -> None:
    assert loss_sink_id(Quantity.CARBON) == loss_sink_id(Quantity.CARBON)
    assert loss_sink_id(Quantity.CARBON) != loss_sink_id(Quantity.WATER)


def test_is_loss_sink_discriminates() -> None:
    assert is_loss_sink(loss_sink_id(Quantity.CARBON))
    # A legitimate boundary-exchange reservoir is not a loss-sink.
    assert not is_loss_sink(StockId("boundary.outside_c"))
    assert not is_loss_sink(StockId("bio.plant_c"))


# --- loss_sink / loss_sinks ------------------------------------------------
def test_loss_sink_is_a_zeroed_boundary_reservoir() -> None:
    s = loss_sink(Quantity.CARBON)
    assert s.kind is StockKind.BOUNDARY
    assert s.quantity is Quantity.CARBON
    assert s.unit == canonical_unit(Quantity.CARBON)
    assert s.domain == BOUNDARY_DOMAIN
    assert s.amount == 0.0
    assert s.unclamped is False  # a sink is never withdrawn from


def test_loss_sinks_cover_every_asserted_quantity_keyed_by_id() -> None:
    sinks = loss_sinks()  # defaults to ASSERTED_QUANTITIES
    assert set(sinks) == {loss_sink_id(q) for q in ASSERTED_QUANTITIES}
    # ENERGY joined the conserved set in Phase 5, so it is covered too. It has no
    # POPULATION stock (extinction routes biomass carbon), so its loss-sink is never
    # used — harmless, and no production caller builds it (build_season / build_demo
    # pass an explicit {CARBON}).
    assert Quantity.ENERGY in {s.quantity for s in sinks.values()}
    # Keyed by the stock's own id, so the dict merges straight into State.stocks.
    for sid, s in sinks.items():
        assert sid == s.id


def test_loss_sinks_accepts_an_explicit_quantity_set() -> None:
    sinks = loss_sinks([Quantity.CARBON])
    assert set(sinks) == {loss_sink_id(Quantity.CARBON)}


# --- source ----------------------------------------------------------------
def test_source_is_unclamped_boundary_by_default() -> None:
    s = source(StockId("boundary.solar"), Quantity.ENERGY, amount=1000.0)
    assert s.kind is StockKind.BOUNDARY
    assert s.unclamped is True  # min-scaling never throttles a source (#13)
    assert s.unit == canonical_unit(Quantity.ENERGY)
    assert s.domain == BOUNDARY_DOMAIN


def test_source_can_be_made_clamped() -> None:
    s = source(StockId("boundary.tank"), Quantity.WATER, amount=50.0, unclamped=False)
    assert s.unclamped is False


# --- sink ------------------------------------------------------------------
def test_sink_is_a_clamped_boundary_accumulator() -> None:
    s = sink(StockId("boundary.outside_c"), Quantity.CARBON)
    assert s.kind is StockKind.BOUNDARY
    assert s.unclamped is False
    assert s.amount == 0.0
    assert s.domain == BOUNDARY_DOMAIN


# --- boundary closes an otherwise-unbalanced flow (conservation half) -------
def test_boundary_closes_an_unbalanced_harvest() -> None:
    # Harvest removes carbon from plant biomass; unbalanced against the modeled
    # stocks alone, but balanced once the boundary sink is counted (#13).
    out = sink(StockId("boundary.outside_c"), Quantity.CARBON)
    stocks = {PLANT_C: _plant_c(), out.id: out}
    harvest = FlowResult(legs=(Leg(PLANT_C, -2.0), Leg(out.id, 2.0)))
    assert_flow_balanced(harvest, stocks)  # balances with the boundary counted


def test_loss_sink_routing_conserves_quantity() -> None:
    # Step 7 routes an extinct POPULATION residual into the loss-sink; this pins
    # that the routing *conserves* (a withdrawal + the matching loss-sink deposit
    # balances per-quantity) without pulling step 7's mechanism forward.
    ls = loss_sink(Quantity.CARBON)
    stocks = {PLANT_C: _plant_c(amount=0.0), ls.id: ls}
    routed = FlowResult(legs=(Leg(PLANT_C, -0.5), Leg(ls.id, 0.5)))
    assert_flow_balanced(routed, stocks)
