"""The Station cabin gas loop: couple Crew ↔ ECLSS at the cabin air (P6.2).

Step 2's assembly — the second cross-domain seam, one level richer than Step 1's
single-quantity heat closure. It imports **both** the Crew and ECLSS domains and wires
them at the shared cabin-air stocks; **no domain imports another** (finding #1). Where
Step 1 coupled one quantity (ENERGY), the cabin couples **three** (CARBON / OXYGEN /
WATER) and turns on the one non-trivial representation decision of the phase: **CO₂ as a
composition ``{CARBON:1, OXYGEN:2}`` stock**, which is what makes OXYGEN *close*.

**The seam (per the phase-6 plan / finding #2).**

  * The ECLSS forced ``CrewMetabolism`` stand-in and its three ``metabolic_*``
    reservoirs are **dropped**; the standalone crew ``o2_store`` / ``OxygenConsumption``
    are dropped too. The real crew now breathes cabin air via the merged
    :class:`station.flows.CrewRespiration`
    (``food_store + cabin_o2 → cabin_co2 + fecal_waste``) and adds humidity via the crew
    ``WaterBalance`` (``water_store → cabin_h2o + urine``).
  * **Every stock the CO₂ passes through is composition ``{C:1, O:2}``** — ``cabin_co2``
    *and* the scrubber sink ``co2_removed`` — and every O₂ stock is ``{O:2}`` —
    ``cabin_o2`` *and* the makeup source ``o2_supply``. Without this the scrubber would
    unbalance OXYGEN (remove 2 O per CO₂ from a pool that only booked carbon), and
    respiration would consume O₂ with nowhere for those atoms to go. So OXYGEN closes
    over the augmented loop ``o2_supply → cabin_o2 → cabin_co2 → co2_removed`` **only**
    because of the composition annotation + the atom-coupled respiration flow. The
    ledger *refuses* the decoupled (pure-carbon-CO₂) version — the non-vacuous gate (see
    ``test_cabin_run``).

**Closure sense (honest — do not overclaim).** This is closure in the **augmented /
atom-conservation** sense (each quantity balances every step over cabin pools + boundary
reservoirs, and the crew's O/C atoms are now tied together), **not** a closed O₂/CO₂
*cycle*: O₂ still enters from the ``o2_supply`` tank and CO₂ still leaves to the
``co2_removed`` scrubber sink. The recycled cabin cycle (plants returning O₂ the crew
breathes) arrives in **Step 3** (biosphere ↔ cabin). Standalone Thermal's permanent
``boundary.space`` is the analogue: a real, still-open boundary that a later step moves
inward.

**The four assembly pieces mirror ``domains.*.system`` one level up.**

  * :func:`build_cabin` — the composition-aware stock set + the five coupled flows.
  * :func:`cabin_resolver` — the merged forcing (the two crew intake rates; O₂ intake is
    **derived** via RQ = 1, so it is *not* a forcing var — the ECLSS control loops read
    stocks, not ``env``).
  * :func:`cabin_steady_state` — the emergent per-species cabin steady states (the
    ``eclss.steady_state`` analogue, with the crew load now atom-coupled at RQ = 1).
  * The stepping loop is the generic :func:`station.system.run_station` (reused verbatim
    — it is domain-agnostic, just an integrator loop with the every-step conservation
    gate).

Pure stdlib only in the spine; the crew split fractions + ECLSS control coefficients
load via the sibling loaders.
"""

from collections.abc import Mapping
from dataclasses import dataclass

from domains.crew.flows import CrewParams, WaterBalance
from domains.crew.stocks import (
    FECAL_WASTE,
    FOOD_INTAKE_VAR,
    FOOD_STORE,
    URINE,
    WATER_INTAKE_VAR,
    WATER_STORE,
    food_store_stock,
    water_store_stock,
)
from domains.crew.system import WATER_BALANCE
from domains.eclss.flows import CO2Scrubber, Condenser, EclssParams, O2Makeup
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_REMOVED,
    ECLSS_DOMAIN,
    HUMIDITY_CONDENSATE,
    O2_SUPPLY,
    cabin_h2o_stock,
)
from domains.eclss.system import CO2_SCRUBBER, CONDENSER, O2_MAKEUP
from simcore import boundary
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.environment import SourceResolver, constant
from simcore.flow import Flow
from simcore.ids import FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock
from station.flows import CrewRespiration
from station.scenario import CABIN_GAS_SCENARIO, CabinScenario

# The station-owned merged respiration flow id (the cross-domain seam; ASCII so str sort
# == future Rust byte sort, #15). The other four flows reuse their domains' canonical
# ids.
CREW_RESPIRATION: FlowId = FlowId("station.crew_respiration")

