"""Phase-3 Step-3 tests: the closed water cycle (WATER only; the first closed ring).

Step 3 closes the one cycle Phase 1/2 left open. Transpiration (plants-owned, retargeted
to the in-system ``water_vapor`` via ``ChamberWiring.vapor_target``) feeds the ring; the
two **new** flows here close it:

```
soil_water (soil) --Transpiration--> water_vapor (atmosphere)
water_vapor (atmosphere) --Condensation--> condensate (water)
condensate (water) --Recycling--> soil_water (soil)
```

Both new flows are first-order donor-controlled WATER transfers (the
engineered-condenser framing — see ``water_cycle.py``), self-limiting like decomposition
/ mineralization. The sealed chamber drops ``Irrigation`` + ``water_source`` for genuine
closure (the nonzero irrigation flux would pump water into a sealed chamber).

Four layers:

* **Rate laws** — ``condensation_flux`` / ``recycling_flux`` are ``k·pool`` (→ 0 as the
  pool → 0; structural positivity).
* **Flow level** — ``Condensation`` / ``Recycling`` each withdraw from their donor POOL
  and deposit the *same* amount into the receiver; single-currency WATER; dt-linear.
* **Behavioral wiring** (the check the ledger identity *cannot* do — both sides of the
  identity move with a mislabel, P3.1): the three cycle flows carry **only** WATER,
  in the ring directions soil → atmosphere → water → soil.
* **Integration (the sealed season)** — the WATER-scoped per-compartment boundary ledger
  balances every step (soil / atmosphere / water); the closed loop conserves total water
  ``soil_water + water_vapor + condensate``; ``rationed == 0``.

The open-field golden stays byte-identical (the water leaf is empty there) — pinned by
``test_regression_season``; the sealed golden is the new closed-water behaviour
(``test_regression_sealed_season``).

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.compartments import (
    ATMOSPHERE,
    SOIL,
    WATER,
    compartment_boundary_ledger,
)
from domains.biosphere.loader import load_water_cycle_params
from domains.biosphere.season import (
    CONDENSATE,
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    SOIL_WATER,
    WATER_VAPOR,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.stocks import RN_VAR, TEMP_VAR, VPD_VAR
from domains.biosphere.water_cycle import (
    Condensation,
    Recycling,
    WaterCycleParams,
    condensation_flux,
    recycling_flux,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- rate laws ---------------------------------------------------------------
def test_condensation_flux_is_first_order_in_vapor() -> None:
    # condensation = k_cond · water_vapor: a hand value + linearity in the standing
    # vapor.
    assert math.isclose(
        condensation_flux(2.0, condensation_rate=0.5), 1.0, rel_tol=1e-12
    )
    assert math.isclose(
        condensation_flux(4.0, condensation_rate=0.5), 2.0, rel_tol=1e-12
    )


def test_recycling_flux_is_first_order_in_condensate() -> None:
    # recycling = k_rec · condensate: a hand value + linearity in the standing
    # condensate.
    assert math.isclose(recycling_flux(2.0, recycling_rate=0.5), 1.0, rel_tol=1e-12)
    assert math.isclose(recycling_flux(4.0, recycling_rate=0.5), 2.0, rel_tol=1e-12)


def test_fluxes_are_zero_at_empty_pools() -> None:
    # Self-limiting: no standing pool ⇒ no flux (positivity is structural).
    assert condensation_flux(0.0, condensation_rate=0.5) == 0.0
    assert recycling_flux(0.0, recycling_rate=0.5) == 0.0


# --- flow level --------------------------------------------------------------
_WV = StockId("test.water_vapor")
_CD = StockId("test.condensate")
_SW = StockId("test.soil_water")


def _pool(sid: StockId, amount: float) -> Stock:
    """A 1:1 WATER POOL stock (composition defaults to ``{WATER: 1.0}``)."""
    return Stock(
        id=sid,
        domain=SOIL,  # domain irrelevant to flow evaluation (legs/balance only)
        quantity=Quantity.WATER,
        unit=canonical_unit(Quantity.WATER),
        amount=amount,
        kind=StockKind.POOL,
    )


def _state(*, vapor: float = 0.0, cond: float = 0.0, soil: float = 0.0) -> State:
    return State(
        n=0,
        stocks={_WV: _pool(_WV, vapor), _CD: _pool(_CD, cond), _SW: _pool(_SW, soil)},
        rng_seed=0,
    )


def _condensation(rate: float = 0.5) -> Condensation:
    return Condensation(
        FlowId("biosphere.condensation"),
        0,
        water_vapor=_WV,
        condensate=_CD,
        params=WaterCycleParams(condensation_rate=rate, recycling_rate=0.0),
    )


def _recycling(rate: float = 0.5) -> Recycling:
    return Recycling(
        FlowId("biosphere.recycling"),
        0,
        condensate=_CD,
        soil_water=_SW,
        params=WaterCycleParams(condensation_rate=0.0, recycling_rate=rate),
    )


def _env(state: State, dt: float):
    # The water-cycle flows read no forcing; a trivial bound resolver satisfies the sig.
    return SourceResolver(forcings={}).bind(state, dt)


def test_condensation_transfers_vapor_to_condensate() -> None:
    # The condensed water leaves the vapor pool and lands in condensate — the SAME
    # amount, so the transfer is conservative by construction.
    state = _state(vapor=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _condensation().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    condensed = 0.5 * 2.0
    assert math.isclose(legs[_WV], -condensed, rel_tol=1e-12)  # withdrawn from vapor
    assert math.isclose(legs[_CD], condensed, rel_tol=1e-12)  # into condensate


def test_recycling_transfers_condensate_to_soil_water() -> None:
    state = _state(cond=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _recycling().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    recycled = 0.5 * 2.0
    assert math.isclose(
        legs[_CD], -recycled, rel_tol=1e-12
    )  # withdrawn from condensate
    assert math.isclose(legs[_SW], recycled, rel_tol=1e-12)  # into soil_water


@pytest.mark.parametrize("flow_donor", [("cond", "vapor"), ("rec", "cond")])
def test_cycle_flows_balance_water_only(flow_donor: tuple[str, str]) -> None:
    # Single-currency WATER: each flow balances WATER and touches no other quantity.
    which, donor = flow_donor
    state = _state(**{donor: 2.0})
    flow = _condensation() if which == "cond" else _recycling()
    result = flow.evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    residual = per_quantity_residual(result, state.stocks)
    assert set(residual) == {Quantity.WATER}  # CARBON/OXYGEN/NITROGEN untouched


def test_cycle_flows_are_dt_linear() -> None:
    # flux = daily·dt — the increment-form contract (RK4 order; here Euler-daily).
    state = _state(vapor=2.0, cond=2.0)
    for flow, donor in ((_condensation(), _WV), (_recycling(), _CD)):
        half = {
            leg.stock: leg.amount
            for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        }
        full = {
            leg.stock: leg.amount
            for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        }
        assert math.isclose(full[donor], 2.0 * half[donor], rel_tol=1e-12)


def test_cycle_flows_self_limit_at_empty_pool() -> None:
    # No standing pool ⇒ zero-amount legs (a clamped POOL draw never goes negative).
    state = _state()  # all pools empty
    assert all(
        leg.amount == 0.0
        for leg in _condensation().evaluate(state, _env(state, 1.0), 1.0).legs
    )
    assert all(
        leg.amount == 0.0
        for leg in _recycling().evaluate(state, _env(state, 1.0), 1.0).legs
    )


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_rates() -> None:
    params = load_water_cycle_params()
    assert params.condensation_rate == 0.5
    assert params.recycling_rate == 0.5


def test_loader_rejects_negative_rate(tmp_path: Path) -> None:
    bad = tmp_path / "water_cycle.yaml"
    bad.write_text(
        "name: chamber\nprocess: water_cycle\nparameters:\n"
        '  condensation_rate:\n    value: -0.1\n    unit: "1/day"\n    source: "t"\n'
        '  recycling_rate:\n    value: 0.5\n    unit: "1/day"\n    source: "t"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="condensation_rate must be >= 0"):
        load_water_cycle_params(bad)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    bad = tmp_path / "water_cycle.yaml"
    bad.write_text(
        "name: chamber\nprocess: water_cycle\nparameters:\n"
        '  condensation_rate:\n    value: 0.5\n    unit: "1/year"\n    source: "t"\n'
        '  recycling_rate:\n    value: 0.5\n    unit: "1/day"\n    source: "t"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="must be declared in"):
        load_water_cycle_params(bad)


# --- behavioral wiring (the check the ledger identity cannot do) --------------
def _ring_state() -> State:
    """A synthetic 3-pool WATER state with each pool nonzero (every flux positive).

    Uses the canonical stock ids + their leaf domains so the registry's *actual* flow
    objects evaluate against it and the leg directions are classified by real domain.
    """
    water = canonical_unit(Quantity.WATER)

    def pool(sid: StockId, dom, amount: float) -> Stock:
        return Stock(
            id=sid,
            domain=dom,
            quantity=Quantity.WATER,
            unit=water,
            amount=amount,
            kind=StockKind.POOL,
        )

    return State(
        n=0,
        stocks={
            SOIL_WATER: pool(SOIL_WATER, SOIL, 1000.0),  # ≫ sw_critical ⇒ f_water = 1
            WATER_VAPOR: pool(WATER_VAPOR, ATMOSPHERE, 5.0),
            CONDENSATE: pool(CONDENSATE, WATER, 5.0),
        },
        rng_seed=0,
    )


def test_three_cycle_flows_carry_only_water_in_ring_order() -> None:
    # The wiring assertion P3.1 says the ledger identity cannot make (both sides move
    # with a mislabel): the three cycle flows each carry ONLY water, source → sink in
    # the ring soil → atmosphere → water → soil. Evaluates the registry's *real* flow
    # objects against a synthetic all-positive state, so a builder mis-wiring is caught
    # here.
    _, registry = build_season(SEALED_CHAMBER_SCENARIO)
    flows = {str(f.id): f for f in registry.flows}
    state = _ring_state()
    domains = {sid: stock.domain for sid, stock in state.stocks.items()}
    # PM transpiration needs weather; condensation/recycling read no forcing.
    resolver = SourceResolver(
        forcings={
            RN_VAR: constant(200.0),
            VPD_VAR: constant(1000.0),
            TEMP_VAR: constant(20.0),
        }
    )
    bound = resolver.bind(state, 1.0)
    expected = {
        "biosphere.transpiration": (SOIL, ATMOSPHERE),
        "biosphere.condensation": (ATMOSPHERE, WATER),
        "biosphere.recycling": (WATER, SOIL),
    }
    for fid, (src_dom, snk_dom) in expected.items():
        result = flows[fid].evaluate(state, bound, 1.0)
        # WATER only: every touched stock carries WATER and nothing else; balanced.
        touched_quantities: set[Quantity] = set()
        for leg in result.legs:
            touched_quantities |= set(state.stocks[leg.stock].composition)
        assert touched_quantities == {Quantity.WATER}, (fid, touched_quantities)
        assert_flow_balanced(result, state.stocks)
        # Direction: exactly one source (negative leg) and one sink (positive leg), in
        # the expected ring compartments.
        sources = [leg for leg in result.legs if leg.amount < 0.0]
        sinks = [leg for leg in result.legs if leg.amount > 0.0]
        assert len(sources) == 1 and len(sinks) == 1, (fid, result.legs)
        assert domains[sources[0].stock] == src_dom, fid
        assert domains[sinks[0].stock] == snk_dom, fid


# --- integration: the sealed season -----------------------------------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], Registry, SourceResolver, int, tuple]:
    """The canonical sealed multi-year run + registry/resolver (for the ledger test).

    Returns the trajectory plus the ``registry`` and ``resolver`` so the
    per-compartment boundary-ledger test can reconstruct each step's ``FlowResult``s
    (``StepReport`` does not expose them) — sound under Euler + ``rationed == 0`` + no
    WATER extinction: with no arbitration scaling the applied legs equal the
    ``evaluate()`` legs, and WATER carries no POPULATION stock to route to the
    loss-sink.
    """
    weather = _weather() * SEALED_CHAMBER_YEARS
    steps = len(weather)
    state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, SEALED_CHAMBER_SCENARIO)
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, steps
    )
    return states, registry, resolver, rationed, events


def _water_total(s: State) -> float:
    """Total water around the closed ring (the only WATER stocks when sealed)."""
    return (
        s.stocks[SOIL_WATER].amount
        + s.stocks[WATER_VAPOR].amount
        + s.stocks[CONDENSATE].amount
    )


def test_sealed_closed_water_loop_is_conserved(
    sealed: tuple[list[State], Registry, SourceResolver, int, tuple],
) -> None:
    # The closure proof beyond the every-step global gate: with the boundaries gone, the
    # ring total soil_water + water_vapor + condensate is invariant (each leg is a 1:1
    # WATER transfer). soil_water O(1e3) kg ⇒ a looser absolute band (the
    # float-subtraction noise of the large pool), as in test_sealed_chamber's WATER
    # conservation.
    states, *_ = sealed
    tot0 = _water_total(states[0])
    for s in states:
        assert math.isclose(_water_total(s), tot0, rel_tol=0.0, abs_tol=1e-7)


def test_sealed_water_cycle_is_active_and_distributes(
    sealed: tuple[list[State], Registry, SourceResolver, int, tuple],
) -> None:
    # Non-vacuity: the ring genuinely moves water — vapor + condensate build from their
    # 0 start (transpiration → condensation → recycling), so the conservation/ledger
    # checks are exercising a live cycle, not a frozen one.
    states, *_ = sealed
    assert states[0].stocks[WATER_VAPOR].amount == 0.0
    assert states[0].stocks[CONDENSATE].amount == 0.0
    assert max(s.stocks[WATER_VAPOR].amount for s in states) > 1e-3
    assert max(s.stocks[CONDENSATE].amount for s in states) > 1e-3


def test_sealed_water_scoped_compartment_ledger_balances_every_step(
    sealed: tuple[list[State], Registry, SourceResolver, int, tuple],
) -> None:
    # The advisor's load-bearing catch: the per-compartment boundary ledger's
    # apply-integrity residual ≈ 0 for Quantity.WATER on EVERY step, for soil /
    # atmosphere / water (the compartments the cycle touches). Scoped to WATER
    # deliberately: the residual identity holds only on a clean step (rationed == 0 AND
    # no extinction routing); the sealed producer may go extinct, but extinction routes
    # CARBON to the loss-sink and touches no WATER stock, so WATER stays clean even if
    # the plant dies. The full-ledger-every-step assertion (handling the extinction
    # exception) is Step 5.
    states, registry, resolver, rationed, events = sealed
    assert rationed == 0  # the ledger identity's preconditions (no arbitration scaling)
    water_compartments = {SOIL, ATMOSPHERE, WATER}
    saw_crossing = False
    for i in range(len(states) - 1):
        # Reconstruct the step's evaluated flows (Euler: one evaluation at the start-of-
        # step state, bound to that same snapshot — the #16 seam, mirroring the engine).
        bound = resolver.bind(states[i], 1.0)
        results = [flow.evaluate(states[i], bound, 1.0) for flow in registry.flows]
        ledger = compartment_boundary_ledger(states[i], states[i + 1], results)
        for entry in ledger:
            if entry.quantity is not Quantity.WATER:
                continue
            assert entry.domain in water_compartments, (i, entry.domain)
            assert abs(entry.residual) < 1e-7, (i, entry)
            if entry.crossing_in > 0.0 or entry.crossing_out > 0.0:
                saw_crossing = True
    assert saw_crossing  # non-vacuity: the ledger saw real WATER crossing flux


def test_sealed_water_cycle_never_rations(
    sealed: tuple[list[State], Registry, SourceResolver, int, tuple],
) -> None:
    # Structural positivity through the new flows: each first-order draw (k·pool·dt,
    # k·dt = 0.5 < 1) self-limits against the start-of-step pool, so the Euler backstop
    # never fires (the decomposition / mineralization positivity pattern).
    *_, rationed, _ = sealed
    assert rationed == 0
