"""The Station assembly layer: couple Power → Thermal at a shared stock (P6.1).

The first ``src/station/`` module — the layer that **imports every domain and owns all
cross-domain wiring** (finding #1 / the Phase-3 "coupling machinery lives outside the
coupled units" discipline, now cross-domain). **No domain imports another**; this module
imports both Power and Thermal and chooses *which stock id* each flow points at. Its
four pieces mirror the standalone ``domains.*.system`` modules one level up:

  * :func:`build_station` — concatenate the chosen domains' stocks + flows into **one**
    ``Registry`` + ``State``, with the cross-domain seam applied by id choice.
  * :func:`station_resolver` — the merged ``SourceResolver`` (for Step 1 it is exactly
    Power's — Thermal contributes no forcing once its ``HeatInput`` is dropped).
  * :func:`run_station` — the ``run_power`` / ``run_thermal`` stepping analogue.
  * :func:`equilibrium_node_heat` / :func:`predicted_equilibrium_temperature` /
    :func:`mean_dissipated_power` — the derived quantities tying the node's equilibrium
    to Power's **actual** output (the ``balanced_load_w`` / ``equilibrium_temperature``
    analogues, now cross-domain).

**The seam (Step 1): redirect Power's dissipation from ``boundary.waste_heat`` into
``thermal.node``.** Standalone Power's ``SolarCharge`` / ``LoadDraw`` take a
``waste_heat: StockId`` constructor arg and dumped into a terminal
``boundary.waste_heat`` sink; standalone Thermal's ``HeatInput`` forced a stand-in
``heat_load`` into its node from ``boundary.heat_source``. Here both stand-ins are
removed: the Power flows are constructed with ``waste_heat = thermal.node`` (their heat
now lands **in-system**), ``HeatInput`` is **not built** (Power's dissipation *is* the
heat input now), and ``RadiatorReject`` rejects that real load to the permanent
deep-space boundary. So ``boundary.waste_heat`` and ``boundary.heat_source`` **do not
exist** in the station state — the redirection is structural, not a shadow sink
(asserted in ``test_station_run.py``). **Zero domain change, zero core change** — purely
the assembly choosing which id to pass (finding #1).

**Single-quantity (ENERGY) — the cleanest possible first integration.** Every station
stock holds joules; the combined ledger balances ENERGY every step over ``solar_source +
battery + node + space`` (the payload). The radiator now carries a **real** load — the
node reaches an equilibrium set by Power's *actual* dissipation, not Thermal's
*constructed* ``heat_load``. This module also stands up the station harness every later
step reuses.

**The battery is unperturbed by coupling.** The Power flows do not read the node; only
their heat *destination* changed. So the battery + solar_source trajectories stay
bit-identical to standalone Power (verified in the run test) — the single cleanest
statement that coupling is pure sink re-wiring.

Pure stdlib only in the spine; the charge + radiator params load via the sibling
loaders.
"""

from domains.power.flows import ChargeParams, LoadDraw, SolarCharge
from domains.power.stocks import BATTERY, SOLAR_SOURCE, battery_stock
from domains.power.system import (
    LOAD_DRAW,
    SOLAR_CHARGE,
    balanced_load_w,
    daily_solar_energy,
    power_resolver,
)
from domains.thermal.flows import RadiatorReject, ThermalParams
from domains.thermal.scenario import ThermalScenario
from domains.thermal.stocks import NODE, SPACE, node_stock
from domains.thermal.system import RADIATOR_REJECT, equilibrium_temperature
from simcore import boundary
from simcore.environment import SourceResolver
from simcore.events import Event
from simcore.flow import Flow
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.scenario import HEAT_CLOSURE_SCENARIO, StationScenario

StationIntegrator = EulerIntegrator | Rk4Integrator


def mean_dissipated_power(
    charge: ChargeParams, scenario: StationScenario = HEAT_CLOSURE_SCENARIO
) -> float:
    """The time-average power (W) Power dissipates into the node, over one day.

    Every joule Power degrades becomes heat in the node: the charge-conversion loss
    ``(1−η_c)·S_day`` (``SolarCharge``) plus the 100 %-dissipative load ``load_w·day``
    (``LoadDraw``), where ``S_day`` is :func:`daily_solar_energy`. Divided by the day
    length it is the mean heat-input rate that sets the node's equilibrium. Under the
    daily-balanced scenario (``load_fraction = 1`` ⇒ SOC returns to ``battery0`` daily)
    this equals ``S_day / day`` exactly — *all* supplied solar ends up as heat — but the
    explicit two-source form here stays correct if the load is ever unbalanced. This is
    the cross-domain analogue of ``balanced_load_w``: a station quantity derived from
    the sibling's actual output, not a hand-set number.
    """
    ph = scenario.power
    day_seconds = ph.steps_per_day * ph.dt_seconds
    charge_loss_per_day = (1.0 - charge.charge_efficiency) * daily_solar_energy(ph)
    load_per_day = balanced_load_w(charge, ph) * day_seconds
    return (charge_loss_per_day + load_per_day) / day_seconds


