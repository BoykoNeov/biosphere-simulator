"""The plants compartment builder (P3.2) — the producer: organs, gas exchange, uptake.

Owns the four organ carbon pools + ``plant_n``, and — in the open field — the canopy
water sink (``vapor_sink``) + the ``litter_sink`` BOUNDARY senescence sheds to (sealed
closes both loops, so neither boundary is built). Drives the
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
    crop_param_set,
    load_allocation_params,
    load_canopy_params,
    load_nitrogen_params,
    load_phenology_params,
    load_photoperiod_params,
    load_photosynthesis_params,
    load_respiration_params,
    load_root_depth_params,
    load_senescence_params,
    load_stem_reserve_params,
    load_transpiration_params,
    load_vernalization_params,
)
from domains.biosphere.mineralization import NitrogenSenescence
from domains.biosphere.nitrogen import NitrogenUptake
from domains.biosphere.phenology import (
    DroughtDevelopmentParams,
    ThermalTimeAccumulation,
    VernalizationAccumulation,
)
from domains.biosphere.root_depth import RootDepthExtension
from domains.biosphere.scenario import SeasonScenario
from domains.biosphere.stem_reserves import StemRemobilization
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
    ROOTED_DEPTH,
    SOIL_N,
    SOIL_WATER,
    SOIL_WATER_VAR,
    STEM_C,
    STEM_RESERVE_C,
    STORAGE_C,
    SUBSOIL_WATER,
    TEMP_VAR,
    THERMAL_TIME,
    VAPOR_SINK,
    VERNALIZATION_DAYS,
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
    crop = crop_param_set(scenario.crop)
    return CarbonContext(
        leaf_c=LEAF_C,
        stem_c=STEM_C,
        root_c=ROOT_C,
        par_var=PAR_VAR,
        ci_var=CI_VAR,
        temp_var=TEMP_VAR,
        soil_water_var=SOIL_WATER_VAR,
        wssg=scenario.wssg,
        rooted_depth_aux=ROOTED_DEPTH,
        soil_extractable_water=scenario.soil_extractable_water,
        plant_n=PLANT_N,
        photo=load_photosynthesis_params(crop.paths["photosynthesis"]),
        canopy=load_canopy_params(crop.paths["canopy"]),
        resp=load_respiration_params(crop.paths["respiration"]),
        nitro=load_nitrogen_params(crop.paths["nitrogen"]),
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
    # The crop's param files (``None`` → the frozen winter-wheat reference, so every
    # frozen scenario reads exactly the files it always did).
    crop = crop_param_set(scenario.crop)
    # Loaded ONCE and shared by the carbon Senescence flow and the N-shedding flow: the
    # two are legs of a single physical event, so they must read identical rdr_* values.
    sen_params = load_senescence_params(crop.paths["senescence"])
    pheno = load_phenology_params(crop.paths["phenology"])
    # The stem-reserve params load ONLY when the crop has the mechanism, so a crop that
    # turns it off never even reads the file it would have fallen back to (the
    # ``vernalization`` precedent). ``None`` then threads through as the inert default
    # on ``Allocation`` and as "build no drain flow" below.
    reserve_params = (
        load_stem_reserve_params(crop.paths["stem_reserves"])
        if scenario.stem_reserves
        else None
    )

    stocks: list[Stock] = [
        organ_stock(LEAF_C, PLANTS, scenario.leaf_c0),
        organ_stock(STEM_C, PLANTS, scenario.stem_c0),
        organ_stock(ROOT_C, PLANTS, scenario.root_c0),
        organ_stock(STORAGE_C, PLANTS, scenario.storage_c0),
        pool_stock(PLANT_N, PLANTS, Quantity.NITROGEN, nitrogen, scenario.plant_n0),
    ]
    if reserve_params is not None:
        # ⚠ Starts EMPTY, and there is no scenario field for it. A seedling carries no
        # shielded starch — the reserve is formed from stem growth, so at sowing there
        # has been none. A settable initial amount would be a number no source gives.
        stocks.append(
            pool_stock(
                STEM_RESERVE_C,
                PLANTS,
                Quantity.CARBON,
                canonical_unit(Quantity.CARBON),
                0.0,
            )
        )
    if not scenario.sealed:
        # Open field: transpiration drains to the vapor BOUNDARY and senescence sheds
        # organ carbon to a boundary sink (both loops are open). Sealed closes them —
        # the water cycle (transpiration → water_vapor, P3.3) and the decomposer
        # (senescence → litter_carbon) — so neither boundary is built (genuine closure).
        stocks.append(boundary.sink(VAPOR_SINK, Quantity.WATER))
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
            alloc=load_allocation_params(crop.paths["allocation"]),
            o2_pool=wiring.o2_pool,
            # The FORMATION half of stem reserves: a share of this flow's own stem leg
            # is deposited as shielded starch instead ([E] §3.2.4). Both fields are the
            # inert defaults when the crop has no reserve, so its legs are byte-for-byte
            # what they were.
            stem_reserve_c=STEM_RESERVE_C if reserve_params is not None else None,
            fstr=(
                reserve_params.remobilizable_fraction
                if reserve_params is not None
                else 0.0
            ),
            # The split's upper end — the SAME number the drain flow below stops
            # on, read from the same params object so the two halves cannot disagree
            # about where the mechanism ends ([E]'s ``FINISH DS = 2.``; the science
            # and the quotes are in stem_reserves.py).
            reserve_cessation_dvs=(
                reserve_params.cessation_dvs if reserve_params is not None else 0.0
            ),
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
            params=sen_params,
            canopy=load_canopy_params(crop.paths["canopy"]),
            ground_area=scenario.ground_area,
        ),
        Transpiration(
            FlowId("biosphere.transpiration"),
            0,
            soil_water=SOIL_WATER,
            vapor_sink=wiring.vapor_target,
            rn_var=RN_VAR,
            vpd_var=VPD_VAR,
            temp_var=TEMP_VAR,
            params=load_transpiration_params(crop.paths["transpiration"]),
            ground_area=scenario.ground_area,
            rooted_depth_aux=ROOTED_DEPTH,
            soil_extractable_water=scenario.soil_extractable_water,
            wssg=scenario.wssg,
        ),
        NitrogenUptake(
            FlowId("biosphere.nitrogen_uptake"),
            0,
            soil_n=SOIL_N,
            plant_n=PLANT_N,
            # Demand-deficit uptake reads biomass: Greenwood's W excludes fibrous roots
            # (leaf+stem+storage), while the deficit applies to f_N's own denominator
            # (leaf+stem+root) — see NitrogenUptake for the measured two-pool delta.
            leaf_c=LEAF_C,
            stem_c=STEM_C,
            root_c=ROOT_C,
            storage_c=STORAGE_C,
            params=nitro,
            ground_area=scenario.ground_area,
            sn_residual=scenario.sn_residual,
            sn_critical=scenario.sn_critical,
            rooted_depth_aux=ROOTED_DEPTH,
            soil_layer_depth=scenario.soil_layer_depth,
        ),
    ]
    if reserve_params is not None:
        # The DRAIN half: the shielded starch moves into the grain between anthesis and
        # maturity (the fill above shares that upper bound, from the same params).
        # Unconditional on ``sealed`` — this is plant-internal, so it needs no chamber
        # wiring and behaves identically in the open field and in a closed chamber.
        flows.append(
            StemRemobilization(
                FlowId("biosphere.stem_remobilization"),
                0,
                stem_reserve_c=STEM_RESERVE_C,
                storage_c=STORAGE_C,
                thermal_time_aux=THERMAL_TIME,
                pheno=pheno,
                params=reserve_params,
            )
        )
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
                # N shedding is now DRIVEN by carbon senescence, so this flow needs the
                # organ stocks and the SAME SenescenceParams object the carbon
                # Senescence flow above was built with — one physical event, two
                # currency legs, and they must not be able to drift apart.
                leaf_c=LEAF_C,
                stem_c=STEM_C,
                root_c=ROOT_C,
                sen_params=sen_params,
                nitro_params=nitro,
                # ...and the SAME canopy/footprint the carbon Senescence flow got, so
                # the mutual-shading term enters both legs of the one physical event.
                canopy=load_canopy_params(crop.paths["canopy"]),
                ground_area=scenario.ground_area,
            )
        )
    # Two accumulators (scope (B) inc. 1): vernalization days accrue from temperature,
    # and thermal time accrues *gated by them* through the vegetative phase. Both read
    # the same forcing; the gating is a snapshot read, so their order is immaterial.
    #
    # Both modifiers are OPTIONAL (the phenology.py seam) and gated by the scenario. The
    # frozen winter wheat keeps both (defaults True → the aux tuple below is unchanged →
    # goldens byte-identical). A DAY-NEUTRAL crop turns both off: no VernalizationAccum-
    # ulation is built, and ThermalTimeAccumulation carries neither modifier, so thermal
    # time advances at the plain degree-day rate (byte-for-byte, per phenology.py). The
    # vern params load only when needed.
    vern = (
        load_vernalization_params(crop.paths["phenology"])
        if scenario.vernalization
        else None
    )
    thermal_time = ThermalTimeAccumulation(
        id=AuxId("biosphere.thermal_time"),
        accumulator=THERMAL_TIME,
        temp_var=TEMP_VAR,
        params=pheno,
        vernalization=vern,
        vernalization_accumulator=(
            VERNALIZATION_DAYS if scenario.vernalization else None
        ),
        photoperiod=(
            load_photoperiod_params(crop.paths["phenology"])
            if scenario.photoperiod
            else None
        ),
        daylength_var=DAYLENGTH_VAR if scenario.photoperiod else None,
        # The THIRD modifier: drought acceleration ([F] Eqn 15.8). Optional on the same
        # pattern — absent ⇒ the rate is byte-for-byte what it was. ⚠ Unlike the other
        # two it is switched by a VALUE (``wssd``) rather than a boolean, because the
        # source publishes the coefficient for only some crops; ``None`` means "[F] does
        # not give one for this crop", which is potato's case.
        drought=(
            DroughtDevelopmentParams(
                wssd=scenario.wssd,
                wssg=scenario.wssg,
                soil_extractable_water=scenario.soil_extractable_water,
                ground_area=scenario.ground_area,
            )
            if scenario.wssd is not None
            else None
        ),
        soil_water=SOIL_WATER if scenario.wssd is not None else None,
        rooted_depth_aux=ROOTED_DEPTH if scenario.wssd is not None else None,
    )
    # The THIRD accumulator (post-roadmap root functional coupling): rooted depth,
    # which gates NitrogenUptake's supply term. Unconditional — unlike
    # vernalization/photoperiod
    # it is not a crop-optional modifier; every crop roots. ⚠ It is bit-identically
    # inert
    # on every frozen scenario (see root_depth.py), so its presence here is what its
    # unit-level pins protect, not any golden.
    root_depth = RootDepthExtension(
        id=AuxId("biosphere.rooted_depth"),
        accumulator=ROOTED_DEPTH,
        thermal_time_aux=THERMAL_TIME,
        temp_var=TEMP_VAR,
        soil_water=SOIL_WATER,
        subsoil_water=SUBSOIL_WATER,
        params=load_root_depth_params(crop.paths["root_depth"]),
        photo=ctx.photo,
        pheno=pheno,
        wssg=scenario.wssg,
        soil_depth=scenario.soil_depth,
        soil_extractable_water=scenario.soil_extractable_water,
        ground_area=scenario.ground_area,
    )
    aux: tuple[AuxProcess, ...] = (thermal_time, root_depth)
    if scenario.vernalization:
        assert vern is not None  # loaded just above when the flag is set
        aux = (
            thermal_time,
            root_depth,
            VernalizationAccumulation(
                id=AuxId("biosphere.vernalization_days"),
                accumulator=VERNALIZATION_DAYS,
                temp_var=TEMP_VAR,
                params=vern,
            ),
        )
    return CompartmentBuild(
        stocks=tuple(stocks), flows=tuple(flows), aux=aux, shared={}
    )
