"""Pins for the crew-coupled loop diagnosis (post-roadmap, 2026-08-10).

`docs/plans/post-roadmap-crew-coupled-loop.md`. Read-only assertions: no fixture, no
unfreeze, no golden moved. They pin the measurements that document rests on, so that a
future change either keeps them true or goes red.

The expensive ones (a coupled season is ~440k sub-steps, ~90 s) are `slow`. They are
deliberately grouped into as few functions as possible: `sealed_tier2_run`'s
session-scoped cache is per *worker* under xdist, and these cannot use it anyway (they
need their own scenarios), so each extra slow function is another full recomputation --
the lesson `docs/test-suite-runtime.md` and the acceptance-gate diagnosis both record.
"""

from dataclasses import replace
from itertools import pairwise

import pytest

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import load_canopy_params
from domains.biosphere.scenario import (
    DEFAULT_SCENARIO,
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    SeasonScenario,
)
from domains.biosphere.season import build_season, run_season, weather_resolver
from domains.biosphere.step import BIO_DT, steps_for
from domains.biosphere.stocks import CARBON_POOL, LEAF_C, O2_POOL, STORAGE_C
from domains.crew.loader import load_crew_params
from domains.eclss.loader import load_eclss_params
from domains.power.loader import load_charge_params
from domains.thermal.loader import load_thermal_params
from sealed_tier2_helper import total_organic_c, weather
from simcore import arbitration
from simcore.conservation import assert_conserved
from simcore.ids import StockId
from simcore.integrator import EulerIntegrator
from simcore.state import State
from station.driver import run_master_day
from station.lighting import lamp_par
from station.loader import (
    load_harvest_params,
    load_lamp_params,
    load_water_recovery_params,
)
from station.scenario import (
    DEFAULT_LIGHTING_SCENARIO,
    GREENHOUSE_BIO_SCENARIO,
    SEALED_STATION_SCENARIO,
    SealedStationScenario,
)
from station.sealed import (
    build_sealed_station,
    sealed_bio_resolver,
    sealed_fast_resolver,
    sealed_reset,
)

# --- the external references (both first-hand, both already in the repo) -------------
#
# [BVAD] NASA/TP-2015-218570 Rev 2 (public domain; NTRS 20210024855).
#   Table 3-31, via `docs/bvad-reference.md`: CO2 load 1.085 kg/CM-d / 44.009
#     = 24.654 mol C per crewmember-day.
#   Table 4-91 p. 173, wheat CO2 uptake 77.00 g/m2-d / 44.009 = 1.7496 mol C/m2-d.
#     Imported as the arithmetic, not re-derived -- `test_chamber_scale.py` owns the
#     rotated-table hazard note for this row (a naive `pdftotext -layout` read files
#     RICE's numbers under the name Wheat).
BVAD_CREW_C_PER_DAY: float = 24.654
BVAD_WHEAT_C_PER_M2_DAY: float = 77.00 / 44.009

# `open_season`, the only frozen scenario at field scale -- the pinned peaks the coupled
# chamber is compared against (`test_senescence_form.py` / `test_nitrogen_form.py`).
# ⚠ 5.191 -> 5.462364 on 2026-08-12 (the stem-reserve build grows the open-field
# crop). Re-measured rather than left stale: this constant is the DENOMINATOR of the
# "still field-scale" ratio below, so a stale value would silently flatter the
# comparison — the ratio would read 1.01x against a crop that no longer exists.
#
# ⚠⚠ **AND IT WENT STALE ANYWAY, 2026-08-14 — the comment above names the hazard
# exactly and did not prevent it.** The step unfreeze moved the open field to
# **5.571922** and nothing here noticed, because a hand-copied number cannot notice.
# The comment was the whole defence, and a warning is not a check.
#
# So the duplication is now SELF-CHECKING: PIN 6 re-runs `open_season` and asserts this
# constant against it, which costs one extra ~2 s season and turns the next drift red
# instead of silently flattering the ratio. *Re-measuring a copied constant fixes one
# occurrence; tying it to its source fixes the class.*
OPEN_SEASON_PEAK_LAI: float = 5.571922
OPEN_SEASON_PEAK_W_EXCL_ROOTS: float = 12.633

# The V-K&S mutual-shading threshold the canopy regulator is built on (Penning de Vries
# 1989 p. 101, quoting Van Keulen & Seligman 1987).
VKS_LAI_THRESHOLD: float = 6.0


