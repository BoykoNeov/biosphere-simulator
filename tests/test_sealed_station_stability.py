"""Step-7 (P6.7) sealed-station stability — Tier 1 (energy) + Tier 2 (multi-year).

The Phase-4 analogue at station scale, split in two (advisor-reviewed; never conflated):

* **Tier 1 — the energy decade (the clean Phase-4 analogue).** Power → Thermal via the
  single-rate ``run_station`` at the diurnal ``dt = 3600 s`` (so ``n`` advances and the
  half-sine solar drives a real daily cycle), run 15 yr. ENERGY only: a genuine emergent
  ``T_eq`` attractor, a permanent ``boundary.space``, daily-periodic forcing. The node/T
  is
  a **period-1 fixed point** (every year's peak identical), the SOC a period-1 daily
  cycle;
  ENERGY conserves to round-off (``drift.py`` slope flat, *relative* — station joules
  are
  ~1e9, so the chamber-scale absolute bounds do NOT transfer, advisor-flagged);
  ``rationed == 0`` / ``events == ()``.

* **Tier 2 — the combined-ledger multi-year run (the genuinely NEW thing).** The fully-
  coupled two-rate sealed station (biosphere-slow + everything-fast) over ``years``
  annual
  cycles, ``close_feces=False`` / ``with_harvest=False`` (the Tier-2 scope). Energy and
  matter share no stock, so "the combined ledger conserves" is two disjoint per-quantity
  ledgers each conserving — what is *new* is that the **full ~11-flow assembly across
  five
  domains sustains** every-quantity **and** ENERGY conservation to round-off with no
  drift
  over many annual cycles (axis-(a) relative drift flat per quantity on the day-boundary
  trace), while the regulated pools stay stationary and the **coupled biosphere biomass
  stays bounded** under the pinned-CO₂ regime the freeze never validated (the mandatory
  biomass watch: the plant is period-1, the decomposer pool *converges* — its year-over-
  year diffs SHRINK, not merely stay non-amplifying, so a linear ramp is not mistaken
  for
  convergence; advisor-flagged). Whole-system matter stationarity is honestly
  **deferred**
  (the crew stores drain; feces open) — this gate never claims "the station is
  stationary."

The Tier-2 run is the expensive artifact (~1.3 M sub-steps, ~3 min), shared with the
regression golden via the session-scoped ``sealed_tier2_run`` fixture (``conftest``).
The
whole module is marked ``slow``.
"""

import pytest

from domains.biosphere.drift import (
    is_stationary,
    non_collapsing,
    same_phase_diffs,
    year_summaries,
)
from domains.power.loader import load_charge_params
from domains.power.stocks import BATTERY
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import NODE
from sealed_tier2_helper import (
    QUANTITIES,
    REL_DRIFT_BOUND,
    REL_SLOPE_BOUND,
    peak_organic_c,
    relative_drift,
    weather,
)
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.scenario import (
    HEAT_CLOSURE_SCENARIO,
    SEALED_ENERGY_DAYS,
    SEALED_STATION_SCENARIO,
    SEALED_STATION_SEASON_DAYS,
    SealedStationScenario,
)
from station.sealed import (
    build_sealed_station,
    run_sealed,
    sealed_bio_resolver,
    sealed_fast_resolver,
)
from station.system import (
    build_station,
    predicted_equilibrium_temperature,
    run_station,
    station_resolver,
)

pytestmark = pytest.mark.slow


# --- Tier 1: the energy decade -------------------------------------------------------


@pytest.fixture(scope="module")
def energy_states() -> list[State]:
    """The 15-yr Power → Thermal trajectory (single-rate, diurnal). Cheap (~4 s)."""
    charge = load_charge_params()
    thermal = load_thermal_params()
    state, registry = build_station(charge, thermal, HEAT_CLOSURE_SCENARIO)
    states, rationed, events = run_station(
        EulerIntegrator(registry),
        state,
        station_resolver(charge, HEAT_CLOSURE_SCENARIO),
        HEAT_CLOSURE_SCENARIO.power.dt_seconds,
        SEALED_ENERGY_DAYS * HEAT_CLOSURE_SCENARIO.power.steps_per_day,
    )
    assert rationed == 0, "Tier-1 energy decade must be well-fed (no arbitration)"
    assert events == (), "Tier-1 energy decade must be event-free"
    return states


def test_tier1_energy_conserved_no_drift(energy_states) -> None:
    # ENERGY conserves to round-off over 15 yr: relative max|drift| and slope at the
    # round-off floor (the leak detector, station-scale relative bounds).
    rel_abs, rel_slope = relative_drift(energy_states, Quantity.ENERGY)
    assert rel_abs < REL_DRIFT_BOUND, f"ENERGY relative drift {rel_abs:.2e} too large"
    assert rel_slope < REL_SLOPE_BOUND, (
        f"ENERGY relative slope {rel_slope:.2e} — a leak?"
    )


