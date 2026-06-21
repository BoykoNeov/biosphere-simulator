"""Phase-1 Step-11 tests: the coupled carbon budget (the buffer-dissolution rewiring).

``domains.biosphere.carbon_budget`` dissolves the provisional ``plant_c`` pool and
sources every carbon fate from the unclamped ``co2_atmos`` boundary + the organ pools.
The three budget-coupled flows share **one** :class:`CarbonContext` so they cannot drift
on the gross-assimilation / maintenance / limitation recompute. Layers:

* **The shared ``CarbonContext``**: ``budget`` returns ``(GASS, MRES, available)``
  recomputed from the step-entry snapshot; ``limitation`` forms ``f_water · f_N`` and is
  applied inside ``budget`` (so all three flows are limited identically). Cross-checked
  against the standalone rate laws (``daily_canopy_assimilation`` etc.).
* **The three flows** (carbon-balanced ``FlowResult``s, dt-linear):
  ``Allocation`` (``co2_atmos -> {leaf, stem, root, storage}``, the DVS-split ``DMI``),
  ``GrowthRespiration`` (``co2_atmos -> co2_resp``, ``GRES``), and
  ``MaintenanceRespiration`` (``{co2_atmos(covered), organs(shortfall)} -> co2_resp``,
  the maintenance-first covered/shortfall split — biomass shrinks in a carbon deficit).
* **The structural-agreement + limitation seams**: all three flows derive their flux
  from the same ``(GASS, MRES, available)``; ``f_water``/``f_N`` scale each flow's gross
  assimilation by the identical factor.

The non-limiting operating point (``soil_water`` above the band, ``plant_n`` above the
critical concentration ⇒ ``limitation = 1``) reproduces the Step-6 pinned GRES literal,
so the rewiring is shown to preserve the standalone carbon numbers.
"""

import math

from domains.biosphere.allocation import AllocationParams, PartitionRow, partition
from domains.biosphere.canopy import CanopyParams
from domains.biosphere.carbon_budget import (
    Allocation,
    CarbonContext,
    GrowthRespiration,
    MaintenanceRespiration,
)
from domains.biosphere.nitrogen import NitrogenParams
from domains.biosphere.phenology import PhenologyParams, development_stage
from domains.biosphere.photosynthesis import (
    PhotosynthesisParams,
    daily_canopy_assimilation,
)
from domains.biosphere.respiration import (
    RespirationParams,
    available_for_growth,
    growth_respiration_flux,
    maintenance_respiration_flux,
)
from simcore.environment import SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

# --- committed placeholders (mirror the yamls; same operating point as Step 5/6/9) --
_VCMAX, _JMAX, _ALPHA, _THETA = 100.0, 180.0, 0.3, 0.7
_GAMMA_STAR, _KC, _KO, _O2 = 42.75, 404.9, 278.4, 210.0
_TMIN, _TOPT_LO, _TOPT_HI, _TMAX = 0.0, 15.0, 25.0, 35.0
_SLA_PER_MOL_C, _K = 0.5872044444444445, 0.6
_M_REF, _Q10, _T_REF, _YG = 0.02, 2.0, 25.0, 0.75
_TSUM_ANTHESIS, _TSUM_MATURITY = 1100.0, 750.0
_T_BASE, _T_CAP = 0.0, 30.0
# Nitrogen f_N thresholds (loader-folded kg N / mol C) + a soil-water band (scenario).
_N_RESIDUAL, _N_CRITICAL = 0.001, 0.002
_SW_WILTING, _SW_CRITICAL = 10.0, 30.0

_TABLE = (
    PartitionRow(dvs=0.0, fl=0.55, fs=0.10, fr=0.35, fo=0.00),
    PartitionRow(dvs=1.0, fl=0.30, fs=0.50, fr=0.20, fo=0.00),
    PartitionRow(dvs=2.0, fl=0.00, fs=0.10, fr=0.10, fo=0.80),
)

# stock ids / forcing-var names ----------------------------------------------
_BIO = DomainId("biosphere")
_BND = DomainId("boundary")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_STORAGE_C = StockId("biosphere.storage_c")
_PLANT_N = StockId("biosphere.plant_n")
_SOIL_WATER = StockId("biosphere.soil_water")
_CO2_ATMOS = StockId("boundary.co2_atmos")
_CO2_RESP = StockId("boundary.co2_resp")
_PAR, _CI, _TEMP, _DAYLEN, _SW = "par", "ci", "temp", "daylength_s", "soil_water"
_THERMAL_TIME = "thermal_time"


