"""The atmosphere compartment builder (P3.2) — the chamber gas pools.

Owns the carbon/oxygen atmosphere the gas-exchange flows draw from and return to. The
flows themselves live in the **plants** compartment (photosynthesis / respiration are
canopy fluxes); atmosphere *builds the stock objects* those flows name through
:class:`~domains.biosphere.stocks.ChamberWiring` — the shared-stock interface (P3.3)
made concrete. Self-selects its content off ``scenario.sealed``:

* **Sealed chamber** (Step 3): one finite CO₂ POOL ``carbon_pool`` (``{CARBON:1,
  OXYGEN:2}``) — *both* the gas source and the respiration sink (the flows detect
  ``source == sink`` to net the assimilate-respired round trip) — plus an ``o2_pool``
  counterpart (``{OXYGEN:2}``). The gas loop is closed. Owns the ``co2_pool`` shared-map
  entry so FvCB derives Ci from the live pool (#16, the draw-down feedback). Also owns
  the ``water_vapor`` POOL (the water cycle's atmosphere leg, P3.3) and the
  ``Condensation`` flow ``water_vapor → condensate`` — the condenser. ``condensate`` is
  the water compartment's stock, read from the **catalog** (no builder imports another,
  P3.3).
* **Open field** (Phase 1): the unclamped ``co2_atmos`` BOUNDARY source + a separate
  ``co2_resp`` BOUNDARY sink. Single-currency CARBON; the loop is open; no shared entry;
  no water vapor (transpiration drains to a ``vapor_sink`` BOUNDARY instead).

Pure stdlib + ``simcore`` + ``domains`` (no builder imports another, P3.3).
"""

from domains.biosphere.compartments import ATMOSPHERE
from domains.biosphere.loader import load_water_cycle_params
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CARBON_POOL,
    CO2_ATMOS,
    CO2_POOL_VAR,
    CO2_RESP,
    CONDENSATE,
    O2_POOL,
    WATER_VAPOR,
    ChamberWiring,
    CompartmentBuild,
    pool_stock,
)
from domains.biosphere.water_cycle import Condensation
from simcore import boundary
from simcore.flow import Flow
from simcore.ids import FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock


def build_atmosphere(
    scenario: SeasonScenario, wiring: ChamberWiring
) -> CompartmentBuild:
    """Build the atmosphere compartment's stocks + the condenser (``wiring`` unused
    here).

    The carbon-budget flows moving gas through the CO₂/O₂ pools are owned by ``plants``
    and reference the pool ids via ``wiring``; atmosphere only stands up those stock
    objects. The water cycle's atmosphere leg (``water_vapor`` stock + the
    ``Condensation`` flow) *is* atmosphere-owned — it names the water compartment's
    ``condensate`` from the catalog (P3.3). Sealed only.
    """
    del wiring  # the gas-exchange flows (which consume the wiring) live in `plants`
    carbon = canonical_unit(Quantity.CARBON)
    oxygen = canonical_unit(Quantity.OXYGEN)
    water = canonical_unit(Quantity.WATER)
    if scenario.sealed:
        stocks: tuple[Stock, ...] = (
            Stock(
                id=CARBON_POOL,
                domain=ATMOSPHERE,
                quantity=Quantity.CARBON,
                unit=carbon,
                amount=scenario.chamber_co2_mol0,
                kind=StockKind.POOL,
                composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
            ),
            Stock(
                id=O2_POOL,
                domain=ATMOSPHERE,
                quantity=Quantity.OXYGEN,
                unit=oxygen,
                amount=scenario.chamber_o2_mol0,
                kind=StockKind.POOL,
                composition={Quantity.OXYGEN: 2.0},
            ),
            pool_stock(
                WATER_VAPOR, ATMOSPHERE, Quantity.WATER, water, scenario.water_vapor0
            ),
        )
        # FvCB reads the live carbon pool as the shared ``co2_pool`` var (#16).
        shared: dict[str, StockId] = {CO2_POOL_VAR: CARBON_POOL}
        # The condenser: water_vapor → condensate (the water compartment's pool,
        # catalog).
        flows: tuple[Flow, ...] = (
            Condensation(
                FlowId("biosphere.condensation"),
                0,
                water_vapor=WATER_VAPOR,
                condensate=CONDENSATE,
                params=load_water_cycle_params(),
            ),
        )
    else:
        stocks = (
            boundary.source(CO2_ATMOS, Quantity.CARBON, scenario.co2_atmos0),
            boundary.sink(CO2_RESP, Quantity.CARBON),
        )
        shared = {}
        flows = ()
    return CompartmentBuild(stocks=stocks, flows=flows, aux=(), shared=shared)
