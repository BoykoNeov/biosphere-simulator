"""Phase-5 Step 5: the standalone Thermal run — equilibrium-temperature validation.

Step 5 assembles the Thermal stocks/flows into a runnable standalone system
(``build_thermal`` / ``thermal_resolver`` / ``run_thermal``) and validates it on the
**equilibrium-temperature** scenario: a cold node under a constant heat load that warms
until Stefan-Boltzmann radiation to deep space balances the input.

**Honest framing (the contrast with Power).** Power's two flows were both *forced*, so
its SOC was a restoring-force-free accumulator and its boundedness had to be
*constructed* by an exactly-balanced derived load. Thermal is genuinely different:
``RadiatorReject`` is **donor-controlled and nonlinear** (``T⁴``), so the system has a
**real restoring force** and a **genuine emergent equilibrium temperature** ``T_eq`` —
any constant load lands there, no tuning. So the non-vacuous claims this validates are:

* **ENERGY conserved every step** — the augmented ledger (``heat_source + node +
  space``) balances to round-off (the energy-closure payload).
* **``rationed == 0`` by sizing** — ``τ = C/(4εσA·T_eq³) >> dt`` (tens of steps) keeps
  Euler from overshooting the nonlinear radiator (the Power ``LoadDraw`` well-fed
  discipline, NOT a structural ``k·dt < 1`` claim).
* **the emergent equilibrium** — ``T`` converges to ``equilibrium_temperature`` (the
  radiator balances the load), and two runs from different initial heat **contract
  together** (monotone, not geometric — the ``T⁴`` nonlinearity). The no-radiator
  contrast (forced input alone keeps the difference *constant*) isolates the radiator as
  the restoring force.
* a **monotonic** heat-rejected diagnostic (the ``space`` sink), **determinism**, **RK4
  ≢ Euler** (a tolerance agreement — the radiator is state-dependent), and
  **registration-order independence**.

Pure-stdlib spine; the radiator params load from the committed ``radiator.yaml``.
"""

import math

import pytest

from domains.thermal.flows import HeatInput, radiated_power, temperature
from domains.thermal.loader import load_thermal_params
from domains.thermal.scenario import (
    EQUILIBRIUM_SCENARIO,
    EQUILIBRIUM_STEPS,
    ThermalScenario,
)
from domains.thermal.stocks import HEAT_SOURCE, NODE, SPACE
from domains.thermal.system import (
    HEAT_INPUT,
    build_thermal,
    equilibrium_temperature,
    relaxation_time,
    run_thermal,
    thermal_resolver,
)
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

_PARAMS = load_thermal_params()
_SCENARIO = EQUILIBRIUM_SCENARIO
_STEPS = EQUILIBRIUM_STEPS
_DT = _SCENARIO.dt_seconds
_T_EQ = equilibrium_temperature(_PARAMS, _SCENARIO)


def _run(
    scenario: ThermalScenario = _SCENARIO,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
) -> tuple[list[State], int, tuple]:
    state, registry = build_thermal(_PARAMS, scenario)
    resolver = thermal_resolver(scenario)
    return run_thermal(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, _STEPS
    )


@pytest.fixture(scope="module")
def equilibrium() -> tuple[list[State], int, tuple]:
    return _run()


def _node_temps(states: list[State]) -> list[float]:
    return [
        temperature(
            s.stocks[NODE].amount,
            heat_capacity=_PARAMS.heat_capacity,
            space_temperature=_PARAMS.space_temperature,
        )
        for s in states
    ]


def _energy_total(state: State) -> float:
    # The augmented-system ENERGY total: the unclamped source (cumulative supply, goes
    # very negative) + the node POOL + the monotonic space sink.
    return (
        state.stocks[HEAT_SOURCE].amount
        + state.stocks[NODE].amount
        + state.stocks[SPACE].amount
    )


