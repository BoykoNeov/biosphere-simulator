"""Phase-6 Step 2 (P6.2): the coupled Crew ↔ ECLSS cabin — gas-loop validation.

Step 2 wires the **real** crew into the ECLSS cabin air at three shared stocks
(``cabin_o2`` / ``cabin_co2`` / ``cabin_h2o``), turning on the phase's one non-trivial
representation decision — **CO₂ as a composition ``{CARBON:1, OXYGEN:2}`` stock** — and
proves that this is what makes **OXYGEN close** across the crew↔cabin loop. The ECLSS
forced ``CrewMetabolism`` stand-in and the crew ``o2_store`` are dropped; the merged
:class:`station.flows.CrewRespiration`
(``food_store + cabin_o2 → cabin_co2 + fecal_waste``) + the crew ``WaterBalance``
(``water_store → cabin_h2o + urine``) feed the cabin, and the three ECLSS control loops
keep each species at a steady state.

**What this validates (the non-vacuous payload).**

* **All three quantities conserved every step over the augmented ledger** — CARBON /
  OXYGEN / WATER each balance to round-off across cabin pools + boundary reservoirs (the
  Phase-5 multi-quantity payload, now cross-domain). This is *trivial* per flow (every
  flow balances internally), so it is **not** the real content — the next two tests are.
* **The decoupled version is REFUSED (the "it bit" gate).** Build the cabin with a
  *pure-carbon* ``cabin_co2`` (drop its OXYGEN composition): the first step raises
  ``ConservationError`` for OXYGEN — respiration draws 2 O per CO₂ out of ``cabin_o2``
  with nowhere for those atoms to land (at the clean-cabin start the scrubber and makeup
  are dormant, so it is the respiration leg that breaks first). OXYGEN closure is *real*
  precisely because the ledger rejects the decoupled model (finding #2; the analogue of
  the N-limited "``f_N`` actually bit" gate — a non-biting run is unpinnable).
* **O₂ is genuinely drawn FROM the cabin (positive content).** ``cabin_o2`` starts at
  the setpoint (regulator idle) and is pulled **below** it by respiration, settling at
  ``o2_eq < o2_setpoint``. That downward draw — not "the ledger balances" — is what
  "OXYGEN closes across the augmented loop" means: the O₂ the crew burns comes out of
  the cabin, and the makeup only tops up the deficit. (Closure in the augmented / atom-
  conservation sense — O₂ still enters from ``o2_supply`` and CO₂ still leaves to
  ``co2_removed``; the recycled cabin cycle is Step 3.)
* **Each species reaches its emergent steady state** —
  ``cabin_o2 → o2_setpoint − P/k_makeup``, ``cabin_co2 → P/k_scrub``,
  ``cabin_h2o → (f_ins·water)/k_cond`` with ``P = f_resp·food`` (RQ = 1). Closed-form
  (``cabin.cabin_steady_state``), emergent from the crew load + params.
* **The stores deplete but stay well-fed (the hybrid, honestly).** ``food_store`` /
  ``water_store`` are forced draws with no resupply, so they run down monotonically
  (open-loop, like standalone Crew — regeneration is Steps 4/6); sizing keeps them
  positive so ``rationed == 0``. Monotonic output sinks. ``events == ()`` (no POPULATION
  stock).
* **RK4 ≢ Euler on the cabin, ≡ on the stores.** The three ECLSS control loops read
  stocks, so the cabin species differ between integrators during the transient (a
  tolerance agreement); ``CrewRespiration`` / ``WaterBalance`` are *forced*, so the
  stores stay bit-identical (the Step-1 battery/node split, now stores/cabin).
  Determinism and registration-order independence round it out.

Pure-stdlib spine; the crew split fractions + ECLSS control coefficients load from the
committed sibling YAMLs.
"""

import pytest

from domains.crew.loader import load_crew_params
from domains.crew.stocks import (
    FECAL_WASTE,
    FOOD_STORE,
    O2_STORE,
    URINE,
    WATER_STORE,
)
from domains.eclss.loader import load_eclss_params
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
from simcore.conservation import compute_ledger
from simcore.flow import ConservationError
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.cabin import (
    CabinSteadyState,
    build_cabin,
    cabin_resolver,
    cabin_steady_state,
)
from station.scenario import CABIN_GAS_SCENARIO, CABIN_GAS_STEPS, CabinScenario
from station.system import run_station

_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_SCENARIO = CABIN_GAS_SCENARIO
_DT = _SCENARIO.dt_seconds
_STEPS = CABIN_GAS_STEPS
_SS: CabinSteadyState = cabin_steady_state(_CREW, _ECLSS, _SCENARIO)

_LEDGER_ABS_TOL = 1e-9
# Each species reaches its steady state to within this over the 900-step (≈27 τ) run.
_EQ_BAND = 1e-3
# The three mass quantities the coupled cabin tracks (no ENERGY/NITROGEN stock).
_MASS_QUANTITIES = {Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER}