def predicted_equilibrium_temperature(
    charge: ChargeParams,
    thermal_params: ThermalParams,
    scenario: StationScenario = HEAT_CLOSURE_SCENARIO,
) -> float:
    """The node's predicted equilibrium temperature ``T_eq`` (K), set by dissipation.

    Reuses Thermal's closed-form :func:`equilibrium_temperature` with the forced load
    replaced by Power's :func:`mean_dissipated_power` — i.e. the temperature at which
    Stefan-Boltzmann rejection balances Power's *actual* mean output. It is a
    **mean-power** prediction: the true diurnal attractor sits slightly *below* it by
    the T⁴-convexity offset (the average of the T⁴ ripple exceeds T⁴ of the average), a
    small, honest gap the run test allows a band for. Emergent from the sibling output +
    radiator params, never stored.
    """
    load = mean_dissipated_power(charge, scenario)
    return equilibrium_temperature(thermal_params, ThermalScenario(heat_load_w=load))


def equilibrium_node_heat(
    charge: ChargeParams,
    thermal_params: ThermalParams,
    scenario: StationScenario = HEAT_CLOSURE_SCENARIO,
) -> float:
    """The node's predicted equilibrium sensible heat ``Q_eq = C·(T_eq − T_space)`` (J).

    The initial node heat :func:`build_station` uses by default — the node starts at the
    equilibrium Power's actual dissipation implies
    (:func:`predicted_equilibrium_temperature`), so the coupled run begins near the
    attractor. This is a *prediction placed as an initial condition*, verified (not
    assumed) by the run test's boundedness + two-start convergence checks.
    """
    t_eq = predicted_equilibrium_temperature(charge, thermal_params, scenario)
    return thermal_params.heat_capacity * (t_eq - thermal_params.space_temperature)


def build_station(
    charge: ChargeParams,
    thermal_params: ThermalParams,
    scenario: StationScenario = HEAT_CLOSURE_SCENARIO,
    node0: float | None = None,
) -> tuple[State, Registry]:
    """Assemble the coupled Power → Thermal station's initial ``State`` + ``Registry``.

    Four stocks — the ``power.battery`` POOL (initial SOC ``scenario.power.battery0``),
    the unclamped ``boundary.solar_source``, the ``thermal.node`` POOL (initial heat
    ``node0``), and the ``boundary.space`` monotonic sink — and three ENERGY-balanced
    flows: ``SolarCharge`` + ``LoadDraw`` **wired to deposit into ``thermal.node``**
    (``waste_heat = NODE`` — the Step-1 seam) and ``RadiatorReject`` rejecting the node
    to deep space. Thermal's forced ``HeatInput`` is **not built** (Power's dissipation
    is the input now); ``boundary.waste_heat`` / ``boundary.heat_source`` are therefore
    absent — the redirection is structural. **No loss-sinks** (no POPULATION stock; only
    ENERGY is checked). The ``Registry`` re-sorts flows by id, so the build order is
    inert.

    ``node0`` defaults (``None``) to :func:`equilibrium_node_heat` — the node starts at
    the equilibrium Power's actual dissipation implies. Tests pass an explicit ``node0``
    to bracket that equilibrium (the two-start convergence proof that the equilibrium is
    set by dissipation, not by the initial condition).
    """
    ph = scenario.power
    battery = battery_stock(ph.battery0)
    solar_source = boundary.source(SOLAR_SOURCE, Quantity.ENERGY, 0.0)
    node_heat = (
        equilibrium_node_heat(charge, thermal_params, scenario)
        if node0 is None
        else node0
    )
    node = node_stock(node_heat)
    space = boundary.sink(SPACE, Quantity.ENERGY, 0.0)
    stocks = {s.id: s for s in (battery, solar_source, node, space)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        SolarCharge(
            SOLAR_CHARGE,
            0,
            solar_source=SOLAR_SOURCE,
            battery=BATTERY,
            waste_heat=NODE,  # the seam: dissipation lands in the thermal node
            params=charge,
        ),
        LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=NODE),
        RadiatorReject(
            RADIATOR_REJECT, 0, node=NODE, space=SPACE, params=thermal_params
        ),
    ]
    return state, Registry(flows, stocks)


def station_resolver(
    charge: ChargeParams, scenario: StationScenario = HEAT_CLOSURE_SCENARIO
) -> SourceResolver:
    """The merged station forcing — for Step 1, exactly Power's diurnal resolver.

    Thermal's standalone forcing (the constant ``heat_load``) fed only ``HeatInput``,
    which the coupling drops, so the station needs **no** Thermal forcing — the radiator
    reads the node stock, not ``env``. Hence the merged resolver is just
    ``power_resolver`` (``solar_power`` + the derived ``load_power``). Multi-resolver
    merging (with disjointness checks) is deferred to the first step that actually needs
    a second domain's forcing — the bespoke-until-second-instance rhythm.
    """
    return power_resolver(charge, scenario.power)


def run_station(
    integrator: StationIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    The ``run_power`` / ``run_thermal`` analogue (no reset hook — the station has no
    phenology / intervention in Step 1). ``states`` is the full trajectory including the
    initial state (length ``steps + 1``) — the validation reads it for the node band,
    the two-start convergence, the monotonic ``space`` sink, and the
    battery-matches-standalone check, and the golden pins its final state.
    ``total_rationed`` sums the Euler-backstop firings (validation asserts ``== 0``);
    ``events`` are extinction events (empty — no POPULATION stock). The every-step
    conservation gate runs inside ``integrator.step_report`` (covering ENERGY over the
    *combined* stock set), so a completed run is itself proof the combined ledger
    balanced every step.
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