def test_tier1_node_is_period_1_fixed_point(energy_states) -> None:
    # The node/T reaches a real emergent equilibrium (an attractor, not a construction):
    # every year's peak node temperature is identical (a period-1 fixed point — no
    # seasonal
    # forcing in Power), stationary and non-collapsing, near the predicted mean-power
    # T_eq.
    charge = load_charge_params()
    thermal = load_thermal_params()
    steps_per_year = (
        HEAT_CLOSURE_SCENARIO.power.steps_per_day * SEALED_STATION_SEASON_DAYS
    )

    def peak_temp(segment) -> float:
        return max(
            thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity
            for s in segment
        )

    peaks = year_summaries(energy_states, steps_per_year, peak_temp)
    assert len(peaks) >= 3
    diffs = same_phase_diffs(peaks, period=1)
    assert is_stationary(diffs, bound=0.1, slope_tol=1e-3), (
        f"node peak-T must be stationary over the decade, diffs={diffs[:3]}"
    )
    assert non_collapsing(peaks, floor=100.0), "node must not collapse toward T_space"
    t_eq = predicted_equilibrium_temperature(charge, thermal, HEAT_CLOSURE_SCENARIO)
    assert abs(peaks[-1] - t_eq) < 1.0, (
        f"node peak-T {peaks[-1]:.3f} must sit near the dissipation-set T_eq {t_eq:.3f}"
    )


def test_tier1_soc_returns_daily(energy_states) -> None:
    # The battery SOC is a period-1 daily cycle: at each day boundary it returns to the
    # same level (daily-balanced), so the day-boundary spread is at the round-off floor.
    spd = HEAT_CLOSURE_SCENARIO.power.steps_per_day
    day_boundary_soc = [
        energy_states[i].stocks[BATTERY].amount
        for i in range(0, len(energy_states), spd)
    ]
    spread = max(day_boundary_soc) - min(day_boundary_soc)
    assert spread < 1.0, f"SOC day-boundary spread {spread:.3e} J — not daily-periodic"


# --- Tier 2: the combined-ledger multi-year run --------------------------------------


def test_tier2_rationed_and_events(sealed_tier2_run) -> None:
    # Well-fed (structural + sizing) and event-free — the annual re-sows are handled by
    # the
    # driver's slow-reset hook, NOT extinction events; nothing goes extinct.
    assert sealed_tier2_run.rationed == 0, "Tier-2 sealed run must be well-fed"
    assert sealed_tier2_run.events == (), "Tier-2 sealed run must be extinction-free"


def test_tier2_every_quantity_conserved_no_drift(sealed_tier2_run) -> None:
    # The payload: every conserved quantity AND ENERGY conserves to round-off over the
    # multi-year run — relative drift + slope at the round-off floor on the day-boundary
    # trace (the full ~11-flow, five-domain assembly leaks no quantity). A completed run
    # already proved per-sub-step closure (the driver's every-sub-step gate); this adds
    # the
    # no-slow-drift teeth over the day-boundary trace.
    states = sealed_tier2_run.states
    for quantity in QUANTITIES:
        rel_abs, rel_slope = relative_drift(states, quantity)
        assert rel_abs < REL_DRIFT_BOUND, (
            f"{quantity} relative drift {rel_abs:.2e} over the multi-year run — a leak?"
        )
        assert rel_slope < REL_SLOPE_BOUND, (
            f"{quantity} relative slope {rel_slope:.2e} — a systematic leak?"
        )


def test_tier2_biomass_bounded_and_converging(sealed_tier2_run) -> None:
    # The mandatory pinned-CO₂ biomass watch (advisor-flagged): the coupled biosphere
    # runs
    # under scrubber-held CO₂ the freeze never validated, and a slowly-growing biosphere
    # is
    # mass-conserving (axis-(a) stays flat and MASKS it) — so watch total organic C, NOT
    # peak leaf_c (which excludes the only pool that moves, the decomposer). The plant
    # is
    # period-1; the decomposer pool CONVERGES to its steady state — the year-over-year
    # same-phase diffs SHRINK (geometric), which is stronger than is_stationary's non-
    # amplifying clause (that clause passes a linear RAMP; a shrinking diff proves
    # genuine
    # convergence). Bounded is the point; NOT "the station is stationary."
    states = sealed_tier2_run.states
    year = SEALED_STATION_SCENARIO.season_days
    peaks = year_summaries(states, year, peak_organic_c)
    assert len(peaks) >= 3, "biomass watch needs ≥3 year summaries to see convergence"
    assert non_collapsing(peaks, floor=1.0), "the coupled biosphere must not collapse"
    diffs = same_phase_diffs(peaks, period=1)
    # Bounded past the year-1 spin-up. After the scope-B decomposer calibration
    # (docs/plans/post-roadmap-decomposer-calibration.md) the soil equilibria are ~2-3x
    # larger, so year 1 -- the only year without a prior annual plant-dump (~60 mol C
    # shed into litter by ``annual_reset``) already in the soil -- is a one-time
    # soil-establishment transient (year-1->2 diff ~7.85, was ~0.09 pre-calibration).
    # ``transient=1`` skips it and checks the settled tail (years 2-4, diffs 0.329/
    # 0.012); bound stays 1.0 << the Tier-3 landmine's ~1e4 ramp -- the run must be
    # non-amplifying past year 1. Skipping a documented spin-up is NOT relaxing the
    # bound.
    assert is_stationary(diffs, bound=1.0, slope_tol=1e-2, transient=1), (
        f"coupled biomass must be bounded/non-amplifying past spin-up, diffs={diffs}"
    )
    # Converging, not ramping: the later same-phase diff is strictly smaller (the
    # decomposer pool + soil establishment approach steady state -- the year-1->2
    # spin-up dwarfs the settled year-3->4 diff, so the shrink is emphatic).
    assert abs(diffs[-1]) < abs(diffs[0]), (
        f"biomass diffs must SHRINK (converge), not persist (ramp): {diffs}"
    )


