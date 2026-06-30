"""Dormant-machinery scenario: the N-limited open field (additive, NOT a frozen ref).

The seven frozen reference scenarios all keep ``f_N ≡ 1`` (the plant-N concentration is
far above the critical-N band — verified by ``test_season_is_potential_production`` /
``test_perennial_f_n_stays_one`` / ``test_sealed_f_n_stays_one``), so the
``nitrogen_stress_factor`` ramp wired into the carbon-budget limiter
(``CarbonContext.limitation``) has **never run hot**. ``N_LIMITED_SCENARIO`` drives it
on purpose, to flush latent bugs in that integration path before Phase 5 builds on it.

The mechanism is **N-limitation by dilution** — the primary mechanism ``nitrogen.py``
names. A deliberately small fixed plant-N reserve (``plant_n0``) puts the whole-plant N
concentration ``plant_n / (leaf+stem+root)`` in the ``(n_residual, n_critical)`` band
at sowing; as biomass grows the concentration falls *through* the band, so ``f_N`` ramps
below 1 and N-limits gross assimilation. Uptake is shut **off** (``soil_n0`` below
``sn_residual`` ⇒ ``soil_n_availability ≡ 0``), so ``plant_n`` is constant and the bite
is **pure dilution** — unconfounded by uptake (and ``rationed`` cannot fire: the frozen
uptake capacity is large relative to the tiny N band, so an in-band uptake would either
flood ``plant_n`` or overdraw ``soil_n`` — the design note in ``scenario.py``).

This is the **inverse** of ``test_season_is_potential_production``: there the open field
asserts ``f_N == 1``; here the (N-starved) open field asserts ``f_N`` genuinely bites,
while every closure invariant (conservation, ``rationed == 0``, ``events == ()``,
loss-sink ``0.0``) still holds. The cascade (lower biomass) is shown against an
otherwise-identical N-replete baseline. The golden is
``test_regression_n_limited_season.py``.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere.drift import total_quantity
from domains.biosphere.loader import load_nitrogen_params
from domains.biosphere.nitrogen import nitrogen_stress_factor, soil_n_availability
from domains.biosphere.scenario import N_LIMITED_SCENARIO, N_LIMITED_YEARS
from domains.biosphere.season import (
    LEAF_C,
    PLANT_N,
    ROOT_C,
    STEM_C,
    build_season,
    run_season,
    weather_resolver,
)
from domains.biosphere.stocks import SOIL_N
from simcore.boundary import loss_sink_id
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_NITRO = load_nitrogen_params()
# A meaningful bite, not float noise: f_N must drop at least this far below 1 for the
# dormant ramp to count as "run hot" (the advisor's bar — ≤ ~0.9 sustained, not 0.999).
_BITE_CEILING = 0.9


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run(scenario) -> tuple[list[State], int, tuple]:
    weather = _weather() * N_LIMITED_YEARS
    state, registry = build_season(scenario)
    return run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )


@pytest.fixture(scope="module")
def n_limited() -> tuple[list[State], int, tuple]:
    return _run(N_LIMITED_SCENARIO)


def _vegetative(state: State) -> float:
    return (
        state.stocks[LEAF_C].amount
        + state.stocks[STEM_C].amount
        + state.stocks[ROOT_C].amount
    )


def _f_n(state: State) -> float:
    return nitrogen_stress_factor(
        state.stocks[PLANT_N].amount,
        _vegetative(state),
        n_residual_per_mol_c=_NITRO.n_residual_per_mol_c,
        n_critical_per_mol_c=_NITRO.n_critical_per_mol_c,
    )


# --- the headline: the dormant f_N ramp actually ran hot ----------------------------
def test_n_limited_f_n_bites(n_limited: tuple[list[State], int, tuple]) -> None:
    # f_N drops MEANINGFULLY below 1 (not float noise) for a sustained window, and stays
    # strictly in (0, 1] throughout — biting (not 1) yet never zero (the plant is
    # N-stressed, not N-dead). This is the whole point of the scenario: the
    # nitrogen_stress_factor ramp wired into the carbon budget is exercised.
    states, _, _ = n_limited
    f_ns = [_f_n(s) for s in states]
    assert min(f_ns) < _BITE_CEILING  # a real bite (~0.55 by probe)
    assert all(0.0 < f <= 1.0 for f in f_ns)  # stressed, never N-dead
    assert (
        sum(1 for f in f_ns if f < _BITE_CEILING) > 30
    )  # sustained, not a one-step blip


def test_n_limited_is_pure_dilution(n_limited: tuple[list[State], int, tuple]) -> None:
    # The bite is pure dilution: uptake is shut off (soil_n below sn_residual ⇒
    # availability ≡ 0, so NitrogenUptake yields a zero leg every step), so plant_n is
    # CONSTANT at its sowing value — the falling f_N is wholly the growing biomass
    # diluting a fixed N reserve, not a shifting N supply. Documents the regime + the
    # soil_n_availability shutoff / structural-positivity path.
    states, _, _ = n_limited
    for s in states:
        avail = soil_n_availability(
            s.stocks[SOIL_N].amount,
            sn_residual=N_LIMITED_SCENARIO.sn_residual,
            sn_critical=N_LIMITED_SCENARIO.sn_critical,
        )
        assert avail == 0.0  # uptake shut off
        assert s.stocks[PLANT_N].amount == N_LIMITED_SCENARIO.plant_n0  # constant


def test_n_limited_cascade_vs_replete(
    n_limited: tuple[list[State], int, tuple],
) -> None:
    # Direction-only cascade (the anti-flakiness rule — no magnitude, no day index):
    # against an otherwise-IDENTICAL N-replete baseline (only plant_n0 up → f_N ≡ 1),
    # the N-limited run reaches a LOWER peak vegetative biomass. Isolates f_N: the one
    # changed field is plant_n0, so the growth shortfall is the limiter, nothing else.
    states, _, _ = n_limited
    replete, _, _ = _run(replace(N_LIMITED_SCENARIO, plant_n0=0.5))
    assert max(_f_n(s) for s in replete) == 1.0  # baseline is genuinely N-replete
    assert min(_f_n(s) for s in replete) == 1.0
    assert max(_vegetative(s) for s in states) < max(_vegetative(s) for s in replete)


# --- the closure invariants still hold under the bite -------------------------------
def test_n_limited_never_rations(n_limited: tuple[list[State], int, tuple]) -> None:
    # f_N throttles gross assimilation (reduces draws) — it never forces the Euler
    # backstop. rationed == 0 stays structural under the limiter.
    _, rationed, _ = n_limited
    assert rationed == 0


def test_n_limited_no_extinction(n_limited: tuple[list[State], int, tuple]) -> None:
    # The stress is a growth throttle, not a kill: no extinction event fires and the
    # carbon loss-sink (the numerical guard, decision #6) stays exactly 0.0.
    states, _, events = n_limited
    assert events == ()
    carbon_loss_sink = loss_sink_id(Quantity.CARBON)
    assert all(s.stocks[carbon_loss_sink].amount == 0.0 for s in states)


@pytest.mark.parametrize(
    ("quantity", "abs_tol"),
    [
        (Quantity.CARBON, 1e-12),
        (Quantity.WATER, 1e-7),
        (Quantity.NITROGEN, 1e-9),
    ],
)
def test_n_limited_conserves_every_quantity(
    n_limited: tuple[list[State], int, tuple], quantity: Quantity, abs_tol: float
) -> None:
    # The conserved totals (boundaries explicit) hold flat under the limiter — the
    # always-on gate already asserts this per step; folded explicitly here. OXYGEN is
    # absent in the open field (no chamber O₂ pool), so the three currencies in play.
    states, _, _ = n_limited
    q0 = total_quantity(states[0], quantity)
    for s in states:
        assert math.isclose(
            total_quantity(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol
        )


def test_n_limited_is_deterministic(
    n_limited: tuple[list[State], int, tuple],
) -> None:
    # Bit-identical on a re-run (the golden's premise).
    states, rationed, events = n_limited
    states2, rationed2, events2 = _run(N_LIMITED_SCENARIO)
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)
