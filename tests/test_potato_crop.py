"""Potato — the first SECOND species — vs the WOFOST oracle. A DIAGNOSTIC, never a fit.

Plan of record: ``docs/plans/post-roadmap-potato-crop.md``.

Every crop the biosphere had grown before this one was winter wheat. Even the
"day-neutral crop" is not a counterexample — by its own record it is the same
winter-wheat files with the cold and daylength gates switched off, "not a new param
file". Potato is the first crop with **its own cited parameters**
(``params/crops/potato/``: phenology, allocation, canopy), reached through the crop
param-set seam (``tests/test_crop_param_set.py``).

WHAT THIS FILE CLAIMS, AND WHAT IT REFUSES TO CLAIM
It pins *a literature-cited, physically-sane potato, runnable as habitat content, with
its gaps to WOFOST measured*. It does **not** claim the crop was validated against the
oracle. Two separate reasons, and they cut in opposite directions:

  * **Phenology is same-family.** Our development stage and WOFOST's are both
    two-phase linear thermal time, so timing agreement would test our *parameter
    choice*, not our model — the near-tautology the day-neutral crop's write-up
    flagged. Here it does not even agree, which is more informative than if it had.
  * **Canopy and biomass are cross-family.** WOFOST assimilates through an
    AMAX/light-response formalism; we use FvCB. Where those differ we **cannot fit and
    do not** (ruling B: the oracle is a diagnostic, never a fit target).

THE HEADLINE FINDING IS A DISAGREEMENT BETWEEN TWO SOURCES, NOT AN ERROR ON OUR SIDE
Our partition curve is [E] Table 18's van Heemst potato, and it begins filling the tuber
almost immediately (positive share once development passes 0.15, first tuber carbon on
day 7). The WOFOST oracle holds tuber weight at exactly 0 until day 46 — development
stage 1.027, i.e. flowering. Two independent parameterizations of the same organ of the
same crop disagree **qualitatively** about when filling starts. That single difference
is also the cause of both downstream gaps measured below — a starved early canopy and an
over-filled tuber — so they are one finding with two symptoms, not two findings.

Nothing here is closed by moving a value. The partition table stays as [E] has it.

Our-side numbers carry ``pytest.approx`` bands: leaf area and organ carbon flow through
FvCB (``exp``/``sqrt``), so they are last-ULP libm-sensitive off Windows/UCRT (the
``tests/golden_platform.py`` trap). Oracle-side values are exact (committed JSON).
PCSE-free: both fixtures are read as JSON.
"""

from __future__ import annotations

import json
from dataclasses import replace
from pathlib import Path
from typing import Any

import pytest

from domains.biosphere.canopy import leaf_area_index
from domains.biosphere.loader import (
    crop_param_set,
    load_canopy_params,
    load_nitrogen_params,
    load_phenology_params,
)
from domains.biosphere.phenology import development_stage
from domains.biosphere.scenario import (
    POTATO_SCENARIO,
    SEALED_CHAMBER_SCENARIO,
    SeasonScenario,
)
from domains.biosphere.season import (
    LEAF_C,
    LITTER_CARBON,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

_ORACLE_DIR = Path(__file__).parent / "oracle"
_REFERENCE = _ORACLE_DIR / "potato_reference.json"
_WEATHER = _ORACLE_DIR / "potato_weather.json"

_GROUND_AREA = 1.0  # m² — the open-field plot (SeasonScenario default)
_RUN_DAYS = 150  # past our own maturity (day 108); the oracle's run is 97 days


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"]


def _oracle() -> list[dict[str, Any]]:
    """The post-emergence oracle rows — day 0 == emergence, aligned to our
    ``thermal_time`` = 0 at emergence (matched-DVS discipline, not matched-date)."""
    payload = json.loads(_REFERENCE.read_text(encoding="utf-8"))
    return [r for r in payload["trajectory"] if r["DVS"] is not None]


def _provenance() -> dict[str, Any]:
    return json.loads(_REFERENCE.read_text(encoding="utf-8"))["provenance"]


def _run(scenario: SeasonScenario, steps: int = _RUN_DAYS) -> list[State]:
    """Run ``scenario`` under the oracle's own weather; assert it is well-behaved."""
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, steps
    )
    # A crop that only conserves by rationing is not a crop that ran (the rationing
    # gate's rule: conservation is not survival).
    assert rationed == 0
    assert events == ()
    return states