def _crew_carbon_per_day(scenario: SealedStationScenario) -> float:
    """The scenario's own crew CO2 production (mol C/d) = f_resp * intake * 86400."""
    return (
        load_crew_params().respired_carbon_fraction
        * scenario.cabin.food_intake_rate
        * 86400.0
    )


def _run_station(
    scenario: SealedStationScenario, *, days: int, with_harvest: bool
) -> tuple[list[State], int, tuple[object, ...]]:
    """Drive the sealed station THE WAY ITS OWN GOLDEN DRIVES IT (`run_master_day` +
    the annual re-sow hook), for `days` master days."""
    charge = load_charge_params()
    lamp = load_lamp_params()
    state, bio_reg, fast_reg = build_sealed_station(
        charge,
        load_thermal_params(),
        load_crew_params(),
        load_eclss_params(),
        load_water_recovery_params(),
        lamp,
        load_harvest_params(),
        scenario,
        with_harvest=with_harvest,
        close_feces=False,
    )
    return run_master_day(
        EulerIntegrator(bio_reg),
        EulerIntegrator(fast_reg),
        state,
        sealed_bio_resolver(weather(scenario.years), lamp, scenario),
        sealed_fast_resolver(charge, scenario),
        days=days,
        steps_per_day=scenario.steps_per_day,
        slow_dt=scenario.bio_dt,
        fast_dt=scenario.cabin_dt,
        slow_steps_per_day=scenario.bio_steps_per_day,
        slow_reset=sealed_reset(scenario),
    )


def _open_season_peak_lai() -> float:
    """``open_season``'s peak LAI, MEASURED — the denominator PIN 6 compares against.

    It is a transcribed constant a few lines up (``OPEN_SEASON_PEAK_LAI``) because the
    pin that owns it (``test_senescence_form.py``) states a BAND, ``5.0 < peak < 8.0``,
    so there is no exact value to import. A band cannot keep a copy honest, so PIN 6
    measures it and asserts the copy against the measurement instead.
    """
    w = weather(1)
    state, registry = build_season(DEFAULT_SCENARIO)
    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(w, DEFAULT_SCENARIO),
        BIO_DT,
        steps_for(len(w)),
    )
    assert rationed == 0
    return _peak_lai(states, DEFAULT_SCENARIO.ground_area)


def _peak_lai(states: list[State], ground_area: float) -> float:
    sla = load_canopy_params().sla_per_mol_c
    return max(
        leaf_area_index(
            s.stocks[LEAF_C].amount, sla_per_mol_c=sla, ground_area=ground_area
        )
        for s in states
    )


# --- 1. the atmosphere is not where the 1137x lives ----------------------------------


def test_the_coupled_cabins_carbon_buffer_is_HOURS_not_orders() -> None:
    """PIN 1. The seam paragraph offered this route for "~1,137x" more carbon. True --
    and all of it is in `crew.food_store`, a pool the plant cannot breathe.

    The *atmosphere* is 3.70 h of one BVAD crewmember, against the chamber-scale
    census's 3.4 h for the frozen jar's ENTIRE inventory: 1.09x, not three orders.
    """
    co2 = GREENHOUSE_BIO_SCENARIO.chamber_co2_mol0
    assert co2 == 3.796
    hours_vs_one_cm = co2 / BVAD_CREW_C_PER_DAY * 24.0
    assert 3.6 < hours_vs_one_cm < 3.8, hours_vs_one_cm

    # ...and against the crew the scenario actually carries, it is MINUTES.
    minutes = co2 / _crew_carbon_per_day(SEALED_STATION_SCENARIO) * 24.0 * 60.0
    assert 16.0 < minutes < 17.5, minutes

    # The sealed station reuses the greenhouse gas fill verbatim (only litter differs),
    # so this is one number covering both, asserted rather than assumed.
    assert SEALED_STATION_SCENARIO.bio.chamber_co2_mol0 == co2
    assert SEALED_STATION_SCENARIO.bio.chamber_air_mol == (
        GREENHOUSE_BIO_SCENARIO.chamber_air_mol
    )


