"""Phase-2 Step-6 tests: the nitrogen return loop (senescence-N + net mineralization).

Step 6 closes the **nitrogen** cycle the way Steps 4–5 closed the carbon cycle. Phase 1
left ``plant_n`` monotone-growing (uptake fed it; nothing withdrew it) and refilled
``soil_n`` from an external ``n_source``. Step 6 returns plant N to the soil internally:

* **N-senescence** ``plant_n → litter_n`` — the plant sheds N into a finite ``litter_n``
  POOL (the N analogue of carbon senescence feeding ``litter_carbon``); first-order in
  plant_n, so ``plant_n`` is now drained (no longer monotone-growing).
* **LitterNitrogenTransfer** ``litter_n → microbial_n`` and **MicrobialNitrogenRelease**
  ``microbial_n → soil_n`` — the **microbe-mediated** return leg (post-roadmap, the
  N-cycle form gap option (B)). These replaced a *direct* ``litter_n → soil_n``
  mineralization at a free ``mineralization_rate``; each now carries the nitrogen
  belonging to the carbon its decomposer sibling already moved, so the free rate is
  retired and ``params/mineralization.yaml`` no longer exists.

All are single-currency NITROGEN flows (no core change). Four layers:

* **Rate laws** — the shedding flux, plus the one ``carried_nitrogen`` kernel behind
  both return legs (→ 0 as the donor → 0; positivity structural and *inherited* from the
  carbon leg's own ``k·dt < 1`` bound).
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

from config.paths import BIOSPHERE_PARAMS_DIR, WINTER_WHEAT_WEATHER
from domains.biosphere.allocation import SenescenceParams
from domains.biosphere.canopy import CanopyParams
from domains.biosphere.decomposition import (
    Decomposition,
    DecompositionParams,
    decomposition_flux,
)
from domains.biosphere.humification import HumificationParams
from domains.biosphere.loader import load_nitrogen_params
from domains.biosphere.microbial_respiration import (
    MicrobialRespiration,
    MicrobialRespirationParams,
)
from domains.biosphere.mineralization import (
    LitterNitrogenTransfer,
    MicrobialNitrogenRelease,
    NitrogenSenescence,
    carried_nitrogen,
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
from domains.biosphere.step import BIO_DT, steps_for
from simcore.environment import SourceResolver
from simcore.flow import assert_flow_balanced, per_quantity_residual
from simcore.ids import DomainId, FlowId, StockId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity, StockKind, canonical_unit
from simcore.state import State, Stock

_WEATHER_FIXTURE = WINTER_WHEAT_WEATHER

_BIO = DomainId("biosphere")
_PLANT_N = StockId("biosphere.plant_n")
_LITTER_N = StockId("biosphere.litter_n")
_LEAF_C = StockId("biosphere.leaf_c")
_STEM_C = StockId("biosphere.stem_c")
_ROOT_C = StockId("biosphere.root_c")
_SOIL_N = StockId("biosphere.soil_n")
_MICROBIAL_N = StockId("biosphere.microbial_n")
_LITTER_C = StockId("biosphere.litter_carbon")
_MICROBIAL_C = StockId("biosphere.microbial_carbon")
_O2 = StockId("biosphere.o2_pool")
_HUMUS_C = StockId("biosphere.humus_carbon")
_HUMUS_N = StockId("biosphere.humus_n")
_CO2 = StockId("biosphere.carbon_pool")


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


def test_carried_nitrogen_moves_the_donor_pools_own_ratio() -> None:
    # moved_C · (pool_N / pool_C): a hand value + linearity in the carbon moved.
    assert math.isclose(carried_nitrogen(0.5, 2.0, 10.0), 0.1, rel_tol=1e-12)
    assert math.isclose(carried_nitrogen(1.0, 2.0, 10.0), 0.2, rel_tol=1e-12)


def test_carried_nitrogen_is_zero_at_every_degenerate_input() -> None:
    # Never a divide-by-zero, never a negative leg — an empty or absent pool moves
    # nothing, which is what makes positivity structural rather than clamped.
    assert carried_nitrogen(0.0, 2.0, 10.0) == 0.0
    assert carried_nitrogen(0.5, 0.0, 10.0) == 0.0
    assert carried_nitrogen(0.5, 2.0, 0.0) == 0.0
    assert carried_nitrogen(-1.0, 2.0, 10.0) == 0.0


def test_carried_n_on_a_first_order_carbon_flux_is_that_same_rate() -> None:
    """THE IDENTITY THAT RETIRED ``mineralization_rate`` — pinned, not just documented.

    ``Decomposition`` withdraws ``k · litter_C``, so the nitrogen riding it is
    ``k · litter_C · (litter_N / litter_C) == k · litter_N``. The free mineralization
    rate was therefore never independent: stoichiometry forces it to equal the carbon
    decay rate, and that is *why* retiring it is a citation upgrade rather than a
    recalibration.

    ⚠ Pinned as an EQUIVALENCE, never as an implementation. The flow deliberately does
    NOT collapse to ``decomposition_rate · litter_n``: the identity holds only while
    ``Decomposition`` stays first-order, and a collapsed form would read identically
    today and silently outlive that premise. This test is what would go red if the two
    ever diverged.
    """
    k, litter_c, litter_n = 0.011, 7.0, 0.25
    decomposed = decomposition_flux(litter_c, decomposition_rate=k)
    assert math.isclose(
        carried_nitrogen(decomposed, litter_n, litter_c), k * litter_n, rel_tol=1e-12
    )


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
    microbial_n: float = 0.02,
    litter_c: float = 4.0,
    microbial_c: float = 1.0,
    humus_n: float = 0.005,
    humus_c: float = 0.5,
    o2: float = 210.0,
) -> State:
    stocks = {
        _PLANT_N: _n_pool(_PLANT_N, plant_n),
        _LITTER_N: _n_pool(_LITTER_N, litter_n),
        _SOIL_N: _n_pool(_SOIL_N, soil_n),
        # The coupled N-shedding flow reads the organ carbon that is senescing.
        _LEAF_C: _c_pool(_LEAF_C, leaf_c),
        _STEM_C: _c_pool(_STEM_C, 0.0),
        _ROOT_C: _c_pool(_ROOT_C, 0.0),
        # The microbe-mediated return legs read the carbon pools they ride.
        _MICROBIAL_N: _n_pool(_MICROBIAL_N, microbial_n),
        _LITTER_C: _c_pool(_LITTER_C, litter_c),
        _MICROBIAL_C: _c_pool(_MICROBIAL_C, microbial_c),
        # CENTURY's slow SOM and its N counterpart (the humification split, 2026-08-10).
        _HUMUS_N: _n_pool(_HUMUS_N, humus_n),
        _HUMUS_C: _c_pool(_HUMUS_C, humus_c),
        _O2: _c_pool(_O2, o2),
    }
    return State(n=0, stocks=stocks, rng_seed=0)


# Decomposer rates chosen so each leg is a single hand-checkable term.
_DECOMP_PARAMS = DecompositionParams(decomposition_rate=0.011)
# o2_half_saturation = 0 disables f_O2 (the loader permits it), so the release leg's
# arithmetic is exact here; f_O2 gets its own dedicated test below rather than riding
# along in every other one.
_MRESP_PARAMS = MicrobialRespirationParams(
    microbial_respiration_rate=0.016, o2_half_saturation=0.0
)
_AIR_MOL = 1000.0
# The humification partition, driven explicitly so each N leg stays a single
# hand-checkable term. The committed values are asserted against the loader in
# test_decomposition / test_microbial_respiration; here they are arithmetic.
_HUMI_PARAMS = HumificationParams(
    litter_respired_fraction=0.45,
    active_stabilization_co2_fraction=0.85,
    slow_respired_fraction=0.55,
    slow_decomposition_rate=0.0005428571428571428,
)


# The coupled N-shedding flow needs the organ carbon stocks and the senescence rates
# that
# drive it. rdr_leaf = 0.01 on 1.0 mol C of leaf gives shed_C = 0.01 mol C/day, with the
# stem/root rates zero so the arithmetic below stays a single hand-checkable term.
_SEN_PARAMS = SenescenceParams(
    rdr_leaf=0.01, rdr_stem=0.0, rdr_root=0.0, shade_rate=0.05, lai_threshold=6.0
)
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
        # inert at these leaf amounts — see the note in test_allocation.py
        canopy=CanopyParams(sla_per_mol_c=0.6, extinction_coef=0.6),
        ground_area=1.0,
    )


def _litter_transfer() -> LitterNitrogenTransfer:
    return LitterNitrogenTransfer(
        FlowId("biosphere.litter_n_transfer"),
        0,
        litter_n=_LITTER_N,
        microbial_n=_MICROBIAL_N,
        soil_n=_SOIL_N,
        litter_carbon=_LITTER_C,
        o2_pool=_O2,
        params=_DECOMP_PARAMS,
        humification=_HUMI_PARAMS,
        o2_half_saturation=0.0,
        air_mol=_AIR_MOL,
    )


def _microbial_release(
    *, params: MicrobialRespirationParams = _MRESP_PARAMS
) -> MicrobialNitrogenRelease:
    return MicrobialNitrogenRelease(
        FlowId("biosphere.microbial_n_release"),
        0,
        microbial_n=_MICROBIAL_N,
        soil_n=_SOIL_N,
        microbial_carbon=_MICROBIAL_C,
        o2_pool=_O2,
        humus_n=_HUMUS_N,
        params=params,
        humification=_HUMI_PARAMS,
        air_mol=_AIR_MOL,
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


def test_litter_transfer_splits_its_nitrogen_the_way_the_carbon_split() -> None:
    # ⚠ REWRITTEN by the humification split (2026-08-10), not weakened. The WITHDRAWAL
    # is
    # unchanged — still k·litter_C · (litter_N/litter_C), which is what preserves option
    # (B)'s identity that litter's C and N leave on the same flux. What is new is that
    # the carbon no longer all goes one place, so neither does the nitrogen.
    state = _state(litter_n=2.0, litter_c=4.0)
    legs = {
        leg.stock: leg.amount
        for leg in _litter_transfer().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    moved = 0.011 * 4.0 * (2.0 / 4.0)
    assert math.isclose(legs[_LITTER_N], -moved, rel_tol=1e-12)
    assert math.isclose(legs[_MICROBIAL_N], 0.55 * moved, rel_tol=1e-12)
    assert math.isclose(legs[_SOIL_N], 0.45 * moved, rel_tol=1e-12)
    assert legs[_MICROBIAL_N] + legs[_SOIL_N] == -legs[_LITTER_N]  # exact


def test_microbial_release_splits_its_nitrogen_the_way_the_carbon_split() -> None:
    # The N riding the microbial turnover: the respired share (Es) mineralizes to soil,
    # the stabilised share rides its carbon into slow SOM.
    state = _state(microbial_n=0.05, microbial_c=2.0)
    legs = {
        leg.stock: leg.amount
        for leg in _microbial_release().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    moved = 0.016 * 2.0 * (0.05 / 2.0)
    assert math.isclose(legs[_MICROBIAL_N], -moved, rel_tol=1e-12)
    assert math.isclose(legs[_SOIL_N], 0.85 * moved, rel_tol=1e-12)
    assert math.isclose(legs[_HUMUS_N], 0.15 * moved, rel_tol=1e-12)
    assert legs[_SOIL_N] + legs[_HUMUS_N] == -legs[_MICROBIAL_N]  # exact


def test_mineralization_is_the_nitrogen_of_the_carbon_that_LEFT_AS_CO2() -> None:
    """Why "N follows the carbon partition" is the right law here, not a convenience.

    The textbook mineralization/immobilization balance computes the N released as the
    difference between the N in the decomposing substrate and the N the receiving pool
    needs at *its own* C:N. In the limit where the receiving pool carries the SAME C:N
    as the donor — exactly our case, because this tree deliberately imposes no
    homeostatic microbial C:N (option (B) measured that doing so would demand 90-152x
    the litter N present, and refused it) — that difference collapses to *the nitrogen
    of the carbon that left as CO2*.

    So the partition is not an approximation chosen for tidiness: it is what the
    standard
    balance reduces to under this tree's own stoichiometry. The gap to a real soil is
    the
    homeostasis, recorded as a limitation rather than fitted.
    """
    state = _state(litter_n=2.0, litter_c=4.0)
    legs = {
        leg.stock: leg.amount
        for leg in _litter_transfer().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    decayed_c = 0.011 * 4.0
    respired_c = 0.45 * decayed_c
    n_per_c = 2.0 / 4.0  # the litter pool's own N:C
    assert math.isclose(legs[_SOIL_N], respired_c * n_per_c, rel_tol=1e-12)


def test_only_the_RESPIRED_share_reaches_soil_n_the_rest_still_transits() -> None:
    """⚠ RESOLVED, NOT CORRECTED — and replaced by its INVERSE.

    This asserted that ``litter_n`` NEVER reaches ``soil_n`` in one step, on the ground
    that "microbe-mediated" means all of it must transit ``microbial_n``. That was a
    true
    measurement of the option-(B) form, and it was true *because* carbon-use efficiency
    was 1.0: when 100 % of the decayed litter carbon went to microbes, 100 % of its
    nitrogen went with it.

    With the humification split, part of the litter carbon leaves as CO2 at the litter
    step, so the nitrogen it carried is mineralized there. The mediation claim survives
    in
    the form that is still true and is the one worth guarding: **no nitrogen reaches the
    soil except in proportion to carbon respired in the same step**. A collapsed
    ``litter_n -> soil_n`` jump at a free rate — the retired ``Mineralization`` — still
    goes red here, because it would move N with no respired carbon to carry it.

    *A pin guarding a mechanism you removed is decoration* (the option-(B) precedent).
    """
    # Nothing decomposing at the litter step ==> no nitrogen may reach the soil from it.
    state = _state(litter_n=5.0, litter_c=0.0)
    legs = _litter_transfer().evaluate(state, _env(state, 1.0), 1.0).legs
    assert all(leg.amount == 0.0 for leg in legs)
    # With carbon moving, the soil leg is exactly the respired share — never the whole
    # withdrawal, which is what a re-collapsed direct mineralization would give.
    state = _state(litter_n=2.0, litter_c=4.0)
    legs = {
        leg.stock: leg.amount
        for leg in _litter_transfer().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    assert 0.0 < legs[_SOIL_N] < -legs[_LITTER_N]
    assert legs[_MICROBIAL_N] > 0.0  # the stabilised share still transits


def test_flows_balance_nitrogen_only() -> None:
    # Single-currency NITROGEN: each flow balances and touches no CARBON/OXYGEN/WATER.
    # ⚠ Note the release leg READS the O₂ pool (for f_O2) but never puts a LEG on it —
    # the carbon sibling owns the O₂ draw. A leg here would double-count it.
    for flow in (_senescence(), _litter_transfer(), _microbial_release()):
        state = _state()
        result = flow.evaluate(state, _env(state, 1.0), 1.0)
        assert_flow_balanced(result, state.stocks)
        assert set(per_quantity_residual(result, state.stocks)) == {Quantity.NITROGEN}


def test_flows_are_dt_linear() -> None:
    # flux = daily·dt — the increment-form contract (here Euler-daily).
    for flow, donor in (
        (_senescence(), _PLANT_N),
        (_litter_transfer(), _LITTER_N),
        (_microbial_release(), _MICROBIAL_N),
    ):
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


def test_return_legs_self_limit_at_an_empty_donor() -> None:
    # Positivity structural on both legs, from the N side...
    for flow, state in (
        (_litter_transfer(), _state(litter_n=0.0)),
        (_microbial_release(), _state(microbial_n=0.0)),
    ):
        assert all(
            leg.amount == 0.0
            for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        )


def test_return_legs_self_limit_when_no_carbon_is_moving() -> None:
    # ...and from the CARBON side, which is the half the retired rate form could not
    # express: no carbon decomposing/respiring ⇒ no nitrogen released, however much N is
    # standing in the pool. That coupling is the whole point of the change.
    for flow, state in (
        (_litter_transfer(), _state(litter_n=5.0, litter_c=0.0)),
        (_microbial_release(), _state(microbial_n=5.0, microbial_c=0.0)),
    ):
        assert all(
            leg.amount == 0.0
            for leg in flow.evaluate(state, _env(state, 1.0), 1.0).legs
        )


# --- the parameter file is GONE (both its rates retired by FORM changes) -----
def test_there_is_no_mineralization_param_file_or_loader() -> None:
    """Both rates this file ever held were retired by changing the FORM.

    ``n_senescence_rate`` (option (A)) — a 1/day N-shedding rate five rounds of citation
    work established no primary source publishes, the project's highest clean-room risk
    — went when shedding became coupled to the senescing carbon at a *cited* residual
    tissue concentration. ``mineralization_rate`` (option (B)) went when the return leg
    became microbe-mediated and stoichiometric, because
    ``decomposed_C / litter_C == decomposition_rate`` makes a free N rate redundant with
    the carbon one.

    With no parameter left, the file and its loader are gone. This is pinned rather than
    merely deleted because a *stale param file* is the failure mode: a
    ``mineralization.yaml`` still sitting in ``params/`` would be silently unread, and
    the manifest's ``param_files`` gate compares membership — so an accidental
    re-addition must fail here, loudly, at the source.
    """
    import domains.biosphere.loader as loader_module

    assert not hasattr(loader_module, "load_mineralization_params")
    assert not hasattr(loader_module, "MINERALIZATION_PARAMS_PATH")
    params_dir = BIOSPHERE_PARAMS_DIR
    # ⚠ The positive half FIRST. This is a negative assertion about a directory, and a
    # directory that does not exist satisfies it vacuously — which is exactly what a
    # mis-resolved `BIOSPHERE_PARAMS_DIR` would produce after slice S1 moved the params
    # out of the Python package. Without this line the relocation could have silently
    # turned a real check into a green no-op.
    assert params_dir.is_dir(), params_dir
    assert (params_dir / "nitrogen.yaml").is_file(), params_dir
    assert not (params_dir / "mineralization.yaml").exists()


def test_the_retired_provenance_record_is_preserved() -> None:
    """The five rounds of negative retrieval results outlive the parameter.

    This project's own citation work found that *a stale negative result is worse than a
    stale positive one, because it suppresses the next search*. Deleting the file would
    have destroyed exactly the record that stops someone re-running searches already
    known to be exhausted, so it is archived verbatim instead. Pinned so the archive
    cannot be quietly tidied away later.
    """
    archive = Path(__file__).parent.parent / "docs" / "retired" / "mineralization.yaml"
    text = archive.read_text(encoding="utf-8")
    assert "RETIRED PARAMETER FILE" in text
    assert "n_senescence_rate" in text
    assert "mineralization_rate" in text
    # The dated-ceiling warning is the transferable lesson, not decoration.
    assert "one afternoon's access" in text


def test_the_return_legs_take_the_DECOMPOSER_params_not_their_own() -> None:
    """No N rate reappears by the back door — the legs are wired to the carbon rates.

    If someone later gives these flows a parameter of their own, the identity that
    retired ``mineralization_rate`` silently stops holding while every trajectory still
    looks plausible. Pinning the *wiring* catches that at the type level.
    """
    assert isinstance(_litter_transfer().params, DecompositionParams)
    assert isinstance(_microbial_release().params, MicrobialRespirationParams)


# --- the recomputation-drift guard (the NitrogenSenescence idiom, one flow over) ---
def test_transfer_leg_recomputes_EXACTLY_the_carbon_Decomposition_moves() -> None:
    """The N leg's ``decomposed_C`` must equal ``Decomposition``'s own litter leg.

    A flow may read only the step-entry snapshot, so there is no channel by which
    ``Decomposition`` could hand this flow its computed flux — recomputation from the
    same params on the same snapshot is the only pure form. The hazard that creates is
    that the two silently drift apart if someone changes one and not the other, and the
    symptom would be a wrong litter C:N rather than a crash. So the agreement is pinned
    against the *actual sibling flow*, not against a re-derivation of it.
    """
    state = _state(litter_n=2.0, litter_c=4.0)
    carbon_leg = {
        leg.stock: leg.amount
        for leg in Decomposition(
            FlowId("biosphere.decomposition"),
            0,
            litter_carbon=_LITTER_C,
            microbial_carbon=_MICROBIAL_C,
            co2_pool=_CO2,
            o2_pool=_O2,
            params=_DECOMP_PARAMS,
            humification=_HUMI_PARAMS,
            o2_half_saturation=0.0,
            air_mol=_AIR_MOL,
        )
        .evaluate(state, _env(state, 1.0), 1.0)
        .legs
    }
    n_leg = {
        leg.stock: leg.amount
        for leg in _litter_transfer().evaluate(state, _env(state, 1.0), 1.0).legs
    }
    decomposed_c = -carbon_leg[_LITTER_C]
    # The N moved is exactly that carbon at the litter pool's own N:C.
    assert math.isclose(-n_leg[_LITTER_N], decomposed_c * (2.0 / 4.0), rel_tol=1e-12)


def test_release_leg_recomputes_EXACTLY_the_carbon_MicrobialRespiration_burns() -> None:
    """Same guard on the respiration leg — and here ``f_O2`` is the live part.

    ``MicrobialRespiration`` self-throttles as O₂ depletes. If the N leg did not carry
    the same factor it would keep mineralizing nitrogen out of a pool whose carbon had
    stopped moving, which is precisely the decoupling this whole change removes.
    """
    params = MicrobialRespirationParams(
        microbial_respiration_rate=0.016, o2_half_saturation=0.05
    )
    # An O₂ mole fraction low enough that f_O2 is meaningfully below 1.
    state = _state(microbial_n=0.05, microbial_c=2.0, o2=50.0)
    carbon_leg = {
        leg.stock: leg.amount
        for leg in MicrobialRespiration(
            FlowId("biosphere.microbial_respiration"),
            0,
            microbial_carbon=_MICROBIAL_C,
            humus_carbon=_HUMUS_C,
            # Any distinct carbon pool: only the microbial leg is read here.
            co2_pool=_LITTER_C,
            o2_pool=_O2,
            params=params,
            humification=_HUMI_PARAMS,
            air_mol=_AIR_MOL,
        )
        .evaluate(state, _env(state, 1.0), 1.0)
        .legs
    }
    n_leg = {
        leg.stock: leg.amount
        for leg in _microbial_release(params=params)
        .evaluate(state, _env(state, 1.0), 1.0)
        .legs
    }
    respired_c = -carbon_leg[_MICROBIAL_C]
    assert math.isclose(-n_leg[_MICROBIAL_N], respired_c * (0.05 / 2.0), rel_tol=1e-12)


def test_release_leg_throttles_with_f_o2_rather_than_ignoring_it() -> None:
    """The f_O2 factor is load-bearing here, not inherited decoration.

    Guards the specific collapsed form the module docstring warns against: writing the
    leg as ``microbial_respiration_rate · microbial_n`` would pass every other test in
    this file and go red only here.
    """
    params = MicrobialRespirationParams(
        microbial_respiration_rate=0.016, o2_half_saturation=0.05
    )
    rich = _state(microbial_n=0.05, microbial_c=2.0, o2=210.0)
    poor = _state(microbial_n=0.05, microbial_c=2.0, o2=5.0)
    flow = _microbial_release(params=params)
    moved_rich = -next(
        leg.amount
        for leg in flow.evaluate(rich, _env(rich, 1.0), 1.0).legs
        if leg.stock == _MICROBIAL_N
    )
    moved_poor = -next(
        leg.amount
        for leg in flow.evaluate(poor, _env(poor, 1.0), 1.0).legs
        if leg.stock == _MICROBIAL_N
    )
    assert moved_poor < moved_rich
    # And an uncollapsed form is the ONLY way this holds: the bare rate would give the
    # identical number in both states.
    assert not math.isclose(moved_poor, moved_rich, rel_tol=1e-9)


def test_microbial_n_is_a_POOL_so_extinction_can_never_orphan_it() -> None:
    """The named seam from the (B) diagnosis, pinned as a test rather than a comment.

    ``microbial_carbon`` is a POPULATION (``organ_stock``), so an extinction pass could
    in principle zero it and route the residual to the loss-sink. Were its nitrogen
    counterpart also a POPULATION, that pass would orphan N and break the emergent C:N.
    ``microbial_n`` is therefore a POOL, and POOL stocks are never zeroed-with-loss (the
    project's extinction invariant).

    ⚠ The seam this pins is on the CARBON side: if anyone ever raises
    ``microbial_carbon``'s extinction threshold above 0, the N counterpart must be
    zeroed
    with it. A comment would not survive that edit; this goes red.
    """
    from domains.biosphere import scenario as sc

    state, _ = build_season(sc.SEALED_CHAMBER_SCENARIO)
    micro_n = state.stocks[_MICROBIAL_N]
    micro_c = state.stocks[StockId("biosphere.microbial_carbon")]
    assert micro_n.kind is StockKind.POOL
    assert micro_n.quantity is Quantity.NITROGEN
    # The carbon sibling's threshold is what makes the asymmetry safe today.
    assert micro_c.extinction_threshold == 0.0


# --- integration: the sealed season -----------------------------------------
@pytest.fixture(scope="module")
def sealed() -> tuple[list[State], int, tuple]:
    scenario = SeasonScenario(sealed=True)
    state, registry = build_season(scenario)
    resolver = weather_resolver(_weather(), scenario)
    return run_season(
        EulerIntegrator(registry), state, resolver, BIO_DT, steps_for(len(_weather()))
    )


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
