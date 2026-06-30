"""The standalone Power system: assemble + diurnal resolver + run harness (P5.3).

The Power analogue of ``domains.biosphere.season`` — the thin composition layer that
turns the Step-2 stocks/flows into a runnable standalone system: :func:`build_power`
(initial ``State`` + flow ``Registry``), :func:`power_resolver` (the day/night forcing,
the ``weather_resolver`` analogue), and :func:`run_power` (the stepping driver, the
``run_season`` analogue **minus** the annual-reset hook — Power has no phenology).

**What this proves (honest framing).** Both Power flows are *forced*
(state-independent), so the system is a daily-balanced **forced linear** accumulator,
**not** an emergent limit cycle / attractor (there is no restoring force — see
``scenario`` for why the load is sized for exact daily balance). The standalone
validation it enables shows the machinery: ENERGY **conserved every step** over the
augmented system (``Δsolar_source + Δbattery + Δwaste_heat ≈ 0`` — the payload of
Phase-5 energy closure), ``rationed == 0`` by well-fed sizing, a bounded day-periodic
SOC swing, and a monotonic heat-generated diagnostic. "Closed" here is decision #13's
**augmented**-system sense: heat physically leaves to the ``boundary.waste_heat`` sink
(a legitimate Output), the seam the Thermal sibling later moves inward (the
water-cycle-closure analogue for energy).

**No loss-sinks.** Extinction routes a snapped *POPULATION* residual to a quantity's
loss-sink; Power has **no POPULATION stock** (the battery is a POOL), so extinction
never fires and no ENERGY loss-sink is needed. The four mass quantities are absent from
the state ⇒ the conservation gate skips them (``ledger.get → None``); only ENERGY is
checked.

**RK4 ≡ Euler here, bit-for-bit.** Because the flows are state-independent, every RK4
stage derivative is identical (``k1 = k2 = k3 = k4``) and the ⅙-combine reproduces
``k1`` exactly — so integrator choice is the *identity*, not numerical-robustness
evidence. The validation runs Euler (the locked scheme); the cross-check, if any,
asserts that identity.

Pure stdlib only in the spine; the YAML/pint charge param is loaded via ``loader.py``.
"""

import math

from domains.power.flows import ChargeParams, LoadDraw, SolarCharge
from domains.power.scenario import DEFAULT_POWER_SCENARIO, PowerScenario
from domains.power.stocks import (
    BATTERY,
    LOAD_POWER_VAR,
    SOLAR_POWER_VAR,
    SOLAR_SOURCE,
    WASTE_HEAT,
    battery_stock,
)
from simcore import boundary
from simcore.environment import Schedule, SourceResolver, constant
from simcore.events import Event
from simcore.flow import Flow
from simcore.ids import FlowId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

PowerIntegrator = EulerIntegrator | Rk4Integrator

# Flow ids (canonical, ASCII so str sort == future Rust byte sort, #15). Mirrors the
# ids the Step-2 per-flow tests already use.
SOLAR_CHARGE: FlowId = FlowId("power.solar_charge")
LOAD_DRAW: FlowId = FlowId("power.load_draw")


def build_power(
    charge_params: ChargeParams, scenario: PowerScenario = DEFAULT_POWER_SCENARIO
) -> tuple[State, Registry]:
    """Assemble the standalone Power system's initial ``State`` and flow ``Registry``.

    Three stocks — the ``power.battery`` POOL (initial SOC ``scenario.battery0``), the
    unclamped ``boundary.solar_source`` (cumulative supply bookkeeping, starts 0), and
    the ``boundary.waste_heat`` monotonic sink (starts 0) — and the two energy-balanced
    flows ``SolarCharge`` (3-leg, heat-named, carrying the loaded η_c) + ``LoadDraw``
    (2-leg dissipative). **No loss-sinks** (no POPULATION stock; see the module
    docstring). The ``Registry`` re-sorts flows by id, so the build order is inert
    (registration-order independence).
    """
    battery = battery_stock(scenario.battery0)
    solar_source = boundary.source(SOLAR_SOURCE, Quantity.ENERGY, 0.0)
    waste_heat = boundary.sink(WASTE_HEAT, Quantity.ENERGY, 0.0)
    stocks = {s.id: s for s in (battery, solar_source, waste_heat)}
    state = State(n=0, stocks=stocks, rng_seed=0)
    flows: list[Flow] = [
        SolarCharge(
            SOLAR_CHARGE,
            0,
            solar_source=SOLAR_SOURCE,
            battery=BATTERY,
            waste_heat=WASTE_HEAT,
            params=charge_params,
        ),
        LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=WASTE_HEAT),
    ]
    return state, Registry(flows, stocks)


