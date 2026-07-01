"""The standalone Thermal system: assemble + constant resolver + run harness (Step 5).

The Thermal analogue of ``domains.power.system`` — the thin composition layer that turns
the stocks/flows into a runnable standalone system: :func:`build_thermal` (initial
``State`` + flow ``Registry``), :func:`thermal_resolver` (the constant forced heat load,
much simpler than Power's diurnal + derived-load resolver, because the radiator is the
restoring force), and :func:`run_thermal` (the stepping driver, the ``run_power`` /
``run_season`` analogue **minus** any reset hook).

**What this proves.** ``HeatInput`` is *forced*, but ``RadiatorReject`` is
**donor-controlled and nonlinear** (Stefan-Boltzmann ``T⁴``), so — unlike Power's
forced-only ``BOUNDED_SOC`` — the system has a **genuine restoring force** and reaches
an **emergent equilibrium temperature** ``T_eq`` (:func:`equilibrium_temperature`) where
rejection balances input. The standalone validation shows: ENERGY **conserved every
step** over the augmented system (``Δheat_source + Δnode + Δspace ≈ 0`` — the Phase-5
energy-closure payload), ``rationed == 0`` by well-fed sizing (``τ = C/(4εσA·T_eq³) >>
dt``, :func:`relaxation_time`), a **monotonic** heat-rejected diagnostic (the ``space``
sink), and monotone convergence of the temperature to ``T_eq`` (two runs from different
initial heat contract together — a nonlinear, not geometric, contraction).

**"Closed" is decision #13's augmented sense — NOT actually closed.** Heat genuinely
leaves to ``boundary.space`` forever (radiation is the only vacuum rejection mode); the
augmented ledger (node + the two boundary reservoirs) balances every step even though
heat physically leaves. Standalone Thermal does not close the energy loop; it builds the
in-system **receiver** (node + radiator) that **Phase 6 wires Power's dissipation
into**.

**No loss-sinks / no POPULATION stock** — as for Power: extinction never fires (the node
is a POOL), the four mass quantities are absent (the gate skips them), only ENERGY is
checked.

**RK4 ≢ Euler here (a tolerance agreement).** The radiator is state-dependent and
nonlinear, so the forced-only bit-identity does not hold (``k1 ≠ k2``); the integrators
agree to ``O(dt²)``. The validation runs Euler (the locked scheme).

Pure stdlib only in the spine; the YAML/pydantic radiator params load via ``loader.py``.
"""

