"""The sealed chambers' season-low CO₂ stays above the CO₂ compensation point.

**The claim this file gives contract standing.** Below the CO₂ compensation point (``Ci
= Γ*``) the FvCB model's net assimilation is *exactly* zero — it is a hard floor, not a
soft discouragement. A chamber whose air is drawn below it and which goes on fixing
carbon is not a guard going off; it is **the answer being wrong**, silently, with every
automatic gate green. That is precisely what the shipped one-day step was doing to the
perennial and consumer chambers until 2026-08-14 (``domains/biosphere/step.py``), and
**nothing in the tree would have caught it** — the diagnosis came from a probe, three
work items and several weeks after the defect shipped.

The direction plan named this band on 2026-08-13 as *"exactly the shape
``science_bands`` exists for — it would have caught this on day one"*, and filed it
behind the step decision because it was **red on the frozen tree**. The step moved; the
band lands.

⚠ **THE LOCUS. It is the PERENNIAL and CONSUMER chambers, not the sealed one.** The
proposal was written as *"the sealed chamber's season-low CO₂"* and carried that way
through the direction plan and five files for three days. Measured, the sealed chamber
in **its own golden's configuration** (``run_season``, which never re-sows) reads 75.75
ppm at ``dt = 1`` and **never crossed**; the 57.89 ppm attached to its name came from
running it through ``run_perennial``'s unconditional annual re-sow, which its reference
does not perform. The crossing was always perennial (56.03 ppm) and consumer. A band
written on ``sealed_chamber`` alone would have frozen the wrong scenario — so this file
gates **all five**, each through the driver its own golden uses, and the drivers are not
interchangeable. See ``step.py``'s corrected docstring for the full pairing.

**The bound is DERIVED, never typed.** ``Γ* = 42.75 µmol/mol`` is one of
``photosynthesis.yaml``'s ``TODO(cite)`` entries, and the direction plan is explicit
that the literal must not enter a science claim while that is true. So the floor is
computed from the frozen params at run time and the literal appears only as a
**tripwire** — :func:`test_the_floor_is_where_the_frozen_params_put_it` fails loudly if
``Γ*`` or ``ci_ratio`` moves, which is an unfreeze event and should be loud.

⚠ **And the uncited value is not load-bearing, which is measured rather than asserted.**
The only route to the same quantity on the shelf is Teh eq. 6.19 — ``Γ* = O₂/(2·τ)``
with the specificity factor ``τ = 2600 µmol/µmol`` tabulated at 25 °C (Teh,
*Introduction to Mathematical Modeling of Crop Growth*, Table 6.2). On our own ``o2``
param that gives ``Γ* = 40.385`` and a floor of **57.69 ppm — BELOW the shipped 61.07**.
So the shipped value is the *conservative* one: every margin here is understated by the
citation gap, and closing it can only move the verdict further from the floor. That is
the honest version of shipping a band whose constant is uncited, and it is why the band
does not wait for the retrieval.

⚠ **The horizon question, asked because a 15-year band can be green while the attractor
is not.** The humification split pushed the chamber settling transient from ~3 yr to ~35
(``docs/log/cue-humification.md``) — past every frozen horizon — so "green on the
golden" is not by itself "green at equilibrium". Measured to 50 yr, both re-sowing
chambers take their **global** minimum *inside* the frozen horizon (perennial at year 2,
consumer at year 5) and rise monotonically to a settled attractor thereafter (75.84 /
75.06 ppm). **The worst case is what the golden already runs**, so the band's horizon
and the golden's coincide rather than merely being asserted to.

Probes: ``M:/claud_projects/temp/co2-band/``. Nothing here regenerates anything; the
values below were measured on the shipped tree before this file existed.
"""

from __future__ import annotations

import json
from functools import cache
from pathlib import Path

import pytest