def _dvs(states: list[State]) -> list[float]:
    pheno = load_phenology_params(crop_param_set("potato").paths["phenology"])
    return [
        development_stage(
            s.aux["thermal_time"],
            tsum_anthesis=pheno.tsum_anthesis,
            tsum_maturity=pheno.tsum_maturity,
        )
        for s in states
    ]


def _lai(states: list[State]) -> list[float]:
    canopy = load_canopy_params(crop_param_set("potato").paths["canopy"])
    return [
        leaf_area_index(
            s.stocks[LEAF_C].amount,
            sla_per_mol_c=canopy.sla_per_mol_c,
            ground_area=_GROUND_AREA,
        )
        for s in states
    ]


def _first_at(series: list[float], threshold: float) -> int | None:
    return next((i for i, v in enumerate(series) if v >= threshold), None)


def _peak(series: list[float]) -> tuple[int, float]:
    index = max(range(len(series)), key=lambda i: series[i])
    return index, series[index]


# --- the crop is what it says it is -----------------------------------------


def test_potato_overrides_exactly_three_files_and_shares_five() -> None:
    # The honest-reuse claim, asserted rather than asserted-in-a-comment. Potato's own
    # science is phenology, allocation and canopy; the other five fall back to the
    # reference crop. The FvCB block in particular is SHARED, and that is defensible
    # only because those twelve params were never wheat-specific — they are TODO(cite)
    # placeholders tagged "literature-typical C3", and potato is a C3 plant. If someone
    # later gives potato its own photosynthesis file, this test must be updated
    # deliberately, which is the point.
    crop = crop_param_set("potato")
    assert crop.name == "potato"
    assert crop.overridden == ("allocation", "canopy", "phenology")
    assert crop.shared == (
        "nitrogen",
        "photosynthesis",
        "respiration",
        "senescence",
        "transpiration",
    )


def test_potato_params_pass_the_frozen_loaders_and_carry_the_cited_values() -> None:
    # A crop's files go through the SAME pydantic schema, unit guards and bound checks
    # as the frozen reference — a crop directory is not a way around the guards.
    crop = crop_param_set("potato")
    pheno = load_phenology_params(crop.paths["phenology"])
    # [E] Table 13's potato row: 7.,0.01, 18.,1.0, 29.,0.01 — base at the lower zero,
    # cap at the OPTIMUM (a cap at the 29 °C upper zero would have us accumulating
    # 22 degC*day/day where the source says development has stopped).
    assert pheno.t_base == 7.0
    assert pheno.t_cap == 18.0
    # TSUM = (T_ref - t_base)/r with T_ref = 18: (18-7)/0.035 and (18-7)/0.015.
    assert pheno.tsum_anthesis == pytest.approx(11.0 / 0.035, abs=0.01)
    assert pheno.tsum_maturity == pytest.approx(11.0 / 0.015, abs=0.01)
    # The shape this gives potato is the mirror image of our winter wheat's: a SHORT
    # vegetative phase and a LONG filling phase, where wheat is the other way round.
    assert pheno.tsum_anthesis < pheno.tsum_maturity
    wheat = load_phenology_params()
    assert wheat.tsum_anthesis > wheat.tsum_maturity


def test_potato_leaf_is_thinner_than_wheats_by_the_factor_the_source_gives() -> None:
    # [E] Table 19: potato 300 kg/ha vs winter wheat 425 kg/ha of leaf per unit area,
    # inverted to specific leaf area. The ratio is the source's, not our conversion's.
    potato = load_canopy_params(crop_param_set("potato").paths["canopy"])
    wheat = load_canopy_params()
    assert potato.sla_per_mol_c > wheat.sla_per_mol_c
    ratio = potato.sla_per_mol_c / wheat.sla_per_mol_c
    assert ratio == pytest.approx(33.33 / 22.0, rel=1e-3)


