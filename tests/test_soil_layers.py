"""Pins for the below-root store + root-zone capture (`WSTORG`/`EWAT`).

⚠ **WHAT THE GOLDENS DO AND DO NOT COVER HERE.** Unlike the root-depth gate — which is
bit-identically inert everywhere, so *nothing* in the regression suite sees it — this
mechanism does move goldens: 10 of them carry a `soil_water` shifted by the season's
capture. But a golden only pins the *number at the end of one run*. It cannot see:

* that the capture is clamped to the donor (`min`, [F] Eqn 14.10) rather than allowed to
  overdraw,
* that a dry subsoil stops root extension (`If WSTORG = 0 Then GRTD = 0`),
* that the re-sow returns the abandoned zone's water instead of ratcheting,
* that the capture flow and the depth accumulator use the **same gated rate**,
* or that any of this makes a difference to a crop.

Each of those is pinned below and was **mutation-verified**: the assertion was seen to
fail against a deliberately broken variant before being committed (a passing test proves
nothing until it has been seen to fail).

Design record: `docs/plans/post-roadmap-soil-layers.md`.
"""

from __future__ import annotations

import json
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere.root_depth import RootDepthParams, extension_rate
from domains.biosphere.scenario import (
    DEEP_WATER_SCENARIO,
    DEFAULT_SCENARIO,
    PERENNIAL_CHAMBER_SCENARIO,
    WATER_BITING_SCENARIO,
    SeasonScenario,
)
from domains.biosphere.season import (
    LEAF_C,
    SOIL_WATER,
    STORAGE_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.soil_layers import WATER_DENSITY, RootZoneCapture, captured_water
from domains.biosphere.step import BIO_DT, steps_for
from domains.biosphere.stocks import ROOTED_DEPTH, SUBSOIL_WATER
from simcore.integrator import EulerIntegrator
from simcore.registry import Registry
from simcore.state import State

_WEATHER = json.loads(
    (Path(__file__).parent / "oracle" / "winter_wheat_weather.json").read_text(
        encoding="utf-8"
    )
)["weather"]


def _run(scenario: SeasonScenario = DEFAULT_SCENARIO, years: int = 1) -> list[State]:
    weather = _WEATHER * years
    state, registry = build_season(scenario)
    states, rationed, events = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        BIO_DT,
        steps_for(len(weather)),
    )
    assert rationed == 0, "the capture must never need the arbitration backstop"
    assert events == ()
    return states


# --- the geometry --------------------------------------------------------------------
def test_captured_water_is_the_column_arithmetic() -> None:
    # m x (m3/m3) x kg/m3 x m2 = kg. A hand value, so a transposed factor cannot hide:
    # 1 m of soil at EXTR 0.13 over 2 m2 holds 260 kg of extractable water.
    assert captured_water(1.0, soil_extractable_water=0.13, ground_area=2.0) == 260.0
    # Linear in each argument (it is a product), and zero for a zero increment.
    assert captured_water(0.0, soil_extractable_water=0.13, ground_area=1.0) == 0.0
    assert WATER_DENSITY == 1000.0


def test_both_stores_are_derived_from_geometry_not_chosen() -> None:
    """[F] Eqns 14.26-14.28, pinned on both sides.

    WARNING: **THIS PIN HELD A FORMULA [F] DOES NOT HAVE, UNTIL 2026-08-12.** It
    asserted ``subsoil_water0 == soil_depth * EXTR * rho * A``, which is [F]'s
    **IPATSW** (14.27) — the water in the WHOLE profile, root zone included. 14.28 is
    ``WSTORG = IPATSW - ATSW``, so the shipped default double-counted the root zone's
    own 19.5 kg. It was defensible only while ``soil_water0`` was not geometric at all
    (there was no ATSW to subtract); the re-basing removed that excuse, so the value and
    the pin moved together.

    ATSW   = DEPORT            * EXTR * rho * A * MAI    (14.26)
    IPATSW = SOLDEP            * EXTR * rho * A * MAI    (14.27)
    WSTORG = (SOLDEP - DEPORT) * EXTR * rho * A * MAI    (14.28, the difference)
    """
    s = DEFAULT_SCENARIO
    atsw = (
        captured_water(
            s.rooted_depth0,
            soil_extractable_water=s.soil_extractable_water,
            ground_area=s.ground_area,
        )
        * s.soil_moisture_index
    )
    ipatsw = (
        captured_water(
            s.soil_depth,
            soil_extractable_water=s.soil_extractable_water,
            ground_area=s.ground_area,
        )
        * s.soil_moisture_index
    )
    assert s.soil_water0 == atsw
    assert s.subsoil_water0 == ipatsw - atsw
    # The two stores together ARE the profile: nothing is created or lost by the split.
    assert s.soil_water0 + s.subsoil_water0 == pytest.approx(ipatsw)
    # [F] Ch. 13's cited value, and a soil deeper than the crop's own 1.3 m cap (so the
    # CROP cap is the binding one on the frozen roster).
    assert s.soil_extractable_water == 0.13
    assert s.soil_depth > 1.3
    # MAI defaults to the drained upper limit, which is what makes FTSW0 = 1.
    assert s.soil_moisture_index == 1.0