def _run(
    scenario: CabinScenario = _SCENARIO,
    integrator_cls: type[EulerIntegrator] | type[Rk4Integrator] = EulerIntegrator,
    steps: int = _STEPS,
) -> tuple[list[State], int, tuple]:
    state, registry = build_cabin(_CREW, _ECLSS, scenario)
    resolver = cabin_resolver(scenario)
    return run_station(
        integrator_cls(registry), state, resolver, scenario.dt_seconds, steps
    )


@pytest.fixture(scope="module")
def cabin() -> tuple[list[State], int, tuple]:
    return _run()


# --- the payload: three quantities conserved every step over the augmented ledger ----
def test_cabin_three_quantities_conserved_every_step(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # Per-step CARBON / OXYGEN / WATER residuals ≈ 0 across cabin pools + boundary
    # reservoirs — the augmented-ledger payload (three quantities at once,
    # cross-domain). Trivial per flow; the non-vacuous content is the next two tests
    # (composition is load-bearing, O₂ drawn from cabin).
    states, _, _ = cabin
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= _LEDGER_ABS_TOL


def test_cabin_only_mass_quantities_present(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # The coupled cabin tracks exactly CARBON / OXYGEN / WATER (no ENERGY — the heat
    # station is a separate assembly; no NITROGEN stock).
    states, _, _ = cabin
    quantities = {ql.quantity for ql in compute_ledger(states[0], states[1])}
    assert quantities == _MASS_QUANTITIES


# --- the "it bit" gate: the decoupled (pure-carbon CO₂) version is refused ------------
def test_cabin_decoupled_co2_breaks_oxygen_closure() -> None:
    # Build the cabin with a PURE-CARBON cabin_co2 (drop its {O:2}) — the
    # standalone-crew decoupled model. The first step raises ConservationError for
    # OXYGEN: respiration pulls 2 O per CO₂ out of cabin_o2 with no matching deposit
    # (residual = 2·respired). At the clean-cabin start the scrubber (cabin_co2 = 0) and
    # makeup (cabin_o2 = setpoint) are both dormant, so the respiration leg is what
    # breaks first — the scrubber's own {C:1,O:2} requirement is enforced later in a
    # full run. The ledger REFUSING this is what makes OXYGEN closure real (finding #2).
    # The coupled build (default composition) does not raise — asserted implicitly by
    # every other test.
    state, registry = build_cabin(
        _CREW, _ECLSS, _SCENARIO, cabin_co2_composition={Quantity.CARBON: 1.0}
    )
    resolver = cabin_resolver(_SCENARIO)
    with pytest.raises(ConservationError, match="OXYGEN"):
        run_station(EulerIntegrator(registry), state, resolver, _DT, 1)


# --- positive content: O₂ is genuinely drawn from the cabin --------------------------
def test_cabin_o2_drawn_down_from_setpoint(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # The crew breathes CABIN O₂ (not a store): cabin_o2 starts at the setpoint (makeup
    # idle) and respiration pulls it BELOW, settling at o2_eq < o2_setpoint. That
    # downward draw is "OXYGEN closes across the augmented loop" — the O₂ burned comes
    # out of the cabin; makeup only tops up the deficit. Monotone (constant crew load ⇒
    # monotone relaxation).
    states, _, _ = cabin
    o2 = [s.stocks[CABIN_O2].amount for s in states]
    assert o2[0] == pytest.approx(_ECLSS.o2_setpoint)  # starts idle at the setpoint
    assert all(b <= a for a, b in zip(o2, o2[1:], strict=False))  # drawn down monotone
    assert o2[-1] < _ECLSS.o2_setpoint  # genuinely below the setpoint
    assert _SS.cabin_o2 < _ECLSS.o2_setpoint  # the eq itself is below (a real deficit)
    assert o2[-1] == pytest.approx(_SS.cabin_o2, abs=_EQ_BAND)


def test_cabin_breathes_cabin_not_a_store(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # The inward move: standalone crew's o2_store + the ECLSS metabolic_* seam
    # reservoirs are ALL absent — the crew draws O₂ from cabin_o2 and exhales into
    # cabin_co2 directly, no decoupled stand-ins. The state is exactly the ten coupled
    # stocks.
    stocks = set(cabin[0][0].stocks)
    for absent in (
        O2_STORE,
        METABOLIC_O2_SINK,
        METABOLIC_CO2_SOURCE,
        METABOLIC_H2O_SOURCE,
    ):
        assert absent not in stocks
    assert stocks == {
        CABIN_O2,
        CABIN_CO2,
        CABIN_H2O,
        FOOD_STORE,
        WATER_STORE,
        O2_SUPPLY,
        CO2_REMOVED,
        HUMIDITY_CONDENSATE,
        FECAL_WASTE,
        URINE,
    }


# --- each species reaches its emergent steady state ----------------------------------
def test_cabin_reaches_steady_states(cabin: tuple[list[State], int, tuple]) -> None:
    # Over ≈27 τ each species converges to its closed-form steady state: cabin_co2 rises
    # from 0 to P/k_scrub, cabin_h2o from 0 to (f_ins·water)/k_cond, cabin_o2 falls from
    # the setpoint to o2_setpoint − P/k_makeup (P = f_resp·food, RQ = 1).
    final = cabin[0][-1]
    assert final.stocks[CABIN_O2].amount == pytest.approx(_SS.cabin_o2, abs=_EQ_BAND)
    assert final.stocks[CABIN_CO2].amount == pytest.approx(_SS.cabin_co2, abs=_EQ_BAND)
    assert final.stocks[CABIN_H2O].amount == pytest.approx(_SS.cabin_h2o, abs=_EQ_BAND)


# --- well-fed, event-free, monotone sinks --------------------------------------------
def test_cabin_never_rations(cabin: tuple[list[State], int, tuple]) -> None:
    # CO₂/H₂O positivity is structural (k·dt < 1, donor-controlled); cabin_o2 and the
    # two crew stores are well-fed sizing (the draws are a small fraction of each pool).
    # The Euler backstop never fires.
    _, rationed, _ = cabin
    assert rationed == 0


def test_cabin_no_events(cabin: tuple[list[State], int, tuple]) -> None:
    # No POPULATION stock anywhere ⇒ extinction can never fire.
    _, _, events = cabin
    assert events == ()


def test_cabin_stores_deplete_but_stay_well_fed(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # The stores are forced draws with no resupply (open-loop, like standalone Crew):
    # they run down monotonically to a material but safe drawdown (≈22 % food, ≈14 %
    # water) — the incompleteness Steps 4/6 close by regenerating them.
    states, _, _ = cabin
    for store in (FOOD_STORE, WATER_STORE):
        amounts = [s.stocks[store].amount for s in states]
        assert all(b <= a for a, b in zip(amounts, amounts[1:], strict=False))
        assert amounts[-1] < amounts[0]  # material drawdown
        assert amounts[-1] > 0.0  # stayed well-fed (rationed == 0)


def test_cabin_output_sinks_monotonic(
    cabin: tuple[list[State], int, tuple],
) -> None:
    # The four disposal sinks (scrubbed CO₂, condensed humidity, feces, urine) only ever
    # receive ⇒ monotonic-increasing free cumulative-output diagnostics, each strictly
    # accumulating from 0.
    states, _, _ = cabin
    for sink in (CO2_REMOVED, HUMIDITY_CONDENSATE, FECAL_WASTE, URINE):
        amounts = [s.stocks[sink].amount for s in states]
        assert all(a <= b for a, b in zip(amounts, amounts[1:], strict=False))
        assert amounts[-1] > amounts[0] == 0.0


# --- integrator split: forced stores bit-identical, state-dependent cabin differs -----
def test_cabin_forced_stores_bit_identical_under_rk4() -> None:
    # CrewRespiration / WaterBalance are forced (read intake rates, never a stock), so
    # the two crew stores are BIT-IDENTICAL under Euler vs RK4 the whole run — the
    # coupling did not make the crew draws state-dependent (the Step-1 forced-battery
    # analogue).
    euler = _run(integrator_cls=EulerIntegrator)[0]
    rk4 = _run(integrator_cls=Rk4Integrator)[0]
    for e, k in zip(euler, rk4, strict=False):
        assert e.stocks[FOOD_STORE].amount == k.stocks[FOOD_STORE].amount
        assert e.stocks[WATER_STORE].amount == k.stocks[WATER_STORE].amount


def test_cabin_state_dependent_species_differ_under_rk4() -> None:
    # The three ECLSS control loops read stocks, so the cabin species differ between
    # Euler and RK4 during the transient (a tolerance agreement to O(dt²)) — checked
    # mid-relaxation (by 900 steps everything has converged to the same fixed point,
    # washing the difference out). The nonzero difference is the "RK4 ≢ Euler"
    # statement.
    transient = 20
    euler = _run(integrator_cls=EulerIntegrator, steps=transient)[0][-1]
    rk4 = _run(integrator_cls=Rk4Integrator, steps=transient)[0][-1]
    for stock in (CABIN_CO2, CABIN_O2, CABIN_H2O):
        e = euler.stocks[stock].amount
        k = rk4.stocks[stock].amount
        assert e != k  # not bit-identical (state-dependent)
        assert e == pytest.approx(k, rel=5e-2)  # but agree to tolerance


# --- determinism / registration-order independence -----------------------------------
def test_cabin_is_deterministic(cabin: tuple[list[State], int, tuple]) -> None:
    states, rationed, events = cabin
    states2, rationed2, events2 = _run()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


def test_cabin_registration_order_independent() -> None:
    # The Registry sorts flows by id, so rebuilding with the flows reversed yields a
    # bit-identical run (#15).
    state, registry = build_cabin(_CREW, _ECLSS, _SCENARIO)
    reversed_registry = Registry(list(reversed(registry.flows)), state.stocks)
    resolver = cabin_resolver(_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(reversed_registry), state, resolver, _DT, _STEPS
    )
    baseline, base_rationed, base_events = _run()
    assert states[-1] == baseline[-1]
    assert (rationed, events) == (base_rationed, base_events)