def test_carbon_fraction_agrees_across_every_crop_set() -> None:
    # TRAP 1 (designed for, not discovered). carbon_fraction lives in BOTH canopy.yaml
    # and nitrogen.yaml, and the reference file says they MUST be equal — divergence
    # "models a silently inconsistent plant". Potato overrides canopy but NOT nitrogen,
    # so the constraint now spans a CROP BOUNDARY. Checked for every crop that exists,
    # not just the default, because the next crop will hit this too.
    #
    # Both loaders fold the fraction rather than exposing it (sla -> m2/mol C;
    # N thresholds -> kg N/mol C), so the check is on the fold: nitrogen's
    # dm_kg_per_mol_c is M_C/carbon_fraction, and canopy's sla_per_mol_c is
    # sla_m2_per_kg * that same quantity. Dividing recovers the SLA, which must match
    # the file's declared value exactly if the two fractions agree.
    for name in (None, "potato"):
        crop = crop_param_set(name)
        canopy = load_canopy_params(crop.paths["canopy"])
        nitro = load_nitrogen_params(crop.paths["nitrogen"])
        declared_sla = {None: 22.0, "potato": 33.33}[name]
        assert canopy.sla_per_mol_c / nitro.dm_kg_per_mol_c == pytest.approx(
            declared_sla, rel=1e-12
        ), f"carbon_fraction disagrees between canopy and nitrogen for crop {name!r}"


# --- the diagnostic: measured gaps to the oracle, none of them fitted -------


def test_oracle_fixture_is_the_run_we_think_it_is() -> None:
    # Guards the comparison's premise: an oracle regenerated against a different crop,
    # site or production mode would silently change every number below.
    provenance = _provenance()
    assert provenance["crop_name"] == "potato"
    assert provenance["mode"] == "pp"  # potential production, like our own PP plot
    assert provenance["grid_no"] == 31031
    # The oracle's CULTIVAR, pinned because it is load-bearing on the headline finding
    # below. We name our own cultivar explicitly (cv Mara — [E] carries two potato
    # cultivars whose vegetative rates differ by 1.6x), so the oracle's has to be on the
    # record too: otherwise a reader cannot tell whether the tuber-onset disagreement is
    # cross-MODEL or merely cross-CULTIVAR, which is precisely the distinction this file
    # is careful about everywhere else. The demo DB holds 46 potato varieties.
    #
    # ⚠ AND THE HONEST READING IS THAT WE CANNOT FULLY SEPARATE THEM. Variety 2830's
    # parameter values are PCSE's and are deliberately never read (clean-room), so we
    # know WHICH cultivar the oracle ran but not how its partition curve is shaped. The
    # finding therefore stands as "two cited parameterizations disagree", and does NOT
    # claim the disagreement is purely structural. Recorded rather than papered over.
    assert provenance["variety_no"] == 2830
    assert provenance["milestones_days_since_emergence"] == {
        "DOE": 0,
        "DOA": 44,
        "DOM": 96,
    }
    # Our run must be driven from the same site the oracle ran at, or the astronomical
    # daylength (hence PAR) is wrong.
    assert POTATO_SCENARIO.latitude == 37.64
    # And potato is day-neutral BY THE SOURCE'S MARKING ([E] Table 12's "-" legend),
    # so neither modifier is built.
    assert POTATO_SCENARIO.vernalization is False
    assert POTATO_SCENARIO.photoperiod is False


def test_gap_1_phenology_vegetative_fast_reproductive_long() -> None:
    # SAME-FAMILY, so this measures our PARAMETER choice, not our model. Recorded
    # because it did NOT agree, which is more informative than agreement would be.
    #
    #   ours   anthesis day 33, maturity day 108  (veg 33 d, fill 75 d)
    #   oracle anthesis day 44, maturity day  96  (veg 44 d, fill 52 d)
    #
    # Note the direction: we chose cv Mara, the LATE cultivar, and still run the
    # vegetative phase ~25 % fast. The cultivar choice was made on source-internal
    # grounds before this was run (see the param file's header) and is NOT revisited to
    # improve the number.
    dvs = _dvs(_run(POTATO_SCENARIO))
    assert _first_at(dvs, 1.0) == 33
    assert _first_at(dvs, 2.0) == 108
    provenance = _provenance()["milestones_days_since_emergence"]
    assert provenance["DOA"] == 44
    assert provenance["DOM"] == 96