def test_the_crew_is_187x_oversized_against_BVADs_OWN_area_arithmetic() -> None:
    """PIN 2. Two BVAD numbers and nothing else: the assembly's crew needs 187.45 m2
    of wheat and has 1.00 m2.

    Kept deliberately apart from the food-return ratio (PIN 6): different denominators.
    """
    crew_cm = _crew_carbon_per_day(SEALED_STATION_SCENARIO) / BVAD_CREW_C_PER_DAY
    assert 13.29 < crew_cm < 13.32, crew_cm

    m2_per_cm = BVAD_CREW_C_PER_DAY / BVAD_WHEAT_C_PER_M2_DAY
    assert 14.08 < m2_per_cm < 14.10, m2_per_cm

    assert SEALED_STATION_SCENARIO.bio.ground_area == 1.0
    oversize = crew_cm * m2_per_cm / SEALED_STATION_SCENARIO.bio.ground_area
    assert 187.0 < oversize < 188.0, oversize


# --- 2. area-scaling is an exact similarity transform ---------------------------------

_PLANT_SOIL_ICS = (
    "leaf_c0",
    "stem_c0",
    "root_c0",
    "litter_carbon0",
    "soil_water0",
    # The below-root store is EXTENSIVE (kg over the plot), so it scales with area like
    # its sibling. This pin caught its omission the first time it was added — which is
    # what the pin is for: `subsoil_water0`'s default is derived from `ground_area`, and
    # a scenario that scales the area without scaling the store is not the same soil.
    "subsoil_water0",
    "soil_n0",
    "plant_n0",
)
_CABIN_GAS_ICS = ("chamber_air_mol", "chamber_co2_mol0", "chamber_o2_mol0")


def _scaled_chamber(base: SeasonScenario, a: float) -> SeasonScenario:
    """A standalone chamber scaled by `a`: area + every extensive IC, gas pools too
    (a standalone chamber's own jar grows with its crop -- in the STATION the cabin
    deliberately does not, and that asymmetry is the coupling)."""
    fields = _PLANT_SOIL_ICS + _CABIN_GAS_ICS
    return replace(
        base,
        ground_area=base.ground_area * a,
        **{f: getattr(base, f) * a for f in fields},
    )


@pytest.mark.slow
def test_area_scaling_is_an_EXACT_similarity_transform() -> None:
    """PIN 3. The licensing step for using `ground_area` as the sizing lever.

    Scaling plant STOCKS by A would model one enormous plant (light interception
    saturates in LAI); scaling AREA preserves LAI and models A replicated 1-m2 canopies
    sharing one atmosphere. That is a claim about the model, so it is measured: every
    stock at every step is exactly A x the base run, to float round-off.
    """
    a = 187.4523
    base = SEALED_CHAMBER_SCENARIO
    scaled = _scaled_chamber(base, a)
    assert scaled.ground_area == pytest.approx(a)

    def drive(scen: SeasonScenario) -> list[State]:
        w = weather(SEALED_CHAMBER_YEARS)
        state, registry = build_season(scen)
        states, rationed, _ = run_season(
            EulerIntegrator(registry),
            state,
            weather_resolver(w, scen),
            BIO_DT,
            steps_for(len(w)),
        )
        assert rationed == 0
        return states

    base_states, scaled_states = drive(base), drive(scaled)
    assert len(base_states) == len(scaled_states)

    worst = 0.0
    for s1, sa in zip(base_states, scaled_states, strict=True):
        for sid, st in s1.stocks.items():
            want = st.amount * a
            got = sa.stocks[sid].amount
            worst = max(worst, abs(got - want) / (abs(want) or 1.0))
    assert worst < 1e-13, f"area scaling is not a similarity transform: {worst:.3e}"

    sla = load_canopy_params().sla_per_mol_c
    worst_lai = max(
        abs(
            leaf_area_index(
                s1.stocks[LEAF_C].amount,
                sla_per_mol_c=sla,
                ground_area=base.ground_area,
            )
            - leaf_area_index(
                sa.stocks[LEAF_C].amount,
                sla_per_mol_c=sla,
                ground_area=scaled.ground_area,
            )
        )
        for s1, sa in zip(base_states, scaled_states, strict=True)
    )
    assert worst_lai < 1e-14, worst_lai


