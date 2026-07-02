"""Shared canonical Tier-2 sealed-station run (P6.7) — one source of truth.

The Tier-2 combined-ledger run is the expensive artifact (~915 master days × 1440
sub-steps ≈ 1.3 M sub-steps, ~3 min): the fully-coupled sealed station over ``years``
annual cycles. Both the stability gate (``test_sealed_station_stability``) and the
regression golden (``test_regression_sealed_station``) assert on the *same* trajectory,
so
this module centralizes the build + run (and a session-scoped ``conftest`` fixture
caches
it so it runs **once** per session, not once per file). Not a test module (no ``test_``
prefix / no test functions) — pytest imports it, does not collect it.

The run is Euler-only (the biosphere is Euler-locked by its freeze),
``with_harvest=False``
/ ``close_feces=False`` (the Tier-2 scope — harvest starves the re-sow, the litter loop
is
the unregulated one, both spike-measured; see ``station.sealed`` / the Tier-3 landmine).
"""

import json
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

from domains.biosphere.drift import drift_slope, mass_drift_trace, max_abs
from domains.biosphere.stocks import (
    LEAF_C,
    LITTER_CARBON,
    MICROBIAL_CARBON,
    ROOT_C,
    STEM_C,
    STORAGE_C,
)
from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from domains.power.loader import load_charge_params
from domains.thermal.loader import load_thermal_params
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State
from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)
from station.scenario import SEALED_STATION_SCENARIO, SealedStationScenario
from station.sealed import (
    build_sealed_station,
    run_sealed,
    sealed_bio_resolver,
    sealed_fast_resolver,
)

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The five conserved quantities the combined sealed ledger spans (matter + ENERGY).
QUANTITIES = (
    Quantity.CARBON,
    Quantity.OXYGEN,
    Quantity.WATER,
    Quantity.NITROGEN,
    Quantity.ENERGY,
)

# Relative drift bounds — the station-scale re-derivation of ``drift.py``'s absolute
# bounds
# (advisor-flagged: station stocks span 1e0–1e10, so ``MASS_DRIFT_ABS_BOUND = 1e-9``
# false-fails). The normalization is by :func:`quantity_scale` — the **max single-stock
# magnitude** carrying the quantity, NOT ``total_q(0)``: round-off in a fold of terms of
# magnitude ``M`` is ~``eps·M``, and the boundary reservoirs (``o2_supply`` etc.) grow
# far
# larger than the small conserved total (OXYGEN's total ~27 mol while ``o2_supply``
# reaches
# ~2e5), so ``total_q(0)`` is the wrong scale (it makes round-off look like a 4e-9
# "leak").
# PROVENANCE (1-yr measurement, 2026-07-02): worst-case relative ``max|d_q|`` ~ 1e-11
# (CARBON), relative slope ~ 3e-14 (machine-ε); horizon-invariant (both drift and scale
# grow together). A real leak puts relative drift orders higher, so these keep teeth.
REL_DRIFT_BOUND: float = 1e-10
REL_SLOPE_BOUND: float = 1e-12

# The biosphere organic-carbon stocks — the biomass watch's "total organic C" (the plant
# organs + grain + the decomposer pool). The decomposer pool is the only piece that
# MOVES
# year-over-year (the plant is period-1), so "total organic C" — not "peak leaf_c" — is
# the honest bounded-ness metric (peak leaf_c would hide the decomposer's approach to
# its
# steady state; advisor-flagged).
BIO_ORGANIC_C = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, LITTER_CARBON, MICROBIAL_CARBON)


def total_organic_c(state: State) -> float:
    """Total biosphere organic carbon (mol) — the biomass-watch summary quantity."""
    return sum(state.stocks[s].amount for s in BIO_ORGANIC_C)


def peak_organic_c(segment: Sequence[State]) -> float:
    """Per-year summary: peak total-organic-C over a year segment (grown-plant peak).

    ``year_summaries`` slices each year including its boundary, so the peak is the fully
    grown biosphere just before the re-sow — the year's biomass level. Bounded ⇔ these
    per-year peaks converge (the decomposer pool reaches its steady state).
    """
    return max(total_organic_c(s) for s in segment)


def quantity_scale(states: Sequence[State], quantity: Quantity) -> float:
    """Max single-stock magnitude of ``quantity`` over states — the round-off scale.

    A conservation fold ``Σ amount·composition`` of terms of magnitude ``M`` carries
    round-off ~``eps·M``; the boundary reservoirs grow far larger than the (small)
    conserved
    total, so this — not ``total_q(0)`` — is the scale relative drift must be measured
    against. Never zero (falls back to ``1.0``).
    """
    return (
        max(
            max(
                abs(s.amount * s.composition.get(quantity, 0.0))
                for s in st.stocks.values()
            )
            for st in states
        )
        or 1.0
    )


def relative_drift(states: Sequence[State], quantity: Quantity) -> tuple[float, float]:
    """``(relative max|drift|, relative |slope|)`` of ``quantity`` vs round-off scale.

    The station-scale conservation-drift diagnostic: ``max|d_q| / scale`` (accumulation)
    and ``|slope| / scale`` (the systematic-leak signature). Both at the round-off floor
    ⇒
    conserved with no drift; a nonzero relative slope ⇒ a leak.
    """
    trace = mass_drift_trace(states, quantity)
    scale = quantity_scale(states, quantity)
    return max_abs(trace) / scale, abs(drift_slope(trace)) / scale


def weather(years: int) -> list[dict[str, float | str]]:
    """The winter-wheat season tiled ``years×`` — covers ``[0, years·season)`` with no
    ``_table`` end-clamp (each tile is one 305-day season = the reset period)."""
    season = json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]
    return season * years


@dataclass(frozen=True)
class Tier2Run:
    """The cached Tier-2 result: day-boundary trace + the run's rationed/events."""

    states: list[State]
    rationed: int
    events: tuple[object, ...]


def run_tier2(scenario: SealedStationScenario = SEALED_STATION_SCENARIO) -> Tier2Run:
    """Build + run the canonical Tier-2 sealed station; return the day-boundary trace.

    ``build_sealed_station`` (``with_harvest=False`` / ``close_feces=False``) + the
    two-rate :func:`station.sealed.run_sealed` with the annual re-sow hook, over
    ``scenario.days`` (= ``years · season_days``) master days. The every-sub-step
    conservation gate runs *inside* the driver, so a completed run is itself proof the
    combined ledger balanced every sub-step; the returned ``states`` (one per day
    boundary,
    length ``days + 1``) carry the day-boundary trace the drift / biomass /
    regulated-pool
    assertions and the final-State golden read. ``rationed`` must be ``0`` and
    ``events``
    ``()`` (the callers' gates assert it).
    """
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
        close_feces=False,
    )
    states, rationed, events = run_sealed(
        EulerIntegrator(bio_reg),
        EulerIntegrator(fast_reg),
        state,
        sealed_bio_resolver(weather(scenario.years), lamp, scenario),
        sealed_fast_resolver(charge, scenario),
        scenario,
    )
    return Tier2Run(states=states, rationed=rationed, events=events)