def test_gap_2_the_headline_the_two_sources_disagree_on_tuber_onset() -> None:
    # THE FINDING. [E]/van Heemst's partition curve starts filling the tuber essentially
    # at emergence; WOFOST's potato holds it until flowering. Both are "potato". This is
    # a cross-source disagreement recorded, not a defect calibrated away.
    states = _run(POTATO_SCENARIO)
    dvs = _dvs(states)
    ours = next(i for i, s in enumerate(states) if s.stocks[STORAGE_C].amount > 0.0)
    assert ours == 7
    assert dvs[ours] == pytest.approx(0.192, abs=0.01)

    oracle = _oracle()
    theirs = next(i for i, r in enumerate(oracle) if (r["TWSO"] or 0.0) > 0.0)
    assert theirs == 46
    assert oracle[theirs]["DVS"] == pytest.approx(1.027, abs=0.001)

    # The gap is not marginal — it is most of the season, and it is qualitative: one
    # source fills the harvested organ throughout, the other only after flowering.
    assert theirs - ours > 35


def test_gap_3_the_canopy_is_starved_downstream_of_gap_2() -> None:
    # A DOWNSTREAM CONSEQUENCE of gap 2, not an independent defect: assimilate diverted
    # into the tuber from day 7 is assimilate the leaves never got, and our development
    # runs fast on top of that, so the leaf tap shuts early. Our canopy peaks at 3.18 on
    # day 34; the oracle's reaches 8.88 on day 51 — a 2.8x shortfall.
    #
    # ⚠ CORRECTED 2026-08-11: that attribution was asserted, not measured, and it is
    # OVERSTATED. Measured on this harness (see docs/plans/
    # post-roadmap-wheat-partition-backfill.md), holding the tuber to anthesis lifts the
    # peak 3.184 -> 5.406 — 39 % of the gap, not all of it. Winter wheat reproduces a
    # shortfall of the same magnitude class from ROOT share alone, with a storage organ
    # that opens at DVS 0.77. The statement that survives both crops: peak canopy is set
    # by assimilate diverted during the COMPOUNDING phase (~DVS < 0.6), whichever organ
    # does the diverting. The 39 % is not pinned here — a stated deferral, not an
    # oversight; pinning it needs the four-way harness lifted into tests/.
    #
    # Worth contrasting with the day-neutral crop, where our peak leaf area landed
    # within 2 % of its oracle: that agreement was NOT a general property of our canopy,
    # and this is the case that shows it. Both facts survive together only because
    # neither was fitted.
    lai = _lai(_run(POTATO_SCENARIO))
    our_day, our_peak = _peak(lai)
    assert our_day == 34
    assert our_peak == pytest.approx(3.184, rel=1e-3)

    oracle_lai = [r["LAI"] for r in _oracle()]
    their_day, their_peak = _peak(oracle_lai)
    assert their_day == 51
    assert their_peak == pytest.approx(8.885, rel=1e-3)
    assert their_peak / our_peak == pytest.approx(2.79, rel=0.02)


def test_gap_4_the_tuber_over_fills_the_other_symptom_of_gap_2() -> None:
    # The second symptom of the same cause. At the oracle's own maturity (day 96) our
    # tuber holds ~14.3 t/ha of dry matter against the oracle's 7.25 t/ha — about 2x.
    #
    # This comparison is deliberately made on the TUBER and not on total above-ground
    # biomass: our leaf/stem carbon senesces to litter, so our standing above-ground
    # total is not comparable to the oracle's TAGP (a cumulative production figure that
    # includes dead material). The tuber does not senesce in our model, so it is the
    # one organ that compares like for like.
    states = _run(POTATO_SCENARIO)
    canopy = load_canopy_params(crop_param_set("potato").paths["canopy"])
    nitro = load_nitrogen_params(crop_param_set("potato").paths["nitrogen"])
    del canopy  # (kept above to document that both loaders share the carbon fraction)
    # mol C on 1 m2 -> kg dry matter per hectare.
    kg_per_ha = nitro.dm_kg_per_mol_c * 1e4
    ours = states[96].stocks[STORAGE_C].amount * kg_per_ha
    theirs = _oracle()[96]["TWSO"]
    assert ours == pytest.approx(14260.0, rel=0.01)
    assert theirs == pytest.approx(7249.8, rel=1e-4)
    assert ours / theirs == pytest.approx(1.97, rel=0.02)


