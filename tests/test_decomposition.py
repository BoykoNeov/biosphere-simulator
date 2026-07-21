"""Phase-2 Step-4 tests: litter + decomposition (CARBON only; the first decomposer).

Step 4 promotes the Phase-1 ``litter_sink`` BOUNDARY to a finite ``litter_carbon`` POOL
(senescence-fed) and adds the ``Decomposition`` flow ``litter_carbon → microbial`` — a
first-order donor-controlled decay (Olson 1963), self-limiting like senescence. It is
deliberately **single-currency CARBON**: aerobic decomposition genuinely consumes O₂,
but in this model that is gate-forced (CO₂ into the ``{CARBON:1, OXYGEN:2}`` pool drags
2 oxygens pure-carbon litter cannot supply), so the CO₂-releasing, O₂-consuming
microbial respiration is **Step 5** — a pure addition. See ``decomposition.py`` and the
plan's Step-4 design.

Three layers:

* **Rate law** — ``decomposition_flux`` is ``k·litter`` (→ 0 as litter → 0).
* **Flow level** — ``Decomposition`` withdraws the decayed carbon from the litter POOL
  and deposits the *same* amount into microbial biomass; it balances CARBON and touches
  no other quantity (single-currency).
* **Integration (the sealed season)** — litter accumulates (senescence) then is drained
  into microbial biomass, which is in turn fed AND respired (Step 5 supersedes the
  Step-4 monotone-growth claim: microbial respiration now burns it back to CO₂), total
  CARBON is conserved exactly (the sealed chamber has no boundary carbon source/sink),
  OXYGEN stays exactly conserved *through* the microbial O₂ sink (PQ=1), and
  ``rationed == 0`` (the draws self-limit). The open field grows no decomposer stocks
  and keeps ``litter_sink`` (the regression golden's path). The microbial-respiration
  leg behaviour itself is pinned in ``test_microbial_respiration.py``.

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.decomposition import (
    Decomposition,
    DecompositionParams,
    decomposition_flux,
)
from domains.biosphere.loader import load_decomposition_params
from domains.biosphere.season import (
    CARBON_POOL,
    LITTER_CARBON,
    LITTER_SINK,
    MICROBIAL_CARBON,
    O2_POOL,
    SeasonScenario,
    build_season,
    run_season,
    weather_resolver,
)
from simcore.environment import SourceResolver
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

_WEATHER_FIXTURE = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"

_BIO = DomainId("biosphere")
_LITTER = StockId("biosphere.litter_carbon")
_MICROBIAL = StockId("biosphere.microbial_carbon")


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- rate law ----------------------------------------------------------------
def test_flux_is_first_order_in_litter() -> None:
    # decay = k · litter (Olson 1963): a hand value + linearity in the standing litter.
    assert math.isclose(
        decomposition_flux(2.5, decomposition_rate=0.02), 0.05, rel_tol=1e-12
    )
    assert math.isclose(
        decomposition_flux(5.0, decomposition_rate=0.02), 0.10, rel_tol=1e-12
    )


def test_flux_is_zero_at_zero_litter() -> None:
    # Self-limiting: no standing litter ⇒ no decomposition (positivity is structural).
    assert decomposition_flux(0.0, decomposition_rate=0.02) == 0.0


# --- flow level --------------------------------------------------------------
def _state(*, litter: float, microbial: float = 0.0) -> State:
    carbon = canonical_unit(Quantity.CARBON)
    stocks = {
        _LITTER: Stock(
            id=_LITTER,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=litter,
            kind=StockKind.POOL,
        ),
        _MICROBIAL: Stock(
            id=_MICROBIAL,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=microbial,
            kind=StockKind.POPULATION,
            extinction_threshold=0.0,
        ),
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _decomposition(rate: float = 0.02) -> Decomposition:
    return Decomposition(
        FlowId("biosphere.decomposition"),
        0,
        litter_carbon=_LITTER,
        microbial_carbon=_MICROBIAL,
        params=DecompositionParams(decomposition_rate=rate),
    )


def _env(state: State, dt: float):
    # Decomposition reads no forcing; a trivial bound resolver satisfies the signature.
    return SourceResolver(forcings={}).bind(state, dt)


def test_decomposition_transfers_litter_to_microbial() -> None:
    # The decayed carbon leaves litter and lands in microbial biomass — the SAME amount,
    # so the transfer is conservative by construction.
    state = _state(litter=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _decomposition().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    decayed = 0.02 * 2.0
    assert math.isclose(legs[_LITTER], -decayed, rel_tol=1e-12)  # withdrawn from litter
    assert math.isclose(legs[_MICROBIAL], decayed, rel_tol=1e-12)  # into microbes


def test_decomposition_balances_carbon_only() -> None:
    # Single-currency CARBON: the flow balances CARBON and touches no other quantity
    # (both pools are pure carbon — no oxygen, the Step-4/Step-5 split).
    state = _state(litter=2.0)
    result = _decomposition().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    residual = per_quantity_residual(result, state.stocks)
    assert set(residual) == {Quantity.CARBON}  # OXYGEN/WATER/NITROGEN untouched


def test_decomposition_is_dt_linear() -> None:
    # flux = daily·dt — the increment-form contract (RK4 order; here Euler-daily).
    state = _state(litter=2.0)
    half = {
        leg.stock: leg.amount
        for leg in _decomposition().evaluate(state, _env(state, 0.5), 0.5).legs
    }
    full = {
        leg.stock: leg.amount
        for leg in _decomposition().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert math.isclose(full[_MICROBIAL], 2.0 * half[_MICROBIAL], rel_tol=1e-12)


def test_decomposition_self_limits_at_zero_litter() -> None:
    # No standing litter ⇒ zero-amount legs (a clamped POOL draw never goes negative).
    state = _state(litter=0.0)
    legs = _decomposition().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_rate() -> None:
    # 0.011/day since the scope-B decomposer calibration (was 0.02; 7.3 -> 4.0/yr,
    # docs/plans/post-roadmap-decomposer-calibration.md).
    assert load_decomposition_params().decomposition_rate == 0.011


def test_loader_rejects_negative_rate(tmp_path: Path) -> None:
    bad = tmp_path / "decomposition.yaml"
    bad.write_text(
        "name: chamber\nprocess: decomposition\nparameters:\n"
        '  decomposition_rate:\n    value: -0.01\n    unit: "1/day"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="decomposition_rate must be >= 0"):
        load_decomposition_params(bad)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    bad = tmp_path / "decomposition.yaml"
    bad.write_text(
        "name: chamber\nprocess: decomposition\nparameters:\n"
        '  decomposition_rate:\n    value: 0.02\n    unit: "1/year"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="must be declared in"):
        load_decomposition_params(bad)


# --- integration: the sealed season -----------------------------------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    scenario = SeasonScenario(sealed=True)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, len(_weather()))


def _total_carbon(s: State) -> float:
    return sum(
        stock.amount * stock.composition.get(Quantity.CARBON, 0.0)
        for stock in s.stocks.values()
    )


def _total_oxygen(s: State) -> float:
    return sum(
        stock.amount * stock.composition.get(Quantity.OXYGEN, 0.0)
        for stock in s.stocks.values()
    )


def test_sealed_litter_accumulates_then_drains(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Senescence feeds the litter POOL; decomposition drains it. The pool is
    # non-vacuous: it rises above its 0 start AND is drawn down on some step (not a
    # monotone pile-up; decomposition removes carbon) — the Step-4 emergent behaviour.
    states, _, _ = sealed
    litter = [s.stocks[LITTER_CARBON].amount for s in states]
    assert litter[0] == 0.0
    assert max(litter) > 1e-3  # senescence genuinely builds standing litter
    # decomposition removes carbon from the pool on some step (a strict draw-down).
    assert any(b < a for a, b in zip(litter, litter[1:], strict=False))


def test_sealed_microbial_is_fed_and_respired(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Step 5 supersedes the Step-4 monotone-growth claim: microbial biomass now has BOTH
    # a source (decomposition deposits) and a sink (microbial respiration burns it to
    # CO₂), so it is no longer monotone — it accumulates AND is genuinely drawn down on
    # some step. (The decomposer mirror of the Step-2→Step-3 open→closed transition; the
    # detailed respiration-leg behaviour is pinned in test_microbial_respiration.py.)
    states, _, _ = sealed
    mic = [s.stocks[MICROBIAL_CARBON].amount for s in states]
    assert mic[0] == 0.0
    assert max(mic) > 1e-3  # decomposition genuinely built standing microbial biomass
    # respiration genuinely withdraws carbon on some step (a strict draw-down).
    assert any(b < a for a, b in zip(mic, mic[1:], strict=False))


def test_sealed_conserves_carbon_exactly(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The strong Step-4 invariant: the sealed chamber has NO boundary carbon source/sink
    # (gas exchange is the closed CO₂ pool; senescence + decomposition are internal
    # CARBON transfers among organs/litter/microbial), so total CARBON is invariant to
    # float — exercising the every-step gate end-to-end through the new decomposer path.
    states, _, _ = sealed
    c0 = _total_carbon(states[0])
    for s in states:
        assert math.isclose(_total_carbon(s), c0, rel_tol=0.0, abs_tol=1e-12)


def test_sealed_conserves_oxygen_through_microbial_respiration(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Decomposition itself is CARBON-only, but microbial respiration (Step 5) now DOES
    # touch the gas system (microbial_C + O₂ → CO₂). Total OXYGEN (2·(CO₂+O₂)) stays
    # exactly conserved regardless: every mol C respired returns a CO₂ (carrying 2
    # oxygens) supplied by 1 consumed O₂ (−2 oxygens) — the PQ=1 composition fold nets
    # OXYGEN to zero. The Step-3 ``2·(CO₂+O₂)`` invariant survives the new O₂ sink
    # (a mis-stoichiometric microbial respiration leg would break this exact check).
    states, _, _ = sealed
    ox0 = _total_oxygen(states[0])
    for s in states:
        assert math.isclose(_total_oxygen(s), ox0, rel_tol=0.0, abs_tol=1e-9)


def test_sealed_never_rations(sealed: tuple[list[State], int, tuple]) -> None:
    # rationed == 0 holds with the decomposer present: the litter draw (k·litter·dt,
    # k·dt = 0.02 ≪ 1) self-limits against the start-of-step pool, so the Euler backstop
    # never fires (the senescence/respiration positivity pattern).
    _, total_rationed, _ = sealed
    assert total_rationed == 0


def test_sealed_no_extinction(sealed: tuple[list[State], int, tuple]) -> None:
    # microbial_carbon is a POPULATION at threshold 0 but only grows (sink-only this
    # step), so it never snaps; no organ goes extinct on the well-fed sealed run.
    _, _, events = sealed
    assert events == ()


# --- open field is untouched (the regression golden's path) ------------------
def test_open_field_has_no_decomposer_stocks() -> None:
    # The open field keeps the Phase-1 ``litter_sink`` BOUNDARY and grows no
    # ``litter_carbon``/``microbial_carbon`` — the sealed-only decomposer is gated on
    # ``scenario.sealed`` so the regression golden's path is byte-identical.
    state, _ = build_season(SeasonScenario(sealed=False))
    assert LITTER_SINK in state.stocks
    assert LITTER_CARBON not in state.stocks
    assert MICROBIAL_CARBON not in state.stocks
    assert CARBON_POOL not in state.stocks
    assert O2_POOL not in state.stocks