def _area_scaled_station(area: float) -> SealedStationScenario:
    """The sealed station with the growing area scaled to `area`, and everything the
    area drags with it: the plant/soil ICs (PIN 3's similarity transform), the lamp
    (PAR is inverse in the area, PIN 4) and the microgrid (a bigger lamp draws more).

    The CABIN gas pools deliberately do NOT scale -- the cabin is the crew's, and that
    asymmetry is the coupling under test.
    """
    bio0 = SEALED_STATION_SCENARIO.bio
    a = area / bio0.ground_area
    bio = replace(
        bio0,
        ground_area=area,
        **{f: getattr(bio0, f) * a for f in _PLANT_SOIL_ICS},
    )
    pw0 = SEALED_STATION_SCENARIO.power
    return replace(
        SEALED_STATION_SCENARIO,
        years=1,
        bio=bio,
        lamp_power_w=SEALED_STATION_SCENARIO.lamp_power_w * a,
        power=replace(
            pw0, battery0=pw0.battery0 * a, solar_peak_w=pw0.solar_peak_w * a
        ),
        battery0=SEALED_STATION_SCENARIO.battery0 * a,
    )


# --- 3. the light is inverse in the area (why the BVAD-sized plot goes dark) ----------


def test_PAR_is_INVERSE_in_the_growing_area_so_the_lamp_must_scale_with_it() -> None:
    """PIN 4. BVAD's 14.091 m2/CM closes the GAS loop and says nothing about the LIGHT.

    `lamp_par` divides by `ground_area`, so a BVAD-sized plot under the same 200 W lamp
    is dimmed by exactly the area factor -- the first of the three nested failures in
    the plan's section 8, and the one that is pure algebra.
    """
    lamp = load_lamp_params()
    base = DEFAULT_LIGHTING_SCENARIO
    wide = replace(
        base, bio=replace(base.bio, ground_area=base.bio.ground_area * 187.0)
    )
    assert lamp_par(lamp, base) == pytest.approx(lamp_par(lamp, wide) * 187.0)

    # ...and restoring the lamp with the area restores PAR exactly.
    relit = replace(wide, lamp_power_w=base.lamp_power_w * 187.0)
    assert lamp_par(lamp, relit) == pytest.approx(lamp_par(lamp, base))


def test_the_ECLSS_regulator_holds_MOLES_not_a_CONCENTRATION() -> None:
    """PIN 5. Why 'just give the cabin more air' is not the fix (plan section 8c).

    The scrubber removes at `k_scrub * pool`, so the equilibrium `P / k_scrub` is an
    AMOUNT and is independent of the air volume -- but `ci` is a mole fraction, so
    scaling the air at a fixed equilibrium amount DILUTES the plant's CO2.
    """
    eclss = load_eclss_params()
    production = _crew_carbon_per_day(SEALED_STATION_SCENARIO) / 86400.0  # mol/s
    equilibrium = production / eclss.co2_scrub_rate
    # The scenario's fill IS that equilibrium -- which is why carbon_pool never moves.
    assert equilibrium == pytest.approx(
        SEALED_STATION_SCENARIO.bio.chamber_co2_mol0, rel=1e-3
    )
    # The equilibrium carries no air term, so 187x the air is 187x less concentration.
    air = SEALED_STATION_SCENARIO.bio.chamber_air_mol
    assert equilibrium / (air * 187.0) == pytest.approx(equilibrium / air / 187.0)


# --- 4. the coupled season: field-scale plant, ~0 % closure, and the per-day ceiling --


