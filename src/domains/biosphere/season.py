"""The single-producer season — the compartment-composition layer (P3.2).

Phase-3 Step 2 split the monolithic ``build_season`` assembly into per-compartment
**builder modules** (``atmosphere`` / ``soil`` / ``plants``); Step 3 added ``water``
(the condensate pool + recycler) when it closed the water cycle. This module is now the
thin **composition**: it calls each builder,
unions the parts (the integrator stays global — one clock, one ledger, one conservation
gate; compartments are a *grouping*, never a sub-solver), adds the cross-cutting carbon
loss-sink, and hands the flat union to the ``Registry`` (which re-sorts flows by id, so
builder/union order is behaviorally inert). The restructure is **behavior-preserving** —
same stocks, flows, ids, amounts, wiring — so the open + sealed regression goldens
pass **byte-identical without regeneration** (the proof it was safe). New science (water
cycle, mortality, perturbations) lands in the *separate* later steps, never mixed here.

**Weather-agnostic (the demo.py precedent).** :func:`build_season` builds the stocks +
flow/aux registry from declared params + a :class:`SeasonScenario`; the forcing resolver
is built separately (:func:`weather_resolver`) from a daily weather table. Both route
through the single :func:`_compartments` aggregator — ``build_season`` unions
stocks/flows/aux, ``weather_resolver`` merges the resolver ``shared`` maps (the #16
live-stock seam) — so shared wiring has one source of truth. Euler at ``dt = 1 day``.

**Re-exports.** The stock-id catalog + ``STOCK_DOMAIN`` now live in ``stocks.py``, the
scenario in ``scenario.py``, and ``_carbon_context`` in ``plants.py``; this module
re-exports every symbol the tests import from ``season`` so no test import path changed.

**DOCUMENTED FINDING — the committed season is NOT a validated oracle match.** The crop
params are uncalibrated ``TODO(cite)`` placeholders, phenology lacks vernalization, so
the trajectory runs ~2 orders of magnitude below the oracle. The season ships the
*machinery* (single-currency flows, the conservation gate, ``rationed == 0`` by
construction, determinism, the golden) — not behavioural validation.

Pure stdlib only (the YAML/pint loading is in ``loader.py``).
"""

from collections.abc import Callable
from dataclasses import replace
from datetime import date

from domains.biosphere.atmosphere import build_atmosphere
from domains.biosphere.consumers import build_consumers
from domains.biosphere.plants import _carbon_context as _carbon_context
from domains.biosphere.plants import build_plants
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_SCENARIO as CONSUMER_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_YEARS as CONSUMER_CHAMBER_YEARS,
)
from domains.biosphere.scenario import (
    DEFAULT_SCENARIO,
    SeasonScenario,
)
from domains.biosphere.scenario import (
    PERENNIAL_CHAMBER_SCENARIO as PERENNIAL_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    PERENNIAL_CHAMBER_YEARS as PERENNIAL_CHAMBER_YEARS,
)
from domains.biosphere.scenario import (
    SEALED_CHAMBER_SCENARIO as SEALED_CHAMBER_SCENARIO,
)
from domains.biosphere.scenario import (
    SEALED_CHAMBER_YEARS as SEALED_CHAMBER_YEARS,
)
from domains.biosphere.soil import build_soil
from domains.biosphere.stocks import (
    CARBON_POOL as CARBON_POOL,
)
from domains.biosphere.stocks import (
    CI_VAR,
    DAYLENGTH_VAR,
    FERTILIZATION_VAR,
    IRRIGATION_VAR,
    PAR_VAR,
    RN_VAR,
    TEMP_VAR,
    THERMAL_TIME,
    VPD_VAR,
    CompartmentBuild,
    chamber_wiring,
)
from domains.biosphere.stocks import (
    CO2_ATMOS as CO2_ATMOS,
)
from domains.biosphere.stocks import (
    CO2_RESP as CO2_RESP,
)
from domains.biosphere.stocks import (
    CONDENSATE as CONDENSATE,
)
from domains.biosphere.stocks import (
    CONSUMER_CARBON as CONSUMER_CARBON,
)
from domains.biosphere.stocks import (
    LEAF_C as LEAF_C,
)
from domains.biosphere.stocks import (
    LITTER_CARBON as LITTER_CARBON,
)
from domains.biosphere.stocks import (
    LITTER_N as LITTER_N,
)
from domains.biosphere.stocks import (
    LITTER_SINK as LITTER_SINK,
)
from domains.biosphere.stocks import (
    MICROBIAL_CARBON as MICROBIAL_CARBON,
)
from domains.biosphere.stocks import (
    O2_POOL as O2_POOL,
)
from domains.biosphere.stocks import (
    PLANT_N as PLANT_N,
)
from domains.biosphere.stocks import (
    ROOT_C as ROOT_C,
)
from domains.biosphere.stocks import (
    SOIL_WATER as SOIL_WATER,
)
from domains.biosphere.stocks import (
    STEM_C as STEM_C,
)
from domains.biosphere.stocks import (
    STOCK_DOMAIN as STOCK_DOMAIN,
)
from domains.biosphere.stocks import (
    STORAGE_C as STORAGE_C,
)
from domains.biosphere.stocks import (
    WATER_VAPOR as WATER_VAPOR,
)
from domains.biosphere.water import build_water
from domains.biosphere.weather import (
    daylength_seconds,
    incident_par,
    net_radiation,
    vapor_pressure_deficit,
)
from simcore import boundary, conservation
from simcore.auxiliary import AuxProcess
from simcore.environment import Schedule, SourceResolver
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State, Stock

