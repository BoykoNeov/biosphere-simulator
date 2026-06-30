"""Phase-5 P5.3: the standalone Power run — bounded-SOC validation (energy closure).

Step 3 assembles the Step-2 stocks/flows into a runnable standalone system
(``build_power`` / ``power_resolver`` / ``run_power``) and validates it on the
**bounded-SOC** scenario: a daily-balanced microgrid whose battery charges by day,
discharges by night, and returns to the same SOC each day.

**Honest framing (the advisor's correction).** Both Power flows are *forced*
(state-independent), so the system is a daily-balanced **forced linear accumulator**,
not an emergent limit cycle / attractor — there is no restoring force. Boundedness is
therefore not emergent; it is *constructed* by the exact daily energy balance the
derived load enforces (``load_fraction = 1`` ⇒ ``E_load_day == η_c · E_solar_day``). So
the non-vacuous claims this validates are:

* **ENERGY conserved every step** — the augmented ledger (``solar_source + battery +
  waste_heat``) balances to round-off (the payload of Phase-5 energy closure, P5.1).
* **``rationed == 0``** — the well-fed ``battery0`` keeps min-SOC above a step's draw.
* **day-over-day return** — SOC at the same day-phase is equal to round-off (the real
  "diurnal cycle" claim, true *only* under exact balance — a drifting run would fail).
* a **genuine** SOC swing (min ≪ max, min > 0), **monotonic** heat-generated,
  **determinism**, and **registration-order / integrator independence**.

Pure-stdlib spine; the η_c param is loaded from the committed ``charge.yaml``.
"""

import math

import pytest

from domains.power.flows import LoadDraw, SolarCharge
from domains.power.loader import load_charge_params
from domains.power.scenario import (
    BOUNDED_SOC_DAYS,
    BOUNDED_SOC_SCENARIO,
    PowerScenario,
)
from domains.power.stocks import BATTERY, SOLAR_SOURCE, WASTE_HEAT
from domains.power.system import (
    LOAD_DRAW,
    SOLAR_CHARGE,
    balanced_load_w,
    build_power,
    daily_solar_energy,
    power_resolver,
    run_power,
    solar_schedule,
)
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

_CHARGE = load_charge_params()
_SCENARIO = BOUNDED_SOC_SCENARIO
_STEPS = BOUNDED_SOC_DAYS * _SCENARIO.steps_per_day
_DT = _SCENARIO.dt_seconds


def _run(
    scenario: PowerScenario = _SCENARIO,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
) -> tuple[list[State], int, tuple]:
    state, registry = build_power(_CHARGE, scenario)
    resolver = power_resolver(_CHARGE, scenario)
    steps = BOUNDED_SOC_DAYS * scenario.steps_per_day
    dt = scenario.dt_seconds
    return run_power(integrator_cls(registry), state, resolver, dt, steps)


@pytest.fixture(scope="module")
def bounded() -> tuple[list[State], int, tuple]:
    return _run()


def _soc(states: list[State]) -> list[float]:
    return [s.stocks[BATTERY].amount for s in states]


def _energy_total(state: State) -> float:
    # The augmented-system ENERGY total: the unclamped source (cumulative supply, goes
    # very negative) + the battery POOL + the monotonic waste-heat sink.
    return (
        state.stocks[SOLAR_SOURCE].amount
        + state.stocks[BATTERY].amount
        + state.stocks[WASTE_HEAT].amount
    )


