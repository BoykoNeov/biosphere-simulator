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
  entry so FvCB derives Ci from the live pool (#16, the draw-down feedback).
* **Open field** (Phase 1): the unclamped ``co2_atmos`` BOUNDARY source + a separate
  ``co2_resp`` BOUNDARY sink. Single-currency CARBON; the loop is open; no shared entry.

Pure stdlib + ``simcore`` + ``domains`` (no builder imports another, P3.3).
"""

from domains.biosphere.compartments import ATMOSPHERE
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CARBON_POOL,
    CO2_ATMOS,
    CO2_POOL_VAR,
    CO2_RESP,
    O2_POOL,
    ChamberWiring,
    CompartmentBuild,
)
from simcore import boundary
from simcore.ids import StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import Stock


def build_atmosphere(
    scenario: SeasonScenario, wiring: ChamberWiring
) -> CompartmentBuild:
    """Build the atmosphere compartment's stocks (no flows/aux; ``wiring`` unused here).

    The carbon-budget flows moving gas through these pools are owned by ``plants`` and
    reference the pool ids via ``wiring``; atmosphere only stands up the stock objects.
    """
    del wiring  # the gas-exchange flows (which consume the wiring) live in `plants`
    carbon = canonical_unit(Quantity.CARBON)
    oxygen = canonical_unit(Quantity.OXYGEN)
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
        )
        # FvCB reads the live carbon pool as the shared ``co2_pool`` var (#16).
        shared: dict[str, StockId] = {CO2_POOL_VAR: CARBON_POOL}
    else:
        stocks = (
            boundary.source(CO2_ATMOS, Quantity.CARBON, scenario.co2_atmos0),
            boundary.sink(CO2_RESP, Quantity.CARBON),
        )
        shared = {}
    return CompartmentBuild(stocks=stocks, flows=(), aux=(), shared=shared)