SeasonIntegrator = EulerIntegrator | Rk4Integrator


def _compartments(scenario: SeasonScenario) -> tuple[CompartmentBuild, ...]:
    """The per-compartment builds for ``scenario`` — the single source of truth.

    Both :func:`build_season` (unions stocks/flows/aux) and :func:`weather_resolver`
    (merges the ``shared`` maps) route through here. The ``ChamberWiring`` (the
    sealed-dependent cross-compartment ids) is computed once, threaded to each builder.
    Builder/union order is inert (Registry re-sorts flows by id; the snapshot serializer
    sorts stocks by id).
    """
    wiring = chamber_wiring(scenario.sealed)
    return (
        build_atmosphere(scenario, wiring),
        build_soil(scenario, wiring),
        build_plants(scenario, wiring),
        build_water(scenario, wiring),
        build_consumers(scenario, wiring),
    )


def build_season(scenario: SeasonScenario = DEFAULT_SCENARIO) -> tuple[State, Registry]:
    """Assemble the season's initial ``State`` and the flow + aux ``Registry``.

    Composes the compartment builds: unions their stocks (keyed by id), flows, and aux,
    then adds the cross-cutting carbon **loss-sink** (extinction routing, #6) at the
    composition level (it spans compartments, so no single builder owns it). The flat
    union goes to ``Registry``, which sorts flows by id — so the assembly is
    order-independent and the goldens reproduce byte-identically.
    """
    builds = _compartments(scenario)
    stocks: dict[StockId, Stock] = {}
    for build in builds:
        for stock in build.stocks:
            stocks[stock.id] = stock
    # Only POPULATION carbon organs are extinction-eligible ⇒ only the carbon loss-sink.
    stocks.update(boundary.loss_sinks({Quantity.CARBON}))
    flows: list[Flow] = [flow for build in builds for flow in build.flows]
    aux_processes: list[AuxProcess] = [aux for build in builds for aux in build.aux]
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
    ``domains.biosphere.weather`` derive the per-day drivers (PAR, net radiation, VPD,
    photoperiod) the flows read. ``Ci``/irrigation/fertilization are constant schedules.
    The resolver ``shared`` map (the #16 live-stock seam — ``soil_water`` always, plus
    the sealed chamber's ``co2_pool``) is **merged from the compartment builds** (one
    source of truth with ``build_season``); #16 makes shared/forcing indistinguishable,
    so this is golden-safe.
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
    shared: dict[str, StockId] = {}
    for build in _compartments(scenario):
        shared.update(build.shared)
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
        shared=shared,
    )


