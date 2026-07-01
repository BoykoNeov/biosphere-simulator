"""The standalone ECLSS system: assemble + constant resolver + run harness (Step 6).

The ECLSS analogue of ``domains.power.system`` / ``domains.thermal.system`` — the thin
composition layer that turns the stocks/flows into a runnable standalone system:
:func:`build_eclss` (initial ``State`` + flow ``Registry``), :func:`eclss_resolver` (the
constant forced crew load — like Thermal's constant ``heat_load``, much simpler than
Power's diurnal + derived-load resolver, because each control loop is the restoring
force), and :func:`run_eclss` (the stepping driver, the ``run_power`` / ``run_thermal``
analogue **minus** any reset hook).

**What this proves.** ``CrewMetabolism`` is *forced*, but the three control flows
(``CO2Scrubber`` / ``Condenser`` / ``O2Makeup``) are **donor-/demand-controlled**, so —
unlike Power's forced-only ``BOUNDED_SOC`` — the system has **genuine restoring forces**
and each species reaches an **emergent steady state** (:func:`steady_state`). The
standalone validation shows: **all three quantities conserved every step** over the
augmented ledger (``Δcabin + Δboundary ≈ 0`` per quantity — the Phase-5 payload, now
*three* quantities at once, the "first multi-quantity sibling" content), ``rationed ==
0`` (structural for CO₂/H₂O, well-fed sizing for O₂), **monotonic** ``co2_removed`` /
``humidity_condensate`` diagnostics, and geometric convergence to the steady states (two
runs differing in one species' initial amount contract by the exact ``d_n = d_0·(1 −
k·dt)^n`` law — linear, not Thermal's nonlinear contraction).

**"Closed" is decision #13's augmented sense — NOT actually closed, and NOT
atom-coupled.** Each quantity balances over its augmented system (cabin pools + boundary
reservoirs) every step, even though (a) matter physically leaves to the
scrubber/condenser sinks and enters from the O₂ tank, and (b) the crew seam does **not**
tie the species' atoms together (real respiration binds inhaled O₂ into exhaled CO₂/H₂O;
standalone ECLSS routes them through three decoupled boundary reservoirs). The atomic
coupling is a **Phase-6** act (crew coupling + composition stocks); standalone ECLSS
builds the cabin-air **receiver**.

**No loss-sinks / no POPULATION stock** — as for Power/Thermal: extinction never fires
(the cabin stocks are POOLs), so no quantity needs a loss-sink. NITROGEN and ENERGY are
absent from the state ⇒ the conservation gate skips them (``ledger.get → None``);
CARBON, OXYGEN and WATER are checked every step.

**RK4 ≢ Euler here (a tolerance agreement).** The three control flows are
state-dependent, so the forced-only bit-identity does not hold (``k1 ≠ k2``); the
integrators agree to ``O(dt²)``. The validation runs Euler (the locked scheme).

Pure stdlib only in the spine; the YAML/pydantic ECLSS params load via ``loader.py``.
"""

from dataclasses import dataclass

from domains.eclss.flows import (
    CO2Scrubber,
    Condenser,
    CrewMetabolism,
    EclssParams,
    O2Makeup,
)
from domains.eclss.scenario import DEFAULT_ECLSS_SCENARIO, EclssScenario
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_PRODUCTION_VAR,
    CO2_REMOVED,
    H2O_PRODUCTION_VAR,
    HUMIDITY_CONDENSATE,
    METABOLIC_CO2_SOURCE,
    METABOLIC_H2O_SOURCE,
    METABOLIC_O2_SINK,
    O2_CONSUMPTION_VAR,
    O2_SUPPLY,
    cabin_co2_stock,
    cabin_h2o_stock,
    cabin_o2_stock,
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

EclssIntegrator = EulerIntegrator | Rk4Integrator

# Flow ids (canonical, ASCII so str sort == future Rust byte sort, #15). Mirrors the ids
# the per-flow tests use.
CREW_METABOLISM: FlowId = FlowId("eclss.crew_metabolism")
CO2_SCRUBBER: FlowId = FlowId("eclss.co2_scrubber")
CONDENSER: FlowId = FlowId("eclss.condenser")
O2_MAKEUP: FlowId = FlowId("eclss.o2_makeup")


@dataclass(frozen=True)
class SteadyState:
    """The emergent per-species cabin steady states (see :func:`steady_state`)."""

    cabin_o2: float
    cabin_co2: float
    cabin_h2o: float


