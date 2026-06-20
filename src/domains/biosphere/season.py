"""The single-producer season — Phase-1 Step 11 integration assembly.

Wires the seven biological processes into one runnable winter-wheat season over a
well-mixed 0-D plot: the four carbon-budget flows (allocation + the two respirations,
``domains.biosphere.carbon_budget``), senescence (``allocation``), transpiration +
irrigation (``transpiration``), nitrogen uptake + fertilization (``nitrogen``), and the
thermal-time aux accumulator (``phenology``). Coupling is the P1 multiplicative
limitation ``f_water · f_N`` inside the shared :class:`CarbonContext`; every flow
is single-currency and balanced, so the every-step conservation gate holds — and because
every clamped withdrawal is self-limiting (organ pools, ``soil_water``, ``soil_n``) the
Euler backstop never fires (``rationed == 0`` by construction).

**Weather-agnostic (the demo.py precedent).** :func:`build_season` builds the stocks +
flow/aux registry from declared params + a :class:`SeasonScenario`; the forcing resolver
is built separately (:func:`weather_resolver`) from a daily weather table, so the same
assembly runs under any weather (a test, or the committed NASAPower fixture, for
the oracle comparison). Euler at ``dt = 1 day`` (P3: crop physiology is Euler-daily; the
crop scenario selects Euler, RK4 stays for the engine gates).

**Potential-production (PP) setup.** To mirror the ``Wofost72_PP`` oracle (no water/N
limitation), the scenario keeps ``f_water = f_N = 1``: ``soil_water`` is sized (and
irrigation tops it) so it never drops below ``sw_critical``; ``plant_n`` is a POOL that
only grows (nothing consumes it in Phase 1), initialised generously so its concentration
stays above the critical-N threshold all season. Water/N-limited variants relax these.

**DOCUMENTED FINDING — the committed season is NOT a validated oracle match.** The crop
params are uncalibrated ``TODO(cite)`` placeholders and phenology has no vernalization
(DVS overruns to maturity mid-season — the Step-8 seam), so the trajectory runs **~2
orders of magnitude below the oracle** (peak LAI ≈ 0.09 vs ≈ 6). Step 11 ships the
**machinery** — single-currency flows, the conservation gate, ``rationed == 0`` by
construction, determinism, the golden — plus a *qualitative* shape match
(``test_oracle_smoke``). The **quantitative** oracle gate (literature-range calibration,
vernalization) is a **deferred** follow-up (a user decision). Do not read the committed
golden as behavioural validation.

Pure stdlib only (the YAML/pint loading is in ``loader.py``; this assembly stays
importable headless, like ``demo.py``).
"""

from dataclasses import dataclass
from datetime import date

from domains.biosphere.allocation import Senescence
from domains.biosphere.carbon_budget import (
    Allocation,
    CarbonContext,
    GrowthRespiration,
    MaintenanceRespiration,
)
from domains.biosphere.decomposition import Decomposition
from domains.biosphere.loader import (
    load_allocation_params,
    load_canopy_params,
    load_decomposition_params,
    load_nitrogen_params,
    load_phenology_params,
    load_photosynthesis_params,
    load_respiration_params,
    load_senescence_params,
    load_transpiration_params,
)
from domains.biosphere.nitrogen import Fertilization, NitrogenUptake
from domains.biosphere.phenology import ThermalTimeAccumulation
from domains.biosphere.transpiration import Irrigation, Transpiration
from domains.biosphere.weather import (
    daylength_seconds,
    incident_par,
    net_radiation,
    vapor_pressure_deficit,
)
from simcore import boundary
from simcore.auxiliary import AuxId, AuxProcess
from simcore.environment import Schedule, SourceResolver
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind, UnitLabel, canonical_unit
from simcore.registry import Registry
from simcore.state import State, Stock

BIOSPHERE: DomainId = DomainId("biosphere")
BOUNDARY: DomainId = DomainId("boundary")

