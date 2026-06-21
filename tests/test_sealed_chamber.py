"""Phase-2 Step-7 tests: the sealed-chamber integration + closed-system validation.

The Phase-2 capstone. Assembles the canonical **multi-year sealed run**
(``SEALED_CHAMBER_SCENARIO``, the season weather tiled ``SEALED_CHAMBER_YEARS×``) and
validates the closed-system phenomena Phase 2 set out to reproduce — with **zero**
``simcore`` changes (``f_O2`` is domain-side, the scenario is ``season.py``, the golden
is ``test_regression_sealed_season.py``).

The run is a deliberately **O₂-poor** chamber (2 mol O₂ in 1000 mol air) seeded with
standing litter (3 mol C). Its emergent three-act story (no control code):

1. **Acute O₂ depletion** — the seeded litter decomposes → microbial respiration draws
   the small O₂ pool down ~99 % to an acute trough (the Biosphere-2 soil-respiration
   O₂-depletion failure mode). ``f_O2`` self-limits the draw so ``rationed == 0`` holds
   on the depleting pool (positivity from kinetics — the central Step-7 check; an
   un-throttled control *does* ration, pinned below).
2. **Producer-driven recovery** — the live plant photosynthesizes at the elevated CO₂
   (Ci ≈ 1600 µmol mol⁻¹) and transiently **refills** O₂ before it matures and dies —
   the producer fighting the decomposer, an emergent swing.
3. **Secular CO₂-rich end state** — the plant dead, decomposition wins; the chamber
   settles O₂-depleted / CO₂-rich (Ci ≈ 1140).

Throughout: all four quantities (CARBON/OXYGEN/WATER/NITROGEN) conserved float-exact;
the O₂↔CO₂ anti-correlation is **exact** (``CO₂_mol + O₂_mol = const``); deterministic.
``f_N ≡ 1`` here (N stays non-limiting), so the N-limited regime is a documented
deferral (Phase 3) — verified, not assumed.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from dataclasses import replace
from pathlib import Path
from unittest.mock import patch

import pytest

import domains.biosphere.plants as plants
import domains.biosphere.soil as soil
from domains.biosphere.loader import (
    load_microbial_respiration_params,
    load_nitrogen_params,
    load_respiration_params,
)
from domains.biosphere.nitrogen import nitrogen_stress_factor
from domains.biosphere.season import (
    CARBON_POOL,
    LEAF_C,
    O2_POOL,
    PLANT_N,
    ROOT_C,
    SEALED_CHAMBER_SCENARIO,
    SEALED_CHAMBER_YEARS,
    STEM_C,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity
from simcore.state import State

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


def _run_canonical(*, k_o2: float | None = None) -> tuple[list[State], int, tuple]:
    """Run the canonical sealed multi-year season (the single source of truth).

    Tiles the season weather ``SEALED_CHAMBER_YEARS×`` (``_table`` reads the rows in
    order, so a repeated list cycles the seasonal forcing). ``k_o2`` overrides the
    committed O₂ half-saturation in both respiration params (used by the load-bearing
    control: ``k_o2 = 0`` disables ``f_O2``) by patching the builders' loaders at their
    new homes (P3.2 split: ``load_respiration_params`` is called in ``plants`` via
    ``_carbon_context``; ``load_microbial_respiration_params`` in ``soil``).
    """
    weather = _weather() * SEALED_CHAMBER_YEARS
    steps = len(weather)
    if k_o2 is None:
        state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    else:
        resp = replace(load_respiration_params(), o2_half_saturation=k_o2)
        mic = replace(load_microbial_respiration_params(), o2_half_saturation=k_o2)
        with (
            patch.object(plants, "load_respiration_params", lambda: resp),
            patch.object(soil, "load_microbial_respiration_params", lambda: mic),
        ):
            state, registry = build_season(SEALED_CHAMBER_SCENARIO)
    resolver = weather_resolver(weather, SEALED_CHAMBER_SCENARIO)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, steps)


@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    return _run_canonical()


def _total(s: State, q: Quantity) -> float:
    return sum(
        stock.amount * stock.composition.get(q, 0.0) for stock in s.stocks.values()
    )


# --- conservation: all four quantities, float-exact every step ---------------
@pytest.mark.parametrize(
    ("quantity", "abs_tol"),
    [
        (Quantity.CARBON, 1e-12),
        (Quantity.OXYGEN, 1e-12),
        (Quantity.WATER, 1e-7),  # soil_water ≈ O(1e3) kg ⇒ looser absolute band
        (Quantity.NITROGEN, 1e-9),  # soil_n-dominated O(1e2)
    ],
)
def test_sealed_conserves_every_quantity(
    sealed: tuple[list[State], int, tuple], quantity: Quantity, abs_tol: float
) -> None:
    # The sealed chamber is closed for CARBON/OXYGEN/NITROGEN (no boundary source/sink);
    # WATER cycles soil↔vapor with the unclamped irrigation/vapor boundaries, so its
    # *total over all stocks incl. boundaries* is what is invariant. Every step the gate
    # holds end-to-end through the f_O2-throttled gas fluxes over the multi-year run.
    states, _, _ = sealed
    q0 = _total(states[0], quantity)
    for s in states:
        assert math.isclose(_total(s, quantity), q0, rel_tol=0.0, abs_tol=abs_tol)


# --- stability: rationed == 0, no extinction, over the multi-year horizon -----
def test_sealed_never_rations(sealed: tuple[list[State], int, tuple]) -> None:
    # The central Step-7 check: rationed == 0 survives the DEPLETING O₂ pool because
    # f_O2 shuts the O₂ draw off before it over-runs the pool (kinetics, not the
    # backstop). That f_O2 is load-bearing here is pinned by the control test below.
    _, rationed, _ = sealed
    assert rationed == 0


def test_sealed_no_extinction(sealed: tuple[list[State], int, tuple]) -> None:
    # POPULATION pools (organs, microbes) self-limit off zero; nothing snaps to the
    # loss-sink, so the chamber stays genuinely closed (no boundary carbon leak).
    _, _, events = sealed
    assert events == ()


# --- the phenomenon: O₂ depletion (the Biosphere-2 failure mode) --------------
def test_sealed_o2_depletes_then_stays_positive(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # O₂ is drawn down a clear fraction (≥ 95 %) toward an acute trough — the depletion
    # phenomenon — yet stays strictly positive (the f_O2 floor; no literal anoxia / no
    # negative pool). The headline closed-system result.
    states, _, _ = sealed
    o2 = [s.stocks[O2_POOL].amount for s in states]
    assert min(o2) > 0.0  # f_O2 self-limit keeps the pool off zero
    assert min(o2) < 0.05 * o2[0]  # ≥ 95 % depletion (acute trough ≈ 0.2 % of the fill)


def test_sealed_o2_co2_anticorrelation_is_exact(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The exact closed-loop signature: only CO₂ and O₂ carry OXYGEN, so OXYGEN
    # conservation means CO₂_mol + O₂_mol = const. Pinned tight (not just qualitative):
    # every mol O₂ consumed appears as a mol CO₂ and vice-versa, through the crash AND
    # the recovery.
    states, _, _ = sealed
    g0 = states[0].stocks[CARBON_POOL].amount + states[0].stocks[O2_POOL].amount
    for s in states:
        g = s.stocks[CARBON_POOL].amount + s.stocks[O2_POOL].amount
        assert math.isclose(g, g0, rel_tol=0.0, abs_tol=1e-12)


def test_sealed_producer_recovers_o2_after_trough(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The emergent producer–decomposer swing (no control code): after the acute trough
    # the live plant photosynthesises at the high CO₂ and REFILLS O₂ before it dies —
    # so O₂ is non-monotone (a local trough then a rise), the dynamic anti-correlation.
    states, _, _ = sealed
    o2 = [s.stocks[O2_POOL].amount for s in states]
    trough = min(range(len(o2)), key=lambda i: o2[i])
    assert max(o2[trough:]) > o2[trough] + 0.1  # recovers well above the trough


# --- f_O2 is load-bearing: an un-throttled control rations -------------------
def test_sealed_f_o2_is_load_bearing(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The proof that f_O2 (not the substrate self-limit alone) is what keeps rationed ==
    # 0 on the depleting pool: re-run with f_O2 DISABLED (k_o2 = 0) and the backstop
    # fires (the O₂ draw over-runs the depleted pool). With f_O2 on it never does.
    _, rationed_on, _ = sealed
    assert rationed_on == 0
    _, rationed_off, _ = _run_canonical(k_o2=0.0)
    assert rationed_off > 0  # the control rations — f_O2 is genuinely load-bearing


# --- determinism -------------------------------------------------------------
def test_sealed_run_is_deterministic(sealed: tuple[list[State], int, tuple]) -> None:
    # Bit-identical on a re-run (registration-order-independent; the golden's premise).
    states, rationed, events = sealed
    states2, rationed2, events2 = _run_canonical()
    assert states2[-1] == states[-1]
    assert (rationed2, events2) == (rationed, events)


# --- f_N gravy: the N loop stays non-limiting here (decoupling verified) ------
def test_sealed_f_n_stays_one(sealed: tuple[list[State], int, tuple]) -> None:
    # The N return loop runs but f_N ≡ 1 (plant_n stays above the critical-N
    # concentration), so it has zero effect on the carbon trajectory; the deliverable is
    # the GAS-side phenomena. The N-limited regime (f_N < 1) is a documented Phase-3
    # deferral; verified here, not assumed.
    states, _, _ = sealed
    nitro = load_nitrogen_params()
    for s in states:
        biomass = (
            s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount
        )
        f_n = nitrogen_stress_factor(
            s.stocks[PLANT_N].amount,
            biomass,
            n_residual_per_mol_c=nitro.n_residual_per_mol_c,
            n_critical_per_mol_c=nitro.n_critical_per_mol_c,
        )
        assert f_n == 1.0