def test_water_biting_declares_one_lean_profile_not_a_dry_layer() -> None:
    """The MAI declaration scales BOTH stores, which is why the override could go.

    ``water_biting`` used to be the one scenario forcing ``subsoil_water0 = 0``, because
    a fixed 195 kg subsoil "would pump ~2.3 kg/day into a 50 kg chamber and abolish the
    water stress this scenario exists to exercise". Under geometry the subsoil scales
    with the same MAI, so it abolishes nothing — and keeping the override would KILL the
    crop (a sealed chamber holding 1.95 kg of total water grows nothing, and its roots
    freeze at the sowing depth besides). Pinned so the retirement stays deliberate.
    """
    s = WATER_BITING_SCENARIO
    mai = s.soil_moisture_index
    assert mai == 0.05
    assert s.soil_water0 == pytest.approx(
        captured_water(
            s.rooted_depth0,
            soil_extractable_water=s.soil_extractable_water,
            ground_area=s.ground_area,
        )
        * mai
    )
    assert s.subsoil_water0 == pytest.approx(
        captured_water(
            s.soil_depth - s.rooted_depth0,
            soil_extractable_water=s.soil_extractable_water,
            ground_area=s.ground_area,
        )
        * mai
    )
    assert s.subsoil_water0 > 0.0  # the retired override


# --- the flow ------------------------------------------------------------------------
def _capture_flow(scenario: SeasonScenario) -> RootZoneCapture:
    _, registry = build_season(scenario)
    (flow,) = [f for f in registry.flows if isinstance(f, RootZoneCapture)]
    return flow


def test_the_capture_is_clamped_to_the_donor() -> None:
    # [F] Eqn 14.10's `min`, and Box 14.1's "If EWAT > WSTORG Then EWAT = WSTORG". Set
    # up a step whose geometric demand exceeds what is actually below: the flow must
    # deliver EXACTLY the remainder, never the demand. Without the clamp this overdraws
    # the donor, which the arbitration backstop would have to catch — and every golden
    # run asserts that backstop fires zero times.
    scenario = replace(DEFAULT_SCENARIO, subsoil_water0=0.001)  # kg, a near-empty store
    state, registry = build_season(scenario)
    flow = _capture_flow(scenario)
    env = weather_resolver(_WEATHER, scenario).bind(state, 1.0)
    legs = {leg.stock: leg.amount for leg in flow.evaluate(state, env, 1.0).legs}
    assert legs[SUBSOIL_WATER] == -0.001  # exactly the store, not the demand
    assert legs[SOIL_WATER] == 0.001
    # ...and the demand it was clamped FROM is genuinely larger, or this pins nothing.
    unclamped = captured_water(
        extension_rate(
            state.aux[ROOTED_DEPTH],
            state.aux["thermal_time"],
            env.get("temp"),
            state.stocks[SOIL_WATER].amount,
            0.001,
            params=flow.params,
            photo=flow.photo,
            pheno=flow.pheno,
            wssg=flow.wssg,
            soil_depth=flow.soil_depth,
            soil_extractable_water=flow.soil_extractable_water,
            ground_area=flow.ground_area,
        ),
        soil_extractable_water=scenario.soil_extractable_water,
        ground_area=scenario.ground_area,
    )
    assert unclamped > 0.001


