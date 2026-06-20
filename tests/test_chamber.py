"""Phase-2 Step-2 tests: the finite chamber atmosphere + the Ci-from-stock seam (P2.2).

The first **emergent feedback**: a sealed chamber sources photosynthesis from a finite
``carbon_pool`` (not the open field's unclamped boundary), and FvCB derives ``Ci`` from
the pool's live draw-down (``chamber.ci_from_co2_pool`` read as a shared stock, #16).
With no control code, the pool draws down → ``Ci`` falls → gross assimilation collapses
toward Γ*. Three layers:

* **The conversion** (:func:`ci_from_co2_pool`) — pure, hand-checkable.
* **The seam** (:class:`CarbonContext` Ci source) — forcing (open) vs pool-derived
  (sealed), and the all-or-nothing wiring guard.
* **The integration** — the sealed season: ``rationed == 0`` across a *non-vacuous*
  draw-down (Ci falls ~2.7× at its minimum; the central numerical check that the finite
  pool self-limits via FvCB's Ci-shutoff, never the Euler backstop), total carbon
  conserved (the sealed chamber has no boundary carbon *source*), and the open-field
  path left untouched.

**Scope (updated at Step 3).** Step 3 promoted the carbon pool to a true CO₂ stock
(``{CARBON:1, OXYGEN:2}``) with an O₂ counterpart and **closed the gas loop**:
respiration now returns CO₂ to the pool instead of draining to a boundary sink, so the
pool is **no longer monotone** (it refills on deficit days) and gross assimilation need
not collapse to ~0. The conversion and seam tests below are unchanged; the integration
tests assert the closed-loop reality. The multi-quantity flow balance + the exact
OXYGEN conservation / O₂↔CO₂ anti-correlation invariants live in
``tests/test_gas_exchange.py``.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere.chamber import ci_from_co2_pool
from domains.biosphere.season import (
    CARBON_POOL,
    CO2_ATMOS,
    LEAF_C,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    SeasonScenario,
    _carbon_context,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.registry import Registry
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_ORGANS = (LEAF_C, STEM_C, ROOT_C, STORAGE_C)


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run_sealed(
    scenario: SeasonScenario,
) -> tuple[list[State], int, tuple]:
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, len(_weather()))


def _total_carbon(s: State) -> float:
    """Total CARBON across every stock (internal + boundary), folding composition.

    The sealed chamber has no boundary carbon *source*, so this is invariant across
    the whole run — the conservation gate's claim, asserted end-to-end here.
    """
    return sum(
        stock.amount * stock.composition.get(Quantity.CARBON, 0.0)
        for stock in s.stocks.values()
    )


# --- the conversion: ci_from_co2_pool (pure, hand-checked) -------------------
def test_ci_from_co2_pool_hand_value() -> None:
    # Ca = 0.5 mol / 1000 mol · 1e6 = 500 µmol mol⁻¹; Ci = 0.7 · 500 = 350.
    assert math.isclose(
        ci_from_co2_pool(0.5, air_mol=1000.0, ci_ratio=0.7), 350.0, rel_tol=1e-12
    )


def test_ci_from_co2_pool_is_linear_in_carbon() -> None:
    a = ci_from_co2_pool(0.2, air_mol=1000.0, ci_ratio=0.7)
    b = ci_from_co2_pool(0.4, air_mol=1000.0, ci_ratio=0.7)
    assert math.isclose(b, 2.0 * a, rel_tol=1e-12)


def test_ci_from_co2_pool_zero_carbon_is_zero_ci() -> None:
    # An exhausted pool reads Ci = 0 (≤ Γ*) → FvCB shuts the source off entirely.
    assert ci_from_co2_pool(0.0, air_mol=1000.0, ci_ratio=0.7) == 0.0


@pytest.mark.parametrize(
    ("co2_mol", "air_mol", "ci_ratio"),
    [
        (0.5, 0.0, 0.7),  # degenerate chamber (no air)
        (0.5, -1.0, 0.7),
        (0.5, 1000.0, 0.0),  # degenerate ratio
        (0.5, 1000.0, -0.7),
        (-0.5, 1000.0, 0.7),  # impossible negative carbon
        (math.inf, 1000.0, 0.7),  # non-finite carbon
        (math.nan, 1000.0, 0.7),
    ],
)
def test_ci_from_co2_pool_rejects_bad_inputs(
    co2_mol: float, air_mol: float, ci_ratio: float
) -> None:
    with pytest.raises(ValueError):
        ci_from_co2_pool(co2_mol, air_mol=air_mol, ci_ratio=ci_ratio)


# --- the seam: CarbonContext Ci source (forcing vs pool) --------------------
def test_carbon_context_open_field_reads_ci_forcing() -> None:
    # Default (open field): Ci is the constant ``ci`` forcing, not pool-derived.
    scenario = SeasonScenario()
    state, _ = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    ctx = _carbon_context(scenario)
    assert ctx.co2_pool_var is None
    assert ctx._ci(resolver.bind(state, 1.0)) == scenario.ci


def test_carbon_context_sealed_derives_ci_from_pool() -> None:
    # Sealed: Ci is derived from the live carbon-pool amount via ci_from_co2_pool.
    scenario = SeasonScenario(sealed=True)
    state, _ = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    ctx = _carbon_context(scenario)
    pool0 = state.stocks[CARBON_POOL].amount
    expected = ci_from_co2_pool(
        pool0, air_mol=scenario.chamber_air_mol, ci_ratio=scenario.ci_ratio
    )
    assert ctx._ci(resolver.bind(state, 1.0)) == expected


def test_carbon_context_partial_chamber_triple_rejected() -> None:
    # The Ci-source triple (co2_pool_var, chamber_air_mol, ci_ratio) is all-or-nothing;
    # a partial wiring is a build bug caught at construction.
    sealed = _carbon_context(SeasonScenario(sealed=True))
    with pytest.raises(ValueError, match="all-or-nothing|set together|left None"):
        replace(sealed, ci_ratio=None)


# --- the integration: the sealed season's emergent draw-down ----------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    return _run_sealed(SeasonScenario(sealed=True))


def test_sealed_never_rations(sealed: tuple[list[State], int, tuple]) -> None:
    # The central P2.2 numerical check: a *clamped* finite pool stays rationed == 0
    # purely from FvCB's Ci-shutoff (the draw self-limits as Ci → Γ*), never the Euler
    # backstop. Phase 1 dodged this with an unclamped boundary; the chamber re-clamps.
    _, total_rationed, _ = sealed
    assert total_rationed == 0


def test_sealed_has_no_extinction_events(
    sealed: tuple[list[State], int, tuple],
) -> None:
    _, _, events = sealed
    assert events == ()


def test_sealed_pool_draws_down_then_refills(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Step 3 closed the gas loop: respiration returns CO₂ to the pool, so the pool is no
    # longer monotone (Step 2's open-loop draw-down). It still nets a real draw-down
    # (carbon accumulates in living biomass + the litter sink) and reaches a minimum
    # below its fill, but *rises* on deficit days when maintenance burns biomass to CO₂.
    # The presence of refill steps is the closed-loop return path made observable.
    states, _, _ = sealed
    pools = [s.stocks[CARBON_POOL].amount for s in states]
    assert pools[-1] < pools[0]  # net draw-down over the season
    assert min(pools) < pools[0]  # drew below the initial fill
    refills = sum(1 for a, b in zip(pools, pools[1:], strict=False) if b > a)
    assert refills > 0  # respiration returned CO₂ to the pool (the closed loop)


def test_sealed_ci_falls_meaningfully(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # NON-VACUOUS draw-down: Ci falls well below half its initial value at its minimum
    # (the probe lands ~0.37×). The closed loop refills the pool, so Ci is no longer
    # monotone (it recovers) — we check the *minimum*, not every step. A chamber so big
    # that Ci barely moved would pass rationed == 0 trivially and verify nothing — this
    # guards against that.
    states, _, _ = sealed
    scenario = SeasonScenario(sealed=True)
    cis = [
        ci_from_co2_pool(
            s.stocks[CARBON_POOL].amount,
            air_mol=scenario.chamber_air_mol,
            ci_ratio=scenario.ci_ratio,
        )
        for s in states
    ]
    assert min(cis) < 0.5 * cis[0]


def test_sealed_assimilation_rises_then_declines(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The emergent feedback's payoff: gross assimilation happens early (liveness — not a
    # dead trajectory) and then collapses by orders of magnitude as the pool draws down
    # (Ci falls) and the plant senesces. The collapse is asserted at the post-peak
    # TROUGH, not the end: Step 3 closed the *gas* loop and Step 5 closes the *carbon*
    # loop (decomposer respiration refills the CO₂ pool), so by season's end the pool
    # has refilled and GASS partially RECOVERS (~4% of peak) — the closed-loop reality,
    # not the Step-2 open-loop decline-to-~0. So the feedback is the deep mid-season
    # trough (GASS hits 0 when Ci ≤ Γ* / dark winter), not a stay-collapsed end.
    # Recompute GASS from each snapshot.
    states, _, _ = sealed
    scenario = SeasonScenario(sealed=True)
    resolver = weather_resolver(_weather(), scenario)
    ctx = _carbon_context(scenario)
    gass = [ctx.budget(s, resolver.bind(s, 1.0))[0] for s in states]
    peak = max(gass)
    peak_idx = gass.index(peak)
    assert peak > 1e-3  # the plant did fix carbon (liveness)
    # after peaking, assimilation collapses ≥ 2 orders at the trough (the feedback) ...
    assert min(gass[peak_idx:]) < 1e-2 * peak
    # ... and the closed carbon loop refills the pool, so GASS recovers above the trough
    # by season's end (the closed-loop signature — an open-loop decline never would).
    assert gass[-1] > 1e-2 * peak


def test_sealed_conserves_total_carbon(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The sealed chamber has no boundary carbon SOURCE (the pool is internal), so total
    # CARBON across all stocks — pool + organs + litter_sink + loss_sink — is invariant.
    # This is the every-step gate's claim, asserted end-to-end. (Step 3 closed the *gas*
    # loop — respiration returns CO₂ to the pool, no co2_resp sink — but the carbon loop
    # is still open: senescence leaks organ carbon to the litter_sink boundary until the
    # decomposer lands at Step 4. No carbon is created or destroyed either way.)
    states, _, _ = sealed
    total0 = _total_carbon(states[0])
    for s in states:
        assert math.isclose(_total_carbon(s), total0, rel_tol=1e-12, abs_tol=1e-12)


# --- determinism + flow-order independence on the sealed path (#7/#15) -------
# Mirror the open-field season's guards over the new sealed wiring (the carbon_pool
# stock + the co2_pool shared resolver entry): a stated Phase-2 invariant ("sealed runs
# are bit-identical and registration-order-independent"). The full sealed golden lands
# at Step 7; these pin the engine invariants for the Step-2 surface now.
def test_sealed_is_deterministic() -> None:
    a, _, _ = _run_sealed(SeasonScenario(sealed=True))
    b, _, _ = _run_sealed(SeasonScenario(sealed=True))
    assert a[-1].aux == b[-1].aux
    for sid, stock in a[-1].stocks.items():
        assert stock.amount == b[-1].stocks[sid].amount  # bit-identical


def test_sealed_is_flow_registration_order_independent() -> None:
    # The registry sorts flows by canonical id, so a reversed registration order gives a
    # bit-identical sealed trajectory — including the carbon_pool draw-down.
    scenario = SeasonScenario(sealed=True)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    forward, _, _ = run_season(
        EulerIntegrator(registry), state, resolver, 1.0, len(_weather())
    )
    shuffled = Registry(
        list(reversed(registry.flows)),
        state.stocks,
        aux_processes=registry.aux_processes,
    )
    state2, _ = build_season(scenario)
    reversed_run, _, _ = run_season(
        EulerIntegrator(shuffled), state2, resolver, 1.0, len(_weather())
    )
    for sid, stock in forward[-1].stocks.items():
        assert stock.amount == reversed_run[-1].stocks[sid].amount


def test_open_field_unchanged_by_chamber_additions() -> None:
    # The default (open field) keeps the unclamped co2_atmos boundary source and grows
    # no carbon_pool — the Phase-1 assembly (and its regression golden) is untouched.
    state, _ = build_season(SeasonScenario())
    assert CO2_ATMOS in state.stocks
    assert CARBON_POOL not in state.stocks
