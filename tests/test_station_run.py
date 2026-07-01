"""Phase-6 Step 1 (P6.1): the coupled Power → Thermal station — heat-closure validation.

Step 1 stands up the ``src/station/`` assembly layer and proves it on the first
cross-domain seam: Power's dissipation legs, which standalone dumped into a terminal
``boundary.waste_heat`` sink, are **redirected into ``thermal.node``**, and the
Stefan-Boltzmann radiator rejects that **real** load to deep space. Single-quantity
(ENERGY) — the cleanest first integration — and the harness (``build_station`` /
``station_resolver`` / ``run_station``) every later step reuses.

**What this validates (the non-vacuous payload).**

* **ENERGY conserved every step over the *combined* ledger** — ``solar_source + battery
  + node + space`` balances to round-off (the energy-closure payload, now across two
  domains, not one). The combined total never leaves ``battery0 + node0``.
* **The seam is structural** — ``boundary.waste_heat`` and ``boundary.heat_source`` are
  **absent** from the station state (heat is not landing in a shadow sink); the node and
  the deep-space boundary are present. ``rationed == 0`` (τ >> dt sizing), ``events ==
  ()`` (no POPULATION stock).
* **Coupling is pure sink re-wiring — the donor is unperturbed.** The ``battery`` +
  ``solar_source`` trajectories are **bit-identical** to a standalone Power run: the
  Power flows do not read the node, only their heat *destination* changed. The single
  cleanest statement that coupling shares a stock and nothing else.
* **The radiator carries a REAL load** — per-day ``ΔSpace`` ≈ Power's per-day heat
  generation (all supplied solar, in daily balance); the sink's accumulation *rate*
  tracks Power's actual dissipation, quantitatively.
* **The node reaches an equilibrium set by dissipation** — its temperature stays within
  a narrow band of :func:`predicted_equilibrium_temperature` (the mean-power prediction;
  the true attractor sits slightly below by the T⁴-convexity offset). That this
  equilibrium is set by dissipation **independent of the initial condition** is proved
  non-circularly by the **two-start convergence** test (two ``node0`` values under
  identical Power forcing contract to one band — the radiator alone governs the
  difference), with the no-radiator contrast (difference stays constant) isolating the
  radiator as the restoring force.
* **Determinism**, **RK4 ≢ Euler** (a tolerance agreement — the radiator is nonlinear;
  the battery half stays bit-identical because it is forced), and **registration-order
  independence**.

Pure-stdlib spine; the charge + radiator params load from the committed sibling YAMLs.
"""

import math
from collections.abc import Mapping

import pytest

from domains.power.loader import load_charge_params
from domains.power.stocks import BATTERY, SOLAR_SOURCE, WASTE_HEAT
from domains.power.system import (
    build_power,
    daily_solar_energy,
    power_resolver,
    run_power,
)
from domains.thermal.flows import RadiatorReject, temperature
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import HEAT_SOURCE, NODE, SPACE
from domains.thermal.system import RADIATOR_REJECT
from simcore.conservation import compute_ledger
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State, Stock
from station.scenario import (
    CONTRACTION_DAYS,
    HEAT_CLOSURE_DAYS,
    HEAT_CLOSURE_SCENARIO,
    StationScenario,
)
from station.system import (
    build_station,
    equilibrium_node_heat,
    mean_dissipated_power,
    predicted_equilibrium_temperature,
    run_station,
    station_resolver,
)

_CHARGE = load_charge_params()
_THERMAL = load_thermal_params()
_SCENARIO = HEAT_CLOSURE_SCENARIO
_SPD = _SCENARIO.power.steps_per_day
_DT = _SCENARIO.power.dt_seconds
_STEPS = HEAT_CLOSURE_DAYS * _SPD
_T_EQ = predicted_equilibrium_temperature(_CHARGE, _THERMAL, _SCENARIO)
_Q_EQ = equilibrium_node_heat(_CHARGE, _THERMAL, _SCENARIO)

