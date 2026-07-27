"""Phase-2 Step-6 tests: the nitrogen return loop (senescence-N + net mineralization).

Step 6 closes the **nitrogen** cycle the way Steps 4–5 closed the carbon cycle. Phase 1
left ``plant_n`` monotone-growing (uptake fed it; nothing withdrew it) and refilled
``soil_n`` from an external ``n_source``. Step 6 returns plant N to the soil internally:

* **N-senescence** ``plant_n → litter_n`` — the plant sheds N into a finite ``litter_n``
  POOL (the N analogue of carbon senescence feeding ``litter_carbon``); first-order in
  plant_n, so ``plant_n`` is now drained (no longer monotone-growing).
* **Mineralization** ``litter_n → soil_n`` — decomposing litter releases mineral N back
  to the soil; direct first-order net mineralization (Stanford & Smith 1972).

Both are single-currency NITROGEN flows (no core change). Four layers:

* **Rate laws** — both fluxes are ``rate·donor`` (→ 0 as the donor → 0; positivity
  structural).
* **Flow level** — each flow transfers the same amount donor → receiver and balances
  NITROGEN only (no CARBON/OXYGEN/WATER residual).
* **Integration (the sealed season)** — ``litter_n`` accumulates then drains,
  ``plant_n`` is drained (declines from its start), total NITROGEN is conserved
  float-exact (an internal cycle soil_n → plant_n → litter_n → soil_n), ``rationed ==
  0``, no extinction.
* **The f_N=1 decoupling (the load-bearing claim, VERIFIED not asserted)** — at the PP
  fill ``plant_n`` stays ~1000× above the critical-N concentration, so ``f_N ≡ 1`` every
  step and the N loop has **zero effect on the carbon trajectory** (the deliverable is
  "N mass cycles and is conserved," not "emergent N feedback" — the N-limited regime is
  Step 7's sized run, mirroring the ``f_O2`` deferral). Recomputed per state and
  asserted ``== 1.0``; the bit-identical carbon run is additionally pinned by the
  *unchanged* prior sealed tests (``test_chamber`` / ``test_gas_exchange`` /
  ``test_decomposition`` / ``test_microbial_respiration``).

Pure-stdlib data path (reads the committed JSON weather fixture; no PCSE).
"""

import json
import math
from pathlib import Path

import pytest
from pydantic import ValidationError

