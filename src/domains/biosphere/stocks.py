"""The biosphere stock-id catalog + compartment-composition types (P3.2).

The **shared interface** every compartment builder reads: the canonical ``StockId``s,
the forcing-var-name constants the flows resolve through ``env.get`` (#16), the
``STOCK_DOMAIN`` declared partition spec, the two small composition types
(:class:`ChamberWiring`, :class:`CompartmentBuild`), the :func:`chamber_wiring` factory,
and the shared :func:`organ_stock` / :func:`pool_stock` constructors.

This is *not* "a compartment" — reading an id here does **not** violate the P3.3 rule
that no builder imports another (the cross-compartment ids a flow names travel either
through this catalog, for stable ids, or through :class:`ChamberWiring`, for the
sealed-dependent ones). Each builder stamps its stocks' ``domain`` as a **literal** leaf
(``PLANTS`` / ``SOIL`` / ``ATMOSPHERE``) — domain assignment is structural — so the old
``season._stock_domain`` lookup is gone; ``STOCK_DOMAIN`` is **retained** here purely as
the declared-partition spec that ``test_compartments`` binds the literal stamps against
(the drift guard).

Imports only ``simcore`` + ``compartments`` (the leaf ``DomainId``s) — no builder, no
``season``. Pure stdlib otherwise.
"""

from collections.abc import Mapping
from dataclasses import dataclass

from domains.biosphere.compartments import (
    ATMOSPHERE,
    CONSUMERS,
    PLANTS,
    SOIL,
    WATER,
)
from simcore.auxiliary import AuxProcess
from simcore.flow import Flow
from simcore.ids import DomainId, StockId
from simcore.quantities import Quantity, StockKind, UnitLabel, canonical_unit
from simcore.state import Stock

# --- stock ids --------------------------------------------------------------
# Ids are byte-identical to Phase 1/2 (the Phase-3 relabel moved only the ``domain``
# label, never an id — so every float reduction stays bit-stable, #15).
LEAF_C: StockId = StockId("biosphere.leaf_c")
STEM_C: StockId = StockId("biosphere.stem_c")
ROOT_C: StockId = StockId("biosphere.root_c")
STORAGE_C: StockId = StockId("biosphere.storage_c")
SOIL_WATER: StockId = StockId("biosphere.soil_water")
SOIL_N: StockId = StockId("biosphere.soil_n")
PLANT_N: StockId = StockId("biosphere.plant_n")
CO2_ATMOS: StockId = StockId("boundary.co2_atmos")
# Sealed chamber (P2.2/Step 3): the finite chamber atmosphere. ``carbon_pool`` is a
# true CO₂ stock (``{CARBON:1, OXYGEN:2}``) and ``o2_pool`` its O₂ counterpart
# (``{OXYGEN:2}``); gas exchange moves CARBON and OXYGEN through both at PQ=1. (At
# Step 2 it was a single-currency ``{CARBON:1}`` draw-down with no O₂; Step 3 promotes
# it and closes the gas loop — respiration returns CO₂ to the pool, not a boundary.)
CARBON_POOL: StockId = StockId("biosphere.carbon_pool")
O2_POOL: StockId = StockId("biosphere.o2_pool")
# Sealed chamber decomposer (P2.3/Steps 4–5): senescence feeds a ``litter_carbon``
# POOL (replacing the open field's ``litter_sink`` BOUNDARY, exactly as Step 2 replaced
# the ``co2_atmos`` boundary with the finite ``carbon_pool``); first-order decomposition
# transfers it into ``microbial_carbon`` (a POPULATION, pure carbon, Step 4), and
# microbial respiration (Step 5) burns that biomass back to CO₂ consuming O₂
# (``microbial_C + O₂ → CO₂``) — closing the carbon loop.
LITTER_CARBON: StockId = StockId("biosphere.litter_carbon")
MICROBIAL_CARBON: StockId = StockId("biosphere.microbial_carbon")
# Sealed chamber nitrogen return loop (P2.3/Step 6): senescence sheds plant N into a
# finite ``litter_n`` POOL (the N analogue of ``litter_carbon``); net mineralization
# returns it to ``soil_n``, closing the cycle soil_n → plant_n → litter_n → soil_n that
# Phase 1 fed externally from ``n_source``.
LITTER_N: StockId = StockId("biosphere.litter_n")
# Sealed chamber water cycle (P3.3/Step 3): the two stocks that close the one cycle
# still open. ``water_vapor`` (in ATMOSPHERE) receives transpired water (the canopy
# flux that drained to the ``vapor_sink`` BOUNDARY in Phase 1/2); first-order
# condensation transfers it to ``condensate`` (in the WATER leaf — its first stocks,
# declared empty since P3.1), which first-order recycling returns to ``soil_water``,
# closing the ring soil → atmosphere → water → soil with no boundary crossing.
WATER_VAPOR: StockId = StockId("biosphere.water_vapor")
CONDENSATE: StockId = StockId("biosphere.condensate")
# Sealed chamber minimal consumer (P3 Step 7): the one herbivore biomass pool, a pure
# carbon POPULATION (the ``microbial_carbon`` analogue, one trophic level up). Grazing
# transfers live ``leaf_c`` into it; consumer respiration burns it back to CO₂
# (consuming
# O₂); mortality routes it to ``litter_carbon`` (death-to-litter, P3.4 — never the
# loss-sink). Sealed + consumer-enabled only.
CONSUMER_CARBON: StockId = StockId("biosphere.consumer_carbon")
CO2_RESP: StockId = StockId("boundary.co2_resp")
VAPOR_SINK: StockId = StockId("boundary.vapor_sink")
LITTER_SINK: StockId = StockId("boundary.litter_sink")
WATER_SOURCE: StockId = StockId("boundary.water_source")
N_SOURCE: StockId = StockId("boundary.n_source")

