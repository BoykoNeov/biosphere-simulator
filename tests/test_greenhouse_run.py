"""Validation: the biosphere ↔ cabin greenhouse (P6.3) — plants offload life support.

Step 3's payload run. The frozen sealed biosphere breathes the crew's cabin air
(its ``CARBON_POOL`` / ``O2_POOL`` ARE the shared cabin gas — the reverse seam,
see ``station.greenhouse``), stepped by the two-rate master-step driver (biosphere
once/day at ``dt=1``, cabin ``substep`` ×1440/day at ``dt=60`` s). The non-vacuous
demonstration:

* **Conservation.** Every quantity (CARBON / OXYGEN / WATER / NITROGEN) balances
  every sub-step over the combined ledger (the driver asserts it after each cabin
  sub-step and inside each biosphere ``step_report``; a completed run *is* the
  proof, and :func:`test_every_day_boundary_conserves` re-checks it
  independently).
* **The plant is a net CO₂ sink / O₂ source** — but the fast ECLSS scrubber
  (τ≈1000 s) relaxes ``CARBON_POOL`` back to ``P/k_scrub`` between the once-daily
  biosphere lumps, so the plant's effect is **erased from the regulated pool at
  every day boundary** and **conserved into** (a) biosphere biomass and (b)
  reduced ECLSS work. The gate is therefore the **offload conservation identity**:
  with plants the plant fixes net carbon, the scrubber removes LESS CO₂, the
  makeup supplies LESS O₂, and the three agree to tolerance (``Δco2_removed ≈
  bio_gain ≈ Δo2_supply``, RQ = 1). The booleans carry the sign robustly (gap
  ~0.06 mol ≫ the ~1e-10 catastrophic-cancellation floor); an un-biting
  (net-source) run would flip them.
* **Live per-step coupling** — right after the biosphere step ``CARBON_POOL`` dips
  below the regulated setpoint (the plant draws *live* cabin CO₂), even though the
  scrubber restores it by day-end.
* **The biosphere's internal water + N loops still close** (Step 3 does not couple
  them to the cabin — the water ring stays independently closed, its unification with
  the cabin humidity deferred to the sealed-station step; N couples in Step 6).

The biosphere is **Euler-locked by its freeze**, so the greenhouse is an Euler run
(no RK4 cross-check — the frozen biosphere's numerics are fixed at ``dt=1``
Euler).
"""

import json
from pathlib import Path