def build_eclss(
    params: EclssParams,
    scenario: EclssScenario = DEFAULT_ECLSS_SCENARIO,
) -> tuple[State, Registry]:
    """Assemble the standalone ECLSS system's initial ``State`` and flow ``Registry``.

    Nine stocks — the three ``eclss.cabin_*`` POOLs (initial inventories from the
    scenario), the unclamped ``boundary.o2_supply`` source, the ``co2_removed`` /
    ``humidity_condensate`` monotonic sinks, and the three ``metabolic_*`` crew-seam
    reservoirs (all boundary reservoirs start at 0) — and the four flows: the forced
    multi-quantity ``CrewMetabolism`` + the three donor-/demand-controlled control loops
    ``CO2Scrubber`` / ``Condenser`` / ``O2Makeup`` (carrying the loaded params). **No
    loss-sinks** (no POPULATION stock). The ``Registry`` re-sorts flows by id, so the
    build order is inert (registration-order independence).
    """
    cabin_o2 = cabin_o2_stock(scenario.cabin_o2_0)
    cabin_co2 = cabin_co2_stock(scenario.cabin_co2_0)
    cabin_h2o = cabin_h2o_stock(scenario.cabin_h2o_0)
    o2_supply = boundary.source(O2_SUPPLY, Quantity.OXYGEN, 0.0)
    co2_removed = boundary.sink(CO2_REMOVED, Quantity.CARBON, 0.0)
    humidity_condensate = boundary.sink(HUMIDITY_CONDENSATE, Quantity.WATER, 0.0)
    metabolic_o2_sink = boundary.sink(METABOLIC_O2_SINK, Quantity.OXYGEN, 0.0)
    metabolic_co2_source = boundary.source(METABOLIC_CO2_SOURCE, Quantity.CARBON, 0.0)
    metabolic_h2o_source = boundary.source(METABOLIC_H2O_SOURCE, Quantity.WATER, 0.0)
    stocks = {
        s.id: s
        for s in (
            cabin_o2,
            cabin_co2,
            cabin_h2o,
            o2_supply,
            co2_removed,
            humidity_condensate,
            metabolic_o2_sink,
            metabolic_co2_source,
            metabolic_h2o_source,
        )
    }
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        CrewMetabolism(
            CREW_METABOLISM,
            0,
            cabin_o2=CABIN_O2,
            cabin_co2=CABIN_CO2,
            cabin_h2o=CABIN_H2O,
            metabolic_o2_sink=METABOLIC_O2_SINK,
            metabolic_co2_source=METABOLIC_CO2_SOURCE,
            metabolic_h2o_source=METABOLIC_H2O_SOURCE,
        ),
        CO2Scrubber(
            CO2_SCRUBBER, 0, cabin_co2=CABIN_CO2, co2_removed=CO2_REMOVED, params=params
        ),
        Condenser(
            CONDENSER,
            0,
            cabin_h2o=CABIN_H2O,
            humidity_condensate=HUMIDITY_CONDENSATE,
            params=params,
        ),
        O2Makeup(O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=CABIN_O2, params=params),
    ]
    return state, Registry(flows, stocks)


def eclss_resolver(
    scenario: EclssScenario = DEFAULT_ECLSS_SCENARIO,
) -> SourceResolver:
    """The forcing: the three constant crew metabolic rates.

    Much simpler than ``power_resolver`` — no diurnal schedule, no derived load (like
    Thermal). Each control loop is the restoring force, so any constant crew load yields
    a unique stable steady state per species (see :func:`steady_state`); the rates are
    plain scenario data. Standalone forcing (no shared stocks — coupling to the Crew
    domain is Phase 6); a flow cannot tell forcing from a shared stock (#16), so this
    same domain code runs coupled later.
    """
    return SourceResolver(
        forcings={
            O2_CONSUMPTION_VAR: constant(scenario.o2_consumption_rate),
            CO2_PRODUCTION_VAR: constant(scenario.co2_production_rate),
            H2O_PRODUCTION_VAR: constant(scenario.h2o_production_rate),
        }
    )


def steady_state(
    params: EclssParams, scenario: EclssScenario = DEFAULT_ECLSS_SCENARIO
) -> SteadyState:
    """The emergent per-species cabin steady states (mol / mol / kg), a closed form.

    At steady state each control loop's removal/supply balances the crew load: ``co2_eq
    = P_co2 / k_scrub`` (scrubber balances CO₂ production), ``h2o_eq = P_h2o / k_cond``
    (condenser balances humidity production), ``o2_eq = o2_setpoint − Con_o2 /
    k_makeup`` (makeup balances O₂ consumption, so cabin O₂ sits just below the
    setpoint). Closed-form (no iteration) — each single-species steady state is
    algebraic. They are *emergent* from the crew load + params, never stored (each
    control loop is the balance — no derived load, unlike Power). The validation asserts
    the run converges here.
    """
    return SteadyState(
        cabin_o2=params.o2_setpoint
        - scenario.o2_consumption_rate / params.o2_makeup_gain,
        cabin_co2=scenario.co2_production_rate / params.co2_scrub_rate,
        cabin_h2o=scenario.h2o_production_rate / params.condense_rate,
    )


def run_eclss(
    integrator: EclssIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    The ``run_power`` / ``run_thermal`` analogue (no reset hook — ECLSS has no phenology
    / intervention). ``states`` is the full trajectory including the initial state
    (length ``steps + 1``) — the validation reads it for the steady-state convergence,
    the monotonic sinks, and the every-step three-quantity ledger, and the golden pins
    its final state. ``total_rationed`` sums the Euler-backstop firings (the validation
    asserts ``== 0``); ``events`` are extinction events (empty — no POPULATION stock).
    The every-step conservation gate runs inside ``integrator.step_report`` (covering
    CARBON / OXYGEN / WATER), so a completed run is itself proof the ledger balanced
    every step.
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