def test_the_capture_is_a_balanced_internal_transfer() -> None:
    # Both legs are in-system soil stocks and sum to zero: no boundary is crossed, so
    # the flow only re-labels which water the crop can reach. (If it ever gained a
    # boundary leg, a sealed chamber's water would stop being conserved.)
    scenario = DEFAULT_SCENARIO
    state, _ = build_season(scenario)
    env = weather_resolver(_WEATHER, scenario).bind(state, 1.0)
    result = _capture_flow(scenario).evaluate(state, env, 1.0)
    assert sum(leg.amount for leg in result.legs) == 0.0
    assert {leg.stock for leg in result.legs} == {SOIL_WATER, SUBSOIL_WATER}
    assert all(not str(leg.stock).startswith("boundary.") for leg in result.legs)


def test_capture_and_depth_use_the_same_gated_rate() -> None:
    # THE ANTI-DRIFT PIN. The accumulator and the flow both call
    # `root_depth.extension_rate`; if either ever recomputed the gates itself they could
    # disagree, and the flow would move water for depth the roots did not gain. Measured
    # as an identity over a whole season: on EVERY step, capture > 0 exactly when the
    # depth increment > 0, and the amount equals the increment's own geometry.
    states = _run()
    for before, after in zip(states, states[1:], strict=False):
        gained = after.aux[ROOTED_DEPTH] - before.aux[ROOTED_DEPTH]
        moved = before.stocks[SUBSOIL_WATER].amount - after.stocks[SUBSOIL_WATER].amount
        assert (gained > 0.0) == (moved > 0.0)
        if gained > 0.0:
            want = captured_water(
                gained,
                soil_extractable_water=DEFAULT_SCENARIO.soil_extractable_water,
                ground_area=DEFAULT_SCENARIO.ground_area,
            )
            assert moved == pytest.approx(want, rel=1e-12)


# --- the stops -----------------------------------------------------------------------
def test_a_dry_subsoil_stops_root_extension() -> None:
    # [F] Box 14.1: "If WSTORG = 0 Then GRTD = 0" — roots do not extend into dry soil.
    # This is what makes a scenario's `subsoil_water0` load-bearing rather than
    # decorative, and it is why `water_biting`/`drought` had to declare theirs.
    dry = _run(replace(DEFAULT_SCENARIO, subsoil_water0=0.0))
    assert {s.aux[ROOTED_DEPTH] for s in dry} == {DEFAULT_SCENARIO.rooted_depth0}
    # ...against the same run with water below, where the roots reach the crop's cap.
    wet = _run()
    assert max(s.aux[ROOTED_DEPTH] for s in wet) > 1.3


def test_a_shallow_soil_caps_rooting_before_the_crop_does() -> None:
    # [F] Box 14.1 `If DEPORT >= SOLDEP Then GRTD = 0`; [E] Listing 7 L33 takes "the
    # shallowest of the rooted depths set by the soil and by the crop". This discharges
    # the ceiling `root_depth.yaml` recorded as deferred — so it is pinned as a
    # BEHAVIOUR, not just as a field that exists.
    shallow = replace(DEFAULT_SCENARIO, soil_depth=0.5, subsoil_water0=195.0)
    states = _run(shallow)
    deepest = max(s.aux[ROOTED_DEPTH] for s in states)
    assert 0.5 <= deepest <= 0.5 + 0.018  # the soil cap binds, within one step's rate
    assert deepest < 1.3  # ...well short of the crop's own cap


@pytest.mark.parametrize("subsoil", [0.0, -1.0])
def test_the_rate_is_exactly_zero_with_nothing_below(subsoil: float) -> None:
    # The gate is `<= 0`, not `== 0`: a store driven a hair negative by round-off must
    # still stop extension rather than run it backwards.
    assert (
        extension_rate(
            0.15,
            0.0,
            20.0,
            1000.0,
            subsoil,
            params=RootDepthParams(max_extension_rate=0.018, max_rooted_depth=1.3),
            photo=_capture_flow(DEFAULT_SCENARIO).photo,
            pheno=_capture_flow(DEFAULT_SCENARIO).pheno,
            wssg=0.30,
            soil_depth=1.5,
            soil_extractable_water=0.13,
            ground_area=1.0,
        )
        == 0.0
    )