from domains.biosphere.stocks import (
    CARBON_POOL,
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
from domains.crew.stocks import FOOD_STORE, WATER_STORE
from domains.eclss.loader import load_eclss_params
from domains.eclss.stocks import CO2_REMOVED, O2_SUPPLY
from simcore.conservation import compute_ledger
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator
from simcore.state import State
from station.greenhouse import (
    build_greenhouse,
    greenhouse_bio_resolver,
    greenhouse_cabin_resolver,
    run_greenhouse,
)
from station.scenario import GREENHOUSE_SCENARIO

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_CREW = load_crew_params()
_ECLSS = load_eclss_params()
_SC = GREENHOUSE_SCENARIO

# The biosphere organic-carbon pools (the plant's cumulative sink lives here).
_BIO_C = (LEAF_C, STEM_C, ROOT_C, STORAGE_C, LITTER_CARBON, MICROBIAL_CARBON)
# The offload identity's catastrophic-cancellation floor (co2_removed ~2056 mol over
# ~10080 additions ⇒ ~1e-10 absolute round-off); the 0.06 mol signal sits far above
# it.
_IDENTITY_ABS_TOL = 1e-8


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(*, with_plants: bool = True) -> tuple[list[State], int, tuple[object, ...]]:
    state, bio_reg, cabin_reg = build_greenhouse(
        _CREW, _ECLSS, _SC, with_plants=with_plants
    )
    return run_greenhouse(
        EulerIntegrator(bio_reg),
        EulerIntegrator(cabin_reg),
        state,
        greenhouse_bio_resolver(_weather(), _SC),
        greenhouse_cabin_resolver(_SC),
        _SC,
    )


def _amt(state: State, sid: StockId) -> float:
    return state.stocks[sid].amount


def _bio_organic_c(state: State) -> float:
    return sum(_amt(state, s) for s in _BIO_C)


def test_rationed_zero_and_event_free() -> None:
    # Well-fed both with AND without plants (the identity needs both stores
    # un-rationed so the crew terms cancel cleanly). No POPULATION goes extinct ⇒ no
    # events.
    for with_plants in (True, False):
        _, rationed, events = _run(with_plants=with_plants)
        assert rationed == 0, f"greenhouse must be well-fed (with_plants={with_plants})"
        assert events == (), (
            f"greenhouse must be event-free (with_plants={with_plants})"
        )


def test_every_day_boundary_conserves() -> None:
    # Independent re-check of the driver's per-sub-step gate: over each master day
    # every conserved quantity balances across the combined biosphere+cabin ledger.
    states, _, _ = _run()
    for before, after in zip(states, states[1:], strict=False):
        for ql in compute_ledger(before, after):
            assert abs(ql.residual) <= 1e-6, (
                f"{ql.quantity} must close across each greenhouse day (residual "
                f"{ql.residual:.2e})"
            )


def test_plant_is_net_carbon_sink() -> None:
    # The single-run sign check (full precision): the growing seedling fixes net
    # carbon — biosphere organic carbon increases over the growth-phase window (DVS
    # 0 start).
    states, _, _ = _run()
    gained = _bio_organic_c(states[-1]) - _bio_organic_c(states[0])
    assert gained > 0.0, "the greenhouse plant must fix net carbon (a net CO₂ sink)"


def test_plant_offloads_life_support() -> None:
    # The cross-domain feedback, as cancellation-proof booleans: with plants the
    # scrubber removes LESS CO₂ and the makeup supplies LESS O₂ (the plant offloads
    # the ECLSS).
    with_p = _run(with_plants=True)[0][-1]
    no_p = _run(with_plants=False)[0][-1]
    assert _amt(with_p, CO2_REMOVED) < _amt(no_p, CO2_REMOVED), (
        "with plants the scrubber must remove less CO₂ (bioregenerative offload)"
    )
    assert _amt(with_p, O2_SUPPLY) > _amt(no_p, O2_SUPPLY), (
        "with plants the makeup must supply less O₂ (o2_supply less depleted)"
    )


def test_offload_conservation_identity() -> None:
    # The three-way identity (RQ = 1): the plant's net carbon fixation EQUALS the
    # CO₂ the scrubber did not remove AND the O₂ the makeup did not supply — to the
    # cancellation tolerance, the physical statement that the plant's fixation is
    # exactly the offload.
    with_p = _run(with_plants=True)[0][-1]
    no_p = _run(with_plants=False)[0][-1]
    bio_gain = _bio_organic_c(with_p) - _bio_organic_c(no_p)
    co2_offloaded = _amt(no_p, CO2_REMOVED) - _amt(with_p, CO2_REMOVED)
    o2_offloaded = _amt(with_p, O2_SUPPLY) - _amt(no_p, O2_SUPPLY)
    assert abs(co2_offloaded - bio_gain) <= _IDENTITY_ABS_TOL, (
        f"scrubber offload {co2_offloaded:.6e} must equal net fixation {bio_gain:.6e}"
    )
    assert abs(o2_offloaded - bio_gain) <= _IDENTITY_ABS_TOL, (
        f"makeup offload {o2_offloaded:.6e} must equal net fixation {bio_gain:.6e}"
    )


def test_intraday_cabin_co2_dip() -> None:
    # Live per-step coupling: the once-daily biosphere step draws CO₂ straight out
    # of the live cabin pool, so CARBON_POOL right after it is below the
    # day-boundary setpoint (which the scrubber later restores). Proof the plant
    # reads/writes cabin gas, not a detached pool.
    state, bio_reg, cabin_reg = build_greenhouse(_CREW, _ECLSS, _SC, with_plants=True)
    before = _amt(state, CARBON_POOL)
    after = (
        EulerIntegrator(bio_reg)
        .step_report(state, greenhouse_bio_resolver(_weather(), _SC), _SC.bio_dt)
        .state
    )
    assert _amt(after, CARBON_POOL) < before, (
        "the biosphere step must draw CO₂ from the live cabin pool (an intra-day dip)"
    )


def test_biosphere_internal_water_loop_closed() -> None:
    # Step 3 does NOT couple the biosphere water cycle to the cabin, so the internal
    # ring soil_water → water_vapor → condensate → soil_water stays closed: its total
    # is conserved to round-off across the whole run. Step 4 (the crew water-recovery
    # loop) does NOT change this: it closes the CREW water independently (humidity +
    # urine → recovery → water_store), while the biosphere ring stays independently
    # closed here — station WATER conserves as (closed ring) + (crew loop). Unifying
    # the two humid-air stocks (biosphere transpiration ⇄ cabin humidity) is a fidelity
    # refinement deferred to the sealed-station step, NOT a closure requirement.
    states, _, _ = _run()
    loop = (SOIL_WATER, WATER_VAPOR, CONDENSATE)
    total0 = sum(_amt(states[0], s) for s in loop)
    totalf = sum(_amt(states[-1], s) for s in loop)
    assert abs(totalf - total0) <= 1e-9, (
        f"biosphere internal water loop must stay closed (drift {totalf - total0:.2e})"
    )


def test_stores_run_down_but_well_fed() -> None:
    # The crew stores are forced open-loop draws (the argument for Steps 4/6): they
    # deplete materially but stay well above 0 over the horizon (well-fed sizing,
    # rationed == 0).
    states, _, _ = _run()
    for store, initial in (
        (FOOD_STORE, _SC.cabin.food_store0),
        (WATER_STORE, _SC.cabin.water_store0),
    ):
        final = _amt(states[-1], store)
        assert 0.0 < final < initial, f"{store} must deplete but stay positive"
        assert final < 0.9 * initial, (
            f"{store} must deplete materially (not a flat line)"
        )


def test_determinism() -> None:
    # Bit-identical re-run (decision #7): the greenhouse is deterministic.
    assert _run()[0][-1] == _run()[0][-1]