# Element compositions for the gas-phase cabin stocks (the biosphere convention: OXYGEN
# is O-atoms, so O₂ books 2 and CO₂ books 2 — the ``atmosphere.py`` carbon_pool /
# o2_pool fold). Every CO₂-bearing stock in the loop must carry CO2_COMPOSITION and
# every O₂-bearing stock O2_COMPOSITION, or OXYGEN fails to close (see the module
# docstring).
CO2_COMPOSITION: Mapping[Quantity, float] = {Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0}
O2_COMPOSITION: Mapping[Quantity, float] = {Quantity.OXYGEN: 2.0}


@dataclass(frozen=True)
class CabinSteadyState:
    """The emergent per-species cabin steady states (see :func:`cabin_steady_state`)."""

    cabin_o2: float
    cabin_co2: float
    cabin_h2o: float


def _gas_pool(
    stock_id: StockId,
    quantity: Quantity,
    amount: float,
    composition: Mapping[Quantity, float],
) -> Stock:
    """A composition-carrying cabin POOL (``cabin_o2``/``cabin_co2``).

    The ECLSS ``cabin_*_stock`` constructors build **single-quantity** (1:1) pools —
    fine standalone, where nothing couples O₂ to CO₂ stoichiometrically — but the
    coupled loop needs the gas-phase composition (``{C:1,O:2}`` / ``{O:2}``).
    ``boundary.py`` / ``eclss.stocks`` take no composition arg, and extending them would
    be a **core change** (``git diff src/simcore/`` must stay empty), so the station
    builds these four gas stocks inline — the assembly-layer-owns-the-wiring discipline
    (finding #1), one level down at the stock level.
    """
    return Stock(
        id=stock_id,
        domain=ECLSS_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=amount,
        kind=StockKind.POOL,
        composition=composition,
    )


def _gas_boundary(
    stock_id: StockId,
    quantity: Quantity,
    composition: Mapping[Quantity, float],
    *,
    unclamped: bool,
) -> Stock:
    """A composition-carrying BOUNDARY reservoir (``co2_removed`` sink / ``o2_supply``).

    The ``boundary.source`` / ``boundary.sink`` helpers can't set composition, so the
    two boundary gas reservoirs are built inline (same rationale as :func:`_gas_pool`).
    Both must carry the loop's gas composition or the scrubber (removing 2 O per CO₂) /
    the makeup (adding 2 O per O₂) would unbalance OXYGEN at the boundary.
    """
    return Stock(
        id=stock_id,
        domain=BOUNDARY_DOMAIN,
        quantity=quantity,
        unit=canonical_unit(quantity),
        amount=0.0,
        kind=StockKind.BOUNDARY,
        unclamped=unclamped,
        composition=composition,
    )


