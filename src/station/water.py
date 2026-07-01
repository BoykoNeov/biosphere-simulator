"""The Station water loop: close the crew's water via recovery (P6.4).

Step 4's assembly — the fourth cross-domain seam, built one assembly **below** the
greenhouse (on the Step-2 cabin). Standalone Crew's ``water_store`` is a finite POOL
that only *depletes*: ``WaterBalance`` splits it into insensible humidity (→
``cabin_h2o`` → the ECLSS ``Condenser``) and urine, both landing in terminal boundary
sinks. Step 4 re-points those two disposal paths **inward** and closes the loop, so the
crew's water becomes **regenerative up to the recovery efficiency**.

**The seam (assembly-level id choices, zero domain / zero core change — finding #1).**

  * A new ``recovered_water`` buffer POOL (the crew analogue of the biosphere's
    ``condensate``): the ECLSS ``Condenser`` product **and** the crew urine collect
    there. Purely a *choice of which id to pass* — the ``Condenser``'s
    ``humidity_condensate`` sink arg and ``WaterBalance``'s ``urine`` arg are pointed at
    ``recovered_water`` instead of at the two terminal sinks. Neither flow class alters.
  * The station-owned :class:`station.flows.WaterRecovery`
    (``recovered_water → water_store (+η_w) + brine (+(1−η_w))``) returns the recovered
    fraction to ``crew.water_store`` and vents the unrecoverable remainder to a
    ``brine`` sink. Its rate + efficiency are the **first station-owned params**
    (``station/params/water_recovery.yaml`` via ``station.loader``).

So the Step-2 cabin's ``humidity_condensate`` / ``urine`` terminal sinks are **absent**
from the Step-4 state (the redirection is structural, not a shadow sink); ``brine`` is
the one remaining WATER boundary — the honest analogue of Thermal's permanent
``boundary.space`` (fully eliminated only at η_w = 1). Everything CARBON / OXYGEN is
identical to the cabin — Step 4 touches only the WATER plumbing.

**The payload — the crew ``water_store`` becomes REGENERATIVE.** Standalone/cabin, the
store drains at the full intake rate; here the recovered water flows back, so the net
drain drops to ``(1−η_w)·intake`` (closed at η_w = 1). The "it bit" gate is
with-vs-without recovery **plus** a conservation identity: because the buffer's
dynamics and the forced intake are both independent of η_w, the water returned to the
store equals exactly ``η_w`` times the water the open-loop baseline (η_w = 0) sends to
``brine`` — ``water_store_with − water_store_without ≈ η_w · brine_without`` — the crew
analogue of Step 3's offload identity.

**Why the cabin, not the greenhouse (advisor).** Closure does **not** require unifying
the biosphere's transpiration with the cabin humidity: the biosphere's internal water
ring (``soil_water → water_vapor → condensate → soil_water``) is already closed and
sealed independently (it needs nothing from the cabin), and the crew loop closes
independently the moment recovery is added. So station WATER conserves as (closed
biosphere ring) + (crew loop closed up to brine); unifying the two humid-air stocks is a
*fidelity refinement* deferred (Step 7 / out of scope), **not** a closure requirement.
Building on the cabin also keeps the biosphere out of the assembly — the biosphere is
Euler-locked by its freeze, so only here can the RK4 ≢ Euler cross-check run: adding
recovery makes ``water_store`` **state-dependent** (its inflow ∝ the buffer level),
breaking the forced RK4 ≡ Euler bit-identity the cabin stores had — the "it earned its
keep" signal.

Pure stdlib only in the spine; the crew split fractions + ECLSS control coefficients +
water-recovery params load via the respective loaders.
"""

from collections.abc import Mapping
from dataclasses import dataclass

from domains.crew.flows import CrewParams, WaterBalance
from domains.crew.stocks import (
    FECAL_WASTE,
    FOOD_STORE,
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
    O2_SUPPLY,
    cabin_h2o_stock,
)
from domains.eclss.system import CO2_SCRUBBER, CONDENSER, O2_MAKEUP
from simcore import boundary
from simcore.boundary import BOUNDARY_DOMAIN
from simcore.environment import SourceResolver
from simcore.flow import Flow
from simcore.ids import FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock
from station.cabin import (
    CO2_COMPOSITION,
    CREW_RESPIRATION,
    O2_COMPOSITION,
    cabin_resolver,
)
from station.flows import CrewRespiration, WaterRecovery, WaterRecoveryParams
from station.scenario import WATER_RECOVERY_SCENARIO, CabinScenario