from domains.biosphere.loader import load_photosynthesis_params
from domains.biosphere.scenario import (
    CONSUMER_CHAMBER_YEARS,
    LONG_HORIZON_YEARS,
    PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
)
from domains.biosphere.season import (
    CARBON_POOL,
    CONSUMER_CHAMBER_SCENARIO,
    PERENNIAL_CHAMBER_SCENARIO,
    SEALED_CHAMBER_SCENARIO,
    SeasonScenario,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from simcore.integrator import EulerIntegrator
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

# The tripwire literal. It is the *expected* value of a quantity read from the frozen
# params below — never the threshold an assertion compares against. See the module note.
FLOOR_PPM = 61.07

# Teh eq. 6.19's independent route, for the robustness assertion. Same role: expected
# value of a computed quantity, not a threshold.
TEH_SPECIFICITY_FACTOR = 2600.0  # µmol/µmol at 25 °C, Teh Table 6.2
TEH_FLOOR_PPM = 57.69


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def floor_ppm() -> float:
    """The CO₂ compensation point in chamber ppm, from the frozen params.

    ``Γ*`` is the compensation point in the *intercellular* air; the chamber gate is on
    the *ambient* air, and the two are related by the C3 set point ``Ci = ci_ratio ·
    Ca`` the sealed carbon budget already uses. So the ambient floor is ``Γ*/ci_ratio``.
    """
    return load_photosynthesis_params().gamma_star / SEALED_CHAMBER_SCENARIO.ci_ratio


@cache
def _season_low_ppm(name: str) -> float:
    """The minimum chamber CO₂ (ppm) over ``name``'s own golden run.

    ⚠ **Each scenario is driven the way its own golden drives it**, which is the whole
    point of the module note: ``sealed_chamber``'s golden uses ``run_season`` and
    re-sows never, the other four use ``run_perennial``'s annual reset. Driving them
    uniformly is how the sealed chamber acquired a crossing it does not have.

    Cached because the two long-horizon entries re-run 15 years and the suite is
    wall-clock-managed (``docs/test-suite-runtime.md``).
    """
    scenario, years, perennial = _CASES[name]
    rows = _weather() * years
    state, registry = build_season(scenario)
    resolver = weather_resolver(rows, scenario)
    integ = EulerIntegrator(registry)
    if perennial:
        states, rationed, events = run_perennial(
            integ,
            state,
            scenario,
            resolver,
            BIO_DT,
            steps_for(len(rows)),
            year=steps_for(len(_weather())),
        )
    else:
        states, rationed, events = run_season(
            integ, state, resolver, BIO_DT, steps_for(len(rows))
        )
    # The band is a claim about a *closed, well-fed* run — the same pre-golden gate
    # every
    # golden in this tree carries. A rationed run's CO₂ trace is not the model's answer.
    assert rationed == 0, f"{name}: band run must be well-fed, rationed {rationed}"
    assert events == (), f"{name}: band run must be extinction-free, got {events}"
    return _min_ppm(states, scenario)


def _min_ppm(states: list[State], scenario: SeasonScenario) -> float:
    air = scenario.chamber_air_mol
    return min(s.stocks[CARBON_POOL].amount / air * 1e6 for s in states)


_CASES: dict[str, tuple[SeasonScenario, int, bool]] = {
    "sealed_chamber": (SEALED_CHAMBER_SCENARIO, SEALED_CHAMBER_YEARS, False),
    "perennial_chamber": (
        PERENNIAL_CHAMBER_SCENARIO,
        PERENNIAL_CHAMBER_YEARS,
        True,
    ),
    "consumer_chamber": (CONSUMER_CHAMBER_SCENARIO, CONSUMER_CHAMBER_YEARS, True),
    "perennial_long_horizon": (PERENNIAL_CHAMBER_SCENARIO, LONG_HORIZON_YEARS, True),
    "consumer_long_horizon": (CONSUMER_CHAMBER_SCENARIO, LONG_HORIZON_YEARS, True),
}


# --- the floor itself, before anything is compared against it -------------------------


def test_the_floor_is_where_the_frozen_params_put_it() -> None:
    """The tripwire. ``Γ*`` or ``ci_ratio`` moving is an unfreeze, and should be loud.

    Everything below compares a measured minimum against :func:`floor_ppm`, which is
    computed rather than typed — so a silent re-value of ``Γ*`` would move every bound
    in this file at once and no assertion would notice. This is the one place the number
    is pinned, which is what lets the rest of the file stay derived.
    """
    assert floor_ppm() == pytest.approx(FLOOR_PPM, abs=5e-3)


def test_the_shipped_floor_is_the_conservative_one_against_the_cited_route() -> None:
    """⚠ The band does not depend on ``Γ*``'s missing citation — measured, not assumed.

    ``gamma_star`` is ``TODO(cite)``. The only route to the same quantity on the shelf
    is Teh eq. 6.19, ``Γ* = O₂/(2·τ)``, with ``τ`` tabulated at 25 °C. It lands
    **below** the shipped value, so the shipped floor is the harder test and closing the
    citation gap can only widen every margin in this file.

    ⚠ This is a statement about the FLOOR, not an endorsement of swapping the value:
    Teh's companion constants (``Kc`` 300, ``Ko`` 300 mmol/mol) disagree with ours
    (404.9 / 278.4), so the two are different parameterizations and mixing them would be
    the co-adaptation this project refuses. The comparison is legitimate *because* it
    only ever moves the bound in the direction that makes the claim harder to pass.
    """
    params = load_photosynthesis_params()
    teh_gamma = params.o2 * 1000.0 / (2.0 * TEH_SPECIFICITY_FACTOR)
    teh_floor = teh_gamma / SEALED_CHAMBER_SCENARIO.ci_ratio
    assert teh_floor == pytest.approx(TEH_FLOOR_PPM, abs=5e-3)
    assert teh_floor < floor_ppm(), (
        "Teh's route no longer sits below the shipped floor — the robustness "
        "argument in "
        "this module's docstring is void and the band's provenance must be re-argued, "
        "not re-tuned."
    )


# --- the band, one gate per frozen scenario ------------------------------------------
#
# ⚠ Five separate gates rather than one parametrized one, because the marker must be a
# literal decorator with literal keyword arguments (``tests/science_gates.py``) — a
# parametrized indirection is invisible to the static enumeration that builds the
# manifest, so it would freeze nothing while looking like it froze five things.


@pytest.mark.science_gate(
    scenario="sealed_chamber",
    field="science_bands",
    quantity="season-low chamber CO₂ (ppm)",
    bound="min > Γ*/ci_ratio (61.07 ppm)",
    source="FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. "
    "1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so "
    "the verdict is provenance-insensitive — test_the_shipped_floor_is_the_"
    "conservative_one_against_the_cited_route",
)
def test_sealed_chamber_stays_above_the_compensation_point() -> None:
    """71.44 ppm — and ⚠ this chamber **never crossed**, at any shipped step.

    ⚠ Said **75.82** until 2026-08-15, which was this scenario's reading in `4d7fdfd`,
    the commit that wrote this file. The light path (`a0ef98b`, six commits later the
    same day) and then the layered canopy moved it; ``test_the_five_margins_are_pinned_
    not_merely_positive`` went red at the first of those and was re-pinned, and these
    docstrings were not. See ``docs/biosphere-reference.md`` — a value in prose
    acquires no owner.

    It is gated anyway. The scenario spent three days named as the crossing's locus, so
    a gate here is worth more than its margin: it pins the configuration
    (``run_season``, no re-sow) that makes the sealed chamber's number what it is.
    Re-measure through ``run_perennial`` and it reads 57.89 ppm at ``dt = 1`` — the same
    tree, a different run.
    """
    assert _season_low_ppm("sealed_chamber") > floor_ppm()


@pytest.mark.science_gate(
    scenario="perennial_chamber",
    field="science_bands",
    quantity="season-low chamber CO₂ (ppm)",
    bound="min > Γ*/ci_ratio (61.07 ppm)",
    source="FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. "
    "1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so "
    "the verdict is provenance-insensitive — test_the_shipped_floor_is_the_"
    "conservative_one_against_the_cited_route",
)
def test_perennial_chamber_stays_above_the_compensation_point() -> None:
    """⚠ **THE ONE THAT WAS RED.** 56.03 ppm at ``dt = 1``; 70.25 at the shipped step.

    This is the scenario the whole step unfreeze was authorised on, and the gate that
    should have existed before it.

    ⚠ **It is now also the TIGHTEST of the five (1.1503×)**, which it was not when this
    file was written — see
    ``test_consumer_chamber_stays_above_the_compensation_point``, whose docstring made
    the opposite claim and was right for six commits. Said "75.48 at
    the shipped step / the margin is 1.24×" until 2026-08-15.
    """
    assert _season_low_ppm("perennial_chamber") > floor_ppm()


@pytest.mark.science_gate(
    scenario="consumer_chamber",
    field="science_bands",
    quantity="season-low chamber CO₂ (ppm)",
    bound="min > Γ*/ci_ratio (61.07 ppm)",
    source="FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. "
    "1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so "
    "the verdict is provenance-insensitive — test_the_shipped_floor_is_the_"
    "conservative_one_against_the_cited_route",
)
def test_consumer_chamber_stays_above_the_compensation_point() -> None:
    """⚠ **WAS the tightest of the five; as of 2026-08-15 it is the LOOSEST.**

    73.34 ppm, margin **1.2009×**. This docstring read *"THE TIGHTEST OF THE FIVE, and a
    number no record carried before this file — 74.42 ppm, margin 1.2186×, below both
    chambers that were quoted in the step write-ups"*, and that was true when written
    (`4d7fdfd`) and false six commits later. The light path and the layered canopy both
    act through canopy **closure**, and this chamber's crop is the one the crew's CO₂
    keeps furthest from closing — so it lost the least (−1.5 % against −7 %) and the
    ranking inverted around it. ``perennial_chamber`` is now the tightest at 1.1503×.

    ⚠ **The lesson the old text carried is untouched and is why this was found**:
    enumerate the roster, not the discussion. What it did not say, and this does, is
    that a *ranking* is a claim about a moment — re-derive it, never quote it.
    """
    assert _season_low_ppm("consumer_chamber") > floor_ppm()


@pytest.mark.slow
@pytest.mark.science_gate(
    scenario="perennial_long_horizon",
    field="science_bands",
    quantity="season-low chamber CO₂ (ppm)",
    bound="min > Γ*/ci_ratio (61.07 ppm)",
    source="FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. "
    "1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so "
    "the verdict is provenance-insensitive — test_the_shipped_floor_is_the_"
    "conservative_one_against_the_cited_route",
)
def test_perennial_long_horizon_stays_above_the_compensation_point() -> None:
    """15 yr: 70.2526 ppm — the *same* minimum as the 5-yr run, taken in year 2.

    The long horizon adds no new low. ⚠ **That identity is the durable half and it
    survived two unfreezes**: the value said 75.4757 until 2026-08-15, and through both
    moves the 15-yr reading has stayed bit-equal to the 5-yr one. A *shape* outlives the
    values it is measured on, which is the argument for pinning shapes.

    ⚠ The 50-yr statement below is **dated 2026-08-14 and has NOT been re-measured**
    since the light path or the layered canopy: *"the per-year minima climb
    monotonically off that year-2 trough to 75.84 and flatten, so the band's worst case
    sits inside the frozen horizon"*. Its two inputs both moved, so it is an open
    question, not restated with new digits it has not earned.
    """
    assert _season_low_ppm("perennial_long_horizon") > floor_ppm()


@pytest.mark.slow
@pytest.mark.science_gate(
    scenario="consumer_long_horizon",
    field="science_bands",
    quantity="season-low chamber CO₂ (ppm)",
    bound="min > Γ*/ci_ratio (61.07 ppm)",
    source="FvCB: net assimilation is exactly zero at Ci = Γ* ([A] Farquhar et al. "
    "1980). ⚠ Γ* is TODO(cite); Teh eq. 6.19 (τ=2600) gives 57.69 ppm, below it, so "
    "the verdict is provenance-insensitive — test_the_shipped_floor_is_the_"
    "conservative_one_against_the_cited_route",
)
def test_consumer_long_horizon_stays_above_the_compensation_point() -> None:
    """15 yr: 73.3386 ppm, taken in year 5 — again the same minimum as the 5-yr run.

    Same reading as the perennial long horizon, including the caveat: the value said
    74.4210 until 2026-08-15, the 15-yr/5-yr identity held through both moves, and the
    50-yr claim (*"the minima rise off year 5 and settle at 75.06"*) is dated 2026-08-14
    and has not been re-measured.
    """
    assert _season_low_ppm("consumer_long_horizon") > floor_ppm()


# --- the margins, pinned so the band cannot pass by a hair unnoticed ------------------


@pytest.mark.slow
def test_the_five_margins_are_pinned_not_merely_positive() -> None:
    """⚠ An inequality that passes tells you nothing about **how nearly** it failed.

    The band above is a one-sided claim, deliberately — it must survive the leaf
    mechanism's golden movement without being re-pinned, so it is written as ``>`` and
    not as a value. But a one-sided claim degrades silently: a change that halves every
    margin leaves all five gates green.

    So the margins are pinned here, loosely (2 %) and in one place, as the *observable*
    rather than the contract. This is the number the next unfreeze's gate report quotes
    — a 15 % larger canopy draws more carbon, so these move even where the inequality
    holds, and that movement is the thing worth reading.
    """
    floor = floor_ppm()
    measured = {name: _season_low_ppm(name) / floor for name in _CASES}
    # ⚠ RE-PINNED 2026-08-14 by the within-day light path — the first unfreeze this pin
    # was written for, one day after it landed, and it did the job it was written for:
    # three of the five moved past 2 % while every inequality above stayed green. Every
    # margin TIGHTENS by ~4-7 %, and the direction is the interesting part: the chamber
    # crop assimilates less per day (concavity) and now also RESPIRES INTO THE POOL AT
    # NIGHT, which raises the pool — yet the net is a lower season-low, so the day-side
    # loss dominates the night-side return. Was: sealed 1.2579, perennial 1.2359,
    # consumer 1.2186.
    expected = {
        "sealed_chamber": 1.1671,
        "perennial_chamber": 1.1543,
        "consumer_chamber": 1.2086,
        "perennial_long_horizon": 1.1543,
        "consumer_long_horizon": 1.2086,
    }
    assert measured == pytest.approx(expected, rel=0.02)