# The node stays within this of the mean-power prediction (≈0.001 K achieved over the
# week — started at equilibrium; the T⁴-convexity offset pulls it only slightly below).
_EQ_BAND_K = 1.0


def _run(
    scenario: StationScenario = _SCENARIO,
    node0: float | None = None,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    days: int = HEAT_CLOSURE_DAYS,
) -> tuple[list[State], int, tuple]:
    state, registry = build_station(_CHARGE, _THERMAL, scenario, node0=node0)
    resolver = station_resolver(_CHARGE, scenario)
    steps = days * scenario.power.steps_per_day
    return run_station(
        integrator_cls(registry), state, resolver, scenario.power.dt_seconds, steps
    )


@pytest.fixture(scope="module")
def station() -> tuple[list[State], int, tuple]:
    return _run()


def _node_temps(states: list[State]) -> list[float]:
    return [
        temperature(
            s.stocks[NODE].amount,
            heat_capacity=_THERMAL.heat_capacity,
            space_temperature=_THERMAL.space_temperature,
        )
        for s in states
    ]


def _energy_total(state: State) -> float:
    # The combined augmented ENERGY total across BOTH domains: the unclamped Power
    # source (cumulative supply, goes very negative) + the battery POOL + the thermal
    # node POOL + the monotonic deep-space sink.
    return (
        state.stocks[SOLAR_SOURCE].amount
        + state.stocks[BATTERY].amount
        + state.stocks[NODE].amount
        + state.stocks[SPACE].amount
    )


# --- the payload: ENERGY conserved every step over the COMBINED ledger ---------------
def test_station_energy_conserved_every_step(
    station: tuple[list[State], int, tuple],
) -> None:
    # Per-step the combined ENERGY ledger residual (Δsolar + Δbattery + Δnode + Δspace)
    # is ≈ 0 — energy closure across two coupled domains (the Step-1 payload). This
    # echoes the gate the integrator runs over the merged stock set; pinning it is the
    # receipt.
    states, _, _ = station
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= 1e-6


def test_station_energy_total_is_invariant(
    station: tuple[list[State], int, tuple],
) -> None:
    # Integral form: the combined ENERGY across all four stocks never leaves the initial
    # battery0 + node0 (every flow has Σ legs == 0). Now nonzero on BOTH stocks — solar
    # charges the battery, dissipation heats the node, the radiator rejects to space;
    # none vanishes.
    states, _, _ = station
    total0 = _energy_total(states[0])
    assert total0 == pytest.approx(_SCENARIO.power.battery0 + _Q_EQ)
    for s in states:
        assert math.isclose(_energy_total(s), total0, rel_tol=0.0, abs_tol=1e-2)


def test_station_only_energy_is_present(
    station: tuple[list[State], int, tuple],
) -> None:
    # The station is a pure-ENERGY assembly in Step 1: the combined ledger names ENERGY
    # and nothing else (no mass quantity — mass domains couple in later steps).
    states, _, _ = station
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == {Quantity.ENERGY}


# --- the seam is structural: no shadow sinks, well-fed, event-free -------------------
def test_station_redirection_is_structural(
    station: tuple[list[State], int, tuple],
) -> None:
    # The Step-1 seam moved Power's dissipation from boundary.waste_heat into
    # thermal.node and dropped Thermal's forced HeatInput. So neither stand-in reservoir
    # exists in the station state — heat is NOT quietly landing in a shadow sink; it
    # flows through the node and out the real deep-space boundary.
    stocks = station[0][0].stocks
    assert WASTE_HEAT not in stocks
    assert HEAT_SOURCE not in stocks
    assert {BATTERY, SOLAR_SOURCE, NODE, SPACE} == set(stocks)


def test_station_never_rations(station: tuple[list[State], int, tuple]) -> None:
    # Battery never empties (Power's well-fed sizing, unchanged) and the radiator
    # rejects only ≈0.4 %/step near equilibrium (τ >> dt) — the Euler backstop never
    # fires.
    _, rationed, _ = station
    assert rationed == 0


