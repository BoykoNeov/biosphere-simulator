"""The water compartment builder (P3.2/P3.3) — the water-recovery loop's recycler.

The fourth leaf compartment, declared empty since P3.1 and populated here by the Step-3
water cycle. Owns the ``condensate`` POOL (the recovered water held between the
condenser and the root zone) and the ``Recycling`` flow that returns it to the soil's
``soil_water`` — the closing leg of the ring:

```
soil_water (soil) --Transpiration--> water_vapor (atmosphere)   # plants-owned
water_vapor (atmosphere) --Condensation--> condensate (water)   # atmosphere-owned
condensate (water) --Recycling--> soil_water (soil)             # water-owned (here)
```

``Recycling`` is a cross-compartment flux — it returns water to the soil's
``soil_water``; water reads that id from the **catalog** (a stable id), not the soil
module (no builder imports another, P3.3). Sealed-chamber only: in the open field the
water cycle is not closed (transpiration drains to a ``vapor_sink`` BOUNDARY, irrigation
refills from a ``water_source`` BOUNDARY — both soil/plants-owned), so this builder
returns an empty ``CompartmentBuild`` and the ``water`` leaf stays empty (the open
golden is byte-identical).

Pure stdlib + ``simcore`` + ``domains``; param values come from ``loader.py``.
"""

from domains.biosphere.compartments import WATER
from domains.biosphere.loader import load_water_cycle_params
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CONDENSATE,
    SOIL_WATER,
    ChamberWiring,
    CompartmentBuild,
    pool_stock,
)
from domains.biosphere.water_cycle import Recycling
from simcore.flow import Flow
from simcore.ids import FlowId
from simcore.quantities import Quantity, canonical_unit
from simcore.state import Stock


def build_water(scenario: SeasonScenario, wiring: ChamberWiring) -> CompartmentBuild:
    """Build the water compartment (``wiring`` unused — water owns its catalog ids).

    Sealed: the ``condensate`` POOL + the ``Recycling`` flow
    (``condensate → soil_water``).
    Open: empty — the water cycle is not closed, so the leaf holds no stocks/flows.
    """
    del wiring  # the water cycle's sealed-dependent ids are catalog-stable, not wiring
    if not scenario.sealed:
        return CompartmentBuild(stocks=(), flows=(), aux=(), shared={})
    water = canonical_unit(Quantity.WATER)
    stocks: tuple[Stock, ...] = (
        pool_stock(CONDENSATE, WATER, Quantity.WATER, water, scenario.condensate0),
    )
    flows: tuple[Flow, ...] = (
        Recycling(
            FlowId("biosphere.recycling"),
            0,
            condensate=CONDENSATE,
            soil_water=SOIL_WATER,
            params=load_water_cycle_params(),
        ),
    )
    return CompartmentBuild(stocks=stocks, flows=flows, aux=(), shared={})