def _photo() -> PhotosynthesisParams:
    return PhotosynthesisParams(
        vcmax=_VCMAX,
        jmax=_JMAX,
        quantum_yield=_ALPHA,
        theta=_THETA,
        gamma_star=_GAMMA_STAR,
        kc=_KC,
        ko=_KO,
        o2=_O2,
        t_min=_TMIN,
        t_opt_lo=_TOPT_LO,
        t_opt_hi=_TOPT_HI,
        t_max=_TMAX,
    )


def _canopy() -> CanopyParams:
    return CanopyParams(sla_per_mol_c=_SLA_PER_MOL_C, extinction_coef=_K)


def _resp() -> RespirationParams:
    return RespirationParams(
        maintenance_coef=_M_REF,
        q10=_Q10,
        t_ref=_T_REF,
        growth_efficiency=_YG,
        o2_half_saturation=0.001,
    )


def _nitro() -> NitrogenParams:
    return NitrogenParams(
        max_uptake_capacity=1.0,  # unused by f_N (it gates uptake, not photosynthesis)
        n_residual_per_mol_c=_N_RESIDUAL,
        n_critical_per_mol_c=_N_CRITICAL,
    )


def _pheno() -> PhenologyParams:
    return PhenologyParams(
        t_base=_T_BASE,
        t_cap=_T_CAP,
        tsum_anthesis=_TSUM_ANTHESIS,
        tsum_maturity=_TSUM_MATURITY,
    )


def _ctx() -> CarbonContext:
    return CarbonContext(
        leaf_c=_LEAF_C,
        stem_c=_STEM_C,
        root_c=_ROOT_C,
        par_var=_PAR,
        ci_var=_CI,
        temp_var=_TEMP,
        daylength_var=_DAYLEN,
        soil_water_var=_SW,
        sw_wilting=_SW_WILTING,
        sw_critical=_SW_CRITICAL,
        plant_n=_PLANT_N,
        photo=_photo(),
        canopy=_canopy(),
        resp=_resp(),
        nitro=_nitro(),
        ground_area=1.0,
    )


def _state(
    *,
    leaf_c: float = 5.0,
    stem_c: float = 0.0,
    root_c: float = 0.0,
    storage_c: float = 0.0,
    soil_water: float = 100.0,  # >> sw_critical ⇒ f_water = 1 (non-limiting)
    plant_n: float = 1.0,  # high conc ⇒ f_N = 1 (non-limiting)
    thermal_time: float = 0.0,  # DVS = 0 (emergence); set per-test
) -> State:
    carbon = canonical_unit(Quantity.CARBON)

    def organ(sid: StockId, amt: float) -> Stock:
        return Stock(
            id=sid,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=amt,
            kind=StockKind.POPULATION,
            extinction_threshold=0.0,
        )

    def boundary(sid: StockId, amt: float, *, unclamped: bool = False) -> Stock:
        return Stock(
            id=sid,
            domain=_BND,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=amt,
            kind=StockKind.BOUNDARY,
            unclamped=unclamped,
        )

    stocks = {
        _LEAF_C: organ(_LEAF_C, leaf_c),
        _STEM_C: organ(_STEM_C, stem_c),
        _ROOT_C: organ(_ROOT_C, root_c),
        _STORAGE_C: organ(_STORAGE_C, storage_c),
        _PLANT_N: Stock(
            id=_PLANT_N,
            domain=_BIO,
            quantity=Quantity.NITROGEN,
            unit=canonical_unit(Quantity.NITROGEN),
            amount=plant_n,
            kind=StockKind.POOL,
        ),
        _SOIL_WATER: Stock(
            id=_SOIL_WATER,
            domain=_BIO,
            quantity=Quantity.WATER,
            unit=canonical_unit(Quantity.WATER),
            amount=soil_water,
            kind=StockKind.POOL,
        ),
        _CO2_ATMOS: boundary(_CO2_ATMOS, 1.0e9, unclamped=True),
        _CO2_RESP: boundary(_CO2_RESP, 0.0),
    }
    return State(n=0, stocks=stocks, rng_seed=0, aux={_THERMAL_TIME: thermal_time})


def _env(snapshot: State, dt: float, *, par: float = 800.0, temp: float = 20.0):  # noqa: ANN202
    resolver = SourceResolver(
        forcings={
            _PAR: constant(par),
            _CI: constant(400.0),
            _TEMP: constant(temp),
            _DAYLEN: constant(43200.0),
        },
        shared={_SW: _SOIL_WATER},  # f_water reads soil_water as a sibling stock (#16)
    )
    return resolver.bind(snapshot, dt)