def run_season(
    integrator: SeasonIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
    *,
    reset: Callable[[int, State], State] | None = None,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    ``states`` is the full trajectory incl. the initial state (length ``steps + 1``):
    used by liveness, the oracle comparison, and the golden. ``total_rationed``
    sums the Euler backstop firings (the golden asserts ``== 0``); ``events`` are the
    extinction events (empty on the well-fed season).

    **Scheduled scenario reset (P3.4, Step 4).** ``reset`` is an optional
    **schedule-agnostic** hook ``(n, state) -> state`` consulted **before each step**:
    it returns ``state`` *unchanged* (the same object) on a non-reset step, or a new
    ``State`` on a reset boundary. The calendar (e.g. ``n % year == 0``) lives **inside
    the caller's closure** — scheduling is a scenario/caller concern, not the driver's.
    When a reset is applied the driver re-asserts the conservation gate across it
    (:func:`conservation.assert_conserved`), so "conserved at every point" stays
    literally true even though the reset is a discrete scenario intervention rather than
    a flow-step (its only non-conserved write is ``thermal_time``, an aux accumulator
    invisible to the gate). The stored trajectory records the **pre-reset** state (the
    reset instant is not appended): the day a reset fires, ``states`` jumps from the
    pre-reset state straight to the post-step state. **Default ``None`` ⇒ the loop is
    byte-identical to the pre-Step-4 season** (the open/sealed regression goldens are
    unaffected) — pure indirection.
    """
    states = [state]
    total_rationed = 0
    events: list[Event] = []
    for _ in range(steps):
        if reset is not None:
            reset_state = reset(state.n, state)
            if reset_state is not state:
                # The redistribution must balance every asserted quantity (carbon is
                # only moved between in-system stocks; the aux thermal_time zero is
                # outside the gate). A violation is a reset bug, not recoverable state.
                conservation.assert_conserved(state, reset_state)
                state = reset_state
        report = integrator.step_report(state, resolver, dt)
        state = report.state
        states.append(state)
        total_rationed += report.rationed
        events.extend(report.events)
    return states, total_rationed, tuple(events)


def annual_reset(state: State, scenario: SeasonScenario) -> State:
    """The annual phenology reset / re-sow (P3.4) — a pure, carbon-conserving transform.

    At each year boundary the **old plant dies/harvests entirely to litter, except the
    seedling carbon retained from the grain (the seed bank)**, and ``thermal_time``
    resets so the new seedling develops from DVS 0:

    * new ``leaf_c``/``stem_c``/``root_c`` := the scenario sowing amounts;
    * ``storage_c`` (grain) := 0 (all of it is either re-sown or shed to litter);
    * ``litter_carbon`` += the **balancing residual**
      ``old_veg + grain − seedling_total`` (the senescence/maintenance idiom — balance
      by construction, not an independent formula), so CARBON is conserved exactly and
      the loss-sink is never touched;
    * ``thermal_time`` := 0 (an aux accumulator, invisible to the conservation gate).

    **Grain → 0 every year is what keeps the cycle sustained** (a seed bank that only
    shed ``seedling_total`` would grow unboundedly and drain the active cycle — the
    damped-cascade trap); the dumped grain decomposes (litter → microbial → CO₂) to
    refuel next year's photosynthesis. **Carbon-only** — ``plant_n`` persists across the
    death (an N *windfall* for the small seedling), harmless only while ``f_N ≡ 1``; a
    full N-reset is a deferred refinement. **Sealed-chamber only:** it sheds to the
    in-system ``litter_carbon`` POOL and re-sows from the in-system grain (the open
    field has no closed loop to re-sow into).

    Raises ``ValueError`` if the seed bank cannot cover a seedling
    (``grain < seedling_total``) — re-sow would conjure carbon or drive ``storage_c``
    negative (the closure caveat: the seedling's carbon comes from an in-system pool).
    """
    seedling = {
        LEAF_C: scenario.leaf_c0,
        STEM_C: scenario.stem_c0,
        ROOT_C: scenario.root_c0,
    }
    seedling_total = scenario.leaf_c0 + scenario.stem_c0 + scenario.root_c0
    stocks = dict(state.stocks)
    grain = stocks[STORAGE_C].amount
    if grain < seedling_total:
        raise ValueError(
            f"annual_reset: seed bank too small to re-sow — storage_c {grain!r} < "
            f"seedling {seedling_total!r}; the seedling's carbon must come from the "
            "in-system grain (closure caveat P3.4)"
        )
    old_veg = sum(stocks[oid].amount for oid in seedling)
    for organ_id, amount in seedling.items():
        stocks[organ_id] = replace(stocks[organ_id], amount=amount)
    stocks[STORAGE_C] = replace(stocks[STORAGE_C], amount=0.0)
    litter_gain = old_veg + grain - seedling_total  # the balancing residual
    stocks[LITTER_CARBON] = replace(
        stocks[LITTER_CARBON], amount=stocks[LITTER_CARBON].amount + litter_gain
    )
    aux = dict(state.aux)
    aux[THERMAL_TIME] = 0.0
    return replace(state, stocks=stocks, aux=aux)


def run_perennial(
    integrator: SeasonIntegrator,
    state: State,
    scenario: SeasonScenario,
    resolver: SourceResolver,
    dt: float,
    steps: int,
    *,
    year: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """:func:`run_season` with :func:`annual_reset` applied every ``year`` steps (P3.4).

    The concrete perennial driver: builds the schedule closure (reset at each ``n`` with
    ``n > 0 and n % year == 0``) and hands it to :func:`run_season`'s ``reset`` hook
    (which re-asserts conservation across each reset). ``year`` is the season length in
    steps (``len(weather)``, the tiling period). Sustained multi-year oscillation, no
    control code — the carbon recycles through the closed loop and each reset re-sows
    the seedling from the grain.
    """

    def reset(n: int, current: State) -> State:
        if n > 0 and n % year == 0:
            return annual_reset(current, scenario)
        return current

    return run_season(integrator, state, resolver, dt, steps, reset=reset)
