"""The standalone Crew system: assemble + constant resolver + run harness (Step 7).

The Crew analogue of ``domains.power.system`` / ``domains.eclss.system`` — the thin
composition layer that turns the stocks/flows into a runnable standalone system:
:func:`build_crew` (initial ``State`` + flow ``Registry``), :func:`crew_resolver` (the
three constant forced intake rates — like ECLSS's constant crew load, much simpler than
Power's diurnal + derived-load resolver, because Crew has no restoring force to balance
against), and :func:`run_crew` (the stepping driver, the ``run_power`` / ``run_eclss``
analogue **minus** any reset hook).

**What this proves.** All three crew flows are **forced** (state-independent), so — like
Power's two-flow ``BOUNDED_SOC``, unlike ECLSS/Thermal — the system has **no restoring
force and no attractor**: each finite store depletes linearly, ``store(n) = store0 −
n·rate·dt``. The standalone validation shows: **all three quantities conserved every
step** over the augmented ledger (``Δstore + Δsinks ≈ 0`` per quantity — the payload,
three quantities at once, as ECLSS), ``rationed == 0`` (well-fed sizing — every store
stays positive over the mission, its endurance ``store0/rate`` exceeding the horizon),
**monotonic** output sinks (exhaled CO₂ / feces / humidity / urine / consumed O₂ — free
cumulative-output diagnostics), and — because no flow reads a stock — **RK4 ≡ Euler
bit-identically** (``k1 = k2 = k3 = k4``; the forced-only identity ECLSS/Thermal broke,
revived).

**"Closed" is decision #13's augmented sense — and Crew is the LEAST closed sibling.**
Each quantity balances over its augmented system (stores + boundary sinks) every step,
but standalone Crew is **open-loop**: matter only ever leaves the stores to the sinks,
with no resupply, so the system runs down. That incompleteness is the argument for
Phase-6 closure: **Phase 6 deletes ECLSS's forced ``CrewMetabolism`` stand-in and wires
Crew's outputs into the cabin** (CO₂ → ``cabin_co2``, humidity → ``cabin_h2o``, O₂
intake ← ``cabin_o2``), and replaces the finite stores with regenerative sources (the
biosphere feeds food, water recovery refills water). Standalone Crew builds the crew
side of that seam. The crew's **atom-level stoichiometry** (``C_food + O₂ → CO₂ + H₂O``)
is also **not** closed here (the O₂-consumed sink and the CO₂/H₂O sources are decoupled)
— the composition-stock Phase-6 act, ECLSS's atom-seam analogue.

**No loss-sinks / no POPULATION stock** — as for Power/Thermal/ECLSS: extinction never
fires (the stores are POOLs; crew count is fixed scenario data, not a stock — modelling
starvation death is out of scope for "forced schedules"), so no quantity needs a
loss-sink. NITROGEN and ENERGY are absent from the state ⇒ the conservation gate skips
them; CARBON, OXYGEN and WATER are checked every step.

Pure stdlib only in the spine; the YAML/pydantic Crew params load via ``loader.py``.
"""

from dataclasses import dataclass

from domains.crew.flows import (
    CrewParams,
    FoodMetabolism,
    OxygenConsumption,
    WaterBalance,
)
from domains.crew.scenario import DEFAULT_CREW_SCENARIO, CrewScenario
from domains.crew.stocks import (
    CREW_HUMIDITY,
    CREW_O2_CONSUMED,
    EXHALED_CO2,
    FECAL_WASTE,
    FOOD_INTAKE_VAR,
    FOOD_STORE,
    O2_INTAKE_VAR,
    O2_STORE,
    URINE,
    WATER_INTAKE_VAR,
    WATER_STORE,
    food_store_stock,
    o2_store_stock,
    water_store_stock,
)
from simcore import boundary
from simcore.environment import SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import FlowId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

CrewIntegrator = EulerIntegrator | Rk4Integrator

# Flow ids (canonical, ASCII so str sort == future Rust byte sort, #15). Mirrors the ids
# the per-flow tests use.
OXYGEN_CONSUMPTION: FlowId = FlowId("crew.oxygen_consumption")
FOOD_METABOLISM: FlowId = FlowId("crew.food_metabolism")
WATER_BALANCE: FlowId = FlowId("crew.water_balance")


@dataclass(frozen=True)
class DepletionTimes:
    """Closed-form per-store time-to-depletion (s); see :func:`depletion_times`."""

    food_store: float
    water_store: float
    o2_store: float


