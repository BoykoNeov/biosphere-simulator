"""The day-neutral crop's PURPOSE — warm-habitat arrest contrast + habitat-runnability.

Plan of record: ``docs/plans/post-roadmap-day-neutral-crop.md``. Ceremony 2
(``post-roadmap-oracle-match.md``) left this open: the frozen winter wheat requires
vernalization (a cold cue), so **dropped into a warm, lamp-lit habitat it would never
flower** — the habitat needs a crop with no cold requirement. This file is the signed
demonstration that the day-neutral crop is that crop, and that it runs in the sealed
chamber (the habitat form) as a well-behaved closed ecosystem.

**The contrast is the payload.** Under a warm constant-temperature regime (20 °C, above
the 12 °C vernalization ceiling, so no vernalization day ever accrues):

  * the **winter wheat** (vernalization ON) is **permanently arrested** — ``verfun`` is
    pinned at 0 in the vegetative phase, so the thermal-time increment is gated to 0,
    DVS never leaves 0, and it never flowers (a real deployment failure, not a
    slowdown);
  * the **day-neutral crop** (vernalization + photoperiod OFF) develops on thermal time
    alone and completes the 0 → 2 arc normally.

Note the lamp-*photoperiod*-control demonstration considered earlier is deliberately
**absent**: a day-neutral crop's flowering does not respond to daylength, so the lamp
controls its PAR/energy (via ``station.lighting``) but not its flowering — see the plan.

Pure stdlib + the biosphere season; no PCSE, no goldens (additive, non-frozen content).
"""

import datetime as _dt

import pytest

from domains.biosphere.loader import load_phenology_params
from domains.biosphere.phenology import development_stage
from domains.biosphere.scenario import DEFAULT_SCENARIO, SeasonScenario
from domains.biosphere.season import (
    LEAF_C,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.integrator import EulerIntegrator, Rk4Integrator
from simcore.state import State

_HABITAT_DAYS = 305


def _warm_weather(temp_c: float = 20.0) -> list[dict[str, float | str]]:
    """A warm constant-temperature habitat forcing (real ISO dates for the daylength
    derivation; ample light, moderate VPD). ``temp_c = 20`` sits above the 12 °C
    vernalization ceiling, so a cold-requiring crop never accrues a vernalization
    day."""
    base = _dt.date(1997, 3, 31)
    return [
        {
            "day": (base + _dt.timedelta(days=i)).isoformat(),
            "TEMP": temp_c,
            "IRRAD": 18.0e6,
            "VAP": 12.0,
        }
        for i in range(_HABITAT_DAYS)
    ]


def _run(scenario: SeasonScenario) -> list[State]:
    state, registry = build_season(scenario)
    resolver = weather_resolver(_warm_weather(), scenario)
    states, rationed, events = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, _HABITAT_DAYS
    )
    assert rationed == 0
    assert events == ()
    return states


def _dvs_series(states: list[State]) -> list[float]:
    pp = load_phenology_params()
    return [
        development_stage(
            s.aux["thermal_time"],
            tsum_anthesis=pp.tsum_anthesis,
            tsum_maturity=pp.tsum_maturity,
        )
        for s in states
    ]


def _first_day_at(series: list[float], threshold: float) -> int | None:
    for i, v in enumerate(series):
        if v >= threshold:
            return i
    return None


# --- the contrast: the crop's reason to exist ---------------------------------


def test_winter_wheat_is_permanently_arrested_in_a_warm_habitat() -> None:
    """The frozen winter wheat NEVER flowers without cold — the deployment failure.

    Warm temperature (20 °C) means every vernalization day is 0, so cumulative
    vernalization stays 0, ``verfun`` is pinned at 0 through the vegetative phase, and
    the thermal-time increment is gated to 0. DVS is stuck at 0 for the whole season —
    permanent arrest, not a slowdown.
    """
    states = _run(DEFAULT_SCENARIO)  # vernalization + photoperiod ON (the frozen crop)
    dvs = _dvs_series(states)
    assert max(dvs) == 0.0
    assert states[-1].aux["thermal_time"] == 0.0  # no thermal time ever accrued


def test_day_neutral_crop_flowers_and_matures_in_the_same_warm_habitat() -> None:
    """The day-neutral crop develops normally where the winter wheat cannot.

    With both modifiers off, thermal time advances at the plain degree-day rate, so the
    crop reaches anthesis and maturity on the warm forcing — the crop the habitat needs.
    """
    states = _run(SeasonScenario(vernalization=False, photoperiod=False))
    dvs = _dvs_series(states)
    assert max(dvs) >= 2.0 - 1e-9  # completes the arc to maturity
    anthesis = _first_day_at(dvs, 1.0)
    maturity = _first_day_at(dvs, 2.0)
    assert anthesis is not None and maturity is not None
    # Fast warm development (±3 d — thermal-time crossings): anthesis ~55, maturity ~93.
    assert anthesis == pytest.approx(55, abs=3)
    assert maturity == pytest.approx(93, abs=3)


def test_the_signed_contrast_winter_arrests_day_neutral_develops() -> None:
    """The payload in one assertion: identical warm habitat, opposite outcomes."""
    winter = max(_dvs_series(_run(DEFAULT_SCENARIO)))
    day_neutral = max(
        _dvs_series(_run(SeasonScenario(vernalization=False, photoperiod=False)))
    )
    assert winter == 0.0  # arrested
    assert day_neutral >= 1.9  # flowered + matured


# --- habitat-runnability: the crop is a well-behaved closed ecosystem ---------


def test_day_neutral_crop_runs_in_the_sealed_chamber_habitat() -> None:
    """The day-neutral crop is habitat-runnable: a sealed chamber closes and conserves.

    The sealed chamber is the habitat form (a closed CARBON/OXYGEN/WATER/NITROGEN loop).
    The day-neutral crop runs it with ``rationed == 0`` / ``events == ()`` under
    **both** integrators and is bit-deterministic — it does not over-draw the
    (un-enlarged) sealed CO₂ pool the way the ~5× consumer chamber did. "Authored ≠
    validated": this earns
    conservation + determinism, not a frozen golden.
    """
    scenario = SeasonScenario(
        sealed=True, litter_carbon0=3.0, vernalization=False, photoperiod=False
    )
    weather = _warm_weather()

    for integrator_cls in (EulerIntegrator, Rk4Integrator):
        state, registry = build_season(scenario)
        resolver = weather_resolver(weather, scenario)
        states, rationed, events = run_season(
            integrator_cls(registry), state, resolver, 1.0, len(weather)
        )
        assert rationed == 0
        assert events == ()
        assert states[-1].stocks[LEAF_C].amount > 0.0

    # Determinism: two independent Euler runs are bit-identical.
    a_state, a_reg = build_season(scenario)
    a_states, _, _ = run_season(
        EulerIntegrator(a_reg),
        a_state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )
    b_state, b_reg = build_season(scenario)
    b_states, _, _ = run_season(
        EulerIntegrator(b_reg),
        b_state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )
    assert a_states[-1].stocks[LEAF_C].amount.hex() == (
        b_states[-1].stocks[LEAF_C].amount.hex()
    )
