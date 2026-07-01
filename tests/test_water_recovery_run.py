"""Phase-6 Step 4 (P6.4): the crew water-recovery loop — WATER closes, store recovers.

Step 4 re-points the Step-2 cabin's two WATER disposal sinks (``humidity_condensate`` /
``urine``) into a ``recovered_water`` buffer POOL feeding a station-owned
``station.flows.WaterRecovery`` flow back to ``crew.water_store``, venting only the
unrecoverable fraction to ``brine``. The crew's finite water store — open-loop and
monotonically depleting standalone/cabin — becomes **regenerative up to the recovery
efficiency** ``η_w``.

**What this validates (the non-vacuous payload).**

* **All three quantities conserved every step** — CARBON / OXYGEN are bit-identical to
  the cabin (Step 4 touches only WATER); WATER now balances across the augmented ledger
  (``cabin_h2o`` / ``recovered_water`` / ``water_store`` / ``brine``). WATER's *total*
  is invariant (``brine`` is the only terminal WATER sink; no WATER source).
* **The store is REGENERATIVE (the "it bit" gate).** With recovery the net drain of
  ``water_store`` is a small fraction of the open-loop (``η_w = 0``) drain — the
  recovered water flows back. Compared against the η_w = 0 baseline (same topology).
* **A conservation identity (advisor, the Step-3 offload analogue).** The buffer's
  dynamics and the forced intake are both independent of η_w, so the water returned to
  the store equals **exactly** ``η_w`` times the water the open-loop baseline sends to
  ``brine``: ``water_store_with − water_store_without ≈ η_w · brine_without``. The
  η-independence of the buffer / ``cabin_h2o`` trajectories (the basis) is checked too.
* **``cabin_h2o`` and ``recovered_water`` reach their emergent steady states** —
  ``cabin_h2o → (f_ins·water)/k_cond``, ``recovered_water → water_intake/k_rec``.
  (``water_store`` has no steady state — a net consumer even here, closed at η_w = 1.)
* **RK4 ≢ Euler on ``water_store`` (the "it earned its keep" signal).** WaterRecovery
  reads the ``recovered_water`` stock, so ``water_store`` becomes state-dependent — the
  forced RK4 ≡ Euler bit-identity the cabin stores had is **broken** (a tolerance
  agreement now), while the forced ``food_store`` stays bit-identical.

Built on the **cabin**, not the greenhouse: the biosphere's internal water ring is
already closed and sealed independently, so station WATER closes without unifying
transpiration with the cabin humidity (a deferred fidelity refinement); and the
biosphere is Euler-locked by its freeze, so only here can the RK4 cross-check run.

Pure-stdlib spine; the crew split fractions, ECLSS control coefficients, and the (first
station-owned) water-recovery params load from the committed YAMLs.
"""

import pytest

from domains.crew.loader import load_crew_params
from domains.crew.stocks import FECAL_WASTE, FOOD_STORE, URINE, WATER_STORE
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import (
    CABIN_CO2,
    CABIN_H2O,
    CABIN_O2,
    CO2_REMOVED,
    HUMIDITY_CONDENSATE,
    O2_SUPPLY,
)
from simcore.conservation import compute_ledger
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.flows import WaterRecoveryParams
from station.loader import load_water_recovery_params
from station.scenario import (
    WATER_RECOVERY_SCENARIO,
    WATER_RECOVERY_STEPS,
    CabinScenario,
)
from station.system import run_station
from station.water import (
    BRINE,
    RECOVERED_WATER,
    WaterRecoverySteadyState,
    build_water_recovery,
    water_recovery_resolver,
    water_recovery_steady_state,
)

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_WR = load_water_recovery_params()
_SCENARIO = WATER_RECOVERY_SCENARIO
_DT = _SCENARIO.dt_seconds
_STEPS = WATER_RECOVERY_STEPS
_SS: WaterRecoverySteadyState = water_recovery_steady_state(
    _CREW, _ECLSS, _WR, _SCENARIO
)

_LEDGER_ABS_TOL = 1e-9
_EQ_BAND = 1e-3
# The identity ws_with − ws_without vs η_w·brine_without: ~20 kg magnitudes over 900
# steps accumulate ~1e-12 round-off (the two runs sum the −intake legs in a different
# interleaving); well above that.
_IDENTITY_ABS_TOL = 1e-9
_MASS_QUANTITIES = {Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER}