# --- expected (GASS, MRES, available) at a state, via the standalone rate laws ----
def _budget(
    *, leaf_c: float, biomass: float, par: float = 800.0, limitation: float = 1.0
):  # noqa: ANN202
    lai = leaf_c * _SLA_PER_MOL_C / 1.0
    gass = daily_canopy_assimilation(
        par,
        lai,
        400.0,
        20.0,
        43200.0,
        params=_photo(),
        canopy=_canopy(),
        ground_area=1.0,
        limitation=limitation,
    )
    mres = maintenance_respiration_flux(biomass, 20.0, params=_resp())
    return gass, mres, available_for_growth(gass, mres)


# --- CarbonContext.budget + limitation --------------------------------------
def test_context_budget_matches_standalone_rate_laws() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    gass, mres, available = _ctx().budget(state, _env(state, 1.0))
    e_gass, e_mres, e_avail = _budget(leaf_c=3.0, biomass=5.0)
    assert math.isclose(gass, e_gass, rel_tol=1e-12)
    assert math.isclose(mres, e_mres, rel_tol=1e-12)
    assert math.isclose(available, e_avail, rel_tol=1e-12)


def test_context_limitation_is_one_at_the_non_limiting_point() -> None:
    state = _state()
    assert _ctx().limitation(state, _env(state, 1.0)) == 1.0


def test_context_limitation_is_f_water_times_f_n() -> None:
    # soil_water at the band midpoint ⇒ f_water = 0.5; plant_n high ⇒ f_N = 1.
    state = _state(soil_water=(_SW_WILTING + _SW_CRITICAL) / 2.0)
    assert math.isclose(_ctx().limitation(state, _env(state, 1.0)), 0.5, rel_tol=1e-12)


def test_context_storage_excluded_from_biomass() -> None:
    # storage_c must NOT enter the maintenance/f_N biomass (grain pays no maintenance).
    no_grain = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, storage_c=0.0)
    with_grain = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, storage_c=100.0)
    _, mres_a, _ = _ctx().budget(no_grain, _env(no_grain, 1.0))
    _, mres_b, _ = _ctx().budget(with_grain, _env(with_grain, 1.0))
    assert math.isclose(mres_a, mres_b, rel_tol=1e-12)  # storage did not change MRES


# --- GrowthRespiration -------------------------------------------------------
def _growth_flow() -> GrowthRespiration:
    return GrowthRespiration(
        id=FlowId("biosphere.growth_respiration"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_ATMOS,
        co2_resp=_CO2_RESP,
    )


def test_growth_flow_leg_is_the_composed_loss_and_pins_the_step6_literal() -> None:
    # leaf_c = 5, all biomass in leaf (the Step-6 GrowthRespiration operating point);
    # limitation = 1 (non-limiting), so the rewiring preserves the pinned GRES literal.
    state = _state(leaf_c=5.0, stem_c=0.0, root_c=0.0)
    _, _, available = _budget(leaf_c=5.0, biomass=5.0)
    expected = growth_respiration_flux(
        *_budget(leaf_c=5.0, biomass=5.0)[:2], growth_efficiency=_YG
    )
    result = _growth_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_CO2_ATMOS], -expected, rel_tol=1e-12)
    assert math.isclose(legs[_CO2_RESP], expected, rel_tol=1e-12)
    assert math.isclose(expected, (1.0 - _YG) * available, rel_tol=1e-12)
    assert math.isclose(expected, 0.32678769775306143, rel_tol=1e-12)


def test_growth_flow_is_carbon_balanced() -> None:
    state = _state(leaf_c=5.0)
    assert_flow_balanced(
        _growth_flow().evaluate(state, _env(state, 1.0), 1.0), state.stocks
    )


def test_growth_flow_clamps_to_zero_in_the_dark() -> None:
    state = _state(leaf_c=5.0)
    result = _growth_flow().evaluate(state, _env(state, 1.0, par=0.0), 1.0)
    for leg in result.legs:
        assert leg.amount == 0.0