# --- compartment assignment (Phase-3 P3.1) — the declared partition spec -----
# Each *modeled* biosphere stock maps to its leaf compartment. After the Step-2 refactor
# this is no longer the *source* of the domain stamp (builders stamp the leaf literal
# directly); it is **retained** as the declared-partition spec that
# ``test_compartments.test_relabel_partitions_modeled_stocks_into_leaves`` binds the
# literal stamps against (``built.domain == STOCK_DOMAIN[sid]`` per modeled stock).
# Boundary stocks are absent (``boundary.source``/``sink``/``loss_sinks`` stamp them
# ``domain="boundary"``).
STOCK_DOMAIN: dict[StockId, DomainId] = {
    LEAF_C: PLANTS,
    STEM_C: PLANTS,
    ROOT_C: PLANTS,
    STORAGE_C: PLANTS,
    PLANT_N: PLANTS,
    SOIL_WATER: SOIL,
    SOIL_N: SOIL,
    LITTER_CARBON: SOIL,
    LITTER_N: SOIL,
    MICROBIAL_CARBON: SOIL,  # decomposer biomass lives in the soil compartment
    CARBON_POOL: ATMOSPHERE,
    O2_POOL: ATMOSPHERE,
    WATER_VAPOR: ATMOSPHERE,  # transpired vapor (the water cycle's atmosphere leg)
    CONDENSATE: WATER,  # the WATER leaf's first stock (recovered condensate)
    CONSUMER_CARBON: CONSUMERS,  # herbivore biomass (the CONSUMERS leaf's one stock)
}

# --- forcing var names (resolved through env.get, #16) ----------------------
PAR_VAR = "par"
CI_VAR = "ci"
TEMP_VAR = "temp"
DAYLENGTH_VAR = "daylength_s"
RN_VAR = "net_radiation"
VPD_VAR = "vpd"
IRRIGATION_VAR = "irrigation"
FERTILIZATION_VAR = "fertilization"
SOIL_WATER_VAR = (
    "soil_water"  # f_water reads soil_water as a shared sibling stock (#16)
)
# Sealed chamber (P2.2): FvCB reads the live carbon pool amount as a shared stock (#16)
# and derives Ci from it (chamber.ci_from_co2_pool) — the draw-down feedback seam.
CO2_POOL_VAR = "co2_pool"
THERMAL_TIME = "thermal_time"


# --- compartment-composition types ------------------------------------------


