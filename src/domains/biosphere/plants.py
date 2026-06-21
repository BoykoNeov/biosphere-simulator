"""The plants compartment builder (P3.2) — the producer: organs, gas exchange, uptake.

Owns the four organ carbon pools + ``plant_n``, the canopy water sink (``vapor_sink``),
and — in the open field — the ``litter_sink`` BOUNDARY senescence sheds to. Drives the
plant carbon budget (``Allocation`` + the two respirations, over the shared
:class:`CarbonContext`), ``Senescence``, ``Transpiration``, ``NitrogenUptake``, and —
sealed — ``NitrogenSenescence``; plus the thermal-time aux accumulator.

The compartment that **consumes** :class:`~domains.biosphere.stocks.ChamberWiring`
— the gas source/sink and senescence litter target are sealed-dependent ids that live
in *other* compartments (atmosphere's ``carbon_pool``/``o2_pool``, soil's
``litter_carbon``). Plants reads them through the wiring; the owner builds the
stock. Stable cross-compartment reads (``soil_water``/``soil_n``/``litter_n``) come
from the catalog (P3.3 — no builder imports another).

Pure stdlib + ``simcore`` + ``domains``; param values come from ``loader.py``.
"""

from domains.biosphere.allocation import Senescence
from domains.biosphere.carbon_budget import (
    Allocation,
    CarbonContext,
    GrowthRespiration,
    MaintenanceRespiration,
)
from domains.biosphere.compartments import PLANTS
from domains.biosphere.loader import (
    load_allocation_params,
    load_canopy_params,
    load_mineralization_params,
    load_nitrogen_params,
    load_phenology_params,
    load_photosynthesis_params,
    load_respiration_params,
    load_senescence_params,
    load_transpiration_params,
)
from domains.biosphere.mineralization import NitrogenSenescence
from domains.biosphere.nitrogen import NitrogenUptake
from domains.biosphere.phenology import ThermalTimeAccumulation
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stocks import (
    CI_VAR,
    CO2_POOL_VAR,
    DAYLENGTH_VAR,
    LEAF_C,
    LITTER_N,
    LITTER_SINK,
    PAR_VAR,
    PLANT_N,
    RN_VAR,
    ROOT_C,
    SOIL_N,
    SOIL_WATER,
    SOIL_WATER_VAR,
    STEM_C,
    STORAGE_C,
    TEMP_VAR,
    THERMAL_TIME,
    VAPOR_SINK,
    VPD_VAR,
    ChamberWiring,
    CompartmentBuild,
    organ_stock,
    pool_stock,
)
from domains.biosphere.transpiration import Transpiration
from simcore import boundary
from simcore.auxiliary import AuxId, AuxProcess
from simcore.flow import Flow
from simcore.ids import FlowId
from simcore.quantities import Quantity, canonical_unit
from simcore.state import Stock


def _carbon_context(scenario: SeasonScenario) -> CarbonContext:
    """Build the shared carbon-budget context from the committed crop params.

    Open field (default): Ci is the ``ci_var`` forcing read. Sealed chamber (P2.2): the
    chamber Ci-source triple is wired so Ci derives from the live ``carbon_pool`` (read
    as the shared ``co2_pool`` var, #16) — the draw-down feedback. Plant-internal: the
    carbon-budget flows share it, and ``NitrogenUptake`` reuses its ``nitro`` params.
    """
    return CarbonContext(
        leaf_c=LEAF_C,
        stem_c=STEM_C,
        root_c=ROOT_C,
        par_var=PAR_VAR,
        ci_var=CI_VAR,
        temp_var=TEMP_VAR,
        daylength_var=DAYLENGTH_VAR,
        soil_water_var=SOIL_WATER_VAR,
        sw_wilting=scenario.sw_wilting,
        sw_critical=scenario.sw_critical,
        plant_n=PLANT_N,
        photo=load_photosynthesis_params(),
        canopy=load_canopy_params(),
        resp=load_respiration_params(),
        nitro=load_nitrogen_params(),
        ground_area=scenario.ground_area,
        co2_pool_var=CO2_POOL_VAR if scenario.sealed else None,
        chamber_air_mol=scenario.chamber_air_mol if scenario.sealed else None,
        ci_ratio=scenario.ci_ratio if scenario.sealed else None,
    )