# --- the re-sow return ---------------------------------------------------------------
@pytest.mark.slow
def test_the_resow_returns_the_abandoned_zones_water_so_there_is_no_ratchet() -> None:
    # OUR rule, not [F]'s (it is single-season and silent). `RootZoneCapture` is one-way
    # within a season, so without a return leg every re-sow would move more of the
    # profile permanently into the root zone. Measured over FIVE cycles: the subsoil
    # returns to the same value each year rather than stepping down — which is the
    # difference between a cycle and a ratchet, and no single golden can show it.
    scenario = PERENNIAL_CHAMBER_SCENARIO
    weather = _WEATHER * 5
    state, registry = build_season(scenario)
    states, rationed, _ = run_perennial(
        EulerIntegrator(registry),
        state,
        scenario,
        weather_resolver(weather, scenario),
        BIO_DT,
        steps_for(len(weather)),
        year=steps_for(len(_WEATHER)),
    )
    assert rationed == 0
    # The subsoil at the same point in each cycle (just after the re-sow refills it).
    # ⚠ In STEPS: it indexes the step-indexed trajectory, and the ``+ 1`` below means
    # "one integration step after the reset", not "one day after".
    year = steps_for(len(_WEATHER))
    at_cycle_start = [
        states[i * year + 1].stocks[SUBSOIL_WATER].amount for i in (1, 2, 3, 4)
    ]
    # ⚠ **THE CLAIM IS SHARPER SINCE THE 2026-08-12 RE-BASING, AND THE PIN SAYS SO.**
    # The old rule returned the abandoned column *at the drained upper limit*, so every
    # cycle start was identical from year 2 — which is what this test asserted. The
    # fraction-based rule (``returned = soil_water · abandoned/old_depth``) instead
    # CONVERGES: one transient cycle, then a fixed point held to round-off (measured
    # spread 7e-14 relative over eight cycles, i.e. the floating-point floor and not a
    # trend). That is a stronger property than "not a ratchet", and it is asserted at a
    # tolerance tight enough that a real drift could not hide inside it — 1e-12, four
    # orders below the 5.5e-4 transient it has to distinguish itself from.
    settled = at_cycle_start[1:]
    assert settled == pytest.approx([settled[0]] * len(settled), rel=1e-12), (
        "after one transient cycle the below-root store must land on a fixed point "
        f"(to round-off), not drift: {at_cycle_start}"
    )
    # And the transient really is one cycle wide and small — not a slow ratchet whose
    # first two steps happen to look flat at this tolerance.
    assert at_cycle_start[0] != settled[0]
    assert abs(at_cycle_start[0] - settled[0]) / settled[0] < 1e-3
    # Direction: a ratchet moves the store one way every cycle. This does not.
    assert at_cycle_start[0] > settled[0]
    # The two soil stores together are conserved across every cycle boundary, which is
    # what says the convergence is a REDISTRIBUTION and not a leak.
    totals = [
        states[i * year + 1].stocks[SUBSOIL_WATER].amount
        + states[i * year + 1].stocks[SOIL_WATER].amount
        for i in (1, 2, 3, 4)
    ]
    assert totals == pytest.approx([totals[0]] * 4, rel=1e-14)
    # Non-vacuity: the store really is drawn down within a year (else "no ratchet" is
    # trivially true of a mechanism that never ran).
    assert min(s.stocks[SUBSOIL_WATER].amount for s in states) < at_cycle_start[0] / 2


def test_water_is_conserved_across_the_resow_transfer() -> None:
    # `annual_reset` moves kg between two stocks INSIDE a state transform, and
    # `run_season` re-asserts conservation across every reset — so this pins that the
    # two legs are equal and opposite, which is what keeps that gate silent.
    from domains.biosphere.season import annual_reset

    scenario = PERENNIAL_CHAMBER_SCENARIO
    states = _run(scenario, years=1)
    before = states[-1]
    after = annual_reset(before, scenario)
    lost = before.stocks[SOIL_WATER].amount - after.stocks[SOIL_WATER].amount
    gained = after.stocks[SUBSOIL_WATER].amount - before.stocks[SUBSOIL_WATER].amount
    assert lost == gained
    assert lost > 0.0  # non-vacuous: a grown crop really did abandon a root zone
    assert after.aux[ROOTED_DEPTH] == scenario.rooted_depth0


