"""Regression gate: the golden sealed-station run (P6.7) — Tier-2 State + Tier-1.

Two additive **NON-frozen** goldens (not in ``docs/biosphere-reference.manifest.json`` —
Step 7 re-wires the frozen biosphere *beside* its freeze), the sealed-station analogues
of
the Phase-4 close:

* **``sealed_station_state.json``** — the Tier-2 fully-coupled multi-year run's **day-
  boundary final ``State``** (``sim_io`` hex-float, byte-compared + load-back). Any bit
  change anywhere in the ~11-flow, five-domain assembly (a biosphere/cabin/Power/Thermal
  flow, the reverse-seam wiring, the two-rate driver + re-sow hook, a param, the
  reduction
  order, the scenario sizing) surfaces here. The **pre-golden gate** bakes Tier-2's
  purpose
  in: ``rationed == 0`` / ``events == ()``, every quantity conserved to round-off (the
  combined-ledger payload), the coupled biosphere biomass bounded (the pinned-CO₂
  watch),
  the regulated pools at their setpoints, and the **feces boundary open** (Tier-2 scope)
  —
  a degenerate/leaking/ramping run is **unpinnable**.

* **``sealed_energy_drift_summary.json``** — the Tier-1 energy-decade **stability
  signature** (per-year peak node temperature + the period class), the Phase-4
  drift-summary
  golden placed on TIER 1 (15 yr, where a period class is characterizable) — NOT Tier 2
  (~3
  yr = too few points, would pin noise). Mass-drift round-off is deliberately NOT pinned
  (it's noise); this pins the *shape* (the node's period-1 fixed point) a single
  final-State
  snapshot cannot.

**Bit-stability caveat** (as for the biosphere / Power goldens): the biosphere weather
conversions + FvCB + the SB radiator use ``math`` transcendentals (not IEEE-754
correctly-rounded), so these goldens are bit-identical **within a build**;
cross-platform
last-ULP differences are tolerance territory. Regenerate (review the diff) if the
toolchain
moves. Marked ``slow`` (the Tier-2 run is ~3 min; shared with the stability gate via the
session-scoped ``sealed_tier2_run`` fixture).
"""

import json
from pathlib import Path

import pytest

import sim_io
from domains.biosphere.drift import (
    is_stationary,
    same_phase_diffs,
    year_summaries,
)
from domains.biosphere.stocks import CARBON_POOL, O2_POOL
from domains.crew.stocks import FECAL_WASTE
from domains.power.loader import load_charge_params
from domains.thermal.loader import load_thermal_params
from domains.thermal.stocks import NODE
from golden_platform import windows_golden_only
from sealed_tier2_helper import (
    QUANTITIES,
    REL_DRIFT_BOUND,
    REL_SLOPE_BOUND,
    peak_organic_c,
    relative_drift,
    run_tier2,
)
from simcore.integrator import EulerIntegrator
from simcore.state import State
from station.scenario import (
    HEAT_CLOSURE_SCENARIO,
    SEALED_ENERGY_DAYS,
    SEALED_ENERGY_YEARS,
    SEALED_STATION_SCENARIO,
    SEALED_STATION_SEASON_DAYS,
)
from station.system import (
    build_station,
    predicted_equilibrium_temperature,
    run_station,
    station_resolver,
)

pytestmark = pytest.mark.slow

GOLDEN_DIR = Path(__file__).parent / "regression" / "golden"
STATE_GOLDEN = GOLDEN_DIR / "sealed_station_state.json"
ENERGY_SUMMARY_GOLDEN = GOLDEN_DIR / "sealed_energy_drift_summary.json"


# --- Tier-2 final-State golden -------------------------------------------------------


def _gate(states: list[State], rationed: int, events: tuple[object, ...]) -> None:
    """The Tier-2 pre-golden gate — a degenerate / ramping / leaking run is unpinnable.

    Bakes Tier-2's purpose into the golden's provenance: well-fed + event-free; every
    conserved quantity closes to round-off over the whole run (relative drift, station
    scale); the regulated CO₂ / O₂ pools sit at their control setpoints; the coupled
    biosphere biomass is bounded (period-1 plant + converging decomposer — the
    pinned-CO₂
    watch); and the feces boundary is OPEN (Tier-2 scope: ``FECAL_WASTE`` accumulated,
    the
    litter loop not closed). Only then is the final State pinnable.
    """
    assert rationed == 0, "golden sealed run must be well-fed (no arbitration)"
    assert events == (), "golden sealed run must be extinction-free"
    for quantity in QUANTITIES:
        rel_abs, rel_slope = relative_drift(states, quantity)
        assert rel_abs < REL_DRIFT_BOUND, (
            f"golden sealed run must keep {quantity} closed to round-off"
        )
        assert rel_slope < REL_SLOPE_BOUND, (
            f"golden sealed run must have no {quantity} leak"
        )
    # Regulated pools at their crew-driven setpoints (the scrubber holds CARBON_POOL
    # near
    # P/k_scrub, the makeup holds O2_POOL near the setpoint) — evidence the fast loops
    # relaxed, sustained multi-year.
    final = states[-1]
    assert 0.0 < final.stocks[CARBON_POOL].amount < 10.0
    assert 0.0 < final.stocks[O2_POOL].amount < 20.0
    # Biomass bounded (the pinned-CO2 watch): year-over-year same-phase diffs shrink.
    # ``transient=1`` skips the year-1 soil-establishment spin-up: after the scope-B
    # decomposer calibration (docs/plans/post-roadmap-decomposer-calibration.md) the
    # soil equilibria are ~2-3x larger, so year 1 -- the only year with no prior annual
    # plant-dump (``annual_reset`` sheds ~60 mol C into litter) already in the soil --
    # is a one-time transient (year-1->2 diff ~7.85). Years 2-4 are the settled tail
    # (diffs 0.329, 0.012): is_stationary sees two shrinking diffs there. Skipping a
    # documented spin-up year is NOT relaxing the amplitude bound (that would pass
    # amplifying drift); the bound stays 1.0 and the run must still be non-amplifying
    # past year 1.
    peaks = year_summaries(states, SEALED_STATION_SCENARIO.season_days, peak_organic_c)
    diffs = same_phase_diffs(peaks, period=1)
    assert is_stationary(diffs, bound=1.0, slope_tol=1e-2, transient=1), (
        f"golden sealed biomass must be bounded past the year-1 spin-up, diffs={diffs}"
    )
    # Feces boundary OPEN (Tier-2 scope, close_feces=False): the FECAL_WASTE sink exists
    # and
    # carried the crew's egested carbon (the litter loop is deliberately not closed
    # here).
    assert final.stocks[FECAL_WASTE].amount > 0.0, (
        "Tier-2 golden must leave the feces boundary open (FECAL_WASTE accumulated)"
    )