@pytest.mark.slow
def test_the_coupled_season_is_field_scale_open_loop_and_carbon_capped() -> None:
    """PINS 6-9, in ONE function because each is a ~90 s coupled season.

    Merged deliberately: `sealed_tier2_run`'s cache is per-worker under xdist and does
    not fit these scenarios anyway, so every extra slow function is another full
    recomputation. Each claim carries its own assertion message.
    """
    scenario = replace(SEALED_STATION_SCENARIO, years=1)
    days = scenario.season_days
    states, rationed, events = _run_station(scenario, days=days, with_harvest=False)
    assert rationed == 0 and events == ()

    # PIN 6: the plant is FIELD-SCALE -- ~0.91x open_season's peak LAI, in a SEALED
    # chamber. Every prior chamber measurement in this repo is of the STANDALONE
    # chambers (52-70 g DM/m2, LAI 0.51-0.63).
    peak_lai = _peak_lai(states, scenario.bio.ground_area)
    # ⚠ 5.03-5.05 -> 5.23-5.25 (2026-08-12, stem reserves) -> this (2026-08-14, the
    # step unfreeze): 5.0879, down 2.9 %. The band keeps its width through all three.
    assert 5.08 < peak_lai < 5.10, f"coupled peak LAI moved: {peak_lai}"

    # ⚠ THE DENOMINATOR IS NOW MEASURED HERE, not transcribed. See the constant's own
    # note: it had gone stale under a comment warning that it must not.
    open_peak = _open_season_peak_lai()
    assert open_peak == pytest.approx(OPEN_SEASON_PEAK_LAI, rel=1e-5), (
        "OPEN_SEASON_PEAK_LAI is stale — re-measure it, do not adjust the ratio below",
        open_peak,
    )

    # ⚠⚠ **THE `> 0.95` CUT IS DROPPED, NOT LOWERED.** The ratio went 0.959 -> 0.9131
    # at `dt = ¼`, and 0.95 was a round number chosen when the measurement was 0.97 —
    # lowering it to 0.90 would be the third re-cut of a bound in this batch and the
    # second in this file. PIN 6's claim is not a threshold: it is that this chamber's
    # canopy is a FIELD canopy, and what makes that legible is the two-sided contrast —
    # within ~10 % of the open field, and an ORDER OF MAGNITUDE above every standalone
    # chamber in the repo (LAI 0.51-0.63). Both are asserted, and the ratio is pinned
    # exactly so any movement is visible rather than absorbed by slack.
    ratio = peak_lai / open_peak
    assert ratio == pytest.approx(0.9131, rel=1e-3)
    assert 0.5 < ratio < 1.5, ("no longer the same scale as the open field", ratio)
    assert peak_lai > 8.0 * 0.63, ("no longer above the standalone chambers", peak_lai)

    # PIN 7: the MECHANISM -- `carbon_pool` is a regulated CONSTANT, so `ci` is
    # functionally the unclamped supply `open_season` has. Carbon-limited by ISOLATION,
    # not by volume.
    pools = [s.stocks[CARBON_POOL].amount for s in states]
    assert max(pools) - min(pools) < 1e-9, (
        "carbon_pool is no longer a regulated constant: spread "
        f"{max(pools) - min(pools)}"
    )
    assert pools[0] == pytest.approx(3.796)

    # PIN 8: the canopy regulator's threshold is NEARLY reached here. Its docstring's
    # "every chamber between 0.068 and 0.632, i.e. 9-88x below it" is true of its own
    # SIX-scenario biosphere roster; the COUPLED chamber sits at 0.84x. A scope
    # finding, not a falsification -- pinned so the scope cannot be lost.
    assert 0.80 < peak_lai / VKS_LAI_THRESHOLD < 0.88, (
        "the coupled chamber's distance to the mutual-shading threshold moved"
    )

    # PIN 9: the per-day CEILING. The biosphere is the SLOW registry -- one Euler step
    # per master day, drawing from the STANDING pool, while the crew's production is
    # delivered by the fast registry afterwards. So the crop's daily carbon is capped by
    # the pool however much area is added.
    organic = [total_organic_c(s) for s in states]
    peak_daily_gain = max(b - a for a, b in pairwise(organic))
    # ⚠ 0.59-0.61 -> 0.62-0.64 (2026-08-12, stem reserves) -> 0.6018 (2026-08-14, the
    # step unfreeze), -4.4 %. ⚠ The BAND IS REPLACED BY AN EXACT PIN rather than
    # re-centred a third time: a ±0.01 window around a measurement is not a claim about
    # anything, it is the measurement with slack, and the slack is what has had to be
    # moved twice. An exact pin catches every change a band would and every change it
    # would not. PIN 9's actual claim — the pool is small next to the daily gain, and
    # smaller still next to the crew's production — is the three assertions below, and
    # none of them moved.
    assert peak_daily_gain == pytest.approx(0.6018, rel=1e-3), peak_daily_gain
    pool = pools[0]
    assert pool / peak_daily_gain > 6.0, "the 1 m2 headroom that hides the ceiling"

    crew_per_day = _crew_carbon_per_day(scenario)
    assert crew_per_day / pool > 80.0, (
        "the crew's PRODUCTION dwarfs the standing pool -- which is exactly the "
        "quantity the once-per-day plant cannot reach"
    )
    # ...and it is exceeded at ONE crewmember's worth of crop, before closure arises.
    one_cm_area = BVAD_CREW_C_PER_DAY / BVAD_WHEAT_C_PER_M2_DAY
    assert peak_daily_gain * one_cm_area / pool > 2.0, (
        "the shared cabin no longer fails to supply one crewmember's worth of crop"
    )


