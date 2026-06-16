"""Step-10 tests: the two-domain Biosphere + Boundary demo.

This is the first assembly that wires *all* prior steps together — two domains,
three flows (one cross-domain), a ``SourceResolver``, both integrators, the
arbitration/extinction/conservation tail. The headline gates:

  * the cross-domain ``Harvest`` balances and touches both domains;
  * the **internal-resolver** case — ``light`` wired as a forcing schedule vs as the
    shared ``boundary.light`` stock — produces a **bit-identical** run (decision #16);
  * the well-fed demo never rations (Euler) / never over-draws (RK4);
  * the run conserves carbon, with the boundary reservoir carrying Inputs/Outputs;
  * the demo flows are dt-linear, and the run is registration-order-independent and
    deterministic.
"""

import pytest
from hypothesis import given
from hypothesis import strategies as st

from domains.biosphere.demo import (
    ATMOSPHERIC_C,
    BIOSPHERE,
    HARVEST,
    LIGHT,
    LIGHT_VAR,
    OUTSIDE_C,
    PLANT_C,
    DemoParams,
    build_demo,
    coupled_resolver,
    forcing_resolver,
    run,
)
from simcore.boundary import BOUNDARY_DOMAIN, loss_sink_id
from simcore.conservation import compute_ledger
from simcore.environment import Environment
from simcore.flow import assert_flow_balanced, domains_touched
from simcore.integrator import EulerIntegrator, Integrator, Rk4Integrator
from simcore.quantities import Quantity, StockKind
from simcore.registry import Registry
from simcore.state import State

INTEGRATORS = [EulerIntegrator, Rk4Integrator]


class _FixedLightEnv:
    """An Environment that resolves only ``light`` (for evaluating flows directly)."""

    def get(self, var: str) -> float:
        if var == LIGHT_VAR:
            return DemoParams().light
        raise KeyError(var)


def _amounts(state: State) -> dict[str, float]:
    return {sid: stock.amount for sid, stock in state.stocks.items()}


# --- assembly sanity --------------------------------------------------------
def test_build_demo_assembly_is_referentially_complete() -> None:
    state, reg = build_demo()
    # Two domains present in the index, with the expected membership.
    assert reg.domain_index[BIOSPHERE] == frozenset({ATMOSPHERIC_C, PLANT_C})
    assert BOUNDARY_DOMAIN in reg.domain_index
    # plant_c is the absorbing-eligible POPULATION; the carbon loss-sink is present
    # for its (here unused) extinction routing.
    assert state.stocks[PLANT_C].kind is StockKind.POPULATION
    assert loss_sink_id(Quantity.CARBON) in state.stocks
    # The energy driver is an unclamped boundary source, never a modeled pool.
    assert state.stocks[LIGHT].quantity is Quantity.ENERGY
    assert state.stocks[LIGHT].unclamped
    assert len(reg) == 3


def test_demo_flows_satisfy_the_integrator_protocol() -> None:
    _state, reg = build_demo()
    assert isinstance(EulerIntegrator(reg), Integrator)
    assert isinstance(Rk4Integrator(reg), Integrator)


# --- cross-domain Harvest ---------------------------------------------------
def test_harvest_is_cross_domain_and_carbon_balanced() -> None:
    params = DemoParams()
    state, reg = build_demo(params)
    harvest = next(f for f in reg.flows if f.id == HARVEST)

    result = harvest.evaluate(state, _FixedLightEnv(), params.dt)

    assert_flow_balanced(result, state.stocks)  # carbon balances once boundary counted
    assert domains_touched(result, state.stocks) == {BIOSPHERE, BOUNDARY_DOMAIN}


# --- internal-resolver indistinguishability (headline, decision #16) --------
@pytest.mark.parametrize("integrator_cls", INTEGRATORS)
def test_forcing_and_coupled_resolvers_run_bit_identically(
    integrator_cls: type,
) -> None:
    params = DemoParams()
    state, reg = build_demo(params)
    integ = integrator_cls(reg)

    forced, _, _ = run(integ, state, forcing_resolver(params), params.dt, 50)
    coupled, _, _ = run(integ, state, coupled_resolver(), params.dt, 50)

    # Bit-identical (exact ==): the reader cannot tell forcing from shared stock.
    assert _amounts(forced) == _amounts(coupled)


