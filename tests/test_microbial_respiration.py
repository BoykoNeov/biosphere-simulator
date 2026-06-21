"""Phase-2 Step-5 tests: microbial respiration as a multi-quantity (CARBON+OXYGEN) flow.

Step 5 closes the carbon loop. Step 4 left microbial biomass a pure intermediate that
only grew (litter decayed into it; nothing withdrew it); Step 5 adds aerobic microbial
respiration ``microbial_C + O₂ → CO₂`` — the decomposer's mirror of plant maintenance
respiration's biomass-burned shortfall, and the chamber's decomposer O₂ sink (the
Biosphere-2 soil-respiration mechanism). It is the gate-forced O₂ coupling the Step-4
doc deferred to here: CO₂ into the ``{CARBON:1, OXYGEN:2}`` pool drags 2 oxygens the
pure-carbon microbes cannot supply, so they come from the O₂ pool (PQ=1). Three layers:

* **Rate law** — ``microbial_respiration_flux`` is ``m_resp·microbial`` (→ 0 as
  microbial → 0; positivity structural).
* **Flow level** — ``MicrobialRespiration`` withdraws the respired carbon from the
  microbial POPULATION, returns the SAME amount as CO₂ to the pool, and consumes the
  same amount of O₂; it balances CARBON *and* OXYGEN in one flow (the P2.1 composition
  fold) and touches no WATER/NITROGEN. Always three legs (sealed-only; ``microbial ≠``
  the pool, so no ``source == sink`` netting).
* **Integration (the sealed season)** — microbial biomass is now genuinely respired (a
  strict draw-down, not the Step-4 monotone pile-up), total CARBON stays conserved
  exactly (microbial → pool is an internal carbon transfer) and total OXYGEN stays
  conserved exactly *through* the new O₂ sink (PQ=1), ``rationed == 0`` holds with the
  microbial O₂ draw present, and O₂ stays far from its floor (the ``f_O2``-deferral
  guard; see ``microbial_respiration.py`` — O₂ self-limitation lands at Step 7's run).

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest

from domains.biosphere.loader import load_microbial_respiration_params
from domains.biosphere.microbial_respiration import (
    MicrobialRespiration,
    MicrobialRespirationParams,
    microbial_respiration_flux,
)
from domains.biosphere.season import (
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
_MICROBIAL = StockId("biosphere.microbial_carbon")
_CO2_POOL = StockId("biosphere.carbon_pool")
_O2_POOL = StockId("biosphere.o2_pool")


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- rate law ----------------------------------------------------------------
def test_flux_is_first_order_in_microbial() -> None:
    # resp = m_resp · microbial: a hand value + linearity in the standing biomass.
    assert math.isclose(
        microbial_respiration_flux(2.0, microbial_respiration_rate=0.05),
        0.10,
        rel_tol=1e-12,
    )
    assert math.isclose(
        microbial_respiration_flux(4.0, microbial_respiration_rate=0.05),
        0.20,
        rel_tol=1e-12,
    )


def test_flux_is_zero_at_zero_microbial() -> None:
    # Self-limiting: no standing microbes ⇒ no respiration (positivity is structural).
    assert microbial_respiration_flux(0.0, microbial_respiration_rate=0.05) == 0.0


# --- flow level --------------------------------------------------------------
def _state(*, microbial: float, co2: float = 0.357, o2: float = 210.0) -> State:
    carbon = canonical_unit(Quantity.CARBON)
    oxygen = canonical_unit(Quantity.OXYGEN)
    stocks = {
        _MICROBIAL: Stock(
            id=_MICROBIAL,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=microbial,
            kind=StockKind.POPULATION,
            extinction_threshold=0.0,
        ),
        # The CO₂ pool: a true molecular stock — 1 mol C + 2 mol O per mol CO₂.
        _CO2_POOL: Stock(
            id=_CO2_POOL,
            domain=_BIO,
            quantity=Quantity.CARBON,
            unit=carbon,
            amount=co2,
            kind=StockKind.POOL,
            composition={Quantity.CARBON: 1.0, Quantity.OXYGEN: 2.0},
        ),
        # The O₂ counterpart: 2 mol OXYGEN per mol O₂.
        _O2_POOL: Stock(
            id=_O2_POOL,
            domain=_BIO,
            quantity=Quantity.OXYGEN,
            unit=oxygen,
            amount=o2,
            kind=StockKind.POOL,
            composition={Quantity.OXYGEN: 2.0},
        ),
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _respiration(
    rate: float = 0.05, *, o2_half_saturation: float = 0.0
) -> MicrobialRespiration:
    # o2_half_saturation defaults to 0 (f_O2 ≡ 1 for O₂ > 0) so the rate-law/balance
    # unit tests isolate the base flux; f_O2 gets its own dedicated tests.
    return MicrobialRespiration(
        FlowId("biosphere.microbial_respiration"),
        0,
        microbial_carbon=_MICROBIAL,
        co2_pool=_CO2_POOL,
        o2_pool=_O2_POOL,
        params=MicrobialRespirationParams(
            microbial_respiration_rate=rate, o2_half_saturation=o2_half_saturation
        ),
        air_mol=1000.0,
    )


def _env(state: State, dt: float):
    # Microbial respiration reads no forcing; a trivial bound resolver suffices.
    return SourceResolver(forcings={}).bind(state, dt)


def test_respiration_burns_microbial_to_co2_consuming_o2() -> None:
    # PQ=1: each mol C respired leaves microbial biomass, returns to the pool as CO₂,
    # and consumes 1 mol O₂ — all the SAME ``respired`` magnitude.
    state = _state(microbial=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _respiration().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    respired = 0.05 * 2.0
    assert math.isclose(legs[_MICROBIAL], -respired, rel_tol=1e-12)  # burned biomass
    assert math.isclose(
        legs[_CO2_POOL], respired, rel_tol=1e-12
    )  # CO₂ returned to pool
    assert math.isclose(
        legs[_O2_POOL], -respired, rel_tol=1e-12
    )  # O₂ consumed = C burned


def test_respiration_balances_carbon_and_oxygen() -> None:
    # The whole point of P2.1: ONE flow balances CARBON *and* OXYGEN via the composition
    # fold (the CO₂ pool's 2 oxygens are supplied by the consumed O₂, not created); it
    # touches no WATER/NITROGEN.
    state = _state(microbial=2.0)
    result = _respiration().evaluate(state, _env(state, 1.0), 1.0)
    assert_flow_balanced(result, state.stocks)
    residual = per_quantity_residual(result, state.stocks)
    assert set(residual) == {
        Quantity.CARBON,
        Quantity.OXYGEN,
    }  # WATER/NITROGEN untouched


def test_respiration_is_dt_linear() -> None:
    # flux = daily·dt — the increment-form contract (RK4 order; here Euler-daily).
    state = _state(microbial=2.0)
    half = {
        leg.stock: leg.amount
        for leg in _respiration().evaluate(state, _env(state, 0.5), 0.5).legs
    }
    full = {
        leg.stock: leg.amount
        for leg in _respiration().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert math.isclose(full[_CO2_POOL], 2.0 * half[_CO2_POOL], rel_tol=1e-12)


def test_respiration_self_limits_at_zero_microbial() -> None:
    # No standing microbes ⇒ zero-amount legs (a clamped POPULATION draw never goes
    # negative; the O₂/CO₂ legs are zero too, so no spurious gas exchange).
    state = _state(microbial=0.0)
    legs = _respiration().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_rate() -> None:
    assert load_microbial_respiration_params().microbial_respiration_rate == 0.05


def test_loader_rejects_negative_rate(tmp_path: Path) -> None:
    bad = tmp_path / "microbial_respiration.yaml"
    bad.write_text(
        "name: chamber\nprocess: microbial_respiration\nparameters:\n"
        '  microbial_respiration_rate:\n    value: -0.01\n    unit: "1/day"\n'
        '    source: "test"\n'
        '  o2_half_saturation:\n    value: 0.001\n    unit: "mol/mol"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="microbial_respiration_rate must be >= 0"):
        load_microbial_respiration_params(bad)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    bad = tmp_path / "microbial_respiration.yaml"
    bad.write_text(
        "name: chamber\nprocess: microbial_respiration\nparameters:\n"
        '  microbial_respiration_rate:\n    value: 0.05\n    unit: "1/year"\n'
        '    source: "test"\n'
        '  o2_half_saturation:\n    value: 0.001\n    unit: "mol/mol"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="must be declared in"):
        load_microbial_respiration_params(bad)


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


def test_sealed_microbial_respiration_is_active(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The Step-5 emergent: microbial biomass is now genuinely respired — drawn down on
    # some step (the sink runs), no longer the Step-4 monotone pile-up. The carbon loop
    # closes: litter → microbial → CO₂ → photosynthesis.
    states, _, _ = sealed
    mic = [s.stocks[MICROBIAL_CARBON].amount for s in states]
    assert max(mic) > 1e-3  # decomposition genuinely fed the microbes
    assert any(b < a for a, b in zip(mic, mic[1:], strict=False))  # respiration drains


def test_sealed_conserves_carbon_exactly(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Microbial respiration moves carbon microbial → CO₂ pool, an internal transfer; the
    # sealed chamber has no boundary carbon source/sink, so total CARBON is invariant to
    # float — the every-step gate end-to-end through the new gas flux.
    states, _, _ = sealed
    c0 = _total_carbon(states[0])
    for s in states:
        assert math.isclose(_total_carbon(s), c0, rel_tol=0.0, abs_tol=1e-12)


def test_sealed_conserves_oxygen_exactly(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # Total OXYGEN = 2·(CO₂+O₂) stays invariant to float though microbial respiration
    # now consumes O₂: the CO₂ returned carries 2 oxygens supplied by the 1 consumed O₂
    # (PQ=1; the composition fold). The Step-3 invariant survives the O₂ sink.
    states, _, _ = sealed
    ox0 = _total_oxygen(states[0])
    for s in states:
        assert math.isclose(_total_oxygen(s), ox0, rel_tol=0.0, abs_tol=1e-9)


def test_sealed_never_rations_with_microbial_o2_draw(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # rationed == 0 still holds now that microbial respiration ALSO draws the finite O₂
    # pool: the microbial draw self-limits (∝ its own amount, m_resp·dt ≪ 1) and O₂
    # stays far from its floor, so the Euler backstop never fires (the central check).
    _, total_rationed, _ = sealed
    assert total_rationed == 0


def test_sealed_no_extinction(sealed: tuple[list[State], int, tuple]) -> None:
    # microbial_carbon is a POPULATION (threshold 0) now WITH a sink (respiration), but
    # the self-limiting draw keeps it positive, so it never snaps; nothing goes extinct.
    _, _, events = sealed
    assert events == ()


def test_sealed_o2_stays_far_from_rationing(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The ``f_O2``-deferral GUARD, extended to the microbial O₂ sink. The realistic O₂
    # fill (~210 mol) is ~4 orders above the standing-microbial-driven O₂ draw, so O₂
    # never approaches arbitration rationing — WHY no O₂ self-limitation (``f_O2``) is
    # needed yet (it lands at Step 7's depleting run). If a future change pushes O₂
    # toward zero, THIS test breaks and flags that ``f_O2`` has become load-bearing.
    states, _, _ = sealed
    o2 = [s.stocks[O2_POOL].amount for s in states]
    assert (
        min(o2) > 0.5 * o2[0]
    )  # stayed within 2× of the fill — nowhere near rationing