# --- the mechanism does something ----------------------------------------------------
def _run_without_capture(scenario: SeasonScenario) -> list[State]:
    """The subject scenario with ONLY the ``RootZoneCapture`` flow removed.

    ⚠ **THE PREVIOUS CLEAN CONTROL WAS DESTROYED BY THE 2026-08-12 RE-BASING, AND
    SILENTLY.** It was ``soil_extractable_water = 0``, justified as removing the water
    transfer while leaving rooted depth to grow exactly as in the subject. That held
    while ``EXTR`` appeared in one place. It now appears in TWO: the transfer *and*
    ``TTSW = DEPORT · EXTR · ρ · A``, the denominator of every stress reading. So
    ``EXTR = 0`` makes the capacity zero, hence ``FTSW = 0``, hence ``WSFG = 0`` — it
    does not remove the transfer, it kills the crop outright (measured: peak leaf
    0.0500, the seed). A control that changes more than it claims is worse than no
    control, and this one would have kept passing while measuring something else.

    Dropping the flow from the registry is the control the claim actually needs: every
    parameter identical, the depth accumulator untouched, one transfer gone.
    """
    weather = _WEATHER
    state, registry = build_season(scenario)
    trimmed = Registry(
        [f for f in registry.flows if str(f.id) != "biosphere.root_zone_capture"],
        state.stocks,
        aux_processes=list(registry.aux_processes),
    )
    states, rationed, events = run_season(
        EulerIntegrator(trimmed),
        state,
        weather_resolver(weather, scenario),
        BIO_DT,
        steps_for(len(weather)),
    )
    assert rationed == 0
    assert events == ()
    return states


@pytest.mark.slow
def test_reaching_the_subsoil_is_what_saves_the_deep_water_crop() -> None:
    """THE HEADLINE CLAIM, measured against a control that removes ONLY the transfer.

    The scenario declares a supply deliberately below demand (1 mm/day against a 5.7744
    kg/day peak), so what the roots can reach decides the season. ⚠ It used to declare
    NO supply; the re-basing made that unwinnable for a reason worth keeping in view —
    a 1.3 m root system over 1 m² can reach at most 169 kg of extractable water against
    a 582 kg season demand. See the scenario comment.
    """
    subject = _run(DEEP_WATER_SCENARIO)
    control = _run_without_capture(DEEP_WATER_SCENARIO)

    def peak_leaf(states: list[State]) -> float:
        return max(s.stocks[LEAF_C].amount for s in states)

    # Same root system in both — that is what makes this a water measurement. (Both
    # reach the cap; the control gets there via the subsoil gate being satisfied, since
    # `subsoil_water` is still full — it is simply never drawn from.)
    # (Both reach the ~1.30 m cap within one step's extension of each other; the depth
    # law is shared, and the tiny difference is the last step before the cut-off.)
    assert max(s.aux[ROOTED_DEPTH] for s in subject) == pytest.approx(
        max(s.aux[ROOTED_DEPTH] for s in control), abs=0.02
    )
    # 15x the canopy and 7x the grain — a STRONGER effect than the 2.5x the previous
    # declaration produced, because the crop is now genuinely supply-limited.
    # ⚠ 16.878x / 8.440x since WSFD (2026-08-12): the CONTROL is more water-limited than
    # the subject, so drought-accelerated development costs it more and the ratio GREW.
    # The bounds below did NOT go red, and that is the point — they had slack, and
    # `deep_water` has no golden, so nothing here would have told you the headline
    # number had moved. Re-measured, not assumed (docs/log/water-stress-curves.md).
    assert peak_leaf(subject) > 10.0 * peak_leaf(control)
    assert subject[-1].stocks[STORAGE_C].amount > 7.0 * (
        control[-1].stocks[STORAGE_C].amount
    )
    assert subject[-1].stocks[STORAGE_C].amount > 2.5


