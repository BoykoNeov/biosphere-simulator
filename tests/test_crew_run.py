"""Phase-5 Step 7: the standalone Crew run — the net-consumer mission validation.

Step 7 assembles the Crew stocks/flows into a runnable standalone system (``build_crew``
/ ``crew_resolver`` / ``run_crew``) and validates it on the **mission** scenario: a
provisioned crew under a constant forced load, every store depleting monotonically but
staying well-fed over the horizon.

**Honest framing (the contrast with the siblings).** Crew is the first **net-consumer /
open-loop** domain — unlike ECLSS/Thermal it has **no restoring force and no
attractor**; the stores just run down (``store(n) = store0 − n·rate·dt``). It is like
Power's *forced* two-flow ``BOUNDED_SOC`` — but *monotone depletion*, not a *constructed
balance* (there is no resupply standalone). Because no flow reads a stock, the
forced-only **RK4 ≡ Euler bit-identity** (which ECLSS/Thermal broke) *returns*. The
non-vacuous claims this validates:

* **all three quantities conserved every step** — the augmented ledger (stores +
  boundary sinks) balances CARBON, OXYGEN and WATER to round-off (multi-quantity, as
  ECLSS — the payload).
* **``rationed == 0``** — by **well-fed sizing** (every store's endurance
  ``store0/rate`` exceeds the mission, so no forced draw over-draws a store). The
  mission is a **material** drawdown (each store falls to ≈ 70 %) that stays positive.
* **monotone depletion + closed-form endurance** — each store falls monotonically; the
  ``depletion_times`` closed form matches the linear draw.
* **monotonic** output sinks (exhaled CO₂ / feces / humidity / urine / consumed O₂ —
  cumulative-output diagnostics), **determinism**, **RK4 ≡ Euler bit-identical**
  (forced-only), and **registration-order independence**.

Pure-stdlib spine; the Crew params load from the committed ``crew.yaml``.
"""

import pytest

from domains.crew.loader import load_crew_params
from domains.crew.scenario import MISSION_DAYS, MISSION_SCENARIO, CrewScenario
from domains.crew.stocks import (
    CREW_HUMIDITY,
    CREW_O2_CONSUMED,
    EXHALED_CO2,
    FECAL_WASTE,
    FOOD_STORE,
    O2_STORE,
    URINE,
    WATER_STORE,
)
from domains.crew.system import (
    FOOD_METABOLISM,
    OXYGEN_CONSUMPTION,
    WATER_BALANCE,
    build_crew,
    crew_resolver,
    depletion_times,
    run_crew,
)
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

_PARAMS = load_crew_params()
_SCENARIO = MISSION_SCENARIO
_STEPS = MISSION_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds


def _run(
    scenario: CrewScenario = _SCENARIO,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
) -> tuple[list[State], int, tuple]:
    state, registry = build_crew(_PARAMS, scenario)
    resolver = crew_resolver(scenario)
    return run_crew(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, _STEPS
    )


@pytest.fixture(scope="module")
def mission() -> tuple[list[State], int, tuple]:
    return _run()


# --- the payload: all three quantities conserved every step -------------------------
def test_crew_three_quantities_conserved_every_step(
    mission: tuple[list[State], int, tuple],
) -> None:
    # Per step the CARBON / OXYGEN / WATER ledger residuals are all ≈ 0 — the augmented
    # store+sink ledger balances (multi-quantity, three at once — the payload).
    states, _, _ = mission
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for q in (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER):
            assert abs(ledger[q].residual) <= 1e-6


def test_crew_only_the_three_mass_quantities_present(
    mission: tuple[list[State], int, tuple],
) -> None:
    # Crew carries CARBON / OXYGEN / WATER and nothing else (no ENERGY, no NITROGEN
    # stock ⇒ the gate skips them).
    states, _, _ = mission
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == {Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER}


def test_crew_augmented_totals_are_invariant(
    mission: tuple[list[State], int, tuple],
) -> None:
    # Integral form: each quantity's total across its augmented stocks never moves from
    # the initial provisioned inventory (every flow has Σ legs == 0 per quantity). The
    # stores hold the inventories, so no negative-going boundary source is needed —
    # carbon total == food0, water total == water0, oxygen total == o2_0.
    states, _, _ = mission

    def carbon_total(s: State) -> float:
        return (
            s.stocks[FOOD_STORE].amount
            + s.stocks[EXHALED_CO2].amount
            + s.stocks[FECAL_WASTE].amount
        )

    def water_total(s: State) -> float:
        return (
            s.stocks[WATER_STORE].amount
            + s.stocks[CREW_HUMIDITY].amount
            + s.stocks[URINE].amount
        )

    def oxygen_total(s: State) -> float:
        return s.stocks[O2_STORE].amount + s.stocks[CREW_O2_CONSUMED].amount

    for s in states:
        assert carbon_total(s) == pytest.approx(_SCENARIO.food_store0, abs=1e-9)
        assert water_total(s) == pytest.approx(_SCENARIO.water_store0, abs=1e-9)
        assert oxygen_total(s) == pytest.approx(_SCENARIO.o2_store0, abs=1e-9)


# --- rationed == 0 / events == () ----------------------------------------------------
def test_crew_never_rations(mission: tuple[list[State], int, tuple]) -> None:
    # Well-fed sizing: every store's endurance exceeds the mission, so no forced draw
    # over-draws a store and the backstop never fires.
    _, rationed, _ = mission
    assert rationed == 0


