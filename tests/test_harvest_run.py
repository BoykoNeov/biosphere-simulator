"""Validation: the Step-6 biomass/food loop — biosphere grain → crew food (P6.6).

Exercises the coupled harvest greenhouse (``station.harvest``): the reproductive plant
fills ``storage_c`` (grain) once per master day, the station-owned ``Harvest`` flow
drains it into the crew ``food_store`` across the day's cabin sub-steps, so the crew's
finite food becomes **regenerative** (the CARBON twin of Step 4's ``WaterRecovery``).

Covers **both** seams the plan splits (landed as separate commits, exercised together
here). Seam 1 is the ``Harvest`` flow (``with_harvest``); seam 2 (``close_feces``,
default on) re-points crew feces into the biosphere ``LITTER_CARBON`` pool to close the
CARBON ring.

The seam-1 payload is the signed **two-way conservation identity** (``Δfood_store =
cumulative harvest = Δstorage_c``): harvest is ``storage_c``'s only new sink and the
``Allocation`` grain-fill leg is independent of ``storage_c``'s level, so grain fill is
identical with and without harvest → the identity is exact to the fp cancellation floor
(an un-biting run flips the signs). The seam-2 payload is closure + no-shadow-sink +
litter-grows-materially (crew-scale feces makes ``MicrobialRespiration`` **active**, so
``Δlitter ≈ feces`` does *not* hold — a three-way identity is not the gate). Euler-only:
the greenhouse biosphere is Euler-locked by its freeze.
"""

import json
from pathlib import Path

import pytest

from domains.biosphere.stocks import (
    CONDENSATE,
    LEAF_C,
    LITTER_CARBON,
    MICROBIAL_CARBON,
    ROOT_C,
    SOIL_WATER,
    STEM_C,
    STORAGE_C,
    WATER_VAPOR,
)
from domains.crew.loader import load_crew_params
from domains.crew.stocks import FECAL_WASTE, FOOD_STORE
from domains.eclss.loader import load_eclss_params
from simcore.conservation import compute_ledger
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State
from station.driver import run_master_day
from station.harvest import (
    build_harvest,
    harvest_bio_resolver,
    harvest_cabin_resolver,
    run_harvest,
)
from station.loader import load_harvest_params
from station.scenario import HARVEST_SCENARIO

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_HARVEST = load_harvest_params()
_SC = HARVEST_SCENARIO

_MASS_QUANTITIES = (Quantity.CARBON, Quantity.OXYGEN, Quantity.WATER)
# The two-way identity subtracts two ~1580-mol food stores accumulated over ~10080
# sub-steps, so the catastrophic-cancellation floor is ~1.8e-9 (measured); the ~1.2e-2
# mol harvest signal sits 7 orders above it.
_IDENTITY_ABS_TOL = 1e-7


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(
    *, with_harvest: bool = True, close_feces: bool = True
) -> tuple[list[State], int, tuple[object, ...]]:
    state, bio_reg, cabin_reg = build_harvest(
        _CREW,
        _ECLSS,
        _HARVEST,
        _SC,
        with_harvest=with_harvest,
        close_feces=close_feces,
    )
    return run_harvest(
        EulerIntegrator(bio_reg),
        EulerIntegrator(cabin_reg),
        state,
        harvest_bio_resolver(_weather(), _SC),
        harvest_cabin_resolver(_SC),
        _SC,
    )


def _amt(state: State, sid: StockId) -> float:
    return state.stocks[sid].amount


def test_rationed_zero_and_event_free() -> None:
    # Well-fed both with AND without harvest (the two-way identity needs both arms
    # un-rationed so the crew draw cancels cleanly). Harvest positivity is structural
    # (k·dt < 1, donor-controlled). No POPULATION stock ⇒ no extinction events; the
    # plant is past anthesis but short of maturity over the horizon.
    for with_harvest in (True, False):
        _, rationed, events = _run(with_harvest=with_harvest)
        assert rationed == 0, (
            f"harvest run must be well-fed (with_harvest={with_harvest})"
        )
        assert events == (), (
            f"harvest run must be event-free (with_harvest={with_harvest})"
        )