@pytest.mark.slow
def test_the_deep_water_effect_is_water_and_not_the_nitrogen_gate() -> None:
    """The attribution, measured rather than asserted.

    This project has had a causal claim ("one cause, two symptoms") come back at 39 %
    before. The naive control (`subsoil_water0 = 0`) removes the water AND freezes
    rooted depth via the `WSTORG = 0` gate, so it also moves the depth-gated nitrogen
    supply; the clean control removes only the transfer. If the two agreed the gate
    would be irrelevant — they do NOT agree here, and that is the honest result:
    """
    clean = _run_without_capture(DEEP_WATER_SCENARIO)[-1]
    naive = _run(replace(DEEP_WATER_SCENARIO, subsoil_water0=0.0))[-1]
    # The naive control freezes depth at sowing; the clean one lets it grow to the cap.
    assert naive.aux[ROOTED_DEPTH] == pytest.approx(DEEP_WATER_SCENARIO.rooted_depth0)
    assert clean.aux[ROOTED_DEPTH] > 1.0
    # So the two controls are NOT interchangeable, which is exactly why the headline
    # test above uses the flow-removal one. Pinned so nobody "simplifies" it back.
    assert clean.stocks[LEAF_C].amount != naive.stocks[LEAF_C].amount


def test_drought_declares_a_stratified_profile_deliberately() -> None:
    # DROUGHT is DEFINED as a lean plot, so a hidden reservoir under it would contradict
    # its own construction — the reachable subsoil does not weaken that cascade, it
    # abolishes it (measured; see the scenario comment). Pinned so a future default
    # change cannot silently re-water it. WARNING: `water_biting` was in this pin until
    # 2026-08-12 and is not any more: it now declares ONE lean profile via MAI, which is
    # both the honest reading and the survivable one. See the test above.
    from domains.biosphere.scenario import DROUGHT_SCENARIO

    assert DROUGHT_SCENARIO.subsoil_water0 == 0.0
    # Its root zone is still at the drained upper limit — the leanness is the
    # STRATIFICATION (nothing below), not a dry bed.
    assert DROUGHT_SCENARIO.soil_water0 == DEFAULT_SCENARIO.soil_water0
    # And it survives the dry layer only because the sowing depth is a cited nonzero: at
    # depth 0 the root-zone access fraction is 0 and nitrogen uptake would be off.
    assert DROUGHT_SCENARIO.rooted_depth0 > 0.0


# --- every scenario, not just the three that happened to have a pin -------------------
# ⚠ WHY THIS IS PARAMETRIZED OVER THE WHOLE ROSTER. The two identities below held on the
# frozen tree by INHERITANCE — most scenarios take the defaults — so the pins that
# existed covered `DEFAULT`, `water_biting` and `drought` and nothing else. That is
# precisely the gap `harvest` demonstrated on 2026-08-12: it overrode `rooted_depth0`
# (a 1.3 m root system, injected past anthesis) while inheriting the 0.15 m zone's
# water,
# giving `FTSW = 0.115` on day 0 for a grain-filling crop and a 79 %-low grain — and
# nothing went red until a golden moved. Correct-by-inheritance is not covered; it is
# untested. Any scenario overriding `rooted_depth0`, `soil_depth`, `ground_area` or
# `soil_moisture_index` without moving the stores now goes red here.
_STRATIFIED = {
    # DROUGHT declares an EMPTY subsoil on purpose (its root zone is at the upper limit
    # and there is nothing below) — see its scenario comment. Its `soil_water0` is still
    # checked; only the subsoil identity is exempted, and the exemption is named.
    "DROUGHT_SCENARIO": ("subsoil",),
    # DEEP_WATER is the diagnostic for reachability, so both stores are geometric; it
    # differs only in its irrigation capacity. Listed to record that it was checked.
}


def _named_scenarios() -> list[tuple[str, SeasonScenario]]:
    """Every module-level `SeasonScenario` in `scenario.py`, plus the station's four.

    Enumerated from the MODULES, not hand-listed — the roster-vs-manifest lesson
    (`coverage-roster-is-not-the-manifest`): a hand-listed roster silently omits the
    scenario added after it was written.
    """
    import domains.biosphere.scenario as bio
    import station.scenario as st

    out: list[tuple[str, SeasonScenario]] = []
    for mod in (bio, st):
        for name in sorted(dir(mod)):
            value = getattr(mod, name)
            if isinstance(value, SeasonScenario):
                out.append((f"{mod.__name__.split('.')[0]}:{name}", value))
    return out


