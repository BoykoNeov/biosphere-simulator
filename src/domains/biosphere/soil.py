"""The soil compartment builder (P3.2) — water, nitrogen, and the decomposer pools.

Owns ``soil_water`` / ``soil_n`` (the rooting-zone POOLs the plant draws from), the
``water_source`` / ``n_source`` boundary supplies, and — in the sealed chamber — the
decomposer sub-loop (``litter_carbon`` POOL, ``microbial_carbon`` POP, ``litter_n``
POOL). Drives them with ``Irrigation`` / ``Fertilization`` (always) and, sealed,
``Decomposition`` / ``MicrobialRespiration`` / ``Mineralization``.

``MicrobialRespiration`` is a cross-compartment flux — it burns microbial C back to
CO₂ in the atmosphere's ``carbon_pool`` consuming its ``o2_pool``; soil reads those two
ids from the **catalog** (stable ids), not the atmosphere module (no builder imports
another, P3.3). Owns the ``soil_water`` shared-map entry (f_water reads it as a live
sibling stock, #16).

Pure stdlib + ``simcore`` + ``domains``; param values come from ``loader.py``.
"""

from domains.biosphere.compartments import SOIL
from domains.biosphere.decomposition import Decomposition
from domains.biosphere.loader import (
    load_decomposition_params,
    load_microbial_respiration_params,
    load_mineralization_params,
)
from domains.biosphere.microbial_respiration import MicrobialRespiration
from domains.biosphere.mineralization import Mineralization
from domains.biosphere.nitrogen import Fertilization
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CARBON_POOL,
    FERTILIZATION_VAR,
    IRRIGATION_VAR,
    LITTER_CARBON,
    LITTER_N,
    MICROBIAL_CARBON,
    N_SOURCE,
    O2_POOL,
    SOIL_N,
    SOIL_WATER,
    SOIL_WATER_VAR,
    WATER_SOURCE,
    ChamberWiring,
    CompartmentBuild,
    organ_stock,
    pool_stock,
)
from domains.biosphere.transpiration import Irrigation
from simcore import boundary
from simcore.flow import Flow
from simcore.ids import FlowId, StockId
from simcore.quantities import Quantity, canonical_unit
from simcore.state import Stock


def build_soil(scenario: SeasonScenario, wiring: ChamberWiring) -> CompartmentBuild:
    """Build the soil compartment (``wiring`` unused — soil owns its litter ids)."""
    del wiring  # senescence's litter target (the wiring) is consumed by `plants`
    water = canonical_unit(Quantity.WATER)
    nitrogen = canonical_unit(Quantity.NITROGEN)
    carbon = canonical_unit(Quantity.CARBON)

    stocks: list[Stock] = [
        pool_stock(SOIL_WATER, SOIL, Quantity.WATER, water, scenario.soil_water0),
        pool_stock(SOIL_N, SOIL, Quantity.NITROGEN, nitrogen, scenario.soil_n0),
        boundary.source(WATER_SOURCE, Quantity.WATER, scenario.water_source0),
        boundary.source(N_SOURCE, Quantity.NITROGEN, scenario.n_source0),
    ]
    flows: list[Flow] = [
        Irrigation(
            FlowId("biosphere.irrigation"),
            0,
            water_source=WATER_SOURCE,
            soil_water=SOIL_WATER,
            irrigation_var=IRRIGATION_VAR,
            ground_area=scenario.ground_area,
        ),
        Fertilization(
            FlowId("biosphere.fertilization"),
            0,
            n_source=N_SOURCE,
            soil_n=SOIL_N,
            fertilization_var=FERTILIZATION_VAR,
            ground_area=scenario.ground_area,
        ),
    ]
    if scenario.sealed:
        # The decomposer pools (Steps 4–6). ``litter_carbon`` is a finite POOL fed by
        # senescence (flow lives in plants) and drained by first-order decomposition;
        # ``microbial_carbon`` is a pure-carbon POPULATION the decay deposits into and
        # microbial respiration drains back to CO₂; ``litter_n`` is the N analogue.
        stocks.append(
            pool_stock(
                LITTER_CARBON, SOIL, Quantity.CARBON, carbon, scenario.litter_carbon0
            )
        )
        stocks.append(organ_stock(MICROBIAL_CARBON, SOIL, 0.0))
        stocks.append(pool_stock(LITTER_N, SOIL, Quantity.NITROGEN, nitrogen, 0.0))
        # The decomposer (Step 4): first-order decay litter_carbon → microbial_carbon.
        flows.append(
            Decomposition(
                FlowId("biosphere.decomposition"),
                0,
                litter_carbon=LITTER_CARBON,
                microbial_carbon=MICROBIAL_CARBON,
                params=load_decomposition_params(),
            )
        )
        # Microbial respiration (Step 5): microbial_C + O₂ → CO₂ — the cross-compartment
        # gas flux closing the carbon loop and the chamber's decomposer O₂ sink. The
        # CO₂/O₂ pools are the atmosphere's (read from the catalog, P3.3).
        flows.append(
            MicrobialRespiration(
                FlowId("biosphere.microbial_respiration"),
                0,
                microbial_carbon=MICROBIAL_CARBON,
                co2_pool=CARBON_POOL,
                o2_pool=O2_POOL,
                params=load_microbial_respiration_params(),
                air_mol=scenario.chamber_air_mol,
            )
        )
        # Net mineralization (Step 6): litter_n → soil_n, closing the N return loop with
        # the plants-owned ``NitrogenSenescence`` (plant_n → litter_n).
        flows.append(
            Mineralization(
                FlowId("biosphere.mineralization"),
                0,
                litter_n=LITTER_N,
                soil_n=SOIL_N,
                params=load_mineralization_params(),
            )
        )
    shared: dict[str, StockId] = {SOIL_WATER_VAR: SOIL_WATER}
    return CompartmentBuild(
        stocks=tuple(stocks), flows=tuple(flows), aux=(), shared=shared
    )