def test_every_day_boundary_conserves() -> None:
    # The payload: over each master day every conserved mass quantity balances across
    # the combined biosphere+cabin ledger. Harvest moves CARBON storage_c → food_store
    # (both {C:1}); with close_feces the feces → litter re-point also feeds active
    # microbial respiration, so OXYGEN closure here is non-trivial (microbes draw cabin
    # O₂ to decompose the feces), not merely inherited from the greenhouse.
    states, _, _ = _run()
    for before, after in zip(states, states[1:], strict=False):
        ledger = {ql.quantity: ql for ql in compute_ledger(before, after)}
        for quantity in _MASS_QUANTITIES:
            assert abs(ledger[quantity].residual) <= 1e-6, (
                f"{quantity} must close across each harvest day (residual "
                f"{ledger[quantity].residual:.2e})"
            )


def test_harvest_moves_carbon() -> None:
    # The signed "it bit" gate: WITH harvest the crew food_store ends HIGHER
    # (regenerated by grain) and the biosphere storage_c ends LOWER (drained) than the
    # no-harvest baseline. An un-biting run (k = 0 / grain never fills) flips these.
    with_p = _run(with_harvest=True)[0][-1]
    no_p = _run(with_harvest=False)[0][-1]
    assert _amt(with_p, FOOD_STORE) > _amt(no_p, FOOD_STORE), (
        "with harvest the crew food_store must be regenerated (higher than baseline)"
    )
    assert _amt(with_p, STORAGE_C) < _amt(no_p, STORAGE_C), (
        "with harvest the grain storage_c must be drained (lower than baseline)"
    )


def test_two_way_conservation_identity() -> None:
    # The CARBON conservation identity (the Step-3/Step-4 discipline): the food the crew
    # gained EQUALS the grain the harvest removed — to the cancellation tolerance.
    # Because grain fill is identical with/without harvest (Allocation's FO·DMI is
    # independent of storage_c's level, and harvest is its only new sink), Δfood_store
    # == Δstorage_c exactly.
    with_p = _run(with_harvest=True)[0][-1]
    no_p = _run(with_harvest=False)[0][-1]
    d_food = _amt(with_p, FOOD_STORE) - _amt(no_p, FOOD_STORE)  # cumulative harvest
    d_storage = _amt(no_p, STORAGE_C) - _amt(with_p, STORAGE_C)  # cumulative harvest
    assert d_food > 0.0, "the harvest must move a positive amount of carbon"
    assert abs(d_food - d_storage) <= _IDENTITY_ABS_TOL, (
        f"food gained {d_food:.6e} must equal grain removed {d_storage:.6e} (harvest "
        f"is a pure CARBON transfer)"
    )


def test_grain_is_a_regenerative_source() -> None:
    # The grain is a REGENERATIVE source being drained, not a static reservoir emptied:
    # storage_c starts at 0 (storage_c0 = 0), fills post-anthesis, and — drained ~58
    # %/day while refilled once/day — settles to a POSITIVE quasi-steady at every day
    # boundary after the first fill. (The day-boundary snapshot is the intra-day
    # minimum, post the 1440 harvest sub-steps — the driver is slow-first.)
    states, _, _ = _run()
    day_boundary_grain = [_amt(s, STORAGE_C) for s in states[1:]]
    assert all(g > 0.0 for g in day_boundary_grain), (
        f"grain must stay positive through the run (a drained regenerative source), "
        f"got {day_boundary_grain}"
    )
    assert _amt(states[0], STORAGE_C) == 0.0, (
        "the plant starts with no grain (fills it)"
    )


def test_plant_persists() -> None:
    # The reproductive plant persists over the horizon: its vegetative organs stay
    # positive (no organ collapses to the extinction floor — consistent with events ==
    # ()).
    states, _, _ = _run()
    for organ in (LEAF_C, STEM_C, ROOT_C):
        assert _amt(states[-1], organ) > 0.0, (
            f"{organ} must persist over the harvest run"
        )


def test_biosphere_internal_water_loop_closed() -> None:
    # Step 6 couples only CARBON (grain → food); it does NOT touch the biosphere's
    # internal water ring, which stays independently closed (its total conserved to
    # round-off), exactly as under the greenhouse (Step 3) — station WATER = (closed
    # ring) + (crew loop).
    states, _, _ = _run()
    loop = (SOIL_WATER, WATER_VAPOR, CONDENSATE)
    total0 = sum(_amt(states[0], s) for s in loop)
    totalf = sum(_amt(states[-1], s) for s in loop)
    assert abs(totalf - total0) <= 1e-9, (
        f"biosphere internal water loop must stay closed (drift {totalf - total0:.2e})"
    )


