"""Step-7 (P6.7) Tier-3 landmine: the drift instrument EARNS ITS KEEP (assertion-only).

The Tier-2 run scopes out the litter/microbial loop (``close_feces=False``) because it
is
the one *unregulated* loop and is **uncalibrated** at illustrative crew-vs-plant scale
(Step 6's ~3400× mismatch). This test turns that scope boundary into a **test, not a
prose
caveat** (advisor addition): run the *same* sealed station with ``close_feces=True``
(crew
feces → the seedling-scale ``LITTER_CARBON``) and show the drift instrument correctly
flags
the resulting non-stationarity, and that the Euler backstop fires — the empirical Step-9
calibration prerequisite, far stronger than a comment.

**What actually happens (spike-measured, sharper than the design's first framing).**
Litter
reaches a quasi-steady ~2600 mol (microbes consume the influx), but
**``microbial_carbon``
grows unbounded** (linear, ~44 mol/day) so **total organic C ramps**; and the once-daily
microbial O₂ draw overtakes ``O2Makeup`` almost immediately ⇒ **``rationed > 0`` from
~day
25**. The growth is *linear* (constant year-over-year diffs), which ``is_stationary``'s
non-amplifying (slope) clause would pass — but its **bound** clause fails (the ~1e4 diff
≫
any healthy-biomass bound), so the *same* ``is_stationary(bound=1.0)`` that PASSES the
Tier-2 biomass watch (~0.09 diffs) FAILS here. That symmetric discriminator is the
point.

Conservation still holds every sub-step (the run completes under the driver's gate —
rationing is conservation-preserving, it only scales withdrawals), so this is a
**non-stationarity / rationing** finding, not a leak. No golden (it deliberately
produces a
rationing, non-stationary run). Marked ``slow`` (a ~2-yr coupled run).
"""

import pytest

from domains.biosphere.drift import (
    is_stationary,
    non_collapsing,
    same_phase_diffs,
    year_summaries,
)
from domains.biosphere.stocks import MICROBIAL_CARBON
from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from domains.power.loader import load_charge_params
from domains.thermal.loader import load_thermal_params
from sealed_tier2_helper import peak_organic_c, weather
from simcore.integrator import EulerIntegrator
from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)
from station.scenario import SealedStationScenario
from station.sealed import (
    build_sealed_station,
    run_sealed,
    sealed_bio_resolver,
    sealed_fast_resolver,
)

pytestmark = pytest.mark.slow

# Two annual cycles: enough for ≥2 year summaries (the ramp shows as a large
# year-over-year
# diff), while rationing appears within the first month. Cheaper than the Tier-2 horizon
# —
# this is an assertion-only characterization, not a golden.
_LANDMINE_YEARS = 2


@pytest.fixture(scope="module")
def landmine_run():
    """The ``close_feces=True`` sealed run (2 yr) — deliberately non-stationary."""
    scenario = SealedStationScenario(years=_LANDMINE_YEARS)
    charge = load_charge_params()
    thermal = load_thermal_params()
    crew = load_crew_params()
    eclss = load_eclss_params()
    recovery = load_water_recovery_params()
    lamp = load_lamp_params()
    harvest = load_harvest_params()
    state, bio_reg, fast_reg = build_sealed_station(
        charge,
        thermal,
        crew,
        eclss,
        recovery,
        lamp,
        harvest,
        scenario,
        with_harvest=False,
        close_feces=True,
    )
    # The run completes only if conservation holds EVERY sub-step (the driver's gate) —
    # even though the backstop fires. So a completed run proves rationing is
    # conservation-preserving (not a leak).
    states, rationed, events = run_sealed(
        EulerIntegrator(bio_reg),
        EulerIntegrator(fast_reg),
        state,
        sealed_bio_resolver(weather(scenario.years), lamp, scenario),
        sealed_fast_resolver(charge, scenario),
        scenario,
    )
    return states, rationed, events, scenario


def test_landmine_rations(landmine_run) -> None:
    # The Euler backstop fires: the once-daily microbial O₂ draw overtakes O2Makeup's
    # refill. This empirically pins the Step-9 calibration prerequisite — at
    # illustrative
    # crew-vs-plant scale the closed feces loop is not viable (a hard number, not a
    # caveat).
    _, rationed, _, _ = landmine_run
    assert rationed > 0, (
        "close_feces=True at illustrative scale must ration (microbial O₂ draw over "
        "O2Makeup) — the drift/backstop instrument earning its keep"
    )


def test_landmine_biomass_non_stationary(landmine_run) -> None:
    # The instrument correctly flags the non-stationarity: total organic C ramps
    # (the microbial pool grows unbounded), so its year-over-year same-phase diff is
    # ~1e4 —
    # is_stationary FAILS (via the bound clause), the SAME is_stationary(bound=1.0) that
    # PASSES the Tier-2 biomass watch. The symmetric discriminator: a bound that a
    # converged
    # biosphere (~0.09 diffs) clears and a ramping one (~1e4) does not.
    states, _, _, scenario = landmine_run
    peaks = year_summaries(states, scenario.season_days, peak_organic_c)
    assert len(peaks) >= 2
    diffs = same_phase_diffs(peaks, period=1)
    assert not is_stationary(diffs, bound=1.0, slope_tol=1e-2), (
        f"close_feces=True must be flagged NON-stationary (ramping), diffs={diffs}"
    )
    # And it is a GROWTH ramp (not collapse): the later year's peak exceeds the earlier
    # by a
    # wide margin (microbial pool accumulating).
    assert peaks[-1] > peaks[0] * 1.2, (
        f"total organic C must ramp up materially (microbial bloom): {peaks}"
    )
    assert non_collapsing(peaks, floor=1.0)


def test_landmine_microbial_pool_grows(landmine_run) -> None:
    # The concrete unbounded pool: microbial biomass grows year-over-year (the
    # decomposer
    # can't clear the crew-scale feces influx), the driver of the non-stationarity
    # above.
    states, _, _, scenario = landmine_run
    year = scenario.season_days
    micro = [
        states[k * year].stocks[MICROBIAL_CARBON].amount
        for k in range(scenario.years + 1)
    ]
    assert micro[-1] > micro[1] * 1.5, (
        f"microbial_carbon must grow unbounded under close_feces=True: {micro}"
    )