# --- the payload: ENERGY conserved every step over the augmented system -------------
def test_thermal_energy_conserved_every_step(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # Per-step the ENERGY ledger residual (Δheat_source + Δnode + Δspace) is ≈ 0 —
    # energy closure (P5.1), now carried by a nonlinear radiator (not just the forced
    # Power flows).
    states, _, _ = equilibrium
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= 1e-6


def test_thermal_energy_total_is_invariant(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # Integral form: the total ENERGY across all three stocks never leaves the initial
    # node0 (every flow has Σ legs == 0). Heat moves source → node → space; none
    # vanishes.
    states, _, _ = equilibrium
    total0 = _energy_total(states[0])
    assert total0 == pytest.approx(_SCENARIO.node0)  # source/space start at 0
    for s in states:
        assert math.isclose(_energy_total(s), total0, rel_tol=0.0, abs_tol=1e-4)


def test_thermal_only_energy_is_present(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # Thermal is a pure-ENERGY domain: the ledger names ENERGY and nothing else.
    states, _, _ = equilibrium
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == {Quantity.ENERGY}


# --- rationed == 0 / events == () : well-fed sizing (τ >> dt) ------------------------
def test_thermal_relaxation_time_is_many_steps() -> None:
    # The load-bearing sizing constraint: τ = C/(4εσA·T_eq³) >> dt keeps Euler from
    # overshooting the nonlinear radiator. Here τ ≈ 65 steps — comfortably many.
    assert relaxation_time(_PARAMS, _SCENARIO) / _DT > 20.0


def test_thermal_never_rations(equilibrium: tuple[list[State], int, tuple]) -> None:
    # The radiator rejects only ≈0.4% of the stored heat per step at equilibrium (τ >>
    # dt), so the Euler backstop never fires — positivity by sizing (the LoadDraw
    # discipline).
    _, rationed, _ = equilibrium
    assert rationed == 0


def test_thermal_no_events(equilibrium: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock ⇒ extinction can never fire ⇒ no events (and no loss-sink).
    _, _, events = equilibrium
    assert events == ()


# --- the emergent equilibrium temperature (the genuine attractor) -------------------
def test_thermal_equilibrium_balances_radiation_against_load() -> None:
    # The defining identity of T_eq: at T_eq the radiated power equals the forced load
    # (the closed-form equilibrium_temperature solves εσA(T_eq⁴ − T_space⁴) =
    # heat_load).
    q_eq = _PARAMS.heat_capacity * (_T_EQ - _PARAMS.space_temperature)
    assert math.isclose(
        radiated_power(q_eq, params=_PARAMS), _SCENARIO.heat_load_w, rel_tol=1e-9
    )


def test_thermal_warms_monotonically_from_cold(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # From node0 = 0 (T = T_space) the node warms monotonically (input > rejection all
    # the way up, since rejection rises with T toward the load) — a monotone approach,
    # not a periodic swing (there is no diurnal forcing).
    states, _, _ = equilibrium
    temps = _node_temps(states)
    assert temps[0] == pytest.approx(_PARAMS.space_temperature)  # starts at the floor
    assert all(a <= b + 1e-9 for a, b in zip(temps, temps[1:], strict=False))


def test_thermal_converges_to_equilibrium_temperature(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # After ~11 τ the node temperature sits within a narrow band of the emergent T_eq —
    # the restoring force pulled it there (a genuine attractor, unlike Power's
    # constructed balance). The tiny residual gap is the T⁴ radiator being near-inert
    # while cold.
    states, _, _ = equilibrium
    t_final = _node_temps(states)[-1]
    assert abs(t_final - _T_EQ) < 0.5  # ≈ 0.04 K achieved; 0.5 K is comfortable margin
    # And it genuinely climbed most of the way (a real approach, not a flat line).
    assert t_final - _PARAMS.space_temperature > 0.9 * (
        _T_EQ - _PARAMS.space_temperature
    )


# --- the restoring force: two runs contract (monotone, not geometric) ---------------
def test_thermal_two_runs_contract_to_the_attractor() -> None:
    # Two runs differing ONLY in node0 (identical forcing ⇒ the HeatInput legs cancel in
    # the difference, leaving only the radiator's). The nonlinear restoring force pulls
    # them together: |d_n| decreases MONOTONICALLY (not the exact geometric law
    # SelfDischarge had — T⁴ is nonlinear) and ends far smaller than it began.
    from dataclasses import replace

    q_eq = _PARAMS.heat_capacity * (_T_EQ - _PARAMS.space_temperature)
    cold, _, _ = _run()  # node0 = 0 (below equilibrium)
    hot, _, _ = _run(replace(_SCENARIO, node0=2.0 * q_eq))  # above equilibrium
    diff = [
        abs(h.stocks[NODE].amount - c.stocks[NODE].amount)
        for c, h in zip(cold, hot, strict=False)
    ]
    # Non-increasing every step (``<=`` not ``<``: if the horizon is ever raised until
    # both land on the same FP fixed point, ``d_n → 0`` and a strict ``<`` would fail
    # though the physics is fine — the "ended much smaller" check below carries the
    # actual contraction claim).
    assert all(b <= a for a, b in zip(diff, diff[1:], strict=False))
    assert diff[-1] < 0.01 * diff[0]  # contracted by >100× over the horizon


def test_thermal_without_radiator_difference_is_constant() -> None:
    # The contrast that makes the contraction meaningful: with the radiator removed
    # (only the forced HeatInput), there is NO restoring force, so a node0 offset
    # propagates undecayed — d_n == d_0 for every n. The radiator is exactly what turns
    # this constant into a contraction (the Power forced-only-difference-is-constant
    # analogue).
    from dataclasses import replace

    other = replace(_SCENARIO, node0=_SCENARIO.node0 + 1.0e9)
    state_a, _ = build_thermal(_PARAMS, _SCENARIO)
    state_b, _ = build_thermal(_PARAMS, other)
    resolver = thermal_resolver(_SCENARIO)
    # A radiator-less registry: HeatInput alone (node grows unbounded, no rejection).
    only_input = [HeatInput(HEAT_INPUT, 0, heat_source=HEAT_SOURCE, node=NODE)]
    reg_a = Registry(only_input, state_a.stocks)
    reg_b = Registry(only_input, state_b.stocks)
    a, ra, _ = run_thermal(EulerIntegrator(reg_a), state_a, resolver, _DT, _STEPS)
    b, rb, _ = run_thermal(EulerIntegrator(reg_b), state_b, resolver, _DT, _STEPS)
    assert (ra, rb) == (0, 0)
    for sa, sb in zip(a, b, strict=False):
        assert sb.stocks[NODE].amount - sa.stocks[NODE].amount == pytest.approx(
            1.0e9, rel=0.0, abs=1e-3
        )


# --- the monotonic heat-rejected diagnostic -----------------------------------------
def test_thermal_space_sink_is_monotonic(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # space only ever receives (radiation is one-way to deep space), so it is
    # non-decreasing every step and strictly grows over the run — the free heat-rejected
    # accumulator (the permanent boundary Thermal cannot move inward).
    states, _, _ = equilibrium
    rejected = [s.stocks[SPACE].amount for s in states]
    assert all(b <= a for b, a in zip(rejected, rejected[1:], strict=False))
    assert rejected[-1] > rejected[0] > -1.0  # strictly accumulated, from 0


# --- determinism / integrator / registration-order independence ---------------------
def test_thermal_is_deterministic(
    equilibrium: tuple[list[State], int, tuple],
) -> None:
    # Bit-identical on a re-run (the golden's premise).
    states, rationed, events = equilibrium
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_thermal_rk4_agrees_with_euler_to_tolerance() -> None:
    # The radiator is state-dependent and nonlinear, so — UNLIKE Power's forced-only
    # BOUNDED_SOC (k1 = k2 = k3 = k4, bit-identical) — RK4 ≢ Euler bit-for-bit. They
    # agree to O(dt²): a real tolerance agreement, the SelfDischarge situation.
    euler, _, _ = _run(integrator_cls=EulerIntegrator)
    rk4, _, _ = _run(integrator_cls=Rk4Integrator)
    e_final = euler[-1].stocks[NODE].amount
    r_final = rk4[-1].stocks[NODE].amount
    assert r_final != e_final  # NOT bit-identical (the forced-only identity is broken)
    assert r_final == pytest.approx(e_final, rel=1e-4)  # but agree to tolerance


def test_thermal_registration_order_independent() -> None:
    # The Registry sorts flows by id, so building with the flows in the opposite order
    # yields a bit-identical run (#15).
    from domains.thermal.flows import RadiatorReject
    from domains.thermal.system import RADIATOR_REJECT

    state, _ = build_thermal(_PARAMS, _SCENARIO)
    reversed_flows = [
        RadiatorReject(RADIATOR_REJECT, 0, node=NODE, space=SPACE, params=_PARAMS),
        HeatInput(HEAT_INPUT, 0, heat_source=HEAT_SOURCE, node=NODE),
    ]
    reversed_registry = Registry(reversed_flows, state.stocks)
    resolver = thermal_resolver(_SCENARIO)
    states, rationed, events = run_thermal(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