def build_plants(scenario: SeasonScenario, wiring: ChamberWiring) -> CompartmentBuild:
    """Build the plants compartment: organ/N stocks, carbon-budget + uptake flows, aux.

    ``wiring`` supplies the sealed-dependent cross-compartment ids: the gas source the
    photosynthesis/respiration flows draw from (``carbon_source``), the respiration sink
    (``resp_sink``, == source when sealed), the O₂ counterpart (``o2_pool``), and the
    senescence litter target (``litter_carbon_target``).
    """
    nitrogen = canonical_unit(Quantity.NITROGEN)
    ctx = _carbon_context(scenario)
    nitro = ctx.nitro
    pheno = load_phenology_params()

    stocks: list[Stock] = [
        organ_stock(LEAF_C, PLANTS, scenario.leaf_c0),
        organ_stock(STEM_C, PLANTS, scenario.stem_c0),
        organ_stock(ROOT_C, PLANTS, scenario.root_c0),
        organ_stock(STORAGE_C, PLANTS, scenario.storage_c0),
        pool_stock(PLANT_N, PLANTS, Quantity.NITROGEN, nitrogen, scenario.plant_n0),
        boundary.sink(VAPOR_SINK, Quantity.WATER),
    ]
    if not scenario.sealed:
        # Open field: senescence sheds organ carbon to a boundary sink (loop is open).
        stocks.append(boundary.sink(LITTER_SINK, Quantity.CARBON))

    flows: list[Flow] = [
        Allocation(
            FlowId("biosphere.allocation"),
            0,
            ctx=ctx,
            co2_atmos=wiring.carbon_source,
            storage_c=STORAGE_C,
            thermal_time_aux=THERMAL_TIME,
            pheno=pheno,
            alloc=load_allocation_params(),
            o2_pool=wiring.o2_pool,
        ),
        GrowthRespiration(
            FlowId("biosphere.growth_respiration"),
            0,
            ctx=ctx,
            co2_atmos=wiring.carbon_source,
            co2_resp=wiring.resp_sink,
        ),
        MaintenanceRespiration(
            FlowId("biosphere.maintenance_respiration"),
            0,
            ctx=ctx,
            co2_atmos=wiring.carbon_source,
            co2_resp=wiring.resp_sink,
            o2_pool=wiring.o2_pool,
            air_mol=scenario.chamber_air_mol if scenario.sealed else None,
        ),
        Senescence(
            FlowId("biosphere.senescence"),
            0,
            leaf_c=LEAF_C,
            stem_c=STEM_C,
            root_c=ROOT_C,
            litter_sink=wiring.litter_carbon_target,
            params=load_senescence_params(),
        ),
        Transpiration(
            FlowId("biosphere.transpiration"),
            0,
            soil_water=SOIL_WATER,
            vapor_sink=VAPOR_SINK,
            rn_var=RN_VAR,
            vpd_var=VPD_VAR,
            temp_var=TEMP_VAR,
            params=load_transpiration_params(),
            ground_area=scenario.ground_area,
            sw_wilting=scenario.sw_wilting,
            sw_critical=scenario.sw_critical,
        ),
        NitrogenUptake(
            FlowId("biosphere.nitrogen_uptake"),
            0,
            soil_n=SOIL_N,
            plant_n=PLANT_N,
            params=nitro,
            ground_area=scenario.ground_area,
            sn_residual=scenario.sn_residual,
            sn_critical=scenario.sn_critical,
        ),
    ]
    if scenario.sealed:
        # The nitrogen return loop's plant side (Step 6): plant_n → litter_n (in soil).
        # The soil side (Mineralization: litter_n → soil_n) is the soil builder's;
        # both load the same mineralization params (identical values, separate objects).
        flows.append(
            NitrogenSenescence(
                FlowId("biosphere.nitrogen_senescence"),
                0,
                plant_n=PLANT_N,
                litter_n=LITTER_N,
                params=load_mineralization_params(),
            )
        )
    aux: tuple[AuxProcess, ...] = (
        ThermalTimeAccumulation(
            id=AuxId("biosphere.thermal_time"),
            accumulator=THERMAL_TIME,
            temp_var=TEMP_VAR,
            params=pheno,
        ),
    )
    return CompartmentBuild(
        stocks=tuple(stocks), flows=tuple(flows), aux=aux, shared={}
    )