def build_cabin(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    scenario: CabinScenario = CABIN_GAS_SCENARIO,
    cabin_co2_composition: Mapping[Quantity, float] = CO2_COMPOSITION,
) -> tuple[State, Registry]:
    """Assemble the coupled Crew ↔ ECLSS cabin's initial ``State`` + ``Registry``.

    Ten stocks and five flows. The stocks: the two composition gas POOLs ``cabin_o2``
    (``{O:2}``) / ``cabin_co2`` (``cabin_co2_composition``), the single-quantity
    ``cabin_h2o`` POOL, the two finite crew POOLs ``food_store`` / ``water_store`` (**no
    ``o2_store`` — the crew breathes cabin O₂**), and five boundary reservoirs
    (``o2_supply`` source + ``co2_removed`` / ``humidity_condensate`` / ``fecal_waste``
    / ``urine`` sinks; the gas ones composition-carrying). The flows: the station-owned
    :class:`CrewRespiration` + the crew ``WaterBalance`` (wired to ``cabin_h2o`` /
    ``urine``) + the three ECLSS control loops ``CO2Scrubber`` / ``Condenser`` /
    ``O2Makeup``. **No loss-sinks** (no POPULATION stock). The ``Registry`` re-sorts
    flows by id, so build order is inert.

    ``cabin_co2_composition`` defaults to the coupled ``{C:1,O:2}``. The validation test
    passes ``{CARBON:1}`` (pure carbon) to build the **decoupled** cabin — which raises
    ``ConservationError`` on the first step: respiration consumes ``cabin_o2`` O with
    nowhere for those atoms to land (at the clean-cabin start the scrubber and makeup
    are dormant, so the respiration leg breaks first; the scrubber's own ``{C:1,O:2}``
    requirement bites later in a full run). That refusal *is* OXYGEN closure becoming
    real (finding #2).
    """
    cabin_o2 = _gas_pool(CABIN_O2, Quantity.OXYGEN, scenario.cabin_o2_0, O2_COMPOSITION)
    cabin_co2 = _gas_pool(
        CABIN_CO2, Quantity.CARBON, scenario.cabin_co2_0, cabin_co2_composition
    )
    cabin_h2o = cabin_h2o_stock(scenario.cabin_h2o_0)
    food_store = food_store_stock(scenario.food_store0)
    water_store = water_store_stock(scenario.water_store0)
    o2_supply = _gas_boundary(
        O2_SUPPLY, Quantity.OXYGEN, O2_COMPOSITION, unclamped=True
    )
    co2_removed = _gas_boundary(
        CO2_REMOVED, Quantity.CARBON, CO2_COMPOSITION, unclamped=False
    )
    humidity_condensate = boundary.sink(HUMIDITY_CONDENSATE, Quantity.WATER, 0.0)
    fecal_waste = boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0)
    urine = boundary.sink(URINE, Quantity.WATER, 0.0)
    stocks = {
        s.id: s
        for s in (
            cabin_o2,
            cabin_co2,
            cabin_h2o,
            food_store,
            water_store,
            o2_supply,
            co2_removed,
            humidity_condensate,
            fecal_waste,
            urine,
        )
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        CrewRespiration(
            CREW_RESPIRATION,
            0,
            food_store=FOOD_STORE,
            cabin_co2=CABIN_CO2,
            cabin_o2=CABIN_O2,
            fecal_waste=FECAL_WASTE,
            respired_carbon_fraction=crew_params.respired_carbon_fraction,
        ),
        WaterBalance(
            WATER_BALANCE,
            0,
            water_store=WATER_STORE,
            crew_humidity=CABIN_H2O,  # the seam: crew humidity lands in the cabin
            urine=URINE,
            params=crew_params,
        ),
        CO2Scrubber(
            CO2_SCRUBBER,
            0,
            cabin_co2=CABIN_CO2,
            co2_removed=CO2_REMOVED,
            params=eclss_params,
        ),
        Condenser(
            CONDENSER,
            0,
            cabin_h2o=CABIN_H2O,
            humidity_condensate=HUMIDITY_CONDENSATE,
            params=eclss_params,
        ),
        O2Makeup(
            O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=CABIN_O2, params=eclss_params
        ),
    ]
    return state, Registry(flows, stocks)


def cabin_resolver(scenario: CabinScenario = CABIN_GAS_SCENARIO) -> SourceResolver:
    """The merged forcing: the two constant crew intake rates (food + water).

    Only the two crew *intake* rates are forcing vars: ``CrewRespiration`` reads
    ``crew_food_intake`` and ``WaterBalance`` reads ``crew_water_intake``. **O₂ intake
    is absent** — it is derived from food intake via RQ = 1 inside ``CrewRespiration``
    (the whole point of the merge). The three ECLSS control loops read *stocks*, not
    ``env``, so they contribute no forcing (like Thermal's radiator in Step 1). A flow
    cannot tell forcing from a shared stock (#16), so the crew ``WaterBalance`` code
    runs unchanged.
    """
    return SourceResolver(
        forcings={
            FOOD_INTAKE_VAR: constant(scenario.food_intake_rate),
            WATER_INTAKE_VAR: constant(scenario.water_intake_rate),
        }
    )


def cabin_steady_state(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    scenario: CabinScenario = CABIN_GAS_SCENARIO,
) -> CabinSteadyState:
    """The emergent per-species cabin steady states (mol / mol / kg), a closed form.

    The ``eclss.steady_state`` analogue, with the crew load now atom-coupled at RQ = 1:
    the crew CO₂ **and** O₂ rates are both ``P = f_resp · food_intake`` (one respired
    mol C makes one CO₂ and consumes one O₂), and the humidity rate is
    ``f_ins · water_intake``. Each ECLSS control loop balances its species' crew load at
    steady state:

      * ``cabin_co2_eq = P / k_scrub``            (scrubber balances CO₂ production)
      * ``cabin_o2_eq  = o2_setpoint − P / k_makeup``  (makeup balances O₂ consumption)
      * ``cabin_h2o_eq = (f_ins · water_intake) / k_cond``  (condenser balances
        humidity)

    Closed-form, emergent from the crew load + the loaded params — never stored. The
    validation asserts the run converges here (and that ``cabin_o2_eq < o2_setpoint``,
    i.e. the crew genuinely draws cabin O₂ down).
    """
    p = crew_params.respired_carbon_fraction * scenario.food_intake_rate
    humidity = crew_params.insensible_water_fraction * scenario.water_intake_rate
    return CabinSteadyState(
        cabin_o2=eclss_params.o2_setpoint - p / eclss_params.o2_makeup_gain,
        cabin_co2=p / eclss_params.co2_scrub_rate,
        cabin_h2o=humidity / eclss_params.condense_rate,
    )