from domains.biosphere.allocation import SenescenceParams
from domains.biosphere.loader import (
    load_mineralization_params,
    load_nitrogen_params,
)
from domains.biosphere.mineralization import (
    Mineralization,
    MineralizationParams,
    NitrogenSenescence,
    mineralization_flux,
    nitrogen_shedding_flux,
)
from domains.biosphere.nitrogen import NitrogenParams, nitrogen_stress_factor
from domains.biosphere.season import (
    LEAF_C,
    LITTER_N,
    PLANT_N,
    ROOT_C,
    STEM_C,
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
_PLANT_N = StockId("biosphere.plant_n")
_LITTER_N = StockId("biosphere.litter_n")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_SOIL_N = StockId("biosphere.soil_n")


def _weather() -> list[dict[str, float | str]]:
    return json.loads(_WEATHER_FIXTURE.read_text(encoding="utf-8"))["weather"]


# --- rate laws ---------------------------------------------------------------
def test_n_shedding_flux_is_proportional_to_the_senescing_carbon() -> None:
    # shed_N = min(tissue_conc, residual_conc) · shed_C — linear in the CARBON flux,
    # which is the point of the coupled form (the rate form it replaced never saw carbon
    # at all).
    assert math.isclose(
        nitrogen_shedding_flux(2.0, 1.0, 100.0, n_residual_per_mol_c=0.001),
        0.002,
        rel_tol=1e-12,
    )
    assert math.isclose(
        nitrogen_shedding_flux(4.0, 1.0, 100.0, n_residual_per_mol_c=0.001),
        0.004,
        rel_tol=1e-12,
    )


def test_n_shedding_sheds_at_the_residual_concentration_when_tissue_is_richer() -> None:
    # The remobilization story: a well-fed plant (tissue 1.0/100 = 0.01 kg N/mol C) does
    # not shed at its live concentration — it retains the difference and sheds only what
    # Van Hecke et al. (2020) measure as N left in mature straw (the cited n_residual).
    rich = nitrogen_shedding_flux(2.0, 1.0, 100.0, n_residual_per_mol_c=0.001)
    assert math.isclose(rich, 0.001 * 2.0, rel_tol=1e-12)  # residual, NOT 0.01


def test_n_shedding_falls_back_to_tissue_conc_when_already_at_residual() -> None:
    # Below residual there is nothing left to remobilize, so shedding runs at the ACTUAL
    # concentration — the min's other branch. tissue = 0.05/100 = 5e-4 < residual 1e-3.
    lean = nitrogen_shedding_flux(2.0, 0.05, 100.0, n_residual_per_mol_c=0.001)
    assert math.isclose(lean, 5e-4 * 2.0, rel_tol=1e-12)


def test_n_shedding_flux_is_zero_at_every_degenerate_input() -> None:
    # Self-limiting / never a divide-by-zero: no carbon shed, no plant N, no biomass.
    assert nitrogen_shedding_flux(0.0, 1.0, 100.0, n_residual_per_mol_c=0.001) == 0.0
    assert nitrogen_shedding_flux(2.0, 0.0, 100.0, n_residual_per_mol_c=0.001) == 0.0
    assert nitrogen_shedding_flux(2.0, 1.0, 0.0, n_residual_per_mol_c=0.001) == 0.0


def test_mineralization_flux_is_first_order_in_litter_n() -> None:
    # min = mineralization_rate · litter_n: a hand value + linearity in the standing N.
    assert math.isclose(
        mineralization_flux(2.0, mineralization_rate=0.03), 0.06, rel_tol=1e-12
    )
    assert math.isclose(
        mineralization_flux(5.0, mineralization_rate=0.03), 0.15, rel_tol=1e-12
    )


def test_mineralization_flux_is_zero_at_zero_litter_n() -> None:
    # Self-limiting: no standing litter N ⇒ no mineralization (positivity structural).
    assert mineralization_flux(0.0, mineralization_rate=0.03) == 0.0


# --- flow level --------------------------------------------------------------
def _n_pool(stock_id: StockId, amount: float) -> Stock:
    return Stock(
        id=stock_id,
        domain=_BIO,
        quantity=Quantity.NITROGEN,
        unit=canonical_unit(Quantity.NITROGEN),
        amount=amount,
        kind=StockKind.POOL,
    )


def _c_pool(stock_id: StockId, amount: float) -> Stock:
    return Stock(
        id=stock_id,
        domain=_BIO,
        quantity=Quantity.CARBON,
        unit=canonical_unit(Quantity.CARBON),
        amount=amount,
        kind=StockKind.POOL,
    )


def _state(
    *,
    plant_n: float = 0.5,
    litter_n: float = 0.1,
    soil_n: float = 100.0,
    leaf_c: float = 1.0,
) -> State:
    stocks = {
        _PLANT_N: _n_pool(_PLANT_N, plant_n),
        _LITTER_N: _n_pool(_LITTER_N, litter_n),
        _SOIL_N: _n_pool(_SOIL_N, soil_n),
        # The coupled N-shedding flow reads the organ carbon that is senescing.
        _LEAF_C: _c_pool(_LEAF_C, leaf_c),
        _STEM_C: _c_pool(_STEM_C, 0.0),
        _ROOT_C: _c_pool(_ROOT_C, 0.0),
    }
    return State(n=0, stocks=stocks, rng_seed=0)


def _params(*, mineral: float = 0.03) -> MineralizationParams:
    return MineralizationParams(mineralization_rate=mineral)


# The coupled N-shedding flow needs the organ carbon stocks and the senescence rates
# that
# drive it. rdr_leaf = 0.01 on 1.0 mol C of leaf gives shed_C = 0.01 mol C/day, with the
# stem/root rates zero so the arithmetic below stays a single hand-checkable term.
_SEN_PARAMS = SenescenceParams(rdr_leaf=0.01, rdr_stem=0.0, rdr_root=0.0)
_NITRO_PARAMS = NitrogenParams(
    max_uptake_capacity=0.0015,
    n_residual_per_mol_c=0.001,
    n_critical_per_mol_c=0.004,
    n_target_coefficient=0.05697,
    n_target_exponent=0.5,
    n_target_w_plateau=1.0,
    dm_kg_per_mol_c=0.026691111111111113,
)


def _senescence() -> NitrogenSenescence:
    return NitrogenSenescence(
        FlowId("biosphere.nitrogen_senescence"),
        0,
        plant_n=_PLANT_N,
        litter_n=_LITTER_N,
        leaf_c=_LEAF_C,
        stem_c=_STEM_C,
        root_c=_ROOT_C,
        sen_params=_SEN_PARAMS,
        nitro_params=_NITRO_PARAMS,
    )


def _mineralization() -> Mineralization:
    return Mineralization(
        FlowId("biosphere.mineralization"),
        0,
        litter_n=_LITTER_N,
        soil_n=_SOIL_N,
        params=_params(),
    )


def _env(state: State, dt: float):
    # Neither flow reads forcing; a trivial bound resolver suffices.
    return SourceResolver(forcings={}).bind(state, dt)


def test_n_senescence_moves_plant_n_to_litter_n() -> None:
    state = _state(plant_n=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _senescence().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    # shed_C = rdr_leaf*leaf_c = 0.01*1.0; tissue conc = 2.0/1.0 = 2.0, far above the
    # residual 0.001, so shedding runs at the residual concentration: 0.001 * 0.01.
    shed = 0.001 * 0.01
    assert math.isclose(legs[_PLANT_N], -shed, rel_tol=1e-12)
    assert math.isclose(legs[_LITTER_N], shed, rel_tol=1e-12)


def test_mineralization_moves_litter_n_to_soil_n() -> None:
    state = _state(litter_n=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _mineralization().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    mineralized = 0.03 * 2.0
    assert math.isclose(legs[_LITTER_N], -mineralized, rel_tol=1e-12)
    assert math.isclose(legs[_SOIL_N], mineralized, rel_tol=1e-12)


def test_flows_balance_nitrogen_only() -> None:
    # Single-currency NITROGEN: each flow balances and touches no CARBON/OXYGEN/WATER.
    for flow in (_senescence(), _mineralization()):
        state = _state()
        result = flow.evaluate(state, _env(state, 1.0), 1.0)
        assert_flow_balanced(result, state.stocks)
        assert set(per_quantity_residual(result, state.stocks)) == {Quantity.NITROGEN}


def test_flows_are_dt_linear() -> None:
    # flux = daily·dt — the increment-form contract (here Euler-daily).
    for flow, donor in ((_senescence(), _PLANT_N), (_mineralization(), _LITTER_N)):
        state = _state()
        half = {
            leg.stock: leg.amount
            for leg in flow.evaluate(state, _env(state, 0.5), 0.5).legs
        }
        full = {
            leg.stock: leg.amount
            for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        }
        assert math.isclose(full[donor], 2.0 * half[donor], rel_tol=1e-12)


def test_n_senescence_self_limits_at_zero_plant_n() -> None:
    state = _state(plant_n=0.0)
    legs = _senescence().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


def test_n_senescence_self_limits_when_no_carbon_is_senescing() -> None:
    """No shed carbon ⇒ no shed nitrogen — the coupling, seen from the zero end.

    Under the retired rate form a standing ``plant_n`` shed nitrogen every step
    regardless of whether any tissue was dying. That is exactly the decoupling this
    change removes, so
    it is pinned from both directions.
    """
    state = _state(plant_n=2.0, leaf_c=0.0)
    legs = _senescence().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


def test_mineralization_self_limits_at_zero_litter_n() -> None:
    state = _state(litter_n=0.0)
    legs = _mineralization().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)


# --- loader ------------------------------------------------------------------
def test_loader_reads_committed_rates() -> None:
    params = load_mineralization_params()
    assert params.mineralization_rate == 0.03


def test_loader_no_longer_accepts_the_retired_shedding_rate(tmp_path: Path) -> None:
    """``n_senescence_rate`` is GONE, and the schema must SAY so rather than ignore it.

    It was the project's highest clean-room risk: a 1/day N-shedding rate that five
    rounds of citation work established no primary source publishes. ``extra="forbid"``
    is what makes the removal load-bearing — a file still carrying the key is a hard
    error, so a stale param file cannot quietly supply a value that nothing reads any
    more.
    """
    stale = tmp_path / "mineralization.yaml"
    stale.write_text(
        "name: chamber\nprocess: mineralization\nparameters:\n"
        '  n_senescence_rate:\n    value: 0.01\n    unit: "1/day"\n'
        '    source: "retired"\n'
        '  mineralization_rate:\n    value: 0.03\n    unit: "1/day"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValidationError):
        load_mineralization_params(stale)


def test_loader_rejects_negative_rate(tmp_path: Path) -> None:
    bad = tmp_path / "mineralization.yaml"
    bad.write_text(
        "name: chamber\nprocess: mineralization\nparameters:\n"
        "  mineralization_rate:\n    value: -0.01\n"
        '    unit: "1/day"\n    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="mineralization_rate must be >= 0"):
        load_mineralization_params(bad)


def test_loader_rejects_bad_unit(tmp_path: Path) -> None:
    bad = tmp_path / "mineralization.yaml"
    bad.write_text(
        "name: chamber\nprocess: mineralization\nparameters:\n"
        '  mineralization_rate:\n    value: 0.03\n    unit: "1/year"\n'
        '    source: "test"\n',
        encoding="utf-8",
    )
    with pytest.raises(ValueError, match="must be declared in"):
        load_mineralization_params(bad)


# --- integration: the sealed season -----------------------------------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    scenario = SeasonScenario(sealed=True)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(EulerIntegrator(registry), state, resolver, 1.0, len(_weather()))


def _total_nitrogen(s: State) -> float:
    return sum(
        stock.amount * stock.composition.get(Quantity.NITROGEN, 0.0)
        for stock in s.stocks.values()
    )


def test_sealed_litter_n_accumulates_then_drains(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # N-senescence feeds the litter-N POOL; mineralization drains it. Non-vacuous: it
    # rises well above its 0 start AND is drawn down on some step (the N analogue of the
    # Step-4 litter_carbon accumulate-then-drain).
    states, _, _ = sealed
    litter = [s.stocks[LITTER_N].amount for s in states]
    assert litter[0] == 0.0
    # RESCALED, not relaxed (post-roadmap: the N-cycle form change). The old bound was
    # 1e-3 kg N, which suited a plant holding 0.5 kg N and shedding 1 %/day of it —
    # about 5e-3 kg N/day. Coupled shedding is ~2 orders smaller: it sheds the RESIDUAL
    # concentration (1.33e-4 kg N/mol C) times the senescing carbon (~1e-2 mol C/day),
    # i.e. ~1e-6 kg N/day, from a plant holding ~2.4e-4 kg N rather than 0.5 kg. The
    # assertion's INTENT is unchanged and still non-vacuous: standing organic N
    # genuinely builds, here to ~1.2e-5 kg N, an order of magnitude above this bound.
    assert max(litter) > 1e-6  # senescence genuinely builds standing organic N
    assert any(b < a for a, b in zip(litter, litter[1:], strict=False))  # mineralized


def test_sealed_plant_n_is_withdrawn_by_shedding(
    sealed: tuple[list[State], int, tuple],
) -> None:
    """N-senescence genuinely WITHDRAWS plant N — the consumption side Phase 1 lacked.

    ⚠ This test used to assert ``plant_n[-1] < plant_n[0]`` — a NET DECLINE over the
    season — and that assertion was **an artefact of the old initial condition, not a
    property of the model**. ``plant_n0`` was 0.5 kg (2055x a seedling's target N),
    chosen
    only to force ``f_N == 1``, and capacity-driven uptake pulled it toward the
    ``max_uptake_capacity / n_senescence_rate`` equilibrium of 0.15 kg; so of course it
    fell. Under demand-deficit uptake ``plant_n`` tracks ``target x biomass``, and
    biomass grows over a season, so a **growing crop accumulating nitrogen** is the
    correct behaviour and the old direction is simply wrong.

    What the test existed to prove is kept and made direct: nitrogen leaves the plant.
    """
    states, _, _ = sealed
    plant_n = [s.stocks[PLANT_N].amount for s in states]
    litter_n = [s.stocks[LITTER_N].amount for s in states]

    # 1. The withdrawal is real: N arrives in litter, which can only come from plant_n.
    assert max(litter_n) > 0.0
    # 2. And it bites step-by-step somewhere (not merely as a net year-end figure):
    #    plant_n falls on some step, i.e. shedding out-runs uptake at least once.
    assert any(b < a for a, b in zip(plant_n, plant_n[1:], strict=False))
    # 3. The NEW invariant that replaces "declines": the target is a FLOOR that
    # demand-deficit uptake maintains. Greenwood's plateau is 5.697 % DM and the chamber
    # crop never leaves the plateau, so the whole-plant concentration sits at the target
    # to within the one-step uptake lag (measured min here: 0.896x).
    #
    #    ⚠ DELIBERATELY A FLOOR AND NOT A BAND — the upper side is unbounded BY
    # CONSTRUCTION, and that is a recorded limitation rather than an oversight. Shedding
    # removes N at the RESIDUAL concentration, so a senescing plant RETAINS most of its
    # nitrogen (remobilization), while the denominator collapses as carbon translocates
    # to storage and dies back. Concentration therefore rises without bound as
    # biomass -> 0: measured up to ~110x target in this 3-year chamber, and ~6e6x in the
    # perennial, where the crop cycles for 5 years. It is harmless for the carbon
    # trajectory (f_N saturates at 1) and nitrogen is conserved exactly, but it is the
    # one-pool model showing through: real remobilized N goes to GRAIN, and we have a
    # single whole-plant pool that cannot represent that. See the plan doc.
    nitro = load_nitrogen_params()
    target = nitro.n_target_coefficient * nitro.dm_kg_per_mol_c
    for state, pn in zip(states, plant_n, strict=True):
        biomass = sum(state.stocks[organ].amount for organ in (LEAF_C, STEM_C, ROOT_C))
        if biomass <= 0.0:
            continue
        assert pn / biomass >= 0.85 * target, (state.n, pn / biomass, target)


def test_sealed_conserves_nitrogen_exactly(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # The N cycle is entirely internal (uptake soil→plant, N-senescence plant→litter,
    # mineralization litter→soil; fertilization is rate 0), so total NITROGEN is
    # invariant to float — the every-step gate end-to-end through the new N flows.
    # abs_tol 1e-9 (total N ≈ 100, soil_n-dominated; cf. the oxygen test, not 1e-12).
    states, _, _ = sealed
    n0 = _total_nitrogen(states[0])
    for s in states:
        assert math.isclose(_total_nitrogen(s), n0, rel_tol=0.0, abs_tol=1e-9)


def test_sealed_f_n_stays_one_carbon_decoupled(
    sealed: tuple[list[State], int, tuple],
) -> None:
    # THE load-bearing claim, verified not asserted: at the PP fill plant_n stays ~1000×
    # above the critical-N concentration, so f_N ≡ 1 EVERY step ⇒ the N loop has zero
    # effect on photosynthesis / the carbon trajectory (mechanism, not feedback; the
    # N-limited regime is Step 7's sized run, the f_O2-deferral mirror). The
    # bit-identical carbon run is additionally pinned by the UNCHANGED prior sealed
    # tests.
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


def test_sealed_never_rations(sealed: tuple[list[State], int, tuple]) -> None:
    # rationed == 0 holds with the N loop present: each draw (rate·pool·dt, rate·dt ≪ 1)
    # self-limits against its start-of-step donor, so the Euler backstop never fires.
    _, total_rationed, _ = sealed
    assert total_rationed == 0


def test_sealed_no_extinction(sealed: tuple[list[State], int, tuple]) -> None:
    # plant_n / litter_n / soil_n are POOLs (never zeroed-with-loss); nothing snaps.
    _, _, events = sealed
    assert events == ()


# --- open field is untouched (the regression golden's path) ------------------
def test_open_field_has_no_litter_n() -> None:
    # The N return loop is sealed-only (litter_n + both flows gated on scenario.sealed),
    # so the open field grows no litter_n and the regression golden's path is byte-
    # identical (it keeps the external n_source / fertilization supply).
    state, _ = build_season(SeasonScenario(sealed=False))
    assert LITTER_N not in state.stocks