from domains.thermal.flows import (
    STEFAN_BOLTZMANN,
    HeatInput,
    RadiatorReject,
    ThermalParams,
)
from domains.thermal.scenario import DEFAULT_THERMAL_SCENARIO, ThermalScenario
from domains.thermal.stocks import (
    HEAT_LOAD_VAR,
    HEAT_SOURCE,
    NODE,
    SPACE,
    node_stock,
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

ThermalIntegrator = EulerIntegrator | Rk4Integrator

# Flow ids (canonical, ASCII so str sort == future Rust byte sort, #15). Mirrors the ids
# the per-flow tests use.
HEAT_INPUT: FlowId = FlowId("thermal.heat_input")
RADIATOR_REJECT: FlowId = FlowId("thermal.radiator_reject")


def build_thermal(
    params: ThermalParams,
    scenario: ThermalScenario = DEFAULT_THERMAL_SCENARIO,
) -> tuple[State, Registry]:
    """Assemble the standalone Thermal system's initial ``State`` and flow ``Registry``.

    Three stocks — the ``thermal.node`` POOL (initial sensible heat ``scenario.node0``),
    the unclamped ``boundary.heat_source`` (cumulative supply bookkeeping, starts 0),
    and the ``boundary.space`` monotonic sink (starts 0) — and the two energy-balanced
    flows ``HeatInput`` (2-leg forced) + ``RadiatorReject`` (2-leg donor-controlled,
    carrying the loaded radiator params). **No loss-sinks** (no POPULATION stock). The
    ``Registry`` re-sorts flows by id, so the build order is inert (registration-order
    independence).
    """
    node = node_stock(scenario.node0)
    heat_source = boundary.source(HEAT_SOURCE, Quantity.ENERGY, 0.0)
    space = boundary.sink(SPACE, Quantity.ENERGY, 0.0)
    stocks = {s.id: s for s in (node, heat_source, space)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        HeatInput(HEAT_INPUT, 0, heat_source=HEAT_SOURCE, node=NODE),
        RadiatorReject(RADIATOR_REJECT, 0, node=NODE, space=SPACE, params=params),
    ]
    return state, Registry(flows, stocks)


def thermal_resolver(
    scenario: ThermalScenario = DEFAULT_THERMAL_SCENARIO,
) -> SourceResolver:
    """The forcing: a constant ``heat_load`` (W) into the node.

    Much simpler than ``power_resolver`` — no diurnal schedule, no derived load. The
    radiator is the restoring force, so any constant load yields a unique stable
    ``T_eq`` (see :func:`equilibrium_temperature`); the load is plain scenario data.
    Standalone forcing (no shared stocks — coupling to Power's dissipation is Phase 6);
    a flow cannot tell forcing from a shared stock (#16), so this same domain code runs
    coupled later.
    """
    return SourceResolver(forcings={HEAT_LOAD_VAR: constant(scenario.heat_load_w)})


def equilibrium_temperature(
    params: ThermalParams, scenario: ThermalScenario = DEFAULT_THERMAL_SCENARIO
) -> float:
    """The emergent steady-state node temperature ``T_eq`` (K), a closed form.

    The temperature at which Stefan-Boltzmann rejection balances the forced input:
    ``ε·σ·A·(T_eq⁴ − T_space⁴) = heat_load`` ⇒ ``T_eq = (heat_load/(εσA) +
    T_space⁴)^(1/4)``. Closed-form (no iteration) because the single-node steady state
    is algebraic. It is *emergent* from the load + params, never stored (the radiator is
    the balance — no derived load, unlike Power). The validation asserts the run
    converges here.
    """
    driving = scenario.heat_load_w / (
        params.emissivity * STEFAN_BOLTZMANN * params.radiator_area
    )
    return (driving + params.space_temperature**4) ** 0.25


def relaxation_time(
    params: ThermalParams, scenario: ThermalScenario = DEFAULT_THERMAL_SCENARIO
) -> float:
    """The linearized relaxation time ``τ = C / (4·ε·σ·A·T_eq³)`` (s) near equilibrium.

    The e-folding time of a small temperature perturbation about ``T_eq`` (the
    derivative of the ``T⁴`` rejection linearized there). The **load-bearing sizing
    quantity**: ``τ >> dt`` is what keeps Euler from overshooting the nonlinear
    radiator, so ``rationed == 0`` holds by sizing (not structurally). The validation
    asserts ``τ / dt`` is large (tens of steps).
    """
    t_eq = equilibrium_temperature(params, scenario)
    return params.heat_capacity / (
        4.0 * params.emissivity * STEFAN_BOLTZMANN * params.radiator_area * t_eq**3
    )


def run_thermal(
    integrator: ThermalIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    The ``run_power`` analogue (no reset hook — Thermal has no phenology /
    intervention). ``states`` is the full trajectory including the initial state (length
    ``steps + 1``) — the validation reads it for the temperature convergence, the
    monotonic ``space`` sink, and the every-step ledger, and the golden pins its final
    state. ``total_rationed`` sums the Euler-backstop firings (the validation asserts
    ``== 0``); ``events`` are extinction events (empty — no POPULATION stock). The
    every-step conservation gate runs inside ``integrator.step_report`` (now covering
    ENERGY), so a completed run is itself proof the ledger balanced every step.
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