@pytest.mark.slow
def test_the_per_day_ceiling_binds_on_CARBON_POOL_in_the_SLOW_registry() -> None:
    """PIN 11. The mechanism behind PIN 9, ISOLATED rather than argued.

    PIN 9's arithmetic only PREDICTS the cap. The measurement at 187.45 m2 is a single
    `rationed` integer, and `run_master_day` sums the slow and fast reports into it --
    in a run that also has a power bus under load. Reading that sum as "the biosphere
    rationed" is the (C)-branch error this repo logs: a location reported under a
    constant it was never measured into.

    So: drive the two registries by hand (the driver's own order, slow-first) to count
    them apart, and record the binding stock through `arbitration.min_scaling`'s own
    demand accumulation. `o2_pool` is checked BY NAME because it was a live candidate --
    the other cabin gas pool the area scaling leaves alone, drawn by decomposers that
    DID scale with the litter.
    """
    area = (
        _crew_carbon_per_day(SEALED_STATION_SCENARIO)
        / BVAD_CREW_C_PER_DAY
        * (BVAD_CREW_C_PER_DAY / BVAD_WHEAT_C_PER_M2_DAY)
    )
    scenario = _area_scaled_station(area)
    days = scenario.season_days

    charge = load_charge_params()
    lamp = load_lamp_params()
    state, bio_reg, fast_reg = build_sealed_station(
        charge,
        load_thermal_params(),
        load_crew_params(),
        load_eclss_params(),
        load_water_recovery_params(),
        lamp,
        load_harvest_params(),
        scenario,
        with_harvest=True,
        close_feces=False,
    )
    bio_int, fast_int = EulerIntegrator(bio_reg), EulerIntegrator(fast_reg)
    bio_res = sealed_bio_resolver(weather(scenario.years), lamp, scenario)
    fast_res = sealed_fast_resolver(charge, scenario)
    reset = sealed_reset(scenario)

    slow_rationed = fast_rationed = 0
    binding: dict[str, int] = {}
    below_one: dict[str, int] = {}
    worst: dict[str, float] = {}

    def record(results: object, stocks: dict[StockId, object]) -> None:
        """`_scale_factors`' own accumulation, verbatim: withdrawals only, unclamped
        skipped (decision #13)."""
        demand: dict[StockId, float] = {}
        for result in results:  # type: ignore[attr-defined]
            for leg in result.legs:
                if leg.amount < 0.0 and not stocks[leg.stock].unclamped:  # type: ignore[attr-defined]
                    demand[leg.stock] = demand.get(leg.stock, 0.0) - leg.amount
        margins = {
            str(sid): stocks[sid].amount / d  # type: ignore[attr-defined]
            for sid, d in demand.items()
            if d > 0.0
        }
        if not margins:
            return
        arg = min(margins, key=lambda k: margins[k])
        binding[arg] = binding.get(arg, 0) + 1
        for k, m in margins.items():
            worst[k] = min(worst.get(k, float("inf")), m)
            if m < 1.0:
                below_one[k] = below_one.get(k, 0) + 1

    real = arbitration.min_scaling

    def wrapped(results: object, stocks: dict[StockId, object]) -> object:
        record(results, stocks)
        return real(results, stocks)  # type: ignore[arg-type]

    for _day in range(days):
        rs = reset(state.n, state)
        if rs is not state:
            assert_conserved(state, rs)
            state = rs
        # The slow (biosphere) sub-steps, and ONLY they, with their arbitration calls
        # recorded. ⚠ This hand-rolls `run_master_day`'s body so it can instrument the
        # slow side alone, so it must take the SAME number of slow sub-steps the driver
        # does — one per master day was right only while the step was a day, and left
        # this run covering a quarter of its intended span at `dt = ¼`.
        arbitration.min_scaling = wrapped  # type: ignore[assignment]
        try:
            for _ in range(scenario.bio_steps_per_day):
                rep = bio_int.step_report(state, bio_res, scenario.bio_dt)
                state = rep.state
                slow_rationed += rep.rationed
        finally:
            arbitration.min_scaling = real  # type: ignore[assignment]
        for _ in range(scenario.steps_per_day):
            before = state
            frep = fast_int.substep(state, fast_res, scenario.cabin_dt)
            state = frep.state
            assert_conserved(before, state)
            fast_rationed += frep.rationed

    # The ceiling is entirely in the SLOW registry -- the power bus is fine.
    assert slow_rationed > 0, "the per-day ceiling stopped binding"
    assert fast_rationed == 0, (
        f"the fast registry now rations too ({fast_rationed}); the slow-side "
        "attribution of PIN 9 is no longer clean"
    )
    # ...and the stock it binds on is CARBON, on essentially every growing day.
    assert below_one == {str(CARBON_POOL): slow_rationed}, (
        f"the binding stock is no longer carbon_pool alone: {below_one}"
    )
    assert binding[str(CARBON_POOL)] > 0.9 * days
    # ⚠ **A MARGIN IS DENOMINATED IN STEPS, so this bound was a step-size observable
    # wearing the clothes of a scarcity measure.** It read ``< 0.25`` — "demand out-runs
    # supply by at least 4x" — and at ``dt = ¼`` each call's demand is a quarter as big,
    # so the same trajectory reads 0.4615. Nothing about the carbon got easier.
    #
    # Restated in DAYS (``margin · dt``), which is what the sentence was about: at its
    # worst the pool covers **0.115 days** of peak draw. ⚠ That is NOT the old 0.25
    # rescaled (0.4615 x 0.25 = 0.1154 against a pre-flip bound of 0.25 day), because
    # the bound was a ceiling rather than a measurement — so this is a genuine
    # tightening of what is known, not a translation of it.
    #
    # The STRUCTURAL claim — the margin is below 1, i.e. the ceiling binds at all — is
    # asserted separately and is the step-invariant half.
    assert worst[str(CARBON_POOL)] < 1.0, worst[str(CARBON_POOL)]
    assert worst[str(CARBON_POOL)] * scenario.bio_dt < 0.25, worst[str(CARBON_POOL)]
    # o2_pool was a LIVE candidate (unscaled, and drawn by the scaled decomposers).
    assert str(O2_POOL) not in below_one, (
        "o2_pool now binds too -- re-read the mechanism"
    )


