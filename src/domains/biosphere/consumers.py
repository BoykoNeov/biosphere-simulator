"""The consumers compartment builder (P3 Step 7) — the minimal herbivore.

The optional stretch leaf, declared empty since the Step-7 hierarchy node was added and
populated here when a consumer scenario is run. Owns the one ``consumer_carbon``
POPULATION (the herbivore biomass — the ``microbial_carbon`` analogue one trophic level
up) and the three trophic flows that compose its closed sub-loop:

```
leaf_c (plants) --Grazing--> consumer_carbon (consumers)            # graze live leaf
consumer_carbon (consumers) + o2_pool (atmos) --ConsumerRespiration--> carbon_pool
consumer_carbon (consumers) --ConsumerMortality--> litter_carbon (soil)   # death→litter
```

All three are **cross-compartment** fluxes — grazing draws from the plants' ``leaf_c``,
respiration returns CO₂ to the atmosphere's ``carbon_pool`` consuming its ``o2_pool``,
and mortality routes the carcass to the soil's ``litter_carbon``. The consumers builder
reads those four ids from the **catalog** (stable ids), not the sibling modules (no
builder imports another, P3.3 — the ``soil`` builder reads ``CARBON_POOL``/``O2_POOL``
for microbial respiration the same way).

**Sealed + consumer-enabled only.** The consumer needs the chamber's finite
``carbon_pool``/``o2_pool`` and the soil's in-system ``litter_carbon``, which exist only
when sealed; and it composes onto the closed perennial ecosystem. It returns an empty
``CompartmentBuild`` unless ``scenario.sealed and scenario.consumer``: every
producer-only run (open field, the sealed/perennial chambers) keeps the consumers leaf
empty and its golden byte-identical (the ``water``-open-field precedent), and
``annual_reset`` is plant-only, so the herbivore persists across the annual re-sow.

Pure stdlib + ``simcore`` + ``domains``; param values come from ``loader.py``.
"""

from domains.biosphere.compartments import CONSUMERS
from domains.biosphere.herbivory import (
    ConsumerMortality,
    ConsumerRespiration,
    Grazing,
)
from domains.biosphere.loader import load_herbivory_params
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CARBON_POOL,
    CONSUMER_CARBON,
    LEAF_C,
    LITTER_CARBON,
    O2_POOL,
    ChamberWiring,
    CompartmentBuild,
    organ_stock,
)
from simcore.flow import Flow
from simcore.ids import FlowId
from simcore.state import Stock


def build_consumers(
    scenario: SeasonScenario, wiring: ChamberWiring
) -> CompartmentBuild:
    """Build the consumers compartment (``wiring`` unused — consumer reads catalog ids).

    Sealed + consumer: the ``consumer_carbon`` POPULATION + the ``Grazing`` /
    ``ConsumerRespiration`` / ``ConsumerMortality`` flows. Otherwise empty — the
    consumer
    needs the chamber pools (``carbon_pool``/``o2_pool``) and the in-system
    ``litter_carbon``, which exist only when sealed, so the leaf holds no stocks/flows
    in
    a producer-only run.
    """
    del wiring  # consumer reads its catalog ids, not the wiring (P3.3)
    if not (scenario.sealed and scenario.consumer):
        return CompartmentBuild(stocks=(), flows=(), aux=(), shared={})
    params = load_herbivory_params()
    stocks: tuple[Stock, ...] = (
        organ_stock(CONSUMER_CARBON, CONSUMERS, scenario.consumer_c0),
    )
    flows: tuple[Flow, ...] = (
        Grazing(
            FlowId("biosphere.grazing"),
            0,
            leaf_c=LEAF_C,
            consumer_carbon=CONSUMER_CARBON,
            params=params,
        ),
        ConsumerRespiration(
            FlowId("biosphere.consumer_respiration"),
            0,
            consumer_carbon=CONSUMER_CARBON,
            co2_pool=CARBON_POOL,
            o2_pool=O2_POOL,
            params=params,
            air_mol=scenario.chamber_air_mol,
        ),
        ConsumerMortality(
            FlowId("biosphere.consumer_mortality"),
            0,
            consumer_carbon=CONSUMER_CARBON,
            litter_carbon=LITTER_CARBON,
            params=params,
        ),
    )
    return CompartmentBuild(stocks=stocks, flows=flows, aux=(), shared={})
