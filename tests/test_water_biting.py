"""Dormant-machinery scenario: the water-biting sealed chamber (additive, NOT a ref).

The frozen sealed chambers tune the closed water cycle (``soil_water → water_vapor →
condensate → soil_water``) **inert**: they start ``soil_water0 = 1000`` kg, so the loop
keeps ``soil_water`` far above the stress band and ``f_water ≡ 1`` (the carbon/O₂/N
trajectory is bit-identical to the pre-water-cycle run — that was the Step-3 sizing
discipline). So the ``water_stress_factor`` ramp wired into the carbon-budget limiter
has **never run hot in a sealed run**. ``WATER_BITING_SCENARIO`` drives it on purpose.

The mechanism: lower the loop's total water to 50 kg (``soil_water0`` inside the
``(sw_wilting, sw_critical) = (20, 60)`` band). Transpiration self-limits via its own
``water_stress_factor``, so the closed loop reaches a stable fixed point with
``soil_water`` settled ~40 kg — well **above** wilting (the plant survives, the bite is
not a kill) — and ``f_water`` holds ~0.5 every step, water-limiting gross assimilation.
The water-loop total stays conserved to round-off the whole season (genuine closure is
preserved; only the *operating point* moved). ``f_N ≡ 1`` (default plant-N), so the bite
is purely water.

The **inverse** of the sealed potential-production assertions: those keep ``f_water ==
1`` (``test_sealed_chamber`` lineage); here the water-starved sealed chamber asserts
``f_water`` genuinely bites while conservation, ``rationed == 0``, ``events == ()`` and
the loss-sink ``0.0`` all still hold. The cascade (lower biomass) is shown against an
otherwise-identical ample-water baseline. The golden is
``test_regression_water_biting_season.py``.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere.drift import total_quantity
from domains.biosphere.loader import load_nitrogen_params
from domains.biosphere.nitrogen import nitrogen_stress_factor
from domains.biosphere.scenario import WATER_BITING_SCENARIO, WATER_BITING_YEARS
from domains.biosphere.season import (
    CONDENSATE,
    LEAF_C,
    PLANT_N,
    ROOT_C,
    SOIL_WATER,
    STEM_C,
    WATER_VAPOR,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.transpiration import water_stress_factor
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_NITRO = load_nitrogen_params()
# A meaningful bite, not float noise (the advisor's bar — ≤ ~0.9 sustained, not 0.999).
_BITE_CEILING = 0.9


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(scenario) -> tuple[list[State], int, tuple]:
    weather = _weather() * WATER_BITING_YEARS
    state, registry = build_season(scenario)
    return run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )


@pytest.fixture(scope="module")
def water_biting() -> tuple[list[State], int, tuple]:
    return _run(WATER_BITING_SCENARIO)


def _vegetative(state: State) -> float:
    return (
        state.stocks[LEAF_C].amount
        + state.stocks[STEM_C].amount
        + state.stocks[ROOT_C].amount
    )


def _f_water(state: State) -> float:
    return water_stress_factor(
        state.stocks[SOIL_WATER].amount,
        sw_wilting=WATER_BITING_SCENARIO.sw_wilting,
        sw_critical=WATER_BITING_SCENARIO.sw_critical,
    )


# --- the headline: the dormant f_water ramp actually ran hot in a sealed run --------
def test_water_biting_f_water_bites(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # f_water drops MEANINGFULLY below 1 for a sustained window and stays strictly in
    # (0, 1] throughout — biting (the closed water cycle limits assimilation) yet never
    # collapsing to zero (soil_water stays above wilting — stressed, not dead).
    states, _, _ = water_biting
    f_ws = [_f_water(s) for s in states]
    assert min(f_ws) < _BITE_CEILING  # a real bite (~0.50 by probe)
    assert all(0.0 < f <= 1.0 for f in f_ws)  # stressed, never fully wilted
    assert sum(1 for f in f_ws if f < _BITE_CEILING) > 30  # sustained


def test_water_biting_stays_above_wilting(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # The closed loop settles to a stable fixed point ABOVE the wilting point — the bite
    # is sustained water stress, not progressive drainage to a dead plant. (soil_water
    # never reaching sw_wilting is what keeps f_water > 0.)
    states, _, _ = water_biting
    assert all(
        s.stocks[SOIL_WATER].amount > WATER_BITING_SCENARIO.sw_wilting for s in states
    )


def test_water_biting_loop_stays_closed(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # Lowering the operating point does NOT break closure: the water-loop total
    # (soil_water + water_vapor + condensate) is conserved to round-off all season —
    # the cycle still recovers every drop of transpired water, it just runs leaner.
    states, _, _ = water_biting

    def loop_total(s: State) -> float:
        return (
            s.stocks[SOIL_WATER].amount
            + s.stocks[WATER_VAPOR].amount
            + s.stocks[CONDENSATE].amount
        )

    total0 = loop_total(states[0])
    assert total0 == pytest.approx(
        WATER_BITING_SCENARIO.soil_water0
    )  # vapor/cond start 0
    for s in states:
        assert math.isclose(loop_total(s), total0, rel_tol=0.0, abs_tol=1e-9)


def test_water_biting_cascade_vs_ample(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # Direction-only cascade: against an otherwise-IDENTICAL ample-water baseline (only
    # soil_water0 raised to the frozen-chamber 1000 kg → f_water ≡ 1), the water-biting
    # run reaches a LOWER peak vegetative biomass. Isolates f_water — the one changed
    # field is soil_water0.
    states, _, _ = water_biting
    ample, _, _ = _run(replace(WATER_BITING_SCENARIO, soil_water0=1000.0))
    assert min(_f_water(s) for s in ample) == 1.0  # baseline is genuinely water-replete
    assert max(_vegetative(s) for s in states) < max(_vegetative(s) for s in ample)


def test_water_biting_f_n_stays_one(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # The bite is purely water: f_N stays == 1 (default plant-N reserve), so the product
    # limitation = f_water · f_N isolates the water factor — no N-side confounder.
    states, _, _ = water_biting
    for s in states:
        f_n = nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            _vegetative(s),
            n_residual_per_mol_c=_NITRO.n_residual_per_mol_c,
            n_critical_per_mol_c=_NITRO.n_critical_per_mol_c,
        )
        assert f_n == 1.0


# --- the closure invariants still hold under the bite -------------------------------
def test_water_biting_never_rations(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # The first-order donor-controlled water flows (and f_water-throttled assimilation)
    # never overdraw — rationed == 0 stays structural under the limiter.
    _, rationed, _ = water_biting
    assert rationed == 0


def test_water_biting_no_extinction(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # A growth throttle, not a kill: no extinction event and the carbon loss-sink stays
    # exactly 0.0 (death routes to litter; the chamber stays genuinely closed).
    states, _, events = water_biting
    assert events == ()
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states)


@pytest.mark.parametrize(
    ("quantity", "abs_tol"),
    [
        (Quantity.CARBON, 1e-12),
        (Quantity.OXYGEN, 1e-11),
        (Quantity.WATER, 1e-7),
        (Quantity.NITROGEN, 1e-9),
    ],
)
def test_water_biting_conserves_every_quantity(
    water_biting: tuple[list[State], int, tuple], quantity: Quantity, abs_tol: float
) -> None:
    # All four sealed-chamber currencies conserved (boundaries explicit) under the bite.
    states, _, _ = water_biting
    q0 = total_quantity(states[0], quantity)
    for s in states:
        assert math.isclose(
            total_quantity(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol
        )


def test_water_biting_is_deterministic(
    water_biting: tuple[list[State], int, tuple],
) -> None:
    # Bit-identical on a re-run (the golden's premise).
    states, rationed, events = water_biting
    states2, rationed2, events2 = _run(WATER_BITING_SCENARIO)
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)