@pytest.mark.slow
def test_harvest_clears_the_seed_bank_on_the_FROZEN_horizon() -> None:
    """PIN 10. A recorded scope reason is STALE.

    `station.sealed` and `SealedStationScenario` both give, as the reason
    `with_harvest` defaults off: "harvest drains storage_c to ~0.01 mol by the year
    boundary -- below the 0.16-mol seed bank". Spike-measured at Phase 6 Step 7; the
    decomposer calibration, the N-cycle form changes and the humification split have all
    moved the plant since.

    Scoped to the frozen horizon on purpose: beyond it BOTH configurations collapse
    (plan section 6), so this pins "the reason is stale", NOT "harvest is safe to
    enable" -- which is a contract question about a frozen scenario's defaults.
    """
    scenario = replace(SEALED_STATION_SCENARIO, years=1)
    seedling = scenario.bio.leaf_c0 + scenario.bio.stem_c0 + scenario.bio.root_c0
    assert seedling == pytest.approx(0.16)

    states, rationed, events = _run_station(
        scenario, days=scenario.season_days, with_harvest=True
    )
    assert rationed == 0, "harvest ON rations on the frozen horizon"
    assert events == ()
    boundary_grain = states[-1].stocks[STORAGE_C].amount
    assert boundary_grain > seedling, (
        f"harvest starves the re-sow after all: {boundary_grain} < {seedling}"
    )
    # The margin is 1.54x, not an order -- stated as a number so it cannot be quoted as
    # comfortable. ⚠ 1.32x -> 1.64x -> 1.54x on 2026-08-12 (the stem-reserve build, then
    # its cessation window): the reserve feeds grain, so the seed bank the re-sow draws
    # on is larger, and the window gives a little of that back. The STALENESS finding
    # this test exists for is unaffected -- the recorded "~0.01 mol, below the 0.16 seed
    # bank" reason is now wrong by a wider margin, not a narrower one.
    assert 1.45 < boundary_grain / seedling < 1.65, boundary_grain / seedling