def test_station_no_events(station: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock anywhere in the coupled system ⇒ extinction can never fire.
    _, _, events = station
    assert events == ()


# --- coupling is pure sink re-wiring: the donor is unperturbed -----------------------
def test_station_battery_matches_standalone_power(
    station: tuple[list[State], int, tuple],
) -> None:
    # The single cleanest statement that coupling shares a stock and nothing else: the
    # Power flows do not read the node (only their heat DESTINATION changed), so the
    # battery + solar_source trajectories are BIT-IDENTICAL to a standalone Power run
    # over the same horizon. Verified empirically, step by step.
    station_states, _, _ = station
    state, registry = build_power(_CHARGE, _SCENARIO.power)
    resolver = power_resolver(_CHARGE, _SCENARIO.power)
    power_states, _, _ = run_power(
        EulerIntegrator(registry), state, resolver, _DT, _STEPS
    )
    assert len(station_states) == len(power_states)
    for st, po in zip(station_states, power_states, strict=False):
        assert st.stocks[BATTERY].amount == po.stocks[BATTERY].amount
        assert st.stocks[SOLAR_SOURCE].amount == po.stocks[SOLAR_SOURCE].amount


# --- the radiator carries a REAL load ------------------------------------------------
def test_station_space_carries_real_dissipation_load(
    station: tuple[list[State], int, tuple],
) -> None:
    # Quantitative "the radiator carries a real load": near equilibrium the node is
    # ~steady, so the deep-space sink's per-day accumulation ≈ Power's per-day heat
    # generation (all supplied solar, in daily balance) = mean_dissipated_power · day.
    states, _, _ = station
    space = [s.stocks[SPACE].amount for s in states]
    # Monotonic (radiation is one-way to deep space) and strictly accumulating from 0.
    assert all(b <= a for b, a in zip(space, space[1:], strict=False))
    assert space[-1] > space[0] == 0.0
    day_seconds = _SPD * _DT
    expected_per_day = mean_dissipated_power(_CHARGE, _SCENARIO) * day_seconds
    assert math.isclose(
        expected_per_day, daily_solar_energy(_SCENARIO.power), rel_tol=1e-9
    )
    for day in range(HEAT_CLOSURE_DAYS):
        rejected = space[(day + 1) * _SPD] - space[day * _SPD]
        assert rejected == pytest.approx(expected_per_day, rel=0.02)


# --- the node reaches an equilibrium set by dissipation ------------------------------
def test_station_node_stays_near_predicted_equilibrium(
    station: tuple[list[State], int, tuple],
) -> None:
    # Started at Q_eq, the node stays within a narrow band of the mean-power prediction
    # T_eq the whole run — the equilibrium is set by Power's ACTUAL dissipation (not
    # Thermal's constructed heat_load). The tiny residual gap is the T⁴-convexity offset
    # (the attractor sits just below the mean-power T_eq).
    temps = _node_temps(station[0])
    assert temps[0] == pytest.approx(_T_EQ)  # starts at the prediction
    assert all(abs(t - _T_EQ) < _EQ_BAND_K for t in temps)


def test_station_two_starts_converge_to_dissipation_equilibrium() -> None:
    # The non-circular core of "equilibrium set by dissipation": two runs differing ONLY
    # in node0 (identical Power forcing ⇒ the SolarCharge/LoadDraw legs cancel in the
    # difference, leaving only the radiator's) contract toward ONE band. The common band
    # is what dissipation sets — independent of the initial condition. |d_n| decreases
    # monotonically (nonlinear, not the geometric SelfDischarge law) and ends far
    # smaller.
    lo, _, _ = _run(node0=0.5 * _Q_EQ, days=CONTRACTION_DAYS)
    hi, _, _ = _run(node0=1.5 * _Q_EQ, days=CONTRACTION_DAYS)
    diff = [
        abs(h.stocks[NODE].amount - c.stocks[NODE].amount)
        for c, h in zip(lo, hi, strict=False)
    ]
    assert all(b <= a for a, b in zip(diff, diff[1:], strict=False))  # non-increasing
    assert diff[-1] < 0.1 * diff[0]  # contracted by >10× over ~3 τ


def test_station_without_radiator_difference_is_constant() -> None:
    # The contrast that makes the contraction meaningful: with the radiator removed
    # (only the two forced Power flows heating the node, no rejection), there is NO
    # restoring force, so a node0 offset propagates undecayed — d_n == d_0 for every
    # step. The radiator is exactly what turns this constant into a contraction (the
    # Thermal / Power forced-only-difference-is-constant analogue, now cross-domain).
    days = CONTRACTION_DAYS
    steps = days * _SPD
    resolver = station_resolver(_CHARGE, _SCENARIO)
    offset = 1.0e9
    state_a, _ = build_station(_CHARGE, _THERMAL, _SCENARIO, node0=_Q_EQ)
    state_b, _ = build_station(_CHARGE, _THERMAL, _SCENARIO, node0=_Q_EQ + offset)
    # Radiator-less registries: the two forced Power flows only (node grows unbounded).
    from domains.power.flows import LoadDraw, SolarCharge
    from domains.power.system import LOAD_DRAW, SOLAR_CHARGE

    def _no_radiator(stocks: Mapping[StockId, Stock]) -> Registry:
        flows = [
            SolarCharge(
                SOLAR_CHARGE,
                0,
                solar_source=SOLAR_SOURCE,
                battery=BATTERY,
                waste_heat=NODE,
                params=_CHARGE,
            ),
            LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=NODE),
        ]
        return Registry(flows, stocks)

    a, ra, _ = run_station(
        EulerIntegrator(_no_radiator(state_a.stocks)), state_a, resolver, _DT, steps
    )
    b, rb, _ = run_station(
        EulerIntegrator(_no_radiator(state_b.stocks)), state_b, resolver, _DT, steps
    )
    assert (ra, rb) == (0, 0)
    for sa, sb in zip(a, b, strict=False):
        assert sb.stocks[NODE].amount - sa.stocks[NODE].amount == pytest.approx(
            offset, rel=0.0, abs=1e-3
        )