# The recovered-water buffer POOL (the crew analogue of the biosphere's ``condensate``):
# where the Condenser product + the crew urine collect before the processor returns them
# to the store. WATER, ECLSS domain (life-support equipment, like ``cabin_h2o``).
RECOVERED_WATER: StockId = StockId("eclss.recovered_water")

# The unrecoverable-water boundary sink (the ``(1−η_w)`` brine leg of WaterRecovery) —
# the one remaining WATER terminal sink, Thermal's permanent ``boundary.space`` twin,
# eliminated only at η_w = 1.
BRINE: StockId = StockId("boundary.brine")

# The station-owned water-recovery flow id (ASCII so str sort == future Rust byte sort,
# #15). The other five flows reuse their domains' canonical ids.
WATER_RECOVERY: FlowId = FlowId("station.water_recovery")


@dataclass(frozen=True)
class WaterRecoverySteadyState:
    """The emergent WATER steady states (see :func:`water_recovery_steady_state`).

    ``water_store`` has **no** steady state — it is a net consumer even with recovery
    (it drains at ``water_store_drain_rate`` once the buffer has filled), the honest
    "regenerative up to η_w, closed only at η_w = 1" statement. The two POOLs that *do*
    have attractors are ``cabin_h2o`` and ``recovered_water``.
    """

    cabin_h2o: float
    recovered_water: float
    # kg/s — the net regenerative drain of ``water_store`` at steady buffer level,
    # ``(1−η_w)·intake`` (0 at η_w = 1: fully closed).
    water_store_drain_rate: float


