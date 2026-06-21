"""The single-producer season — the compartment-composition layer (P3.2).

Phase-3 Step 2 split the monolithic ``build_season`` assembly into per-compartment
**builder modules** (``atmosphere`` / ``soil`` / ``plants``; ``water`` arrives with the
Step-3 water cycle). This module is now the thin **composition**: it calls each builder,
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

from datetime import date

from domains.biosphere.atmosphere import build_atmosphere
from domains.biosphere.plants import _carbon_context as _carbon_context
from domains.biosphere.plants import build_plants
from domains.biosphere.scenario import (
    DEFAULT_SCENARIO,
    SeasonScenario,
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
from domains.biosphere.weather import (
    daylength_seconds,
    incident_par,
    net_radiation,
    vapor_pressure_deficit,
)
from simcore import boundary
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