@windows_golden_only
def test_sealed_station_golden_bytes_match(sealed_tier2_run) -> None:
    _gate(sealed_tier2_run.states, sealed_tier2_run.rationed, sealed_tier2_run.events)
    expected = sim_io.dumps(sealed_tier2_run.states[-1]).encode("utf-8")
    assert expected == STATE_GOLDEN.read_bytes()


@windows_golden_only
def test_sealed_station_golden_loads_back(sealed_tier2_run) -> None:
    text = STATE_GOLDEN.read_text(encoding="utf-8")
    assert sim_io.loads(text) == sealed_tier2_run.states[-1]


# --- Tier-1 energy drift-summary golden ----------------------------------------------


@pytest.fixture(scope="module")
def energy_states() -> list[State]:
    """The 15-yr Power → Thermal trajectory (single-rate diurnal; cheap ~4 s)."""
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
    assert rationed == 0 and events == ()
    return states


def _node_peak_temps(states: list[State]) -> list[float]:
    thermal = load_thermal_params()

    def peak_temp(segment) -> float:
        return max(
            thermal.space_temperature + s.stocks[NODE].amount / thermal.heat_capacity
            for s in segment
        )

    steps_per_year = (
        HEAT_CLOSURE_SCENARIO.power.steps_per_day * SEALED_STATION_SEASON_DAYS
    )
    return year_summaries(states, steps_per_year, peak_temp)


def _to_hex(value: object) -> object:
    if isinstance(value, float):
        return value.hex()
    if isinstance(value, list):
        return [_to_hex(v) for v in value]
    if isinstance(value, dict):
        return {k: _to_hex(v) for k, v in value.items()}
    return value


def _energy_summary_dumps(states: list[State]) -> str:
    peaks = _node_peak_temps(states)
    data = {
        "horizon_years": SEALED_ENERGY_YEARS,
        "node_peak_temp_k": peaks,
        "is_stationary": is_stationary(
            same_phase_diffs(peaks, period=1), bound=0.1, slope_tol=1e-3
        ),
    }
    return json.dumps(_to_hex(data), indent=2, sort_keys=True) + "\n"


@windows_golden_only
def test_energy_drift_summary_bytes_match(energy_states) -> None:
    assert _energy_summary_dumps(energy_states).encode("utf-8") == (
        ENERGY_SUMMARY_GOLDEN.read_bytes()
    )


@windows_golden_only
def test_energy_drift_summary_loads_back(energy_states) -> None:
    parsed = json.loads(ENERGY_SUMMARY_GOLDEN.read_text(encoding="utf-8"))
    assert parsed["horizon_years"] == SEALED_ENERGY_YEARS
    assert [float.fromhex(h) for h in parsed["node_peak_temp_k"]] == _node_peak_temps(
        energy_states
    )
    # The node sits near the dissipation-set equilibrium (the attractor the signature
    # pins).
    charge = load_charge_params()
    thermal = load_thermal_params()
    t_eq = predicted_equilibrium_temperature(charge, thermal, HEAT_CLOSURE_SCENARIO)
    assert abs(_node_peak_temps(energy_states)[-1] - t_eq) < 1.0


def _regenerate() -> None:
    """Rewrite both committed sealed-station goldens from the current engine output.

    A deliberately separate, explicit action — NOT reachable from a test run. Run via::

        uv run python tests/test_regression_sealed_station.py

    Review the diff before committing: a change means the coupled multi-year output
    (Tier 2)
    or the energy stability signature (Tier 1) moved.
    """
    tier2 = run_tier2()
    _gate(tier2.states, tier2.rationed, tier2.events)
    STATE_GOLDEN.write_bytes(sim_io.dumps(tier2.states[-1]).encode("utf-8"))
    print(f"wrote {STATE_GOLDEN}")

    charge = load_charge_params()
    thermal = load_thermal_params()
    state, registry = build_station(charge, thermal, HEAT_CLOSURE_SCENARIO)
    states, _, _ = run_station(
        EulerIntegrator(registry),
        state,
        station_resolver(charge, HEAT_CLOSURE_SCENARIO),
        HEAT_CLOSURE_SCENARIO.power.dt_seconds,
        SEALED_ENERGY_DAYS * HEAT_CLOSURE_SCENARIO.power.steps_per_day,
    )
    ENERGY_SUMMARY_GOLDEN.write_bytes(_energy_summary_dumps(states).encode("utf-8"))
    print(f"wrote {ENERGY_SUMMARY_GOLDEN}")


if __name__ == "__main__":
    _regenerate()
