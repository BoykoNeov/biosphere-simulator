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
from domains.biosphere.stocks import ROOTED_DEPTH, SOIL_N, SUBSOIL_WATER
from simcore.integrator import EulerIntegrator
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
        1.0,
        len(weather),
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


def test_the_default_subsoil_is_the_profile_at_the_drained_upper_limit() -> None:
    # `subsoil_water0`'s default is DERIVED, not chosen: soil_depth x EXTR x rho x area.
    # Pinned because the two sides live in different fields and nothing else couples
    # them — moving `soil_depth` alone would silently make the default incoherent.
    s = DEFAULT_SCENARIO
    assert s.subsoil_water0 == captured_water(
        s.soil_depth,
        soil_extractable_water=s.soil_extractable_water,
        ground_area=s.ground_area,
    )
    # [F] Ch. 13's cited value, and a soil deeper than the crop's own 1.3 m cap (so the
    # CROP cap is the binding one on the frozen roster).
    assert s.soil_extractable_water == 0.13
    assert s.soil_depth > 1.3


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
            sw_wilting=flow.sw_wilting,
            sw_critical=flow.sw_critical,
            soil_depth=flow.soil_depth,
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
            sw_wilting=20.0,
            sw_critical=60.0,
            soil_depth=1.5,
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
        1.0,
        len(weather),
        year=len(_WEATHER),
    )
    assert rationed == 0
    # The subsoil at the same point in each cycle (just after the re-sow refills it).
    year = len(_WEATHER)
    at_cycle_start = [
        states[i * year + 1].stocks[SUBSOIL_WATER].amount for i in (1, 2, 3, 4)
    ]
    assert at_cycle_start == pytest.approx([at_cycle_start[0]] * 4, rel=1e-12), (
        "the below-root store must return to the same level each cycle, not ratchet"
    )
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
@pytest.mark.slow
def test_reaching_the_subsoil_is_what_saves_the_deep_water_crop() -> None:
    """THE HEADLINE CLAIM, measured against a control that removes ONLY the water.

    ⚠ The obvious control (`subsoil_water0 = 0`) is the WRONG one: it removes the water
    *and* freezes rooted depth (the `WSTORG = 0` gate), so it also changes the nitrogen
    gate. `soil_extractable_water = 0` is the clean control — rooted depth grows exactly
    as it does in the subject, and only the transfer is switched off.
    """
    subject = _run(DEEP_WATER_SCENARIO)
    control = _run(replace(DEEP_WATER_SCENARIO, soil_extractable_water=0.0))

    def peak_leaf(states: list[State]) -> float:
        return max(s.stocks[LEAF_C].amount for s in states)

    # Same root system in both — that is what makes this a water measurement.
    assert max(s.aux[ROOTED_DEPTH] for s in subject) == pytest.approx(
        max(s.aux[ROOTED_DEPTH] for s in control), rel=1e-12
    )
    assert peak_leaf(subject) > 2.4 * peak_leaf(control)
    # ...and the categorical one: it is the difference between setting grain and none.
    assert subject[-1].stocks[STORAGE_C].amount > 3.0
    assert control[-1].stocks[STORAGE_C].amount == 0.0


@pytest.mark.slow
def test_the_deep_water_effect_is_water_and_not_the_nitrogen_gate() -> None:
    # The attribution, measured rather than asserted — this project has had a causal
    # claim ("one cause, two symptoms") come back at 39 % before. The naive control and
    # the clean control agree stock-for-stock EXCEPT `soil_n` at one ULP, so the
    # depth-gated nitrogen contributes nothing to the deep-water rescue.
    clean = _run(replace(DEEP_WATER_SCENARIO, soil_extractable_water=0.0))[-1]
    naive = _run(replace(DEEP_WATER_SCENARIO, subsoil_water0=0.0))[-1]
    for sid, stock in clean.stocks.items():
        if sid == SUBSOIL_WATER:
            continue  # the two controls differ in this store BY CONSTRUCTION
        other = naive.stocks[sid].amount
        if sid == SOIL_N:
            assert abs(other - stock.amount) / abs(stock.amount) < 1e-15
        else:
            assert other == stock.amount, f"{sid} moved between the two controls"


def test_water_biting_and_drought_declare_dry_profiles_deliberately() -> None:
    # Both scenarios are DEFINED as water-lean, so a hidden reservoir under them would
    # contradict their own construction — and the default profile does not weaken the
    # drought cascade, it abolishes it (measured; see the scenario comments). Pinned so
    # a future default change cannot silently re-water them.
    from domains.biosphere.scenario import DROUGHT_SCENARIO

    assert WATER_BITING_SCENARIO.subsoil_water0 == 0.0
    assert DROUGHT_SCENARIO.subsoil_water0 == 0.0
    # And they survive it only because the sowing depth is a cited nonzero: at depth 0
    # the root-zone access fraction is 0 and nitrogen uptake would be identically off.
    assert WATER_BITING_SCENARIO.rooted_depth0 > 0.0
    assert DROUGHT_SCENARIO.rooted_depth0 > 0.0