def _gas_pool(
    stock_id: StockId,
    quantity: Quantity,
    amount: float,
    composition: Mapping[Quantity, float],
) -> Stock:
    """A composition-carrying cabin POOL (``cabin_o2`` / ``cabin_co2``).

    The ECLSS ``cabin_*_stock`` constructors build single-quantity pools; the coupled
    loop needs the gas-phase composition (``{O:2}`` / ``{C:1,O:2}``). ``boundary.py`` /
    ``eclss.stocks`` take no composition arg and extending them is a core change, so the
    station builds these inline — the ``station.cabin`` rationale, re-declared here to
    keep ``cabin.py`` untouched (the ``station.greenhouse`` precedent).
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

    Both must carry the gas composition or the scrubber (2 O per CO₂) / the makeup (2 O
    per O₂) unbalances OXYGEN at the boundary — the ``station.cabin`` rationale.
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


def _recovered_water_pool(amount: float = 0.0) -> Stock:
    """The ``recovered_water`` buffer POOL (WATER, ECLSS domain; starts empty).

    A single-quantity WATER POOL — a plain buffer, no composition. Guarded by
    arbitration (WaterRecovery draws it down, donor-controlled ``k_rec·dt < 1``); never
    zeroed-with-loss (POOLs are not extinction-eligible).
    """
    return Stock(
        id=RECOVERED_WATER,
        domain=ECLSS_DOMAIN,
        quantity=Quantity.WATER,
        unit=canonical_unit(Quantity.WATER),
        amount=amount,
        kind=StockKind.POOL,
    )


def build_water_recovery(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    recovery_params: WaterRecoveryParams,
    scenario: CabinScenario = WATER_RECOVERY_SCENARIO,
) -> tuple[State, Registry]:
    """Assemble the coupled crew water-recovery cabin (``State`` + ``Registry``).

    Ten stocks and six flows — the Step-2 cabin with its two WATER disposal sinks
    replaced by the recovery loop. The stocks: the two gas POOLs ``cabin_o2`` /
    ``cabin_co2`` + the single-quantity ``cabin_h2o`` POOL, the two finite crew POOLs
    ``food_store`` / ``water_store``, the ``o2_supply`` source + ``co2_removed`` /
    ``fecal_waste`` sinks (all unchanged from the cabin), **plus** the new
    ``recovered_water`` buffer POOL and the ``brine`` sink — and **no**
    ``humidity_condensate`` / ``urine`` sinks (re-pointed into ``recovered_water``). The
    flows: ``CrewRespiration`` (unchanged) + ``WaterBalance`` with its ``urine`` leg
    re-pointed to ``recovered_water`` + the three ECLSS control loops (``CO2Scrubber`` /
    ``O2Makeup`` unchanged; ``Condenser`` with its sink re-pointed to the buffer) + the
    station-owned ``WaterRecovery``. **No loss-sinks** (no POPULATION stock). The
    ``Registry`` re-sorts flows by id, so build order is inert.

    CARBON / OXYGEN are bit-identical to the cabin (Step 4 touches only WATER). ``η_w``
    and ``k_rec`` ride on ``recovery_params``; ``recovery_efficiency = 0`` builds the
    **open-loop baseline** (the store gets nothing back — the "it bit" gate's arm).
    """
    cabin_o2 = _gas_pool(CABIN_O2, Quantity.OXYGEN, scenario.cabin_o2_0, O2_COMPOSITION)
    cabin_co2 = _gas_pool(
        CABIN_CO2, Quantity.CARBON, scenario.cabin_co2_0, CO2_COMPOSITION
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
    fecal_waste = boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0)
    recovered_water = _recovered_water_pool()
    brine = boundary.sink(BRINE, Quantity.WATER, 0.0)
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
            fecal_waste,
            recovered_water,
            brine,
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
            crew_humidity=CABIN_H2O,
            urine=RECOVERED_WATER,  # the seam: urine collects in the recovery buffer
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
            humidity_condensate=RECOVERED_WATER,  # the seam: condensate → the buffer
            params=eclss_params,
        ),
        O2Makeup(
            O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=CABIN_O2, params=eclss_params
        ),
        WaterRecovery(
            WATER_RECOVERY,
            0,
            recovered_water=RECOVERED_WATER,
            water_store=WATER_STORE,  # the seam: recovered water returns to the store
            brine=BRINE,
            params=recovery_params,
        ),
    ]
    return state, Registry(flows, stocks)


def water_recovery_resolver(
    scenario: CabinScenario = WATER_RECOVERY_SCENARIO,
) -> SourceResolver:
    """The forcing — the two constant crew intake rates (reused from the cabin).

    ``WaterRecovery`` reads a *stock* (``recovered_water``), not ``env``, so it adds no
    forcing var; the two crew intake rates are exactly the cabin's, so the resolver is
    ``station.cabin.cabin_resolver``.
    """
    return cabin_resolver(scenario)


def water_recovery_steady_state(
    crew_params: CrewParams,
    eclss_params: EclssParams,
    recovery_params: WaterRecoveryParams,
    scenario: CabinScenario = WATER_RECOVERY_SCENARIO,
) -> WaterRecoverySteadyState:
    """The emergent WATER steady states + the net ``water_store`` drain rate.

    At steady state the ``recovered_water`` inflow is the condenser throughput
    (``k_cond·cabin_h2o = f_ins·intake`` at balance) plus the urine ``(1−f_ins)·intake``
    — i.e. the **whole** water intake — so:

      * ``cabin_h2o_eq  = (f_ins·water_intake) / k_cond``  (condenser balances humidity)
      * ``recovered_eq  = water_intake / k_rec``           (processor balances inflow)
      * ``water_store``  drains at ``(1−η_w)·water_intake`` (the buffer returns
        ``η_w·water_intake``, the regenerative offset; 0 at η_w = 1 ⇒ fully closed).

    ``recovered_eq`` is independent of η_w (η_w only splits the processor's *output* —
    the fact that makes the with-vs-without-recovery conservation identity exact).
    Emergent from the crew load + the loaded params, never stored.
    """
    humidity_rate = crew_params.insensible_water_fraction * scenario.water_intake_rate
    return WaterRecoverySteadyState(
        cabin_h2o=humidity_rate / eclss_params.condense_rate,
        recovered_water=scenario.water_intake_rate / recovery_params.recovery_rate,
        water_store_drain_rate=(
            (1.0 - recovery_params.recovery_efficiency) * scenario.water_intake_rate
        ),
    )
