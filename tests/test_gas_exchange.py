"""Phase-2 Step-3 tests: gas exchange as multi-quantity (CARBON+OXYGEN) flows.

Step 3 realizes P1's filed deferral — *the genuine multi-quantity stoichiometric flow*.
The sealed chamber's carbon pool is promoted to a true CO₂ stock
(``{CARBON:1, OXYGEN:2}``) with an O₂ counterpart (``{OXYGEN:2}``), and the gas loop is
**closed**: photosynthesis is ``CO₂ → biomass + O₂`` and plant respiration
``biomass + O₂ → CO₂``, each balancing CARBON *and* OXYGEN in one flow at the
photosynthetic quotient PQ=1 (pure-carbon biomass; the trace stoichiometric water is not
tracked — see ``docs/plans/phase-2-closed-chamber.md`` P2.1). Two layers:

* **Flow-level** — each multi-quantity flow balances *both* elements through the
  composition fold (``assert_flow_balanced``): :class:`Allocation` releases O₂ equal to
  the carbon fixed into biomass; the closed-loop :class:`MaintenanceRespiration` burns
  biomass back to CO₂ consuming O₂ equal to the carbon burned; and the
  assimilate-respired round trips (:class:`GrowthRespiration`; the *covered* maintenance
  paid from today's assimilate) are net-zero **no-ops** — a CO₂→CO₂ round trip on the
  single pool whose photosynthetic O₂ release is reconsumed by the respiration.
* **Integration (the sealed season)** — OXYGEN is conserved **exactly** (the
  ``2·(CO₂+O₂)`` invariant; there is no boundary O₂ stock, so this directly exercises
  conservation fold site 2), the CO₂ and O₂ pools anti-correlate at PQ=1 step-for-step,
  O₂ stays far from arbitration rationing (the ``f_O2``-deferral guard), and
  ``rationed == 0`` holds *with* O₂ consumption present.

**Scope (Step 3).** The chamber closes the *gas* loop; the *carbon* loop stays open
until the decomposer lands (Step 4) — senescence leaks organ carbon to ``litter_sink``.
O₂ self-limitation (``f_O2``) is deferred to where O₂ actually depletes
(microbial respiration, Step 5; the O₂-depletion validation, Step 7); the realistic O₂
fill keeps plant respiration ~3 orders from rationing, guarded by
:func:`test_sealed_o2_stays_far_from_rationing`.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.carbon_budget import (
    Allocation,
    CarbonContext,
    GrowthRespiration,
    MaintenanceRespiration,
)
from domains.biosphere.loader import (
    load_allocation_params,
    load_canopy_params,
    load_nitrogen_params,
    load_phenology_params,
    load_photosynthesis_params,
    load_respiration_params,
)
from domains.biosphere.season import (
    CARBON_POOL,
    O2_POOL,
    SeasonScenario,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.environment import Environment, SourceResolver, constant
from simcore.flow import assert_flow_balanced
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# stock ids / forcing-var names (mirror the season's; the unit tests are self-contained)
_BIO = DomainId("biosphere")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_STORAGE_C = StockId("biosphere.storage_c")
_PLANT_N = StockId("biosphere.plant_n")
_SOIL_WATER = StockId("biosphere.soil_water")
_CO2_POOL = StockId("biosphere.carbon_pool")
_O2_POOL = StockId("biosphere.o2_pool")
_PAR, _CI, _TEMP, _DAYLEN, _SW = "par", "ci", "temp", "daylength_s", "soil_water"
_THERMAL_TIME = "thermal_time"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- flow-level: synthetic sealed-chamber snapshots --------------------------
# An open Ci-forcing context is enough for the *leg structure / balance* (the sealed Ci
# seam is tested in test_chamber.py); what makes a flow multi-quantity here is the
# CO₂-composition carbon pool + the O₂ pool it is wired to, not where Ci came from.
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
        sw_wilting=10.0,
        sw_critical=30.0,
        plant_n=_PLANT_N,
        photo=load_photosynthesis_params(),
        canopy=load_canopy_params(),
        resp=load_respiration_params(),
        nitro=load_nitrogen_params(),
        ground_area=1.0,
    )


def _state(
    *,
    leaf_c: float,
    stem_c: float = 0.0,
    root_c: float = 0.0,
    storage_c: float = 0.0,
    co2: float = 0.357,
    o2: float = 210.0,
    soil_water: float = 100.0,  # >> sw_critical ⇒ f_water = 1 (non-limiting)
    plant_n: float = 1.0,  # high conc ⇒ f_N = 1 (non-limiting)
    thermal_time: float = 0.0,
) -> State:
    carbon = canonical_unit(Quantity.CARBON)
    oxygen = canonical_unit(Quantity.OXYGEN)

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
        # The CO₂ pool: a true molecular stock — 1 mol C + 2 mol O per mol CO₂.
        _CO2_POOL: Stock(
            id=_CO2_POOL,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=co2,
            kind=StockKind.POOL,
            composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
        ),
        # The O₂ counterpart: 2 mol OXYGEN per mol O₂.
        _O2_POOL: Stock(
            id=_O2_POOL,
            domain=_BIO,
            quantity=Quantity.OXYGEN,
            unit=oxygen,
            amount=o2,
            kind=StockKind.POOL,
            composition={Quantity.OXYGEN: 2.0},
        ),
    }
    return State(n=0, stocks=stocks, rng_seed=0, aux={_THERMAL_TIME: thermal_time})


def _env(
    state: State, dt: float, *, par: float = 800.0, temp: float = 20.0
) -> Environment:
    resolver = SourceResolver(
        forcings={
            _PAR: constant(par),
            _CI: constant(400.0),
            _TEMP: constant(temp),
            _DAYLEN: constant(43200.0),
        },
        shared={_SW: _SOIL_WATER},
    )
    return resolver.bind(state, dt)


def _allocation() -> Allocation:
    return Allocation(
        id=FlowId("biosphere.allocation"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_POOL,
        storage_c=_STORAGE_C,
        thermal_time_aux=_THERMAL_TIME,
        pheno=load_phenology_params(),
        alloc=load_allocation_params(),
        o2_pool=_O2_POOL,
    )


def _growth() -> GrowthRespiration:
    # Closed chamber: the carbon source IS the respiration sink (the single pool).
    return GrowthRespiration(
        id=FlowId("biosphere.growth_respiration"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_POOL,
        co2_resp=_CO2_POOL,
    )


def _maintenance() -> MaintenanceRespiration:
    return MaintenanceRespiration(
        id=FlowId("biosphere.maintenance_respiration"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_POOL,
        co2_resp=_CO2_POOL,
        o2_pool=_O2_POOL,
    )


# --- Allocation: photosynthesis CO₂ → biomass + O₂ ---------------------------
def test_allocation_releases_o2_equal_to_carbon_fixed() -> None:
    # PQ=1: every mol C fixed into an organ releases 1 mol O₂. The O₂ deposit equals the
    # carbon drawn from the CO₂ pool (and the organ legs), exactly (same leg sum).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)  # DVS 0.5
    legs = {
        leg.stock: leg.amount
        for leg in _allocation().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    fixed = legs[_LEAF_C] + legs[_STEM_C] + legs[_ROOT_C] + legs[_STORAGE_C]
    assert fixed > 0.0  # liveness — carbon actually fixed
    assert math.isclose(legs[_O2_POOL], fixed, rel_tol=1e-12)  # O₂ released = C fixed
    assert math.isclose(legs[_CO2_POOL], -fixed, rel_tol=1e-12)  # CO₂ drawn = C fixed


def test_allocation_balances_carbon_and_oxygen() -> None:
    # The whole point of P2.1: ONE flow balances CARBON *and* OXYGEN via the composition
    # fold (the CO₂ pool's 2 oxygens are accounted to the released O₂, not lost).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    assert_flow_balanced(
        _allocation().evaluate(state, _env(state, 1.0), 1.0), state.stocks
    )


def test_allocation_storage_carbon_releases_o2_too() -> None:
    # Grain fixation is still CO₂ → CH₂O, so storage_c joins the O₂ release (the leg sum
    # must cover all four organs). At DVS 1.5 the storage fraction is nonzero.
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=1100.0 + 375.0)
    legs = {
        leg.stock: leg.amount
        for leg in _allocation().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert legs[_STORAGE_C] > 0.0  # grain fills
    fixed = legs[_LEAF_C] + legs[_STEM_C] + legs[_ROOT_C] + legs[_STORAGE_C]
    assert math.isclose(legs[_O2_POOL], fixed, rel_tol=1e-12)


def test_allocation_open_field_has_no_o2_leg() -> None:
    # o2_pool=None (open field) keeps the single-currency Phase-1 legs — no O₂ leg.
    flow = Allocation(
        id=FlowId("biosphere.allocation"),
        priority=0,
        ctx=_ctx(),
        co2_atmos=_CO2_POOL,
        storage_c=_STORAGE_C,
        thermal_time_aux=_THERMAL_TIME,
        pheno=load_phenology_params(),
        alloc=load_allocation_params(),
    )
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0, thermal_time=550.0)
    stocks = {leg.stock for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs}
    assert _O2_POOL not in stocks


# --- MaintenanceRespiration: closed-loop biomass + O₂ → CO₂ ------------------
def test_maintenance_closed_burns_biomass_to_co2_consuming_o2() -> None:
    # Deficit day (dark ⇒ GASS = 0): the shortfall is drawn from the organs and returned
    # to the CO₂ pool as CO₂, consuming O₂ equal to the carbon burned (PQ=1).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    legs = {
        leg.stock: leg.amount
        for leg in _maintenance().evaluate(state, _env(state, 1.0, par=0.0), 1.0).legs
    }
    burned = -(legs[_LEAF_C] + legs[_STEM_C] + legs[_ROOT_C])
    assert burned > 0.0  # biomass burned for maintenance
    assert math.isclose(
        legs[_CO2_POOL], burned, rel_tol=1e-12
    )  # CO₂ returned to the pool
    assert math.isclose(
        legs[_O2_POOL], -burned, rel_tol=1e-12
    )  # O₂ consumed = C burned


def test_maintenance_closed_balances_carbon_and_oxygen() -> None:
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    assert_flow_balanced(
        _maintenance().evaluate(state, _env(state, 1.0, par=0.0), 1.0), state.stocks
    )


def test_maintenance_closed_partial_deficit_balances() -> None:
    # 0 < GASS < MRES (large biomass, small canopy): the covered part is the dropped
    # round trip; only the organ-burned shortfall returns to the pool + consumes O₂.
    state = _state(leaf_c=0.1, stem_c=50.0, root_c=50.0)
    result = _maintenance().evaluate(state, _env(state, 1.0), 1.0)
    legs = {leg.stock: leg.amount for leg in result.legs}
    burned = -(legs[_LEAF_C] + legs[_STEM_C] + legs[_ROOT_C])
    assert math.isclose(legs[_CO2_POOL], burned, rel_tol=1e-12)
    assert math.isclose(legs[_O2_POOL], -burned, rel_tol=1e-12)
    assert_flow_balanced(result, state.stocks)


def test_maintenance_closed_surplus_day_is_noop() -> None:
    # GASS ≥ MRES ⇒ shortfall = 0 ⇒ the covered maintenance is a pure CO₂→CO₂ round trip
    # on the single pool (no net gas) and is dropped: an empty no-op flow.
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    assert _maintenance().evaluate(state, _env(state, 1.0), 1.0).legs == ()


def test_maintenance_closed_emits_single_pool_leg() -> None:
    # The closed branch nets the covered round trip away, so the pool gets ONE return
    # leg (not a withdraw+deposit pair that would trip the duplicate-leg guard).
    state = _state(leaf_c=3.0, stem_c=1.0, root_c=1.0)
    legs = _maintenance().evaluate(state, _env(state, 1.0, par=0.0), 1.0).legs
    assert sum(1 for leg in legs if leg.stock == _CO2_POOL) == 1


# --- GrowthRespiration: closed-loop no-op ------------------------------------
def test_growth_resp_closed_is_noop() -> None:
    # Growth-conversion carbon is gross-assimilated and immediately respired — a CO₂→CO₂
    # round trip with the O₂ release reconsumed (PQ=1): an empty flow.
    state = _state(leaf_c=5.0)
    assert _growth().evaluate(state, _env(state, 1.0), 1.0).legs == ()


# --- integration: the sealed season -----------------------------------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    scenario = SeasonScenario(sealed=True)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, len(_weather()))


def _total_oxygen(s: State) -> float:
    return sum(
        stock.amount * stock.composition.get(Quantity.OXYGEN, 0.0)
        for stock in s.stocks.values()
    )


def test_sealed_conserves_oxygen_exactly(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # OXYGEN lives in exactly two POOLs (CO₂, O₂) and no boundary stock, so total OXYGEN
    # = 2·(CO₂_mol + O₂_mol) is invariant to float — the every-step gate's claim, here
    # checked end-to-end. The strong Step-3 invariant: it exercises conservation site 2,
    # a per-stock Δamount booking to both CARBON and OXYGEN.
    states, _, _ = sealed
    ox0 = _total_oxygen(states[0])
    for s in states:
        assert math.isclose(_total_oxygen(s), ox0, rel_tol=0.0, abs_tol=1e-9)


def test_sealed_co2_o2_anti_correlate_at_pq1(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # PQ=1, made exact: photosynthesis moves CO₂ down / O₂ up by the SAME mol amount,
    # respiration the reverse, so Δ(O₂_mol) == −Δ(CO₂_mol) every step. Senescence leaks
    # organ carbon to litter without touching the CO₂ pool, so the pool↔O₂ coupling
    # stays exact. The headline emergent O₂↔CO₂ anti-correlation, no control code.
    states, _, _ = sealed
    co2 = [s.stocks[CARBON_POOL].amount for s in states]
    o2 = [s.stocks[O2_POOL].amount for s in states]
    for c0, c1, p0, p1 in zip(co2, co2[1:], o2, o2[1:], strict=False):
        assert math.isclose(p1 - p0, -(c1 - c0), rel_tol=0.0, abs_tol=1e-12)


def test_sealed_exercises_both_gas_directions(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Closed-loop liveness: O₂ both accumulates (surplus-day photosynthesis) and is
    # consumed (deficit-day maintenance burning biomass back to CO₂), so BOTH the
    # ``CO₂ → biomass + O₂`` and ``biomass + O₂ → CO₂`` paths genuinely run.
    states, _, _ = sealed
    o2 = [s.stocks[O2_POOL].amount for s in states]
    deltas = [b - a for a, b in zip(o2, o2[1:], strict=False)]
    assert any(d > 0.0 for d in deltas)  # O₂ released (photosynthesis)
    assert any(d < 0.0 for d in deltas)  # O₂ consumed (respiration)


def test_sealed_o2_stays_far_from_rationing(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The ``f_O2``-deferral GUARD. Plant respiration draws O₂, but the realistic O₂ fill
    # is ~3 orders above the O(0.1) mol C gas fluxes, so O₂ never approaches arbitration
    # rationing — WHY no O₂ self-limitation (Michaelis ``f_O2``) is needed yet (it lands
    # with the depleting microbial O₂ sink (Step 5) / the O₂-depletion check (Step 7).
    # If a future change pushes O₂ toward zero, THIS test breaks and flags that ``f_O2``
    # has become load-bearing.
    states, _, _ = sealed
    o2 = [s.stocks[O2_POOL].amount for s in states]
    assert (
        min(o2) > 0.5 * o2[0]
    )  # stayed within 2× of the fill — nowhere near rationing


def test_sealed_never_rations_with_o2_consumption(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # rationed == 0 still holds now that respiration also draws the finite O₂ pool — the
    # draws self-limit (FvCB Ci-shutoff bounds the CO₂ draw; O₂ far from its floor), so
    # the Euler backstop never fires. The central numerical check, extended to OXYGEN.
    _, total_rationed, _ = sealed
    assert total_rationed == 0