# --- the payload: ENERGY conserved every step over the augmented system -------------
def test_power_energy_conserved_every_step(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # Per-step the ENERGY ledger residual (Δsolar + Δbattery + Δwaste_heat) is ≈ 0 —
    # energy closure (P5.1), now asserted for a real ENERGY carrier (not the inert demo
    # stock). This echoes the gate the integrator runs; pinning it is the receipt.
    states, _, _ = bounded
    for before, after in zip(states, states[1:], strict=False):  # consecutive pairs
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        assert abs(ledger[Quantity.ENERGY].residual) <= 1e-6


def test_power_energy_total_is_invariant(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # The integral form: the total ENERGY across all three stocks never moves from the
    # initial SOC (every flow has Σ legs == 0, so each step's total delta is 0). The
    # "every joule named" structural guarantee, integrated over the whole run.
    states, _, _ = bounded
    total0 = _energy_total(states[0])
    assert total0 == pytest.approx(_SCENARIO.battery0)  # source/sink start at 0
    for s in states:
        assert math.isclose(_energy_total(s), total0, rel_tol=0.0, abs_tol=1e-4)


def test_power_only_energy_is_present(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # Power is a pure-ENERGY domain: the ledger names ENERGY and nothing else (no mass
    # quantity leaks in), so the gate's other-quantity branches are vacuously skipped.
    states, _, _ = bounded
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == {Quantity.ENERGY}


# --- rationed == 0 / events == () : well-fed sizing ---------------------------------
def test_power_never_rations(bounded: tuple[list[State], int, tuple]) -> None:
    # The battery never empties (battery0 sized a few× the within-day drawdown), so the
    # Euler backstop never fires — positivity from sizing, the Phase-1 discipline.
    _, rationed, _ = bounded
    assert rationed == 0


def test_power_no_events(bounded: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock ⇒ extinction can never fire ⇒ no events (and no loss-sink).
    _, _, events = bounded
    assert events == ()


# --- the bounded, day-periodic SOC swing --------------------------------------------
def test_power_soc_swings_and_stays_positive(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # A genuine charge/discharge swing (min materially below max — not a flat line) that
    # never approaches empty (min comfortably above a single step's load draw, which is
    # what keeps rationed == 0 structural).
    states, _, _ = bounded
    soc = _soc(states)
    lo, hi = min(soc), max(soc)
    assert hi - lo > 0.1 * _SCENARIO.battery0  # material swing (~0.72·battery0, probe)
    one_step_draw = balanced_load_w(_CHARGE, _SCENARIO) * _DT
    assert lo > one_step_draw > 0.0  # never empties, with a full step's margin


def test_power_soc_returns_each_day(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # The real "diurnal cycle" claim: at every day boundary (n = k·steps_per_day) the
    # SOC equals the initial SOC to round-off. True ONLY because the day is balanced —
    # a drifting (unbalanced) run would fail this, so it is the non-vacuous test that
    # the derived load actually balances charge against discharge.
    states, _, _ = bounded
    spd = _SCENARIO.steps_per_day
    for day in range(BOUNDED_SOC_DAYS + 1):
        assert math.isclose(
            states[day * spd].stocks[BATTERY].amount,
            _SCENARIO.battery0,
            rel_tol=1e-9,
            abs_tol=1e-6,
        )


def test_power_charges_by_day_discharges_by_night(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # Direction check: over the first day SOC dips to its minimum at the morning
    # crossover (load > stored solar at dawn), peaks in the afternoon — so the in-day
    # minimum is NOT at the day boundary (a real diurnal shape, not a monotone trend).
    states, _, _ = bounded
    spd = _SCENARIO.steps_per_day
    first_day = _soc(states[: spd + 1])
    assert first_day.index(min(first_day)) not in (0, spd)  # interior minimum


# --- the monotonic heat-generated diagnostic ----------------------------------------
def test_power_waste_heat_is_monotonic(
    bounded: tuple[list[State], int, tuple],
) -> None:
    # waste_heat only ever receives (charge loss + the 100%-dissipative load), so it is
    # non-decreasing every step and strictly grows over the run — the free
    # heat-generated / "usefulness is not conserved" accumulator (roadmap line 50).
    states, _, _ = bounded
    heat = [s.stocks[WASTE_HEAT].amount for s in states]
    assert all(b <= a for b, a in zip(heat, heat[1:], strict=False))
    assert heat[-1] > heat[0] > -1.0  # strictly accumulated, from 0


# --- the balance identity (option A's core) -----------------------------------------
def test_balanced_load_matches_daily_stored_solar() -> None:
    # The derived load's daily energy equals the daily STORED solar (η_c · supplied) —
    # the exact-balance condition that makes the SOC bounded. This is the one place a
    # Power resolver reads η_c (Power's load is intrinsically η_c-coupled).
    load_w = balanced_load_w(_CHARGE, _SCENARIO)
    e_load_day = load_w * _SCENARIO.steps_per_day * _SCENARIO.dt_seconds
    e_stored_day = _CHARGE.charge_efficiency * daily_solar_energy(_SCENARIO)
    assert math.isclose(e_load_day, e_stored_day, rel_tol=1e-12)


# --- the diurnal solar shape --------------------------------------------------------
def test_solar_schedule_night_zero_noon_peak() -> None:
    # Half-sine over the 12 h daylight window: 0 at midnight/midnight-adjacent night,
    # peak at solar noon (phase 0.5 ⇒ step steps_per_day/2), periodic across days.
    sched = solar_schedule(_SCENARIO)
    spd = _SCENARIO.steps_per_day
    assert sched(0, _DT) == 0.0  # midnight
    assert math.isclose(sched(spd // 2, _DT), _SCENARIO.solar_peak_w, rel_tol=1e-12)
    assert sched(spd // 2, _DT) == sched(spd // 2 + spd, _DT)  # periodic day-to-day
    # Daylight is the central half: its sunrise edge (phase 0.25) is inclusive but
    # evaluates to 0, and deep night is 0.
    assert sched(spd // 4, _DT) == 0.0  # sunrise edge: in-window but 0
    assert sched(2, _DT) == 0.0  # deep night (phase ≈ 0.083)


# --- determinism / integrator / registration-order independence ---------------------
def test_power_is_deterministic(bounded: tuple[list[State], int, tuple]) -> None:
    # Bit-identical on a re-run (the Step-4 golden's premise).
    states, rationed, events = bounded
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_power_rk4_equals_euler(bounded: tuple[list[State], int, tuple]) -> None:
    # Because the flows are state-independent, every RK4 stage derivative is identical
    # (k1 = k2 = k3 = k4) and the ⅙-combine reproduces k1 exactly — so RK4 ≡ Euler
    # bit-for-bit here. This is that algebraic identity, NOT numerical-robustness
    # evidence; it does double as a guard that the flows stay forced (a future
    # state-dependent flow would break the identity).
    states, _, _ = bounded
    rk4_states, rk4_rationed, rk4_events = _run(integrator_cls=Rk4Integrator)
    assert rk4_states[-1] == states[-1]
    assert (rk4_rationed, rk4_events) == (0, ())


def test_power_registration_order_independent() -> None:
    # The Registry sorts flows by id, so building with the flows in the opposite order
    # yields a bit-identical run (#15). Rebuilds build_power's registry with the two
    # flows reversed and compares the final state.
    state, _ = build_power(_CHARGE, _SCENARIO)
    flows = [
        LoadDraw(LOAD_DRAW, 0, battery=BATTERY, waste_heat=WASTE_HEAT),
        SolarCharge(
            SOLAR_CHARGE,
            0,
            solar_source=SOLAR_SOURCE,
            battery=BATTERY,
            waste_heat=WASTE_HEAT,
            params=_CHARGE,
        ),
    ]
    reversed_registry = Registry(flows, state.stocks)
    resolver = power_resolver(_CHARGE, _SCENARIO)
    states, rationed, events = run_power(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