# --- determinism / integrator / registration-order independence ---------------------
def test_station_is_deterministic(station: tuple[list[State], int, tuple]) -> None:
    # Bit-identical on a re-run (the golden's premise).
    states, rationed, events = station
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_station_rk4_agrees_with_euler_to_tolerance(
    station: tuple[list[State], int, tuple],
) -> None:
    # The radiator is state-dependent and nonlinear, so RK4 ≢ Euler bit-for-bit on the
    # NODE (a tolerance agreement, the Thermal situation). But the battery half is
    # forced, so it stays bit-identical between integrators — the coupling did not make
    # Power state-dependent.
    euler = station[0][-1]
    rk4 = _run(integrator_cls=Rk4Integrator)[0][-1]
    assert (
        rk4.stocks[NODE].amount != euler.stocks[NODE].amount
    )  # node not bit-identical
    assert rk4.stocks[NODE].amount == pytest.approx(euler.stocks[NODE].amount, rel=1e-4)
    assert (
        rk4.stocks[BATTERY].amount == euler.stocks[BATTERY].amount
    )  # forced ⇒ identical


def test_station_registration_order_independent() -> None:
    # The Registry sorts flows by id, so building with the flows in a shuffled order
    # yields a bit-identical run (#15). Rebuild the station registry reversed.
    from domains.power.flows import LoadDraw, SolarCharge
    from domains.power.system import LOAD_DRAW, SOLAR_CHARGE

    state, _ = build_station(_CHARGE, _THERMAL, _SCENARIO)
    reversed_flows = [
        RadiatorReject(RADIATOR_REJECT, 0, node=NODE, space=SPACE, params=_THERMAL),
        LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=NODE),
        SolarCharge(
            SOLAR_CHARGE,
            0,
            solar_source=SOLAR_SOURCE,
            battery=BATTERY,
            waste_heat=NODE,
            params=_CHARGE,
        ),
    ]
    reversed_registry = Registry(reversed_flows, state.stocks)
    resolver = station_resolver(_CHARGE, _SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