def test_tier2_regulated_pools_stationary(sealed_tier2_run) -> None:
    # The ECLSS/recovery/radiator regulators hold their pools stationary: CO₂/O₂/H₂O and
    # the thermal node return to the same day-boundary level year-over-year (the fast
    # regulators fully relax between the once-daily biosphere lumps — Step-3 regulator-
    # erasure, now sustained multi-year). Day-boundary spread over the settled tail is
    # tiny
    # vs each pool's level.
    from domains.biosphere.stocks import CARBON_POOL, O2_POOL
    from domains.eclss.stocks import CABIN_H2O

    states = sealed_tier2_run.states
    # Compare year-boundary snapshots (n = k·season) over the settled years.
    year = SEALED_STATION_SCENARIO.season_days
    for pool in (CARBON_POOL, O2_POOL, CABIN_H2O, NODE):
        levels = [
            states[k * year].stocks[pool].amount
            for k in range(1, SEALED_STATION_SCENARIO.years)
        ]
        assert len(levels) >= 2
        scale = max(abs(v) for v in levels) or 1.0
        spread = max(levels) - min(levels)
        assert spread / scale < 1e-3, (
            f"{pool} not stationary year-over-year: spread {spread:.3e} on {scale:.3e}"
        )


# --- determinism + registration-order independence (cheap short runs) ----------------

# A short horizon: determinism and order-independence are per-step properties, so there
# is
# no need to pay the full multi-year run — 3 master days exercises the full 11-flow
# assembly and both registries.
_SHORT_SCENARIO = SealedStationScenario(years=1, season_days=3)


def _short_run(fast_flow_order=None) -> State:
    """Build + run the sealed station over the short horizon; return the final State.

    ``fast_flow_order`` optionally reorders the fast registry's flows (a callable
    applied
    to the flow list) — used to prove registration-order independence: the ``Registry``
    canonicalizes flow order by id, so any permutation must give the identical
    trajectory.
    """
    from domains.crew.loader import load_crew_params
    from domains.eclss.loader import load_eclss_params
    from simcore.registry import Registry
    from station.loader import (
        load_harvest_params,
        load_lamp_params,
        load_water_recovery_params,
    )

    charge = load_charge_params()
    thermal = load_thermal_params()
    crew = load_crew_params()
    eclss = load_eclss_params()
    recovery = load_water_recovery_params()
    lamp = load_lamp_params()
    harvest = load_harvest_params()
    state, bio_reg, fast_reg = build_sealed_station(
        charge, thermal, crew, eclss, recovery, lamp, harvest, _SHORT_SCENARIO
    )
    if fast_flow_order is not None:
        fast_reg = Registry(fast_flow_order(list(fast_reg.flows)), state.stocks)
    states, _, _ = run_sealed(
        EulerIntegrator(bio_reg),
        EulerIntegrator(fast_reg),
        state,
        sealed_bio_resolver(weather(_SHORT_SCENARIO.years), lamp, _SHORT_SCENARIO),
        sealed_fast_resolver(charge, _SHORT_SCENARIO),
        _SHORT_SCENARIO,
    )
    return states[-1]


def test_sealed_determinism_short() -> None:
    # Bit-identical re-run: two independent builds + runs give the identical final
    # State.
    assert _short_run() == _short_run(), (
        "the sealed station must be deterministic (bit-identical)"
    )


def test_sealed_registration_order_independent() -> None:
    # Registration-order independence (a design-named Tier-2 deliverable): the 11-flow
    # fast
    # registry canonicalizes flow order by id, so building it with the flows REVERSED
    # must
    # give the identical final State — a real cross-domain composition bug (an
    # order-dependent reduction) that plain determinism (same order twice) would not
    # catch.
    assert _short_run() == _short_run(fast_flow_order=lambda fs: list(reversed(fs))), (
        "the sealed station must be registration-order independent (Registry sorts ids)"
    )