def solar_schedule(scenario: PowerScenario = DEFAULT_POWER_SCENARIO) -> Schedule:
    """The diurnal solar forcing (W) — a half-sine over the daylight window, 0 at night.

    The ``weather_resolver`` table analogue, *computed* from the scenario rather than
    tabulated. Day phase ``φ = (n mod steps_per_day) / steps_per_day`` runs ``[0, 1)``
    with ``φ = 0`` at midnight and solar noon at ``φ = 0.5``. Over the daylight window
    ``[sunrise, sunset)`` (length ``daylight_hours/24``, centred at noon) the supply is
    ``peak · sin(π · (φ − sunrise) / daylight_fraction)`` — 0 at sunrise/sunset,
    ``peak`` at noon; outside the window it is exactly 0 (night). Periodic with period
    ``steps_per_day`` (``φ`` depends only on ``n mod steps_per_day``), so every day's
    solar profile — and energy — is identical, which is what makes the daily-balanced
    SOC return day-over-day. Independent of ``dt`` (a power, a function of phase only;
    the ``dt`` arg satisfies the ``Schedule`` signature).
    """
    spd = scenario.steps_per_day
    peak = scenario.solar_peak_w
    daylight_fraction = scenario.daylight_hours / 24.0
    sunrise = 0.5 - daylight_fraction / 2.0
    sunset = 0.5 + daylight_fraction / 2.0

    def schedule(n: int, dt: float) -> float:
        phase = (n % spd) / spd
        if sunrise <= phase < sunset:
            return peak * math.sin(math.pi * (phase - sunrise) / daylight_fraction)
        return 0.0

    return schedule


def daily_solar_energy(scenario: PowerScenario = DEFAULT_POWER_SCENARIO) -> float:
    """The discrete solar energy supplied over one day (J): ``Σ_day solar(n)·dt``.

    Summed over the ``steps_per_day`` steps of a single day in canonical step order
    (the same terms the integrator accumulates), so the derived balanced load makes the
    per-day battery delta cancel to round-off. The basis for :func:`balanced_load_w`.
    """
    solar = solar_schedule(scenario)
    dt = scenario.dt_seconds
    return sum(solar(n, dt) * dt for n in range(scenario.steps_per_day))


def balanced_load_w(
    charge_params: ChargeParams, scenario: PowerScenario = DEFAULT_POWER_SCENARIO
) -> float:
    """The forced load (W) that balances ``load_fraction`` of the daily *stored* solar.

    ``load_w = load_fraction · η_c · (Σ_day solar) / steps_per_day · (1/dt)``, i.e. the
    constant power whose daily energy ``load_w · steps_per_day · dt`` equals
    ``load_fraction · η_c · daily_solar_energy`` (the energy that actually *reaches* the
    battery). At ``load_fraction == 1`` the per-day charge and discharge cancel exactly,
    so SOC is bounded and day-periodic (see ``scenario``). This is the one place a Power
    resolver reads the charge param η_c — Power's load is intrinsically η_c-coupled.
    """
    stored_per_day = charge_params.charge_efficiency * daily_solar_energy(scenario)
    day_seconds = scenario.steps_per_day * scenario.dt_seconds
    return scenario.load_fraction * stored_per_day / day_seconds


def power_resolver(
    charge_params: ChargeParams, scenario: PowerScenario = DEFAULT_POWER_SCENARIO
) -> SourceResolver:
    """The day/night forcing: diurnal ``solar_power`` + the derived ``load_power``.

    ``solar_power`` is the half-sine :func:`solar_schedule`; ``load_power`` is the
    constant :func:`balanced_load_w` (forced demand, sized for daily balance). Both are
    forcing schedules (no shared stocks — standalone, no cross-domain coupling, which is
    Phase 6). A flow cannot tell forcing from a shared stock (#16), so this same domain
    code runs coupled later.
    """
    return SourceResolver(
        forcings={
            SOLAR_POWER_VAR: solar_schedule(scenario),
            LOAD_POWER_VAR: constant(balanced_load_w(charge_params, scenario)),
        }
    )


def run_power(
    integrator: PowerIntegrator,
    state: State,
    resolver: SourceResolver,
    dt: float,
    steps: int,
) -> tuple[list[State], int, tuple[Event, ...]]:
    """Step ``steps`` times, returning ``(states, total_rationed, events)``.

    The ``run_season`` analogue **without** the annual-reset hook (Power has no
    phenology / scenario intervention). ``states`` is the full trajectory including the
    initial state (length ``steps + 1``) — the validation reads it for the SOC swing,
    the day-over-day return, and the monotonic heat check, and the Step-4 golden pins
    its final state. ``total_rationed`` sums the Euler-backstop firings (the validation
    asserts ``== 0``); ``events`` are extinction events (empty — no POPULATION stock).
    The every-step conservation gate runs inside ``integrator.step_report`` (now
    covering ENERGY), so a completed run is itself proof the ledger balanced every step.
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