@pytest.mark.parametrize("name,scenario", _named_scenarios())
def test_every_scenarios_water_stores_are_geometric(
    name: str, scenario: SeasonScenario
) -> None:
    """`ATSW` and `WSTORG` from the declared geometry, on EVERY scenario.

    [F] Eqns 14.26-14.28.
    """
    mai = scenario.soil_moisture_index
    assert 0.0 <= mai <= 1.0, f"{name}: MAI is a fraction of the drained upper limit"
    atsw = (
        captured_water(
            scenario.rooted_depth0,
            soil_extractable_water=scenario.soil_extractable_water,
            ground_area=scenario.ground_area,
        )
        * mai
    )
    assert scenario.soil_water0 == pytest.approx(atsw, rel=1e-12), (
        f"{name}: soil_water0 is not DEPORT x EXTR x rho x A x MAI"
    )
    exempt = _STRATIFIED.get(name.split(":")[-1], ())
    if "subsoil" in exempt:
        assert scenario.subsoil_water0 == 0.0, f"{name}: named stratified, but not dry"
        return
    wstorg = (
        captured_water(
            scenario.soil_depth - scenario.rooted_depth0,
            soil_extractable_water=scenario.soil_extractable_water,
            ground_area=scenario.ground_area,
        )
        * mai
    )
    assert scenario.subsoil_water0 == pytest.approx(wstorg, rel=1e-12), (
        f"{name}: subsoil_water0 is not (SOLDEP - DEPORT) x EXTR x rho x A x MAI"
    )


def test_the_roster_this_covers_is_not_empty_and_includes_the_station() -> None:
    """Non-vacuity for the enumeration above — an empty roster would pass silently."""
    names = [n for n, _ in _named_scenarios()]
    assert len(names) >= 8, names
    assert any(n.startswith("station:") for n in names), names
    assert any(n.endswith("WATER_BITING_SCENARIO") for n in names), names


def test_the_harvest_injection_keeps_depth_and_water_together() -> None:
    """The station's past-anthesis injection, which is where this gap actually bit.

    `build_harvest_station` overrides `rooted_depth0` to 1.3 m on top of a greenhouse
    built for the 0.15 m sowing zone. Before 2026-08-12 it inherited that zone's water:
    19.5 kg inside a 169 kg capacity, `FTSW = 0.115`, grain 79 % low. Both stores
    are now
    re-derived from the injected depth, and this asserts the resulting state — not the
    code path — so a future refactor cannot quietly drop it.
    """
    from domains.crew.loader import load_crew_params
    from domains.eclss.loader import load_eclss_params
    from station.harvest import build_harvest
    from station.loader import load_harvest_params
    from station.scenario import HARVEST_SCENARIO

    state, _, _ = build_harvest(
        load_crew_params(),
        load_eclss_params(),
        load_harvest_params(),
        HARVEST_SCENARIO,
    )
    bio = HARVEST_SCENARIO.greenhouse.bio
    depth = state.aux[ROOTED_DEPTH]
    assert depth == pytest.approx(HARVEST_SCENARIO.rooted_depth0)
    capacity = captured_water(
        depth,
        soil_extractable_water=bio.soil_extractable_water,
        ground_area=bio.ground_area,
    )
    held = state.stocks[SOIL_WATER].amount
    assert held == pytest.approx(capacity * bio.soil_moisture_index, rel=1e-12)
    # ...which is to say the injected crop starts at its declared FTSW, not at 0.115.
    assert held / capacity == pytest.approx(bio.soil_moisture_index, rel=1e-12)
    below = state.stocks[SUBSOIL_WATER].amount
    assert below == pytest.approx(
        captured_water(
            bio.soil_depth - depth,
            soil_extractable_water=bio.soil_extractable_water,
            ground_area=bio.ground_area,
        )
        * bio.soil_moisture_index,
        rel=1e-12,
    )