# --- stock ids --------------------------------------------------------------
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
# Sealed chamber decomposer (P2.3/Step 4): senescence feeds a finite ``litter_carbon``
# POOL (replacing the open field's ``litter_sink`` BOUNDARY, exactly as Step 2 replaced
# the ``co2_atmos`` boundary with the finite ``carbon_pool``); first-order decomposition
# transfers it into ``microbial_carbon`` (a POPULATION, pure carbon). Single-currency
# CARBON — the CO₂-releasing, O₂-consuming microbial respiration is Step 5.
LITTER_CARBON: StockId = StockId("biosphere.litter_carbon")
MICROBIAL_CARBON: StockId = StockId("biosphere.microbial_carbon")
CO2_RESP: StockId = StockId("boundary.co2_resp")
VAPOR_SINK: StockId = StockId("boundary.vapor_sink")
LITTER_SINK: StockId = StockId("boundary.litter_sink")
WATER_SOURCE: StockId = StockId("boundary.water_source")
N_SOURCE: StockId = StockId("boundary.n_source")

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

SeasonIntegrator = EulerIntegrator | Rk4Integrator


@dataclass(frozen=True)
class SeasonScenario:
    """Scenario data (not crop params): plot, initial amounts, soil/atmosphere knobs.

    Defaults are the Phase-1 winter-wheat PP plot (1 m² ground, a small
    sown seedling, N/water kept non-limiting — see the module docstring). All are
    scenario wiring, not flow-logic coefficients (P4); crop coefficients come from the
    param files via the loaders.
    """

    ground_area: float = 1.0  # m²
    # seedling organ carbon (mol C) at sowing — small, nonzero (LAI ≈ 0.03 at emergence)
    leaf_c0: float = 0.05
    stem_c0: float = 0.03
    root_c0: float = 0.08
    storage_c0: float = 0.0
    # CO₂: an unclamped atmosphere (FvCB reads Ci forcing, not the stock) + a resp sink.
    # Started at 0 (it tracks cumulative net exchange, going negative) so amounts stay
    # O(1)–O(1e3) and the conservation gate's relative tolerance holds (a huge source
    # would swamp the small daily flux below float resolution; the demo's amounts note).
    co2_atmos0: float = 0.0
    ci: float = 250.0  # intercellular CO₂ (µmol mol⁻¹ ≈ 0.7·ambient for C3)
    # Sealed chamber (P2.2). ``sealed=False`` keeps the Phase-1 open field (unclamped
    # ``co2_atmos`` boundary + constant ``ci`` forcing; the regression golden is
    # untouched). ``sealed=True`` swaps in a finite ``carbon_pool`` POOL that
    # photosynthesis draws down, and derives Ci from it (the draw-down feedback). The
    # chamber air total + initial fill are sized (see the Step-2 design / probe) so Ci
    # falls meaningfully toward Γ* without exhausting the pool (rationed == 0). The
    # default fill reproduces the Phase-1 Ci=250 at t=0
    # (Ci0 = ci_ratio·co2_mol0/air_mol·1e6).
    sealed: bool = False
    chamber_air_mol: float = 1000.0  # total chamber air (mol); 0-D well-mixed
    # initial pool carbon (mol C); Ci0 = ci_ratio·co2_mol0/air_mol·1e6 ≈ 250 µmol mol⁻¹
    # (continuity with the Phase-1 constant Ci forcing). Sized (Step-2 probe) so the
    # draw-down spans ~40–60 days down toward Ci≈Γ* — Ci falls ~5×, gross assimilation
    # collapses ~4 orders — while withdrawals stay far from exhausting the pool
    # (rationed == 0; FvCB Ci-shutoff self-limits, never the Euler backstop).
    chamber_co2_mol0: float = 0.357
    ci_ratio: float = 0.7  # C3 Ci/Ca draw-down set point (Farquhar & Sharkey 1982)
    # O₂ counterpart pool (mol O₂; Step 3). Sized to a realistic chamber O₂ fraction
    # (~21% of ``chamber_air_mol``) — vastly larger than the O(0.1) mol C gas fluxes, so
    # it never approaches arbitration rationing and plant respiration needs no O₂
    # self-limitation (``f_O2``) yet. The depleting-O₂ regime (where ``f_O2`` becomes
    # load-bearing) arrives with microbial respiration (Step 5) and the O₂-depletion
    # validation (Step 7); a Step-3 test pins O₂ ≫ 0 to guard that deferral.
    # Photosynthesis deposits O₂ here (PQ=1) and respiration draws it, so it
    # anti-correlates with the CO₂ pool: ΔO₂ = −Δ(net CO₂), 2·(CO₂+O₂) conserved.
    chamber_o2_mol0: float = 210.0
    # water (PP, non-limiting): a store sized to stay above the band all season
    soil_water0: float = 1000.0  # kg
    water_source0: float = 0.0  # kg (unclamped supply; tracks cumulative irrigation)
    sw_wilting: float = 20.0  # kg
    sw_critical: float = 60.0  # kg
    irrigation_mm_day: float = 2.0  # mm day⁻¹
    # nitrogen (PP, non-limiting): a generous plant-N reserve + ample soil supply
    soil_n0: float = 100.0  # kg N (>> sn_critical ⇒ availability = 1 all season)
    n_source0: float = 0.0  # kg N (unclamped supply; tracks cumulative fertilization)
    plant_n0: float = 0.5  # kg N — high conc ⇒ f_N = 1 all season (plant_n only grows)
    sn_residual: float = 1.0  # kg N (soil-N availability band, scenario/soil data)
    sn_critical: float = 50.0  # kg N
    fertilization_kg_m2_day: float = 0.0  # kg N m⁻² day⁻¹ (soil store already ample)
    # location (for the astronomical daylength); matches the oracle plot
    latitude: float = 52.0