def _run(
    *,
    recovery_efficiency: float = _WR.recovery_efficiency,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    steps: int = _STEPS,
    scenario: CabinScenario = _SCENARIO,
) -> tuple[list[State], int, tuple]:
    params = WaterRecoveryParams(
        recovery_rate=_WR.recovery_rate, recovery_efficiency=recovery_efficiency
    )
    state, registry = build_water_recovery(_CREW, _ECLSS, params, scenario)
    resolver = water_recovery_resolver(scenario)
    return run_station(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, steps
    )


def _amt(state: State, sid) -> float:
    return state.stocks[sid].amount


@pytest.fixture(scope="module")
def loop() -> tuple[list[State], int, tuple]:
    return _run()


# --- the payload: three quantities conserved every step ------------------------------
def test_three_quantities_conserved_every_step(
    loop: tuple[list[State], int, tuple],
) -> None:
    states, _, _ = loop
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= _LEDGER_ABS_TOL


def test_only_mass_quantities_present(loop: tuple[list[State], int, tuple]) -> None:
    states, _, _ = loop
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == _MASS_QUANTITIES


def test_total_water_is_invariant(loop: tuple[list[State], int, tuple]) -> None:
    # brine is the only terminal WATER sink and there is no WATER source, so total WATER
    # (cabin_h2o + recovered_water + water_store + brine) is conserved over the run.
    states, _, _ = loop
    water = (CABIN_H2O, RECOVERED_WATER, WATER_STORE, BRINE)
    total0 = sum(_amt(states[0], s) for s in water)
    for state in states:
        assert sum(_amt(state, s) for s in water) == pytest.approx(total0, abs=1e-9)


# --- the "it bit" gate: the store regenerates ----------------------------------------
def test_water_store_is_regenerative(loop: tuple[list[State], int, tuple]) -> None:
    # With recovery the net drain of water_store is a small fraction of the open-loop
    # (η_w = 0) drain — the recovered water flows back. Same topology, only η_w differs.
    with_states = loop[0]
    without_states = _run(recovery_efficiency=0.0)[0]
    ws0 = _SCENARIO.water_store0
    drain_with = ws0 - _amt(with_states[-1], WATER_STORE)
    drain_without = ws0 - _amt(without_states[-1], WATER_STORE)
    assert drain_without > 0.0  # the open-loop store genuinely depletes
    assert 0.0 < drain_with < drain_without  # recovery reduces the drain
    # Materially: net drain < 20 % of the open-loop drain (η_w = 0.9 ⇒ ~10 % steady,
    # a touch more over the buffer-filling transient).
    assert drain_with < 0.2 * drain_without
    # And the store ends materially higher than the open-loop baseline.
    assert _amt(with_states[-1], WATER_STORE) > _amt(without_states[-1], WATER_STORE)


def test_recovery_conservation_identity(loop: tuple[list[State], int, tuple]) -> None:
    # The water returned to the store equals EXACTLY η_w times the water the open-loop
    # baseline sends to brine (the recovered_water dynamics + forced intake are both
    # η-independent). The Step-3 offload-identity analogue on WATER.
    with_final = loop[0][-1]
    without_final = _run(recovery_efficiency=0.0)[0][-1]
    returned = _amt(with_final, WATER_STORE) - _amt(without_final, WATER_STORE)
    brine_without = _amt(without_final, BRINE)
    assert returned == pytest.approx(
        _WR.recovery_efficiency * brine_without, abs=_IDENTITY_ABS_TOL
    )


def test_recovery_dynamics_independent_of_efficiency(
    loop: tuple[list[State], int, tuple],
) -> None:
    # The identity's basis: WaterRecovery's DRAW (k_rec·recovered) and the Condenser
    # + forced urine INFLOW are all η-independent (η only splits the OUTPUT), so the
    # recovered_water and cabin_h2o trajectories are BIT-IDENTICAL between η_w = 0.9 and
    # η_w = 0. brine differs (it carries the (1−η) leg); water_store differs (gets the
    # η leg).
    with_states = loop[0]
    without_states = _run(recovery_efficiency=0.0)[0]
    for a, b in zip(with_states, without_states, strict=False):
        assert _amt(a, RECOVERED_WATER) == _amt(b, RECOVERED_WATER)
        assert _amt(a, CABIN_H2O) == _amt(b, CABIN_H2O)


