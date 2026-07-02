"""Step-8 (P6.8): the cross-domain perturbation harness — cascades, no cascade code.

The station analogue of ``tests/test_perturbations.py`` (Phase-3): each perturbation
(``station.perturbations``) is composed **onto** the already-assembled station inputs,
and each is a **cascade with no cascade code** — a disturbance in one domain propagating
into another through a shared stock / shared forcing (#16), while conservation holds and
``rationed`` *behaves*. ``git diff src/{simcore,domains}/`` stays empty; all twenty
existing goldens stay byte-identical; **no new golden** (a behavioural demonstration —
determinism re-runs are the no-golden insurance).

**Two substrates (the Phase-3 asymmetric assignment, now by physics + compute).**
*Energy* perturbations (brownout, radiator failure) run on the cheap single-rate diurnal
``run_station`` (Power → Thermal); *matter* perturbations (leak, crew spike, lighting)
run on a **short** two-rate ``run_sealed`` (``DAYS = 8``, window ``[2, 7)`` — inside
year 1, so ``annual_reset`` never fires and cannot be starved: the short horizon is the
*fix* for the Phase-3 seed-bank landmine, not just a compute win).

**The signatures are the EMERGENT ones (advisor, spike-CONFIRMED).** The station
regulators *erase* the naive pool level — every matter perturbation returns
``CARBON_POOL`` / ``O2_POOL`` to setpoint at the day boundary (the Step-3
regulator-erasure physics), so the tests assert regulator **effort** (``co2_removed`` /
``o2_supply``) + the sinks (``LEAK_SINK``, biomass), never the erased pool. And the two
gas pools **do not fail the same way**: ``CARBON_POOL`` is only *removed* (a leak lowers
Ci ⇒ biomass↓ + scrubber↓), ``O2_POOL`` is *defended* (a leak ⇒ ``O2Makeup`` effort↑,
``cabin_o2`` flat). **Direction-only** asserts (the anti-flakiness rule — per-stock,
never a magnitude / day-index / ``State == State``), each a perturbed-vs-baseline
contrast. Conservation: a completed run *is* the per-sub-step proof (the driver's gate
folds every balanced leg incl. ``LEAK_SINK``); the relative day-boundary drift
(``sealed_tier2_helper``, summing incl. the sink) is the extra teeth.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.perturbations import LEAK_SINK
from domains.biosphere.stocks import (
    CARBON_POOL,
    LEAF_C,
    O2_POOL,
    ROOT_C,
    STEM_C,
    STORAGE_C,
)
from domains.crew.loader import load_crew_params
from domains.crew.stocks import FOOD_STORE
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import CO2_REMOVED, O2_SUPPLY
from domains.power.loader import load_charge_params
from domains.power.stocks import BATTERY
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import NODE, SPACE
from sealed_tier2_helper import REL_DRIFT_BOUND, REL_SLOPE_BOUND, relative_drift
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.driver import run_master_day
from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)
from station.perturbations import (
    ScaledFlow,
    with_brownout,
    with_crew_load_spike,
    with_lighting_failure,
    with_radiator_failure,
    with_station_leak,
)
from station.scenario import HEAT_CLOSURE_SCENARIO, SealedStationScenario
from station.sealed import (
    build_sealed_station,
    sealed_bio_resolver,
    sealed_fast_resolver,
)
from station.system import build_station, run_station, station_resolver

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# ============================ ENERGY: run_station ====================================

_E_SCN = HEAT_CLOSURE_SCENARIO
_SPD = _E_SCN.power.steps_per_day
_E_DT = _E_SCN.power.dt_seconds
_E_DAYS = 12


def _energy_run(resolver, registry, state) -> tuple[list[State], int, tuple]:
    return run_station(
        EulerIntegrator(registry), state, resolver, _E_DT, _E_DAYS * _SPD
    )


def _node_temp(state: State, thermal) -> float:
    return thermal.space_temperature + state.stocks[NODE].amount / thermal.heat_capacity


@pytest.fixture(scope="module")
def energy_baseline():
    """Baseline Power → Thermal diurnal run + the loaded params + the base resolver."""
    charge = load_charge_params()
    thermal = load_thermal_params()
    state, reg = build_station(charge, thermal, _E_SCN)
    resolver = station_resolver(charge, _E_SCN)
    states, rationed, events = _energy_run(resolver, reg, state)
    assert rationed == 0 and events == ()
    return states, charge, thermal, resolver


# --- brownout: solar cut → SOC↓ + node cools; deep → rationing emerges ----------------
def test_brownout_graceful_cools_node_without_rationing(energy_baseline) -> None:
    # A short/shallow afternoon dimming: SOC dips below baseline and the thermal node
    # cools (less dissipation reaches it) — the Power→Thermal cascade — WITHOUT
    # rationing.
    base_states, charge, thermal, resolver = energy_baseline
    state, reg = build_station(charge, thermal, _E_SCN)
    perturbed = with_brownout(resolver, start=85, end=90, factor=0.5)
    states, rationed, events = _energy_run(perturbed, reg, state)
    assert rationed == 0 and events == ()
    base_soc = [s.stocks[BATTERY].amount for s in base_states]
    soc = [s.stocks[BATTERY].amount for s in states]
    assert min(soc) < min(base_soc) and min(soc) > 0.0  # dips but never empties
    base_t = [_node_temp(s, thermal) for s in base_states]
    t = [_node_temp(s, thermal) for s in states]
    assert min(t) < min(base_t)  # node cools — the cross-domain cascade
    rel_abs, _ = relative_drift(states, Quantity.ENERGY)
    assert rel_abs < REL_DRIFT_BOUND  # ENERGY conserved through the perturbation


def test_brownout_deep_emerges_rationing_still_conserving(energy_baseline) -> None:
    # The FAILURE cascade (the exit criterion): a multi-day full blackout empties the
    # battery so LoadDraw cannot be met → rationed > 0 EMERGES (bounded — one firing per
    # unmet step), yet ENERGY still conserves (the Euler backstop conserves as it
    # rations).
    _, charge, thermal, resolver = energy_baseline
    state, reg = build_station(charge, thermal, _E_SCN)
    perturbed = with_brownout(resolver, start=2 * _SPD, end=8 * _SPD, factor=0.0)
    states, rationed, events = _energy_run(perturbed, reg, state)
    assert rationed > 0  # the emergent failure signature
    assert rationed < _E_DAYS * _SPD  # bounded (not every step rations)
    assert events == ()
    rel_abs, _ = relative_drift(states, Quantity.ENERGY)
    assert rel_abs < REL_DRIFT_BOUND


# --- radiator failure: throttle rejection → node heats, still conserving --------------
def test_radiator_failure_heats_node_conserving_no_rationing(energy_baseline) -> None:
    # Throttling RadiatorReject to 0 over a window lets Power's real dissipation pile up
    # in the node → it HEATS (peak T above baseline) — the opposite sign to brownout,
    # the thermal-runaway cascade. Energy stays in-system (conserved); the sink is still
    # monotonic; rationed == 0 (a POOL accumulation, not a withdrawal shortfall).
    base_states, charge, thermal, resolver = energy_baseline
    state, reg = build_station(charge, thermal, _E_SCN)
    perturbed_reg, perturbed_res = with_radiator_failure(
        state, reg, resolver, start=3 * _SPD, end=9 * _SPD, health=0.0
    )
    states, rationed, events = _energy_run(perturbed_res, perturbed_reg, state)
    assert rationed == 0 and events == ()
    base_t = [_node_temp(s, thermal) for s in base_states]
    t = [_node_temp(s, thermal) for s in states]
    assert max(t) > max(base_t)  # node overheats — the cross-domain cascade
    space = [s.stocks[SPACE].amount for s in states]
    assert all(space[i + 1] >= space[i] - 1e-6 for i in range(len(space) - 1))
    rel_abs, rel_slope = relative_drift(states, Quantity.ENERGY)
    assert rel_abs < REL_DRIFT_BOUND and rel_slope < REL_SLOPE_BOUND


def test_radiator_failure_outside_window_is_baseline(energy_baseline) -> None:
    # The ScaledFlow bit-identity: health == 1 outside the window reproduces the wrapped
    # flow EXACTLY (x·1.0 == x), so a failure window the run never reaches gives the
    # byte-identical baseline trajectory — the perturbation is confined to the window.
    base_states, charge, thermal, resolver = energy_baseline
    state, reg = build_station(charge, thermal, _E_SCN)
    perturbed_reg, perturbed_res = with_radiator_failure(
        state, reg, resolver, start=100 * _SPD, end=101 * _SPD, health=0.0
    )
    states, _, _ = _energy_run(perturbed_res, perturbed_reg, state)
    assert states[-1] == base_states[-1]


def test_radiator_failure_is_deterministic(energy_baseline) -> None:
    # The no-golden insurance (energy side): a perturbed run is bit-identical on re-run.
    _, charge, thermal, resolver = energy_baseline
    out = []
    for _ in range(2):
        state, reg = build_station(charge, thermal, _E_SCN)
        preg, pres = with_radiator_failure(
            state, reg, resolver, start=3 * _SPD, end=9 * _SPD, health=0.0
        )
        states, _, _ = _energy_run(pres, preg, state)
        out.append(states[-1])
    assert out[0] == out[1]


def test_scaled_flow_scales_whole_flow_balanced() -> None:
    # The new seam-type's invariant, unit-level: ScaledFlow multiplies EVERY leg by the
    # same alpha, so an internally-balanced flow stays balanced (Σ α·leg = α·Σ leg), and
    # each leg is exactly alpha× the wrapped leg (the "arbitration scales the whole
    # flow" rule as a perturbation). Checked on the real RadiatorReject at a fixed
    # alpha, with a real bound env carrying the health forcing.
    from domains.biosphere.perturbations import with_forcing
    from domains.thermal.system import RADIATOR_REJECT
    from simcore.environment import constant
    from station.perturbations import RADIATOR_HEALTH_VAR

    charge = load_charge_params()
    thermal = load_thermal_params()
    state, reg = build_station(charge, thermal, _E_SCN)
    inner = next(f for f in reg.flows if f.id == RADIATOR_REJECT)
    scaled = ScaledFlow(inner, RADIATOR_HEALTH_VAR)
    assert scaled.id == inner.id and scaled.priority == inner.priority
    resolver = with_forcing(
        station_resolver(charge, _E_SCN), RADIATOR_HEALTH_VAR, constant(0.25)
    )
    env = resolver.bind(state, _E_DT)
    inner_legs = {
        leg.stock: leg.amount for leg in inner.evaluate(state, env, _E_DT).legs
    }
    scaled_res = scaled.evaluate(state, env, _E_DT)
    for leg in scaled_res.legs:
        assert leg.amount == pytest.approx(0.25 * inner_legs[leg.stock])
    # each conserved quantity still balances over the scaled legs
    total: dict[Quantity, float] = {}
    for leg in scaled_res.legs:
        comp = state.stocks[leg.stock].composition
        for q, n in comp.items():
            total[q] = total.get(q, 0.0) + leg.amount * n
    for q, s in total.items():
        assert abs(s) < 1e-9, (q, s)


# ============================ MATTER: short run_sealed ================================

# season_days >> the run horizon ⇒ annual_reset never fires (window stays in year 1)
_M_SCN = SealedStationScenario(years=1, season_days=305)
_M_DAYS = 8
_START, _END = 2, 7  # window (master days) — inside year 1
_K_LEAK = 1.0e-3  # k·dt = 0.06 at 60 s (the k_scrub scale; k·dt < 1 ⇒ rationed == 0)
# Matter conserved quantities that the leak / crew / lighting cascades touch (ENERGY is
# the disjoint energy loop — separately checked; it is inert to matter perturbations).
_MATTER_Q = (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER, Quantity.NITROGEN)


def _sealed_params():
    return (
        load_charge_params(),
        load_thermal_params(),
        load_crew_params(),
        load_eclss_params(),
        load_water_recovery_params(),
        load_lamp_params(),
        load_harvest_params(),
    )


def _sealed_build():
    params = _sealed_params()
    charge, _, _, _, _, lamp, _ = params
    state, bio_reg, fast_reg = build_sealed_station(*params, _M_SCN)
    bio_res = sealed_bio_resolver(_weather(), lamp, _M_SCN)
    fast_res = sealed_fast_resolver(charge, _M_SCN)
    return state, bio_reg, fast_reg, bio_res, fast_res


def _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res):
    return run_master_day(
        EulerIntegrator(bio_reg),
        EulerIntegrator(fast_reg),
        state,
        bio_res,
        fast_res,
        days=_M_DAYS,
        steps_per_day=_M_SCN.steps_per_day,
        slow_dt=_M_SCN.bio_dt,
        fast_dt=_M_SCN.cabin_dt,
    )


def _biomass(state: State) -> float:
    return sum(state.stocks[s].amount for s in (LEAF_C, STEM_C, ROOT_C, STORAGE_C))


def _assert_matter_conserved(states: list[State]) -> None:
    # Relative drift (incl. any LEAK_SINK — quantity_scale/mass_drift_trace fold all
    # stocks) at the round-off floor: total conserved even when the chamber interior's
    # closure breaks (the leak vents to the boundary sink). Do NOT assert closure /
    # loss_sink == 0.
    for quantity in _MATTER_Q:
        rel_abs, rel_slope = relative_drift(states, quantity)
        assert rel_abs < REL_DRIFT_BOUND, (quantity, rel_abs)
        assert rel_slope < REL_SLOPE_BOUND, (quantity, rel_slope)


@pytest.fixture(scope="module")
def sealed_baseline():
    """The unperturbed short sealed run — the contrast each matter cascade uses."""
    states, rationed, events = _sealed_run(*_sealed_build())
    assert rationed == 0 and events == ()
    return states


# --- atmosphere leak on CARBON_POOL: Ci↓ ⇒ biomass↓ + scrubber does less --------------
@pytest.fixture(scope="module")
def carbon_leak():
    state, bio_reg, fast_reg, bio_res, fast_res = _sealed_build()
    state, bio_reg, fast_reg, fast_res = with_station_leak(
        state,
        bio_reg,
        fast_reg,
        fast_res,
        pool=CARBON_POOL,
        k_leak=_K_LEAK,
        start=_START,
        end=_END,
    )
    states, rationed, events = _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res)
    assert rationed == 0 and events == ()
    return states


def test_carbon_leak_lowers_biomass_and_scrubber_effort(
    sealed_baseline, carbon_leak
) -> None:
    # CARBON_POOL is only-removed (the scrubber cannot push it up): the leak lowers Ci
    # within the window, so the plant assimilates LESS (biomass below baseline) AND the
    # scrubber does LESS work (leaked CO₂ was not there to remove) — a signed cascade
    # into BOTH biology and ECLSS. The leak-sink strictly accumulates (the chamber
    # opens).
    assert _biomass(carbon_leak[-1]) < _biomass(sealed_baseline[-1])
    assert carbon_leak[-1].stocks[CO2_REMOVED].amount < (
        sealed_baseline[-1].stocks[CO2_REMOVED].amount
    )
    assert carbon_leak[-1].stocks[LEAK_SINK].amount > 0.0


def test_carbon_leak_conserves_total_with_sink(carbon_leak) -> None:
    _assert_matter_conserved(carbon_leak)


# --- atmosphere leak on O2_POOL: defended pool ⇒ makeup effort, biology untouched -----
@pytest.fixture(scope="module")
def o2_leak():
    state, bio_reg, fast_reg, bio_res, fast_res = _sealed_build()
    state, bio_reg, fast_reg, fast_res = with_station_leak(
        state,
        bio_reg,
        fast_reg,
        fast_res,
        pool=O2_POOL,
        k_leak=_K_LEAK,
        start=_START,
        end=_END,
    )
    states, rationed, events = _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res)
    assert rationed == 0 and events == ()
    return states


def test_o2_leak_is_absorbed_by_makeup_effort(sealed_baseline, o2_leak) -> None:
    # O2_POOL is DEFENDED (O2Makeup is demand-controlled), so — unlike CARBON — the leak
    # surfaces as makeup EFFORT, not a pool/biology change: o2_supply supplies strictly
    # MORE (its cumulative bookkeeping runs further negative), the plant is UNTOUCHED
    # (biomass ≈ baseline), and the leak-sink accumulates. The two pools fail
    # differently.
    assert o2_leak[-1].stocks[O2_SUPPLY].amount < (
        sealed_baseline[-1].stocks[O2_SUPPLY].amount
    )  # more negative = more O₂ supplied
    assert math.isclose(
        _biomass(o2_leak[-1]), _biomass(sealed_baseline[-1]), rel_tol=1e-6
    )
    assert o2_leak[-1].stocks[LEAK_SINK].amount > 0.0


def test_o2_leak_conserves_total_with_sink(o2_leak) -> None:
    _assert_matter_conserved(o2_leak)


# --- crew load spike: both ECLSS regulators work harder + food drains faster ----------
@pytest.fixture(scope="module")
def crew_spike():
    state, bio_reg, fast_reg, bio_res, fast_res = _sealed_build()
    fast_res = with_crew_load_spike(fast_res, start=_START, end=_END, factor=2.0)
    states, rationed, events = _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res)
    assert rationed == 0 and events == ()
    return states


def test_crew_spike_raises_regulator_effort_and_drains_food(
    sealed_baseline, crew_spike
) -> None:
    # A doubled food intake drives more respiration → BOTH regulators work harder
    # (co2_removed up AND o2_supply further negative) and food_store depletes faster.
    # The gas pools are held at setpoint by the regulators — so the signature is EFFORT,
    # not level (the regulator-erasure, asserted below).
    assert crew_spike[-1].stocks[CO2_REMOVED].amount > (
        sealed_baseline[-1].stocks[CO2_REMOVED].amount
    )
    assert crew_spike[-1].stocks[O2_SUPPLY].amount < (
        sealed_baseline[-1].stocks[O2_SUPPLY].amount
    )
    assert crew_spike[-1].stocks[FOOD_STORE].amount < (
        sealed_baseline[-1].stocks[FOOD_STORE].amount
    )


def test_crew_spike_pools_return_to_setpoint(sealed_baseline, crew_spike) -> None:
    # Regulator-erasure (the load-bearing finding): despite the doubled crew load, the
    # day-boundary CARBON_POOL / O2_POOL return to the SAME setpoint as baseline — the
    # naive "pool rises/falls" signature is erased, which is WHY the effort signal above
    # is the real one. (This is the Step-3 regulator-erasure sustained under a
    # disturbance.)
    for pool in (CARBON_POOL, O2_POOL):
        assert math.isclose(
            crew_spike[-1].stocks[pool].amount,
            sealed_baseline[-1].stocks[pool].amount,
            rel_tol=1e-6,
        )


def test_crew_spike_conserves(crew_spike) -> None:
    _assert_matter_conserved(crew_spike)


# --- lighting failure: growth stalls AND battery spared (energy↔biology, #16) ---------
@pytest.fixture(scope="module")
def lighting_failure():
    state, bio_reg, fast_reg, bio_res, fast_res = _sealed_build()
    bio_res, fast_res = with_lighting_failure(bio_res, fast_res, start=_START, end=_END)
    states, rationed, events = _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res)
    assert rationed == 0 and events == ()
    return states


def test_lighting_failure_stalls_growth_and_spares_battery(
    sealed_baseline, lighting_failure
) -> None:
    # The #16 lamp is ONE device with two legs: cutting PAR (a forcing) stalls the
    # biosphere's growth (biomass below baseline), and cutting the lamp draw (the Lamp
    # flow's energy) SPARES the battery (it drains slower ⇒ higher SOC). One
    # intervention, a cascade in both directions — energy↔biology, no shared stock.
    assert _biomass(lighting_failure[-1]) < _biomass(sealed_baseline[-1])
    assert lighting_failure[-1].stocks[BATTERY].amount > (
        sealed_baseline[-1].stocks[BATTERY].amount
    )


def test_lighting_failure_carbon_pool_returns_to_setpoint(
    sealed_baseline, lighting_failure
) -> None:
    # Regulator-erasure again: the plant assimilates less, but the scrubber holds
    # CARBON_POOL at setpoint, so the day-boundary pool is unchanged from baseline — the
    # growth deficit lives in the biomass, not the (regulated) pool.
    assert math.isclose(
        lighting_failure[-1].stocks[CARBON_POOL].amount,
        sealed_baseline[-1].stocks[CARBON_POOL].amount,
        rel_tol=1e-6,
    )


def test_lighting_failure_conserves(lighting_failure) -> None:
    _assert_matter_conserved(lighting_failure)


def test_matter_perturbation_is_deterministic() -> None:
    # The no-golden insurance (matter side): a perturbed two-rate sealed run is
    # bit-identical on re-run (stands in for the absent golden).
    out = []
    for _ in range(2):
        state, bio_reg, fast_reg, bio_res, fast_res = _sealed_build()
        bio_res, fast_res = with_lighting_failure(
            bio_res, fast_res, start=_START, end=_END
        )
        states, _, _ = _sealed_run(state, bio_reg, fast_reg, bio_res, fast_res)
        out.append(states[-1])
    assert out[0] == out[1]
