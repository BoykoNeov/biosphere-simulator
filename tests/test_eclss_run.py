"""Phase-5 Step 6: the standalone ECLSS run — multi-quantity steady-state validation.

Step 6 assembles the ECLSS stocks/flows into a runnable standalone system
(``build_eclss`` / ``eclss_resolver`` / ``run_eclss``) and validates it on the
**steady-state** scenario: a clean cabin under a constant crew load, each species
relaxing until its control loop balances the crew load.

**Honest framing (the contrast with Power and Thermal).** Power's flows were both
*forced*, so its boundedness was *constructed* by an exactly-balanced derived load.
ECLSS is like Thermal — the three control flows (``CO2Scrubber`` / ``Condenser`` /
``O2Makeup``) are donor-/demand-controlled **restoring forces**, so each species has a
**genuine emergent steady state** (any constant crew load lands there, no tuning). But
ECLSS is **linear** (unlike Thermal's ``T⁴``), so its contraction is **geometric** — the
``SelfDischarge`` idiom, per species. The non-vacuous claims this validates:

* **all three quantities conserved every step** — the augmented ledger (cabin + boundary
  reservoirs) balances CARBON, OXYGEN and WATER to round-off (the first multi-quantity
  sibling — the payload).
* **``rationed == 0``** — structural for CO₂/H₂O (``k·dt < 1``, donor-controlled) and by
  well-fed sizing for O₂ (``cabin_o2`` never empties; ``o2_eq > 0``).
* **the emergent steady states** — each species converges to ``steady_state`` (its
  control loop balances the crew load), and two runs differing only in one species'
  initial amount **contract by the exact ``d_n = d_0·(1 − k·dt)^n`` law** (geometric,
  linear). The no-control contrast (crew load alone keeps the difference *constant*)
  isolates the control loop as the restoring force.
* **monotonic** ``co2_removed`` / ``humidity_condensate`` diagnostics, **determinism**,
  **RK4 ≢ Euler** (a tolerance agreement — the control flows are state-dependent), and
  **registration-order independence**.

Pure-stdlib spine; the ECLSS params load from the committed ``eclss.yaml``.
"""

from dataclasses import replace

import pytest

from domains.eclss.loader import load_eclss_params
from domains.eclss.scenario import (
    STEADY_STATE_SCENARIO,
    STEADY_STATE_STEPS,
    EclssScenario,
)
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_REMOVED,
    HUMIDITY_CONDENSATE,
    METABOLIC_CO2_SOURCE,
    METABOLIC_H2O_SOURCE,
    METABOLIC_O2_SINK,
    O2_SUPPLY,
)
from domains.eclss.system import (
    CO2_SCRUBBER,
    CONDENSER,
    CREW_METABOLISM,
    O2_MAKEUP,
    build_eclss,
    eclss_resolver,
    run_eclss,
    steady_state,
)
from simcore.conservation import compute_ledger
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

_PARAMS = load_eclss_params()
_SCENARIO = STEADY_STATE_SCENARIO
_STEPS = STEADY_STATE_STEPS
_DT = _SCENARIO.dt_seconds
_SS = steady_state(_PARAMS, _SCENARIO)


def _run(
    scenario: EclssScenario = _SCENARIO,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
) -> tuple[list[State], int, tuple]:
    state, registry = build_eclss(_PARAMS, scenario)
    resolver = eclss_resolver(scenario)
    return run_eclss(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, _STEPS
    )


@pytest.fixture(scope="module")
def steady() -> tuple[list[State], int, tuple]:
    return _run()


# --- the payload: all three quantities conserved every step -------------------------
def test_eclss_three_quantities_conserved_every_step(
    steady: tuple[list[State], int, tuple],
) -> None:
    # Per step the CARBON / OXYGEN / WATER ledger residuals are all ≈ 0 — the
    # augmented cabin+boundary ledger balances (the first multi-quantity sibling:
    # three quantities, not one, gated simultaneously).
    states, _, _ = steady
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for q in (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER):
            assert abs(ledger[q].residual) <= 1e-6


def test_eclss_only_the_three_mass_quantities_present(
    steady: tuple[list[State], int, tuple],
) -> None:
    # ECLSS carries CARBON / OXYGEN / WATER and nothing else (no ENERGY, no NITROGEN
    # stock ⇒ the gate skips them).
    states, _, _ = steady
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == {Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER}


def test_eclss_augmented_totals_are_invariant(
    steady: tuple[list[State], int, tuple],
) -> None:
    # Integral form: each quantity's total across its augmented stocks never moves
    # from the initial cabin inventory (every flow has Σ legs == 0 per quantity).
    # Carbon/water start at 0 in the cabin (source+sink+cabin sum to 0); oxygen
    # totals to cabin_o2_0.
    states, _, _ = steady

    def oxygen_total(s: State) -> float:
        return (
            s.stocks[CABIN_O2].amount
            + s.stocks[O2_SUPPLY].amount
            + s.stocks[METABOLIC_O2_SINK].amount
        )

    def carbon_total(s: State) -> float:
        return (
            s.stocks[CABIN_CO2].amount
            + s.stocks[CO2_REMOVED].amount
            + s.stocks[METABOLIC_CO2_SOURCE].amount
        )

    def water_total(s: State) -> float:
        return (
            s.stocks[CABIN_H2O].amount
            + s.stocks[HUMIDITY_CONDENSATE].amount
            + s.stocks[METABOLIC_H2O_SOURCE].amount
        )

    for s in states:
        assert oxygen_total(s) == pytest.approx(_SCENARIO.cabin_o2_0, abs=1e-9)
        assert carbon_total(s) == pytest.approx(0.0, abs=1e-9)
        assert water_total(s) == pytest.approx(0.0, abs=1e-9)