def test_growth_flow_scales_linearly_with_dt() -> None:
    state = _state(leaf_c=5.0)
    flow = _growth_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _CO2_RESP
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _CO2_RESP
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- MaintenanceRespiration (the covered/shortfall split) -------------------
def _maintenance_flow() -> MaintenanceRespiration:
    return MaintenanceRespiration(
        id=FlowId("biosphere.maintenance_respiration"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_ATMOS,
        co2_resp=_CO2_RESP,
    )


def test_maintenance_surplus_day_is_fully_covered_by_assimilate() -> None:
    # GASS ≥ MRES ⇒ covered = MRES, shortfall = 0: a pure co2_atmos -> co2_resp flux,
    # no organ withdrawal (biomass does not shrink on a surplus day).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    gass, mres, _ = _ctx().budget(state, _env(state, 1.0))
    assert gass >= mres  # precondition of this scenario
    result = _maintenance_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_CO2_ATMOS], -mres, rel_tol=1e-12)
    assert math.isclose(legs[_CO2_RESP], mres, rel_tol=1e-12)
    assert _LEAF_C not in legs and _STEM_C not in legs and _ROOT_C not in legs


def test_maintenance_deficit_day_draws_the_shortfall_from_organs() -> None:
    # Dark ⇒ GASS = 0 ⇒ covered = 0, shortfall = MRES drawn from the organ pools,
    # proportional to each organ's share — biomass shrinks (the overwintering case).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    mres = maintenance_respiration_flux(5.0, 20.0, params=_resp())
    result = _maintenance_flow().evaluate(state, _env(state, 1.0, par=0.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_CO2_ATMOS] == 0.0  # covered = 0 in full deficit
    assert math.isclose(legs[_LEAF_C], -mres * (3.0 / 5.0), rel_tol=1e-12)
    assert math.isclose(legs[_STEM_C], -mres * (1.0 / 5.0), rel_tol=1e-12)
    assert math.isclose(legs[_ROOT_C], -mres * (1.0 / 5.0), rel_tol=1e-12)
    assert math.isclose(legs[_CO2_RESP], mres, rel_tol=1e-12)


def test_maintenance_partial_deficit_splits_covered_and_shortfall() -> None:
    # Large biomass + a small canopy ⇒ 0 < GASS < MRES: covered = GASS (from co2_atmos),
    # shortfall = MRES − GASS (from organs, proportional). The discriminating case.
    state = _state(leaf_c=0.1, stem_c=50.0, root_c=50.0)
    gass, mres, _ = _ctx().budget(state, _env(state, 1.0))
    assert 0.0 < gass < mres  # precondition: a genuine partial deficit
    biomass = 0.1 + 50.0 + 50.0
    shortfall = mres - gass
    result = _maintenance_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_CO2_ATMOS], -gass, rel_tol=1e-12)
    assert math.isclose(legs[_LEAF_C], -shortfall * (0.1 / biomass), rel_tol=1e-9)
    assert math.isclose(legs[_STEM_C], -shortfall * (50.0 / biomass), rel_tol=1e-9)
    assert math.isclose(legs[_ROOT_C], -shortfall * (50.0 / biomass), rel_tol=1e-9)


def test_maintenance_flow_is_carbon_balanced_both_regimes() -> None:
    surplus = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    deficit = _state(leaf_c=0.1, stem_c=50.0, root_c=50.0)
    flow = _maintenance_flow()
    assert_flow_balanced(
        flow.evaluate(surplus, _env(surplus, 1.0), 1.0), surplus.stocks
    )
    assert_flow_balanced(
        flow.evaluate(deficit, _env(deficit, 1.0), 1.0), deficit.stocks
    )


def test_maintenance_zero_biomass_is_inert() -> None:
    state = _state(leaf_c=0.0, stem_c=0.0, root_c=0.0)
    result = _maintenance_flow().evaluate(state, _env(state, 1.0), 1.0)
    for leg in result.legs:
        assert leg.amount == 0.0


def test_maintenance_flow_scales_linearly_with_dt() -> None:
    state = _state(leaf_c=0.1, stem_c=50.0, root_c=50.0)  # a deficit (organ draw)
    flow = _maintenance_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _CO2_RESP
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _CO2_RESP
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- Allocation --------------------------------------------------------------
def _allocation_flow() -> Allocation:
    return Allocation(
        id=FlowId("biosphere.allocation"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_ATMOS,
        storage_c=_STORAGE_C,
        thermal_time_aux=_THERMAL_TIME,
        pheno=_pheno(),
        alloc=AllocationParams(table=_TABLE),
    )


def test_allocation_legs_are_the_partitioned_increment() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)  # DVS = 0.5
    _, _, available = _budget(leaf_c=3.0, biomass=5.0)
    dmi = _YG * available
    leaf, stem, root, storage = partition(dmi, 0.5, _TABLE)
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(legs[_LEAF_C], leaf, rel_tol=1e-12)
    assert math.isclose(legs[_STEM_C], stem, rel_tol=1e-12)
    assert math.isclose(legs[_ROOT_C], root, rel_tol=1e-12)
    assert math.isclose(legs[_STORAGE_C], storage, abs_tol=1e-12)
    assert math.isclose(legs[_CO2_ATMOS], -dmi, rel_tol=1e-12)