def test_crew_stores_stay_positive_and_well_fed(
    mission: tuple[list[State], int, tuple],
) -> None:
    # The well-fed claim with teeth: every store ends comfortably positive (a material
    # drawdown, not near-empty). The mission (7 days) is ~30 % of each store's ~23-day
    # endurance, so each ends near 70 % of its initial inventory.
    states, _, _ = mission
    final = states[-1]
    for stock, initial in (
        (FOOD_STORE, _SCENARIO.food_store0),
        (WATER_STORE, _SCENARIO.water_store0),
        (O2_STORE, _SCENARIO.o2_store0),
    ):
        end = final.stocks[stock].amount
        assert end > 0.0
        assert 0.6 * initial < end < 0.8 * initial  # material, not near-empty


def test_crew_no_events(mission: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock (crew count is fixed scenario data, not a stock) ⇒ extinction
    # can never fire ⇒ no events (and no loss-sink).
    _, _, events = mission
    assert events == ()


# --- monotone depletion + closed-form endurance -------------------------------------
def test_crew_stores_deplete_monotonically(
    mission: tuple[list[State], int, tuple],
) -> None:
    # Each store falls monotonically (constant forced draw, no resupply ⇒ monotone
    # depletion — no restoring force, no attractor).
    states, _, _ = mission
    for stock in (FOOD_STORE, WATER_STORE, O2_STORE):
        amounts = [s.stocks[stock].amount for s in states]
        assert all(a >= b - 1e-15 for a, b in zip(amounts, amounts[1:], strict=False))
        assert amounts[-1] < amounts[0]  # genuinely ran down


def test_crew_depletion_matches_closed_form(
    mission: tuple[list[State], int, tuple],
) -> None:
    # The linear draw matches the closed-form endurance: after n steps a store holds
    # store0·(1 − n·dt / (store0/rate)). Check the final state against store0 − Σ draws.
    states, _, _ = mission
    final = states[-1]
    dep = depletion_times(_SCENARIO)
    horizon = _STEPS * _DT
    for stock, initial, endurance in (
        (FOOD_STORE, _SCENARIO.food_store0, dep.food_store),
        (WATER_STORE, _SCENARIO.water_store0, dep.water_store),
        (O2_STORE, _SCENARIO.o2_store0, dep.o2_store),
    ):
        expected = initial * (1.0 - horizon / endurance)
        assert final.stocks[stock].amount == pytest.approx(expected, rel=1e-9)


# --- the monotonic output diagnostics -----------------------------------------------
def test_crew_output_sinks_are_monotonic(
    mission: tuple[list[State], int, tuple],
) -> None:
    # Every output sink only ever receives, so each is non-decreasing every step and
    # strictly grows over the mission — the free cumulative-output diagnostics.
    states, _, _ = mission
    for stock in (EXHALED_CO2, FECAL_WASTE, CREW_HUMIDITY, URINE, CREW_O2_CONSUMED):
        amounts = [s.stocks[stock].amount for s in states]
        assert all(a <= b + 1e-15 for a, b in zip(amounts, amounts[1:], strict=False))
        assert amounts[-1] > amounts[0]


# --- determinism / integrator / registration-order independence ---------------------
def test_crew_is_deterministic(mission: tuple[list[State], int, tuple]) -> None:
    states, rationed, events = mission
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_crew_rk4_equals_euler_bit_for_bit() -> None:
    # Because every flow is FORCED (state-independent), every RK4 stage derivative is
    # identical (k1 = k2 = k3 = k4) and the ⅙-combine reproduces k1 exactly — so RK4 ≡
    # Euler BIT-FOR-BIT (the forced-only identity ECLSS/Thermal broke, revived here —
    # the symmetric bookend). NOT a tolerance agreement.
    euler, _, _ = _run(integrator_cls=EulerIntegrator)
    rk4, rk4_rationed, rk4_events = _run(integrator_cls=Rk4Integrator)
    assert rk4[-1] == euler[-1]
    assert (rk4_rationed, rk4_events) == (0, ())


def test_crew_registration_order_independent() -> None:
    # The Registry sorts flows by id, so building with the flows in a different order
    # yields a bit-identical run (#15).
    from domains.crew.flows import FoodMetabolism, OxygenConsumption, WaterBalance

    state, _ = build_crew(_PARAMS, _SCENARIO)
    reversed_flows = [
        WaterBalance(
            WATER_BALANCE,
            0,
            water_store=WATER_STORE,
            crew_humidity=CREW_HUMIDITY,
            urine=URINE,
            params=_PARAMS,
        ),
        OxygenConsumption(
            OXYGEN_CONSUMPTION, 0, o2_store=O2_STORE, o2_consumed=CREW_O2_CONSUMED
        ),
        FoodMetabolism(
            FOOD_METABOLISM,
            0,
            food_store=FOOD_STORE,
            exhaled_co2=EXHALED_CO2,
            fecal_waste=FECAL_WASTE,
            params=_PARAMS,
        ),
    ]
    reversed_registry = Registry(reversed_flows, state.stocks)
    resolver = crew_resolver(_SCENARIO)
    states, rationed, events = run_crew(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