# --- rationed == 0 / events == () ----------------------------------------------------
def test_eclss_never_rations(steady: tuple[list[State], int, tuple]) -> None:
    # CO₂/H₂O positivity is structural (k·dt < 1); O₂ positivity is by sizing
    # (cabin_o2 settles at 8 mol, far from empty). The backstop never fires.
    _, rationed, _ = steady
    assert rationed == 0


def test_eclss_structural_positivity_fractions_below_one() -> None:
    # The structural claim for the donor-controlled loops: k·dt < 1 per step.
    assert _PARAMS.co2_scrub_rate * _DT < 1.0
    assert _PARAMS.condense_rate * _DT < 1.0


def test_eclss_no_events(steady: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock ⇒ extinction can never fire ⇒ no events (and no
    # loss-sink).
    _, _, events = steady
    assert events == ()


# --- the emergent steady states (genuine attractors) --------------------------------
def test_eclss_converges_to_the_steady_states(
    steady: tuple[list[State], int, tuple],
) -> None:
    # After many time constants each species sits within a narrow band of its
    # emergent steady state (co2_eq = 3.0, h2o_eq = 0.04, o2_eq = 8.0) — the
    # restoring forces pulled them there (genuine attractors, unlike Power's
    # constructed balance).
    states, _, _ = steady
    final = states[-1]
    assert final.stocks[CABIN_CO2].amount == pytest.approx(_SS.cabin_co2, abs=1e-6)
    assert final.stocks[CABIN_H2O].amount == pytest.approx(_SS.cabin_h2o, abs=1e-6)
    assert final.stocks[CABIN_O2].amount == pytest.approx(_SS.cabin_o2, abs=1e-6)


def test_eclss_species_move_monotonically_to_steady_state(
    steady: tuple[list[State], int, tuple],
) -> None:
    # From the clean cabin: CO₂ and H₂O rise monotonically from 0 to their eq; O₂
    # falls monotonically from the setpoint (10) to o2_eq (8). No periodic structure
    # (constant crew load ⇒ monotone relaxation).
    states, _, _ = steady
    co2 = [s.stocks[CABIN_CO2].amount for s in states]
    h2o = [s.stocks[CABIN_H2O].amount for s in states]
    o2 = [s.stocks[CABIN_O2].amount for s in states]
    assert co2[0] == pytest.approx(0.0) and h2o[0] == pytest.approx(0.0)
    assert o2[0] == pytest.approx(_SCENARIO.cabin_o2_0)
    assert all(a <= b + 1e-15 for a, b in zip(co2, co2[1:], strict=False))
    assert all(a <= b + 1e-15 for a, b in zip(h2o, h2o[1:], strict=False))
    assert all(a >= b - 1e-15 for a, b in zip(o2, o2[1:], strict=False))


# --- the restoring forces: two runs contract GEOMETRICALLY (per species) ------------
@pytest.mark.parametrize(
    ("field", "stock", "rate"),
    [
        ("cabin_co2_0", CABIN_CO2, _PARAMS.co2_scrub_rate),
        ("cabin_h2o_0", CABIN_H2O, _PARAMS.condense_rate),
        ("cabin_o2_0", CABIN_O2, _PARAMS.o2_makeup_gain),
    ],
)
def test_eclss_two_runs_contract_geometrically(
    field: str, stock: StockId, rate: float
) -> None:
    # Two runs differing ONLY in one species' initial amount (identical crew forcing
    # ⇒ the CrewMetabolism legs cancel in the difference, leaving only that species'
    # control loop). Because each loop is LINEAR, the difference decays by the EXACT
    # geometric law d_n = d_0·(1 − k·dt)^n (the SelfDischarge idiom — NOT Thermal's
    # nonlinear monotone contraction). O₂ uses cabin_o2_0 − 2 so both runs stay ≤
    # setpoint (makeup only adds).
    base_amount = getattr(_SCENARIO, field)
    offset_amount = base_amount - 2.0 if field == "cabin_o2_0" else base_amount + 1.0
    a, _, _ = _run()
    b, _, _ = _run(replace(_SCENARIO, **{field: offset_amount}))
    d0 = abs(b[0].stocks[stock].amount - a[0].stocks[stock].amount)
    decay = 1.0 - rate * _DT
    for n, (x, y) in enumerate(zip(a, b, strict=False)):
        d = abs(y.stocks[stock].amount - x.stocks[stock].amount)
        assert d == pytest.approx(d0 * decay**n, abs=1e-12)
    # And it genuinely contracted (a real approach, not a flat line).
    final_diff = abs(b[-1].stocks[stock].amount - a[-1].stocks[stock].amount)
    assert final_diff < 1e-6 * d0


def test_eclss_without_control_difference_is_constant() -> None:
    # The contrast that makes the contraction meaningful: with the control flows
    # removed (only CrewMetabolism), there is NO restoring force, so a cabin_co2
    # offset propagates undecayed — d_n == d_0 for every n (the Power/Thermal
    # forced-only analogue). O₂ consumption is zeroed here (``base``) so that
    # removing O2Makeup does not deplete cabin_o2 and trip the backstop — an
    # artifact of dropping the makeup, unrelated to the CO₂ restoring-force point
    # this contrast isolates (CrewMetabolism only *produces* CO₂, so cabin_co2 grows
    # unbounded and an offset simply persists).
    from domains.eclss.flows import CrewMetabolism

    base = replace(_SCENARIO, o2_consumption_rate=0.0)
    other = replace(base, cabin_co2_0=base.cabin_co2_0 + 1.0)
    state_a, _ = build_eclss(_PARAMS, base)
    state_b, _ = build_eclss(_PARAMS, other)
    resolver = eclss_resolver(base)
    only_crew = [
        CrewMetabolism(
            CREW_METABOLISM,
            0,
            cabin_o2=CABIN_O2,
            cabin_co2=CABIN_CO2,
            cabin_h2o=CABIN_H2O,
            metabolic_o2_sink=METABOLIC_O2_SINK,
            metabolic_co2_source=METABOLIC_CO2_SOURCE,
            metabolic_h2o_source=METABOLIC_H2O_SOURCE,
        )
    ]
    reg_a = Registry(only_crew, state_a.stocks)
    reg_b = Registry(only_crew, state_b.stocks)
    a, ra, _ = run_eclss(EulerIntegrator(reg_a), state_a, resolver, _DT, _STEPS)
    b, rb, _ = run_eclss(EulerIntegrator(reg_b), state_b, resolver, _DT, _STEPS)
    assert (ra, rb) == (0, 0)
    for sa, sb in zip(a, b, strict=False):
        assert sb.stocks[CABIN_CO2].amount - sa.stocks[
            CABIN_CO2
        ].amount == pytest.approx(1.0, abs=1e-12)


# --- the monotonic diagnostics ------------------------------------------------------
def test_eclss_removal_sinks_are_monotonic(
    steady: tuple[list[State], int, tuple],
) -> None:
    # co2_removed and humidity_condensate only ever receive, so each is
    # non-decreasing every step and strictly grows over the run — the free
    # scrubbed/recovered diagnostics.
    states, _, _ = steady
    for stock in (CO2_REMOVED, HUMIDITY_CONDENSATE):
        amounts = [s.stocks[stock].amount for s in states]
        assert all(a <= b + 1e-15 for a, b in zip(amounts, amounts[1:], strict=False))
        assert amounts[-1] > amounts[0]


# --- determinism / integrator / registration-order independence ---------------------
def test_eclss_is_deterministic(steady: tuple[list[State], int, tuple]) -> None:
    states, rationed, events = steady
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_eclss_rk4_agrees_with_euler_to_tolerance() -> None:
    # The control flows are state-dependent, so — UNLIKE a forced-only system (k1 =
    # k2 = k3 = k4, bit-identical) — RK4 ≢ Euler bit-for-bit. They agree to O(dt²):
    # a real tolerance agreement, the SelfDischarge / Thermal situation.
    euler, _, _ = _run(integrator_cls=EulerIntegrator)
    rk4, _, _ = _run(integrator_cls=Rk4Integrator)
    e_final = euler[-1].stocks[CABIN_CO2].amount
    r_final = rk4[-1].stocks[CABIN_CO2].amount
    assert r_final != e_final  # NOT bit-identical (no forced-only identity)
    assert r_final == pytest.approx(e_final, rel=1e-4)  # but agree to tolerance


def test_eclss_registration_order_independent() -> None:
    # The Registry sorts flows by id, so building with the flows in a different
    # order yields a bit-identical run (#15).
    from domains.eclss.flows import CO2Scrubber, Condenser, CrewMetabolism, O2Makeup

    state, _ = build_eclss(_PARAMS, _SCENARIO)
    reversed_flows = [
        O2Makeup(O2_MAKEUP, 0, o2_supply=O2_SUPPLY, cabin_o2=CABIN_O2, params=_PARAMS),
        Condenser(
            CONDENSER,
            0,
            cabin_h2o=CABIN_H2O,
            humidity_condensate=HUMIDITY_CONDENSATE,
            params=_PARAMS,
        ),
        CO2Scrubber(
            CO2_SCRUBBER,
            0,
            cabin_co2=CABIN_CO2,
            co2_removed=CO2_REMOVED,
            params=_PARAMS,
        ),
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
    ]
    reversed_registry = Registry(reversed_flows, state.stocks)
    resolver = eclss_resolver(_SCENARIO)
    states, rationed, events = run_eclss(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