def test_allocation_dmi_agrees_with_growth_resp_budget() -> None:
    # Agreement by construction (the shared available_for_growth in CarbonContext):
    # at the Step-6 point DMI = Yg·available, GRES = (1−Yg)·available ⇒ DMI = 3·GRES.
    state = _state(leaf_c=5.0, stem_c=0.0, root_c=0.0, thermal_time=550.0)
    _, _, available = _budget(leaf_c=5.0, biomass=5.0)
    dmi = _YG * available
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert math.isclose(-legs[_CO2_ATMOS], dmi, rel_tol=1e-12)
    assert math.isclose(dmi, 3.0 * 0.32678769775306143, rel_tol=1e-12)


def test_allocation_fills_storage_in_the_reproductive_phase() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=1100.0 + 375.0)
    dvs = development_stage(
        1100.0 + 375.0, tsum_anthesis=_TSUM_ANTHESIS, tsum_maturity=_TSUM_MATURITY
    )
    assert math.isclose(dvs, 1.5, rel_tol=1e-12)
    result = _allocation_flow().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    assert legs[_STORAGE_C] > 0.0  # grain fills at DVS 1.5 (fo = 0.40)


def test_allocation_is_carbon_balanced() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    assert_flow_balanced(
        _allocation_flow().evaluate(state, _env(state, 1.0), 1.0), state.stocks
    )


def test_allocation_clamps_to_zero_in_the_dark() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    result = _allocation_flow().evaluate(state, _env(state, 1.0, par=0.0), 1.0)
    for leg in result.legs:
        assert leg.amount == 0.0


def test_allocation_scales_linearly_with_dt() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    flow = _allocation_flow()
    one = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _LEAF_C
    )
    half = next(
        leg.amount
        for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        if leg.stock == _LEAF_C
    )
    assert math.isclose(half, one * 0.5, rel_tol=1e-12)


# --- structural three-site agreement + the limitation seam ------------------
def test_three_flows_share_one_budget() -> None:
    # GRES, DMI, and (covered + shortfall) all derive from the SAME (GASS, MRES,
    # available): GRES + DMI = available, and the maintenance flux = MRES.
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    ctx = _ctx()
    _, mres, available = ctx.budget(state, _env(state, 1.0))
    gres = next(
        leg.amount
        for leg in _growth_flow().evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _CO2_RESP
    )
    dmi = -next(
        leg.amount
        for leg in _allocation_flow().evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _CO2_ATMOS
    )
    mres_flux = next(
        leg.amount
        for leg in _maintenance_flow().evaluate(state, _env(state, 1.0), 1.0).legs
        if leg.stock == _CO2_RESP
    )
    assert math.isclose(gres + dmi, available, rel_tol=1e-12)
    assert math.isclose(mres_flux, mres, rel_tol=1e-12)


def test_limitation_scales_growth_and_allocation_identically() -> None:
    # f_water = 0.5 (soil_water at the band midpoint) must halve BOTH GRES and DMI —
    # the same factor hits every gross-assimilation recompute (no per-flow drift).
    full = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    half = _state(
        leaf_c=3.0,
        stem_c=1.0,
        root_c=1.0,
        thermal_time=550.0,
        soil_water=(_SW_WILTING + _SW_CRITICAL) / 2.0,
    )
    for flow, stock in ((_growth_flow(), _CO2_RESP), (_allocation_flow(), _LEAF_C)):
        f = next(
            leg.amount
            for leg in flow.evaluate(full, _env(full, 1.0), 1.0).legs
            if leg.stock == stock
        )
        h = next(
            leg.amount
            for leg in flow.evaluate(half, _env(half, 1.0), 1.0).legs
            if leg.stock == stock
        )
        # available = max(0, f_water·GASS − MRES) is NOT exactly linear in f_water
        # (MRES is unlimited), so assert the limited run is strictly smaller, and the
        # GASS halving is exact: re-derive via the budget.
        assert abs(h) < abs(f)
    # Exactness on GASS: the limited GASS is half the unlimited GASS.
    g_full, _, _ = _ctx().budget(full, _env(full, 1.0))
    g_half, _, _ = _ctx().budget(half, _env(half, 1.0))
    assert math.isclose(g_half, 0.5 * g_full, rel_tol=1e-12)