# --- emergent steady states ----------------------------------------------------------
def test_reaches_water_steady_states(loop: tuple[list[State], int, tuple]) -> None:
    # cabin_h2o → (f_ins·water)/k_cond and recovered_water → water_intake/k_rec.
    final = loop[0][-1]
    assert _amt(final, CABIN_H2O) == pytest.approx(_SS.cabin_h2o, abs=_EQ_BAND)
    assert _amt(final, RECOVERED_WATER) == pytest.approx(
        _SS.recovered_water, abs=_EQ_BAND
    )


# --- structural redirection: orphaned sinks gone, brine present + monotonic -----------
def test_orphaned_disposal_sinks_absent(loop: tuple[list[State], int, tuple]) -> None:
    # The Step-2 cabin's humidity_condensate / urine terminal sinks are re-pointed into
    # recovered_water, so they are ABSENT (the redirection is structural, not a shadow
    # sink); the state is exactly the ten Step-4 stocks incl. recovered_water + brine.
    stocks = set(loop[0][0].stocks)
    assert HUMIDITY_CONDENSATE not in stocks
    assert URINE not in stocks
    assert stocks == {
        CABIN_O2,
        CABIN_CO2,
        CABIN_H2O,
        FOOD_STORE,
        WATER_STORE,
        O2_SUPPLY,
        CO2_REMOVED,
        FECAL_WASTE,
        RECOVERED_WATER,
        BRINE,
    }


def test_brine_monotonic(loop: tuple[list[State], int, tuple]) -> None:
    # brine is the honest remaining WATER boundary (the (1−η_w) loss), a monotonic sink
    # accumulating from 0 (η_w = 0.9 < 1, so it genuinely grows).
    states, _, _ = loop
    brine = [_amt(s, BRINE) for s in states]
    assert all(a <= b for a, b in zip(brine, brine[1:], strict=False))
    assert brine[-1] > brine[0] == 0.0


# --- well-fed, event-free ------------------------------------------------------------
def test_never_rations(loop: tuple[list[State], int, tuple]) -> None:
    # recovered_water positivity is structural (k_rec·dt = 0.06 < 1, donor-controlled);
    # the stores + cabin_o2 are well-fed sizing. The Euler backstop never fires.
    _, rationed, _ = loop
    assert rationed == 0


def test_no_events(loop: tuple[list[State], int, tuple]) -> None:
    _, _, events = loop
    assert events == ()


def test_stores_stay_well_fed(loop: tuple[list[State], int, tuple]) -> None:
    # food_store is a forced open-loop draw (depletes materially — Step 6 closes it);
    # water_store now regenerates (depletes only slightly). Both stay well above 0.
    states, _, _ = loop
    food = [_amt(s, FOOD_STORE) for s in states]
    assert all(b <= a for a, b in zip(food, food[1:], strict=False))
    assert 0.0 < food[-1] < food[0]
    water = [_amt(s, WATER_STORE) for s in states]
    assert 0.0 < water[-1] < water[0]  # regenerated but still a net consumer


# --- integrator split: state-dependent store differs, forced store bit-identical ------
def test_water_store_state_dependent_under_rk4() -> None:
    # WaterRecovery reads recovered_water, so water_store's recovery inflow is
    # state-dependent — the forced RK4 ≡ Euler bit-identity is BROKEN (a tolerance
    # agreement). Checked mid-transient (by 900 steps everything has converged, washing
    # the difference out). food_store is still forced ⇒ bit-identical (earned its keep).
    transient = 20
    euler = _run(integrator_cls=EulerIntegrator, steps=transient)[0][-1]
    rk4 = _run(integrator_cls=Rk4Integrator, steps=transient)[0][-1]
    e = _amt(euler, WATER_STORE)
    k = _amt(rk4, WATER_STORE)
    assert e != k  # broken bit-identity (state-dependent)
    assert e == pytest.approx(k, rel=5e-2)  # but agree to tolerance
    assert _amt(euler, FOOD_STORE) == _amt(rk4, FOOD_STORE)  # forced ⇒ bit-identical


# --- determinism / registration-order independence -----------------------------------
def test_is_deterministic(loop: tuple[list[State], int, tuple]) -> None:
    states, rationed, events = loop
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_registration_order_independent() -> None:
    params = WaterRecoveryParams(
        recovery_rate=_WR.recovery_rate, recovery_efficiency=_WR.recovery_efficiency
    )
    state, registry = build_water_recovery(_CREW, _ECLSS, params, _SCENARIO)
    reversed_registry = Registry(list(reversed(registry.flows)), state.stocks)
    resolver = water_recovery_resolver(_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