# --- well-fed backstop gate (rationing counter == 0 / no over-draw) ---------
def test_well_fed_euler_run_never_rations() -> None:
    params = DemoParams()
    state, reg = build_demo(params)

    _final, total_rationed, _events = run(
        EulerIntegrator(reg), state, coupled_resolver(), params.dt, 200
    )

    # A nonzero counter is a hard gate failure (dt too large / kinetics mis-scaled).
    assert total_rationed == 0


def test_well_fed_rk4_run_never_over_draws() -> None:
    params = DemoParams()
    state, reg = build_demo(params)

    # RK4 makes a needed scale_f < 1 an ArbitrationError; completing the run *is* the
    # assertion that no stage ever over-drew. rationed is always 0 for RK4.
    _final, total_rationed, _events = run(
        Rk4Integrator(reg), state, coupled_resolver(), params.dt, 200
    )

    assert total_rationed == 0


def test_well_fed_run_emits_no_extinction_events() -> None:
    params = DemoParams()
    state, reg = build_demo(params)

    _final, _rationed, events = run(
        EulerIntegrator(reg), state, coupled_resolver(), params.dt, 200
    )

    assert events == ()


# --- conservation + boundary exchange (#13) ---------------------------------
@pytest.mark.parametrize("integrator_cls", INTEGRATORS)
def test_demo_run_conserves_carbon_via_boundary_exchange(integrator_cls: type) -> None:
    params = DemoParams()
    state, reg = build_demo(params)

    final, _, _ = run(integrator_cls(reg), state, coupled_resolver(), params.dt, 100)

    ledger = {leg.quantity: leg for leg in compute_ledger(state, final)}
    carbon = ledger[Quantity.CARBON]
    # Total carbon unchanged: the boundary's Output exactly offsets ΔStored (#13).
    assert carbon.boundary_delta + carbon.stored_delta == pytest.approx(carbon.residual)
    assert carbon.residual == pytest.approx(0.0, abs=1e-9)
    assert carbon.boundary_delta == pytest.approx(-carbon.stored_delta, abs=1e-9)
    # The harvest actually drained carbon into the boundary reservoir.
    assert final.stocks[OUTSIDE_C].amount > state.stocks[OUTSIDE_C].amount
    # The energy driver carried no flux (structure-only, #8).
    energy = ledger[Quantity.ENERGY]
    assert energy.boundary_delta == 0.0
    assert energy.residual == 0.0


# --- dt-linearity of the demo flows (guards RK4 order; step-6 contract) -----
def test_demo_flow_legs_scale_linearly_with_dt() -> None:
    params = DemoParams()
    state, reg = build_demo(params)
    env: Environment = _FixedLightEnv()
    dt = params.dt

    for flow in reg.flows:
        legs1 = {leg.stock: leg.amount for leg in flow.evaluate(state, env, dt).legs}
        legs2 = {
            leg.stock: leg.amount for leg in flow.evaluate(state, env, 2 * dt).legs
        }
        assert legs1.keys() == legs2.keys()
        for sid, amount in legs1.items():
            assert legs2[sid] == pytest.approx(2.0 * amount)


# --- determinism + registration-order independence --------------------------
def test_demo_run_is_deterministic_across_runs() -> None:
    params = DemoParams()
    state, reg = build_demo(params)
    first, _, _ = run(Rk4Integrator(reg), state, coupled_resolver(), params.dt, 50)
    second, _, _ = run(Rk4Integrator(reg), state, coupled_resolver(), params.dt, 50)
    assert _amounts(first) == _amounts(second)


@pytest.mark.parametrize("integrator_cls", INTEGRATORS)
@given(perm=st.permutations(range(3)))
def test_demo_run_is_registration_order_independent(
    integrator_cls: type, perm: list[int]
) -> None:
    params = DemoParams()
    state, reg = build_demo(params)
    stocks = dict(state.stocks)
    flows = list(reg.flows)
    shuffled = [flows[i] for i in perm]

    base, _, _ = run(
        integrator_cls(Registry(flows, stocks)),
        state,
        coupled_resolver(),
        params.dt,
        30,
    )
    other, _, _ = run(
        integrator_cls(Registry(shuffled, stocks)),
        state,
        coupled_resolver(),
        params.dt,
        30,
    )

    assert _amounts(base) == _amounts(other)