# Module-level default (immutable, frozen dataclass) — used as the param default so the
# signatures don't call SeasonScenario() in their defaults (ruff B008).
DEFAULT_SCENARIO: SeasonScenario = SeasonScenario()


def _carbon_context(scenario: SeasonScenario) -> CarbonContext:
    """Build the shared carbon budget context from the committed crop params.

    Open field (default): Ci is the ``ci_var`` forcing read. Sealed chamber (P2.2): the
    chamber Ci-source triple is wired so Ci is derived from the live ``carbon_pool``
    (read as the shared ``co2_pool`` var, #16) — the draw-down feedback.
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


def build_season(scenario: SeasonScenario = DEFAULT_SCENARIO) -> tuple[State, Registry]:
    """Assemble the season's initial ``State`` and the flow + aux ``Registry``.

    Stocks: four organ pools (POPULATION CARBON, extinction-eligible, ``threshold = 0``
    on the well-fed run), ``soil_water``/``soil_n``/``plant_n`` POOLs, the
    ``water_source``/``n_source`` boundary sources, the ``vapor_sink`` boundary sink,
    plus the carbon loss-sink for extinction routing (#6). The carbon atmosphere is
    scenario-dependent: **open field** adds the unclamped ``co2_atmos`` source, the
    ``co2_resp`` + ``litter_sink`` boundary sinks; **sealed chamber** swaps in the
    finite ``carbon_pool``/``o2_pool`` (gas exchange, Steps 2/3) and the decomposer
    pools ``litter_carbon`` (POOL, senescence-fed) + ``microbial_carbon`` (pure-carbon
    POPULATION, decay-fed) (Step 4), adds the ``Decomposition`` flow. Flows in
    canonical id order; the aux process is a registry construction dependency (advances
    ``State.aux``, P2).
    """
    carbon = canonical_unit(Quantity.CARBON)
    water = canonical_unit(Quantity.WATER)
    nitrogen = canonical_unit(Quantity.NITROGEN)
    oxygen = canonical_unit(Quantity.OXYGEN)

    def organ(stock_id: StockId, amount: float) -> Stock:
        return Stock(
            id=stock_id,
            domain=BIOSPHERE,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=amount,
            kind=StockKind.POPULATION,
            extinction_threshold=0.0,
        )

    def pool(
        stock_id: StockId, quantity: Quantity, unit: UnitLabel, amount: float
    ) -> Stock:
        return Stock(
            id=stock_id,
            domain=BIOSPHERE,
            quantity=quantity,
            unit=unit,
            amount=amount,
            kind=StockKind.POOL,
        )

    stocks: dict[StockId, Stock] = {
        LEAF_C: organ(LEAF_C, scenario.leaf_c0),
        STEM_C: organ(STEM_C, scenario.stem_c0),
        ROOT_C: organ(ROOT_C, scenario.root_c0),
        STORAGE_C: organ(STORAGE_C, scenario.storage_c0),
        SOIL_WATER: pool(SOIL_WATER, Quantity.WATER, water, scenario.soil_water0),
        SOIL_N: pool(SOIL_N, Quantity.NITROGEN, nitrogen, scenario.soil_n0),
        PLANT_N: pool(PLANT_N, Quantity.NITROGEN, nitrogen, scenario.plant_n0),
        VAPOR_SINK: boundary.sink(VAPOR_SINK, Quantity.WATER),
        WATER_SOURCE: boundary.source(
            WATER_SOURCE, Quantity.WATER, scenario.water_source0
        ),
        N_SOURCE: boundary.source(N_SOURCE, Quantity.NITROGEN, scenario.n_source0),
    }
    # The carbon source the gas-exchange flows draw from, and the respiration sink:
    #   * Open field (Phase-1): the unclamped ``co2_atmos`` BOUNDARY source + a separate
    #     ``co2_resp`` BOUNDARY sink. Single-currency CARBON; the loop is open.
    #   * Sealed chamber (Step 3): one finite CO₂ POOL (``{CARBON:1, OXYGEN:2}``) is
    #     *both* the source and the respiration sink, plus an O₂ counterpart POOL. The
    #     gas loop is closed — respiration returns CO₂ to the pool — the flows detect
    #     ``source == sink`` to net the assimilate-respired round trips (see
    #     ``carbon_budget``). O₂ legs balance OXYGEN at PQ=1.
    carbon_source = CARBON_POOL if scenario.sealed else CO2_ATMOS
    resp_sink = carbon_source if scenario.sealed else CO2_RESP
    o2_pool = O2_POOL if scenario.sealed else None
    # Senescence destination (Step 4): a finite ``litter_carbon`` POOL in the sealed
    # chamber (decomposition decays it into microbial biomass) vs the Phase-1
    # ``litter_sink`` BOUNDARY in the open field (the regression golden's path —
    # unchanged). Parametrized exactly like ``carbon_source``/``resp_sink``.
    litter_target = LITTER_CARBON if scenario.sealed else LITTER_SINK
    if scenario.sealed:
        stocks[CARBON_POOL] = Stock(
            id=CARBON_POOL,
            domain=BIOSPHERE,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=scenario.chamber_co2_mol0,
            kind=StockKind.POOL,
            composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
        )
        stocks[O2_POOL] = Stock(
            id=O2_POOL,
            domain=BIOSPHERE,
            quantity=Quantity.OXYGEN,
            unit=oxygen,
            amount=scenario.chamber_o2_mol0,
            kind=StockKind.POOL,
            composition={Quantity.OXYGEN: 2.0},
        )
        # The decomposer pools (Step 4). ``litter_carbon`` is a finite POOL fed by
        # senescence and drained by first-order decomposition; ``microbial_carbon`` is a
        # pure-carbon POPULATION the decay deposits into. Both start at 0 (no standing
        # litter/microbes at sowing). microbial_carbon is a POPULATION (extinction-
        # eligible) but only *grows* this step — nothing withdraws it until Step 5's
        # microbial respiration — so threshold 0 never snaps (the carbon loss-sink below
        # covers it once Step 5 makes it a source).
        stocks[LITTER_CARBON] = pool(LITTER_CARBON, Quantity.CARBON, carbon, 0.0)
        stocks[MICROBIAL_CARBON] = organ(MICROBIAL_CARBON, 0.0)
    else:
        stocks[CO2_ATMOS] = boundary.source(
            CO2_ATMOS, Quantity.CARBON, scenario.co2_atmos0
        )
        stocks[CO2_RESP] = boundary.sink(CO2_RESP, Quantity.CARBON)
        stocks[LITTER_SINK] = boundary.sink(LITTER_SINK, Quantity.CARBON)
    # Only POPULATION carbon organs are extinction-eligible ⇒ only the carbon loss-sink.
    stocks.update(boundary.loss_sinks({Quantity.CARBON}))

    ctx = _carbon_context(scenario)
    nitro = ctx.nitro
    pheno = load_phenology_params()
    flows: list[Flow] = [
        Allocation(
            FlowId("biosphere.allocation"),
            0,
            ctx=ctx,
            co2_atmos=carbon_source,
            storage_c=STORAGE_C,
            thermal_time_aux=THERMAL_TIME,
            pheno=pheno,
            alloc=load_allocation_params(),
            o2_pool=o2_pool,
        ),
        GrowthRespiration(
            FlowId("biosphere.growth_respiration"),
            0,
            ctx=ctx,
            co2_atmos=carbon_source,
            co2_resp=resp_sink,
        ),
        MaintenanceRespiration(
            FlowId("biosphere.maintenance_respiration"),
            0,
            ctx=ctx,
            co2_atmos=carbon_source,
            co2_resp=resp_sink,
            o2_pool=o2_pool,
        ),
        Senescence(
            FlowId("biosphere.senescence"),
            0,
            leaf_c=LEAF_C,
            stem_c=STEM_C,
            root_c=ROOT_C,
            litter_sink=litter_target,
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
        Irrigation(
            FlowId("biosphere.irrigation"),
            0,
            water_source=WATER_SOURCE,
            soil_water=SOIL_WATER,
            irrigation_var=IRRIGATION_VAR,
            ground_area=scenario.ground_area,
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
        # The decomposer (Step 4): first-order decay transfers litter carbon into
        # microbial biomass (pure CARBON, single-currency). Sealed-only; the Registry
        # sorts flows by id, so conditional addition stays order-independent.
        flows.append(
            Decomposition(
                FlowId("biosphere.decomposition"),
                0,
                litter_carbon=LITTER_CARBON,
                microbial_carbon=MICROBIAL_CARBON,
                params=load_decomposition_params(),
            )
        )
    aux_processes: list[AuxProcess] = [
        ThermalTimeAccumulation(
            id=AuxId("biosphere.thermal_time"),
            accumulator=THERMAL_TIME,
            temp_var=TEMP_VAR,
            params=pheno,
        )
    ]
    state = State(n=0, stocks=stocks, rng_seed=0, aux={THERMAL_TIME: 0.0})
    return state, Registry(flows, stocks, aux_processes=aux_processes)


def _table(values: list[float]) -> Schedule:
    """A forcing ``Schedule`` reading a precomputed per-day table (clamped at the end).

    ``schedule(n, dt) = values[min(n, last)]`` — the first genuinely ``n``-dependent
    forcing (P3). Clamping past the last day keeps a longer-than-table run well-defined.
    """
    last = len(values) - 1

    def schedule(n: int, dt: float) -> float:
        return values[min(n, last)]

    return schedule


def weather_resolver(
    weather: list[dict[str, float | str]], scenario: SeasonScenario = DEFAULT_SCENARIO
) -> SourceResolver:
    """Build the forcing resolver from a daily raw-weather table (NASAPower facts).

    Each row is ``{day, TEMP, IRRAD, VAP}``; the clean-room conversions in
    ``domains.biosphere.weather`` derive the per-day drivers (PAR, net radiation,
    VPD, photoperiod) the flows read. ``Ci``/irrigation/fertilization are constant
    schedules; ``soil_water`` is wired **shared** to the stock so ``f_water`` reads the
    live sibling amount (#16). In a sealed chamber (P2.2) the ``co2_pool`` var is also
    wired **shared** to the finite ``carbon_pool`` so FvCB derives Ci from its live
    draw-down (the ``ci`` forcing schedule is then unused but harmless — disjoint var).
    """
    temp: list[float] = []
    par: list[float] = []
    daylen: list[float] = []
    rn: list[float] = []
    vpd: list[float] = []
    for row in weather:
        t = float(row["TEMP"])
        irrad = float(row["IRRAD"])
        vap = float(row["VAP"])
        doy = date.fromisoformat(str(row["day"])).timetuple().tm_yday
        dl = daylength_seconds(scenario.latitude, doy)
        temp.append(t)
        daylen.append(dl)
        par.append(incident_par(irrad, dl))
        rn.append(net_radiation(irrad))
        vpd.append(vapor_pressure_deficit(t, vap))
    return SourceResolver(
        forcings={
            TEMP_VAR: _table(temp),
            PAR_VAR: _table(par),
            DAYLENGTH_VAR: _table(daylen),
            RN_VAR: _table(rn),
            VPD_VAR: _table(vpd),
            CI_VAR: _table([scenario.ci]),
            IRRIGATION_VAR: _table([scenario.irrigation_mm_day]),
            FERTILIZATION_VAR: _table([scenario.fertilization_kg_m2_day]),
        },
        shared=(
            {SOIL_WATER_VAR: SOIL_WATER, CO2_POOL_VAR: CARBON_POOL}
            if scenario.sealed
            else {SOIL_WATER_VAR: SOIL_WATER}
        ),
    )


def run_season(
    integrator: SeasonIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    ``states`` is the full trajectory incl. the initial state (length ``steps + 1``):
    used by liveness, the oracle comparison, and the golden. ``total_rationed``
    sums the Euler backstop firings (the golden asserts ``== 0``); ``events`` are the
    extinction events (empty on the well-fed season).
    """
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(steps):
        report = integrator.step_report(state, resolver, dt)
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    return states, total_rationed, tuple(events)