@dataclass(frozen=True)
class CompartmentBuild:
    """One compartment's contribution to the assembled season (P3.2).

    A pure builder returns this: its own stocks, flows, aux processes, and the resolver
    ``shared``-map entries (forcing-var → live stock, #16) it owns. ``build_season``
    unions the parts (Registry re-sorts flows by id, so union order is inert) and
    ``weather_resolver`` merges the ``shared`` maps — one source of truth.
    """

    stocks: tuple[Stock, ...]
    flows: tuple[Flow, ...]
    aux: tuple[AuxProcess, ...]
    shared: Mapping[str, StockId]


@dataclass(frozen=True)
class ChamberWiring:
    """The handful of stock ids whose identity depends on ``sealed``, computed once.

    The open-vs-sealed difference reduces to these ids, threaded into flows that live in
    different compartments. Consumed **entirely by plants** (the carbon-budget
    flows + ``Senescence``); the **atmosphere** / **soil** builders *build the stock
    objects* these ids point at — the shared-stock interface (P3.3) made
    concrete. Stable cross-compartment targets with a fixed id when they exist
    (e.g. ``NitrogenSenescence → litter_n``) are read from the catalog, not here,
    to keep this lean.
    """

    carbon_source: StockId  # CARBON_POOL (sealed) | CO2_ATMOS (open)
    resp_sink: StockId  # CARBON_POOL (sealed, == source) | CO2_RESP (open)
    o2_pool: StockId | None  # O2_POOL (sealed) | None (open)
    litter_carbon_target: StockId  # LITTER_CARBON (sealed) | LITTER_SINK (open)
    vapor_target: StockId  # WATER_VAPOR (sealed, closed loop) | VAPOR_SINK (open)


def chamber_wiring(sealed: bool) -> ChamberWiring:
    """Select the sealed-dependent cross-compartment ids (a pure id selection).

    Sealed: gas exchange draws from and returns to the one finite ``carbon_pool``
    (``source == sink``), an ``o2_pool`` balances OXYGEN, senescence feeds the finite
    ``litter_carbon`` POOL, and transpiration targets the in-system ``water_vapor``
    stock that closes the water cycle (P3.3/Step 3). Open: unclamped ``co2_atmos``
    source + separate ``co2_resp`` sink, no O₂ pool, senescence sheds to the
    ``litter_sink`` BOUNDARY, and transpiration drains to the ``vapor_sink`` BOUNDARY.

    ``vapor_target`` flips to ``WATER_VAPOR`` (sealed) | ``VAPOR_SINK`` (open) — exactly
    the ``carbon_source`` / ``litter_carbon_target`` pattern. ``Transpiration`` reads
    its sink id off the wiring (the field name ``vapor_sink`` is kept, so the open flow
    object is byte-identical), so the same canopy flux drains to a boundary (open) or
    feeds the closed loop (sealed) with no change to the flow class.
    """
    return ChamberWiring(
        carbon_source=CARBON_POOL if sealed else CO2_ATMOS,
        resp_sink=CARBON_POOL if sealed else CO2_RESP,
        o2_pool=O2_POOL if sealed else None,
        litter_carbon_target=LITTER_CARBON if sealed else LITTER_SINK,
        vapor_target=WATER_VAPOR if sealed else VAPOR_SINK,
    )


# --- shared stock constructors ----------------------------------------------
# Builders stamp the ``domain`` as a leaf literal (PLANTS / SOIL / ATMOSPHERE); these
# factories take it explicitly. Stocks with non-default element composition (the CO₂/O₂
# pools) are built directly in their compartment builder, not through these.


def organ_stock(stock_id: StockId, domain: DomainId, amount: float) -> Stock:
    """A POPULATION CARBON organ pool (extinction-eligible, ``threshold = 0``)."""
    return Stock(
        id=stock_id,
        domain=domain,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POPULATION,
        extinction_threshold=0.0,
    )


def pool_stock(
    stock_id: StockId,
    domain: DomainId,
    quantity: Quantity,
    unit: UnitLabel,
    amount: float,
) -> Stock:
    """A single-currency POOL stock (default ``{quantity: 1.0}`` composition)."""
    return Stock(
        id=stock_id,
        domain=domain,
        quantity=quantity,
        unit=unit,
        amount=amount,
        kind=StockKind.POOL,
    )