def test_feces_closes_into_litter() -> None:
    # Seam 2 "it bit": with close_feces the crew's fecal carbon is routed into the
    # biosphere LITTER_CARBON pool (closing the trophic ring), so litter grows massively
    # vs the open (close_feces=False) baseline — and the litter feeds
    # MicrobialRespiration (microbial biomass grows too). The ~3400x crew-vs-seedling
    # mismatch makes this a dominating perturbation (measured ~342 mol litter vs ~0.01
    # open), not a subtle one.
    closed = _run(close_feces=True)[0][-1]
    open_ = _run(close_feces=False)[0][-1]
    assert _amt(closed, LITTER_CARBON) > _amt(open_, LITTER_CARBON) + 1.0, (
        "closing feces must route crew carbon into litter (litter grows materially)"
    )
    assert _amt(closed, MICROBIAL_CARBON) > _amt(open_, MICROBIAL_CARBON), (
        "the feces-fed litter must feed microbial respiration (active, not throttled)"
    )


def test_no_shadow_fecal_sink() -> None:
    # The re-point is structural (the Step-1 no-shadow-sink property): with close_feces
    # the orphaned FECAL_WASTE boundary sink is ABSENT from the assembled state (feces
    # lands in LITTER_CARBON instead); the open baseline keeps it (and it fills).
    closed, _, _ = build_harvest(_CREW, _ECLSS, _HARVEST, _SC, close_feces=True)
    open_, _, _ = build_harvest(_CREW, _ECLSS, _HARVEST, _SC, close_feces=False)
    assert FECAL_WASTE not in closed.stocks, (
        "closing feces must omit the orphaned FECAL_WASTE sink (no shadow sink)"
    )
    assert FECAL_WASTE in open_.stocks, "the open baseline keeps the FECAL_WASTE sink"


def test_feces_routing_perturbs_food_and_grain_only_at_roundoff() -> None:
    # The two seams are near-orthogonal: routing feces into litter feeds microbial
    # respiration, whose CO₂ enters the shared CARBON_POOL the plant reads for Ci — but
    # the ECLSS scrubber holds CARBON_POOL at its setpoint, so the plant's carbon budget
    # (grain fill → food_store) is unchanged except at the fp-round-off level (the
    # Step-3 "regulators erase the pool perturbation" physics). Not bit-identical, but
    # agreeing to ~1e-12 relative — the increments compose without material
    # interference.
    closed = _run(close_feces=True)[0][-1]
    open_ = _run(close_feces=False)[0][-1]
    assert _amt(closed, FOOD_STORE) == pytest.approx(_amt(open_, FOOD_STORE), rel=1e-12)
    assert _amt(closed, STORAGE_C) == pytest.approx(_amt(open_, STORAGE_C), rel=1e-9)


def test_determinism() -> None:
    # Bit-identical re-run (decision #7): the harvest run is deterministic.
    assert _run()[0][-1] == _run()[0][-1]


def test_registration_order_independence() -> None:
    # Canonical (flow-id) ordering (#7/#15): shuffling the cabin registry's flow list
    # yields a bit-identical final State. The Registry sorts by FlowId, so the order the
    # Harvest flow is appended in (or any flow) cannot change the trajectory.
    state, bio_reg, cabin_reg = build_harvest(_CREW, _ECLSS, _HARVEST, _SC)
    shuffled = Registry(list(reversed(cabin_reg.flows)), state.stocks)

    def _final(cabin: Registry) -> State:
        states, _, _ = run_master_day(
            EulerIntegrator(bio_reg),
            EulerIntegrator(cabin),
            state,
            harvest_bio_resolver(_weather(), _SC),
            harvest_cabin_resolver(_SC),
            days=_SC.greenhouse.days,
            steps_per_day=_SC.greenhouse.steps_per_day,
            slow_dt=_SC.greenhouse.bio_dt,
            fast_dt=_SC.greenhouse.cabin_dt,
        )
        return states[-1]

    assert _final(cabin_reg) == _final(shuffled)