def test_gap_5_we_are_root_heavy_at_mid_vegetative() -> None:
    # A partition-model difference independent of gap 2's timing: at development stage
    # 0.5 our roots hold 36 % of live biomass against the oracle's 20 %. Recorded
    # because the day-neutral crop found the OPPOSITE sign against LINTUL3 (that oracle
    # front-loaded roots relative to us), so this is not a standing bias of our
    # allocation — it is per-crop, per-source.
    states = _run(POTATO_SCENARIO)
    dvs = _dvs(states)
    index = next(i for i, d in enumerate(dvs) if d >= 0.5)
    state = states[index]
    live = (
        state.stocks[LEAF_C].amount
        + state.stocks[STEM_C].amount
        + state.stocks[ROOT_C].amount
    )
    assert state.stocks[ROOT_C].amount / live == pytest.approx(0.360, abs=0.005)

    oracle_row = next(r for r in _oracle() if r["DVS"] >= 0.5)
    their_live = oracle_row["TWLV"] + oracle_row["TWST"] + oracle_row["TWRT"]
    assert oracle_row["TWRT"] / their_live == pytest.approx(0.200, abs=0.005)


# --- the crop is a shippable artifact, not just a report --------------------


def test_potato_runs_in_the_sealed_habitat_under_both_integrators() -> None:
    # TRAP 3 (designed for, not discovered). The chamber-scale finding is that the
    # sealed jar holds ~2 days of ONE crop's carbon, `run_scenario` now RAISES on
    # rationing, and potato grows harder than wheat — so a sealed potato was a live
    # candidate to over-draw. It does not: the FvCB Ci-shutoff self-limits before the
    # arbitration backstop is ever needed, exactly as it does for wheat.
    #
    # "Authored != validated" applies in full: this pins conservation and determinism,
    # not scientific endorsement, and no golden is written.
    sealed = replace(
        SEALED_CHAMBER_SCENARIO,
        crop="potato",
        vernalization=False,
        photoperiod=False,
        latitude=37.64,
    )
    weather = _weather()
    for integrator_cls in (EulerIntegrator, Rk4Integrator):
        state, registry = build_season(sealed)
        states, rationed, events = run_season(
            integrator_cls(registry),
            state,
            weather_resolver(weather, sealed),
            1.0,
            len(weather),
        )
        assert rationed == 0, f"{integrator_cls.__name__} needed the Euler backstop"
        assert events == ()
        final = states[-1]
        # It is alive and it closed the loop: tuber carbon was made, and shed organ
        # carbon reached the in-system litter pool rather than a boundary sink.
        assert final.stocks[STORAGE_C].amount > 0.0
        assert final.stocks[LITTER_CARBON].amount > 0.0


def test_potato_is_deterministic() -> None:
    first = _run(POTATO_SCENARIO)
    second = _run(POTATO_SCENARIO)
    for a, b in zip(first, second, strict=True):
        for stock_id, stock in a.stocks.items():
            assert stock.amount.hex() == b.stocks[stock_id].amount.hex()


def test_the_frozen_wheat_is_untouched_by_any_of_this() -> None:
    # The whole exercise adds a species without moving the reference. Cheap to check,
    # and it is the property the freeze contract actually cares about.
    assert SeasonScenario().crop is None
    wheat = crop_param_set(None)
    assert wheat.overridden == ()
    assert load_phenology_params(wheat.paths["phenology"]).tsum_anthesis == 1100.0
    assert load_canopy_params(wheat.paths["canopy"]).extinction_coef == 0.6