def build_crew(
    params: CrewParams,
    scenario: CrewScenario = DEFAULT_CREW_SCENARIO,
) -> tuple[State, Registry]:
    """Assemble the standalone Crew system's initial ``State`` and flow ``Registry``.

    Eight stocks — the three ``crew.*`` finite provisioned POOLs (initial inventories
    from the scenario) and the five ``boundary.*`` monotonic output sinks (all start at
    0) — and the three forced flows ``OxygenConsumption`` / ``FoodMetabolism`` /
    ``WaterBalance`` (the two split flows carrying the loaded params). **No loss-sinks**
    (no POPULATION stock). The ``Registry`` re-sorts flows by id, so the build order is
    inert (registration-order independence).
    """
    food_store = food_store_stock(scenario.food_store0)
    water_store = water_store_stock(scenario.water_store0)
    o2_store = o2_store_stock(scenario.o2_store0)
    exhaled_co2 = boundary.sink(EXHALED_CO2, Quantity.CARBON, 0.0)
    fecal_waste = boundary.sink(FECAL_WASTE, Quantity.CARBON, 0.0)
    crew_humidity = boundary.sink(CREW_HUMIDITY, Quantity.WATER, 0.0)
    urine = boundary.sink(URINE, Quantity.WATER, 0.0)
    o2_consumed = boundary.sink(CREW_O2_CONSUMED, Quantity.OXYGEN, 0.0)
    stocks = {
        s.id: s
        for s in (
            food_store,
            water_store,
            o2_store,
            exhaled_co2,
            fecal_waste,
            crew_humidity,
            urine,
            o2_consumed,
        )
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        OxygenConsumption(
            OXYGEN_CONSUMPTION, 0, o2_store=O2_STORE, o2_consumed=CREW_O2_CONSUMED
        ),
        FoodMetabolism(
            FOOD_METABOLISM,
            0,
            food_store=FOOD_STORE,
            exhaled_co2=EXHALED_CO2,
            fecal_waste=FECAL_WASTE,
            params=params,
        ),
        WaterBalance(
            WATER_BALANCE,
            0,
            water_store=WATER_STORE,
            crew_humidity=CREW_HUMIDITY,
            urine=URINE,
            params=params,
        ),
    ]
    return state, Registry(flows, stocks)


def crew_resolver(
    scenario: CrewScenario = DEFAULT_CREW_SCENARIO,
) -> SourceResolver:
    """The forcing: the three constant crew intake rates.

    Much simpler than ``power_resolver`` — no diurnal schedule, no derived load. Crew
    has no restoring force to balance against (the stores simply deplete), so the rates
    are plain scenario data. Standalone forcing (no shared stocks — coupling to the
    ECLSS cabin / biosphere is Phase 6); a flow cannot tell forcing from a shared stock
    (#16), so this same domain code runs coupled later.
    """
    return SourceResolver(
        forcings={
            O2_INTAKE_VAR: constant(scenario.o2_intake_rate),
            FOOD_INTAKE_VAR: constant(scenario.food_intake_rate),
            WATER_INTAKE_VAR: constant(scenario.water_intake_rate),
        }
    )


def depletion_times(scenario: CrewScenario = DEFAULT_CREW_SCENARIO) -> DepletionTimes:
    """The closed-form per-store time-to-depletion ``store0 / rate`` (seconds).

    Because each store is drawn at a constant forced rate with no resupply, its
    endurance is exactly ``store0 / rate`` — the mission-margin number. The validation
    asserts the mission horizon is a fraction of every store's endurance (so every store
    stays positive and ``rationed == 0``). Emergent from the scenario data alone (no
    params — the split fractions do not change *how fast* a store depletes, only where
    its output goes).
    """
    return DepletionTimes(
        food_store=scenario.food_store0 / scenario.food_intake_rate,
        water_store=scenario.water_store0 / scenario.water_intake_rate,
        o2_store=scenario.o2_store0 / scenario.o2_intake_rate,
    )


def run_crew(
    integrator: CrewIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    The ``run_power`` / ``run_eclss`` analogue (no reset hook — Crew has no phenology /
    intervention). ``states`` is the full trajectory including the initial state (length
    ``steps + 1``) — the validation reads it for the monotone depletion, the monotonic
    sinks, and the every-step three-quantity ledger, and the golden pins its final
    state. ``total_rationed`` sums the Euler-backstop firings (the validation asserts
    ``== 0``); ``events`` are extinction events (empty — no POPULATION stock). The
    every-step conservation gate runs inside ``integrator.step_report`` (covering CARBON
    / OXYGEN / WATER), so a completed run is itself proof the ledger balanced every
    step.
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
