"""The N-cycle FORM change: Greenwood's target curve, the coupling, and litter C:N.

Post-roadmap work — ``docs/plans/post-roadmap-nitrogen-cycle-form.md``. The gap this
closes is that **nothing tied the nitrogen leaving the plant to the carbon it was part
of**, so litter C:N was the unconstrained ratio of two independent first-order rates
(measured: 0.004 in-run, against wheat straw's ~80). Two paired changes fix it —
N:C-coupled shedding and demand-deficit uptake against a *published* target
concentration — and they must ship together, because coupled shedding removes the
``max_uptake_capacity / n_senescence_rate`` equilibrium that used to pin ``plant_n``.

What this module pins, and why each one is here rather than in prose:

1. **The curve is Greenwood's, including its DOMAIN BOUND.** The plateau below 1 t/ha is
the
   primary's own statement (exponential growth ⇒ constant %N, Ågren 1985), not our
   interpolation, and extrapolating the declining branch below the bound is the one form
   the paper contradicts. A test, because "we read the paper correctly" is otherwise
   ungated.
2. **The 14.42 t/ha crossing point.** ``f_N ≡ 1`` in every frozen scenario holds on a
~12 %
   margin, not structurally: ``open_season`` peaks at 12.633 t/ha and the target meets
   ``n_critical`` at 14.42. The margin is asserted rather than mentioned (the "freeze's
   prose half is ungated" lesson).

   ⚠ **This item originally continued "Anything that grows the open-field crop ~15 %
   moves a frozen golden." THAT INFERENCE IS MEASURED FALSE (2026-07-27** —
   ``docs/plans/post-roadmap-canopy-regulator.md`` **finding 6).** A counterexample at
   **+24.5 %** exists: the DS-keyed form plus the Van Keulen & Seligman canopy regulator
   peaks at 15.725 t/ha, *is* past the crossing, and leaves ``f_N`` at exactly 1.0 for
   all 306 steps. ``f_N`` reads the plant's **actual** concentration, not its target,
   and demand-deficit uptake clamps at zero deficit — so past the crossing the plant
   sits **15-30 % above** its target with no route back down. Peak mass does not even
   *order* the bite (the bare form crosses ``n_critical`` at a **lower** mass, 15.068).
   **The ASSERTIONS below are sound and CONSERVATIVE and are deliberately unchanged** —
   14.4248 fires before the earliest measured bite, which is what a tripwire should do.
   *The value may stand* and *its justification is falsified* are both true, and the
   first does not rescue the second (round 4's ``self_discharge``).
3. **The two carbon legs of one physical event cannot drift.** The shed-N flow
recomputes
   the senescence flux; a test compares it against ``Senescence``'s own litter leg.
4. **litter C:N landed in a defensible place, and WHERE THE RESIDUAL LIVES.** The shed
   material is straw-like (C:N 90, from two cited params). The litter POOL is **two
   regimes, not one number**: a shedding-fed chamber sits ~2x higher (173-192) because
   ``mineralization_rate`` drains N ~2.7x faster than ``decomposition_rate`` drains C,
   while a reset-driven chamber is dominated by the annual dump of N-rich dying tissue
   and reads ~10. Either way the ratio is now OBSERVABLE, which retires the decomposer
   calibration's finding that ``mineralization_rate`` is "behaviorally inert".
   ⚠ This line said "~5x" until 2026-07-27 — a leftover from the discredited 1.894
   factor, still standing after the correction that removed the factor itself. When a
   number is withdrawn, grep for its CONSEQUENCES, not just its name.
"""

from __future__ import annotations

import json
import math
from dataclasses import replace
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.allocation import Senescence
from domains.biosphere.loader import (
    load_nitrogen_params,
    load_senescence_params,
)
from domains.biosphere.nitrogen import target_n_concentration
from domains.biosphere.season import (
    LEAF_C,
    LITTER_CARBON,
    LITTER_N,
    LITTER_SINK,
    PLANT_N,
    ROOT_C,
    STEM_C,
    STORAGE_C,
    build_season,
    run_perennial,
    run_season,
    weather_resolver,
)
from domains.biosphere.step import BIO_DT, steps_for
from simcore.environment import SourceResolver
from simcore.ids import FlowId
from simcore.integrator import EulerIntegrator
from simcore.quantities import Quantity

_WEATHER = Path(__file__).parent / "oracle" / "winter_wheat_weather.json"
_M_C = 0.012011  # kg C / mol C
_CARBON_FRACTION = 0.45  # kg C / kg DM (nitrogen.yaml / canopy.yaml, cited)


def _weather(years: int = 1) -> list[dict[str, float | str]]:
    return json.loads(_WEATHER.read_text(encoding="utf-8"))["weather"] * years


def _t_per_ha(mol_c: float, ground_area: float) -> float:
    """mol C -> t DM/ha (Greenwood's basis). 1 kg/m^2 == 10 t/ha."""
    return ((mol_c * _M_C / _CARBON_FRACTION) / ground_area) * 10.0


# --- 1. the curve is the primary's, bound and all ------------------------------------
def test_target_is_constant_below_greenwoods_domain_bound() -> None:
    """Below 1 t/ha the target is FLAT at ``a`` — the paper's own statement.

    Greenwood omits all data below 1 t/ha, and says why the region is flat rather than
    leaving it undefined: "When growth is exponential plant % N remains constant and the
    critical concentration does not change with increase in plant mass (Agren, 1985)."
    """
    kw = {"coefficient": 0.05697, "exponent": 0.5, "w_plateau": 1.0}
    for w in (0.0, 1e-6, 0.09, 0.35, 0.63, 0.999, 1.0):
        assert target_n_concentration(w, **kw) == 0.05697, w


def test_target_declines_as_a_power_law_above_the_bound() -> None:
    kw = {"coefficient": 0.05697, "exponent": 0.5, "w_plateau": 1.0}
    # 4 t/ha ⇒ 5.697/sqrt(4) = 2.8485 %; 16 ⇒ 1.42425 %. Halving per 4x mass (b = 0.5).
    assert math.isclose(target_n_concentration(4.0, **kw), 0.028485, rel_tol=1e-12)
    assert math.isclose(target_n_concentration(16.0, **kw), 0.01424250, rel_tol=1e-12)
    # strictly decreasing above the bound
    ws = [1.0, 2.0, 5.0, 10.0, 20.0]
    vals = [target_n_concentration(w, **kw) for w in ws]
    assert all(b < a for a, b in zip(vals, vals[1:], strict=False))


def test_target_is_continuous_at_the_bound() -> None:
    """No step at the bound — the plateau meets the curve at ``a`` by construction."""
    kw = {"coefficient": 0.05697, "exponent": 0.5, "w_plateau": 1.0}
    assert math.isclose(
        target_n_concentration(1.0, **kw),
        target_n_concentration(1.0 + 1e-12, **kw),
        rel_tol=1e-9,
    )


def test_target_rejects_a_non_positive_plateau_bound() -> None:
    for bad in (0.0, -1.0):
        with pytest.raises(ValueError, match="n_target_w_plateau"):
            target_n_concentration(2.0, coefficient=0.05, exponent=0.5, w_plateau=bad)


def test_committed_params_are_the_values_read_off_the_primary() -> None:
    """a = 5.697 (eqn 6, NOT the abstract's rounded 5.7), b = 0.5, bound 1.0 t/ha."""
    nitro = load_nitrogen_params()
    assert nitro.n_target_coefficient == 0.05697
    assert nitro.n_target_exponent == 0.5
    assert nitro.n_target_w_plateau == 1.0


# --- 2. the crossing point: f_N == 1 is a 12 % margin, not a structural result --------
def test_the_target_meets_n_critical_at_14_42_t_per_ha() -> None:
    """``W* = (a / n_critical)^(1/b)``. Below it f_N == 1; above it the plant is
    stressed at its own target concentration.

    This file's ``n_critical`` source tag has recorded the same arithmetic since the
    citation scope's round 2 ("ours equals the curve only at W ~ 14.44 t/ha") as a DELTA
    against using a flat threshold. The form change turns it into a mechanism: the
    target IS the curve now, so this is the crop mass at which the two coincide.
    """
    nitro = load_nitrogen_params()
    n_critical_kg_kg = nitro.n_critical_per_mol_c / nitro.dm_kg_per_mol_c
    crossing = (nitro.n_target_coefficient / n_critical_kg_kg) ** (
        1.0 / nitro.n_target_exponent
    )
    assert math.isclose(crossing, 14.4248, rel_tol=1e-4), crossing
    # and the curve really does cross there, in both directions
    kw = {
        "coefficient": nitro.n_target_coefficient,
        "exponent": nitro.n_target_exponent,
        "w_plateau": nitro.n_target_w_plateau,
    }
    assert target_n_concentration(crossing * 0.99, **kw) > n_critical_kg_kg
    assert target_n_concentration(crossing * 1.01, **kw) < n_critical_kg_kg


def _open_season_peak_w() -> float:
    """``open_season``'s peak W on Greenwood's basis — whole plant EXCLUDING fibrous
    roots. Shared by the gate and the margin pin, which were one test until the
    science-gate work split them.
    """
    scenario = sc.DEFAULT_SCENARIO
    state, registry = build_season(scenario)
    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(_weather(), scenario),
        BIO_DT,
        steps_for(len(_weather())),
    )
    assert rationed == 0
    return max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            scenario.ground_area,
        )
        for s in states
    )


@pytest.mark.science_gate(
    scenario="open_season",
    field="science_bands",
    quantity="peak W excl. fibrous roots (t/ha)",
    bound="peak_w < 14.4248",
    source="Greenwood 1990 eqn (6) a=5.697 meets n_critical=1.5",
)
def test_open_season_peaks_below_the_crossing_with_the_margin_pinned() -> None:
    """⚠ THE LOAD-BEARING MARGIN. ``open_season`` is the only frozen scenario that
    enters Greenwood's declining branch at all, and it peaks at 88 % of the crossing
    mass.

    So ``f_N == 1`` across the frozen set is **not structural** — it is a ~12 % margin
    on one scenario. A weather-fixture change, a canopy improvement, or any calibration
    that grows the open-field crop ~15 % pushes the target below ``n_critical``. That is
    exactly the kind of claim that rots silently in prose, so it is asserted here.

    ⚠ **The docstring used to end "…below ``n_critical`` AND MOVES A FROZEN GOLDEN".
    The second conjunct does not follow, and a +24.5 % counterexample was measured
    2026-07-27** (``docs/plans/post-roadmap-canopy-regulator.md`` finding 6, pinned in
    ``test_senescence_form.py::test_the_greenwood_tripwire_fires_WITHOUT_f_n_biting``).
    ``f_N`` reads the plant's actual concentration, not its target. **The assertions
    below are unchanged**: as a guard, 14.4248 fires *before* the earliest measured bite
    (15.068 t/ha), which is the right direction for a tripwire to err in.
    """
    peak_w = _open_season_peak_w()
    assert peak_w < 14.4248, "open_season entered the stressed branch — f_N moved"


def test_open_season_peak_w_margin_to_the_crossing() -> None:
    """The margin narrative and the recorded value — NOT gates, deliberately unmarked.

    The band below characterizes our own number, and the ratio pin's own failure message
    says what it is for: detecting that *prose* went stale. Neither bounds the science
    against an outside source, and freezing the second would let an unfreeze ceremony
    fail because a doc sentence drifted. The sourced half — 14.4248 crossing — is the
    gate above.

    ⚠ This paragraph used to quote ``12.0 < peak_w < 13.0`` as the live band. It had not
    been that since 2026-08-12 and is now ``13.9 < peak_w < 14.4248`` — the docstring
    was describing a band two revisions old while the comment below recorded both moves.
    **Prose that names a number goes stale in the file that owns the number**, which is
    this test's own subject, so it is fixed rather than left as an irony.
    """
    peak_w = _open_season_peak_w()
    # ⚠ RE-MEASURED TWICE ON 2026-08-12: 12.633098 -> 14.019448 (the stem-reserve
    # build) -> 13.939142 (its cessation window), so the original 12.0-13.0 band no
    # longer holds. THE MARGIN THIS TEST GUARDS HAS NARROWED SHARPLY — from 12.4 % under
    # the 14.4248 crossing to 3.4 % — and that is the finding, not the new number. The
    # window gave a little of it back (2.8 % -> 3.4 %), which is worth seeing and is
    # not a reason to like the window. ``test_senescence_form`` carries the paired pin,
    # the reserve-off control that attributes the move, and — since the reserve exists —
    # the SAME quantity measured on BOTH W definitions, because `W` here is
    # leaf+stem+storage and reserve starch is defensibly dry matter too.
    #
    # ⚠⚠ **RE-MEASURED A THIRD TIME 2026-08-14, AND THE PARAGRAPH ABOVE WENT STALE THE
    # WAY IT WAS WRITTEN TO CATCH.** The quarter-day step moved this to **14.107660**
    # and the margin **3.4 % -> 2.20 %** — the narrowest it has ever been — and *nothing
    # went red*, because the band below spans both values and the ratio guard sat at
    # 0.85 while the quantity lives at 0.978. The step-unfreeze ceremony re-pinned every
    # test that FAILED; this one passed, so it was never re-read.
    #
    # ⇒ **A characterization pin wide enough to survive the change it characterizes is
    # not a tripwire, it is decoration.** The ratio guard is therefore tightened to 0.97
    # — ~1 % of headroom below today's 0.978, so the next move of this size trips it.
    # Tightening a tripwire toward the measured value is the opposite of the retune the
    # freeze discipline forbids: nothing here bounds the science (the sourced half, the
    # 14.4248 crossing, is the gate above and is UNCHANGED), and a guard that cannot
    # fire protects nothing.
    #
    # ⚠ The cause is the STEP, not a mechanism: this is the frozen tree, and the same
    # refinement moves peak LAI 5.4624 -> 5.5719 on the same run. Measured in
    # `docs/plans/post-roadmap-leaf-remeasurement.md`, which found it while re-measuring
    # something else — which is why it is committed on its own rather than folded in.
    #
    # ⚠⚠ **RE-MEASURED A FOURTH TIME 2026-08-14 — AND THIS TIME THE GUARD FIRED.** The
    # within-day light path moved peak W to **13.740221** and the margin the *other*
    # way,
    # **2.20 % -> 4.75 %**: less carbon per day (the FvCB light response is concave, so
    # the same photons delivered unevenly fix less) means a smaller crop and more room
    # under the crossing. The 0.97 ratio guard tightened one day earlier went red on the
    # very next change — which is what a tripwire that cannot fire was replaced to do.
    #
    # ⇒ The guard becomes **two-sided**. The one-sided form only ever caught the margin
    # NARROWING, and this move widened it; a narrative pin that is blind to half the
    # directions its subject can move is the same decoration in a different pose. The
    # band is ~1 % either side of today's 0.95254.
    #
    # ⚠ Cause: the light path (`docs/plans/post-roadmap-gross-net-gas-exchange.md`), not
    # the step, which is unchanged at ¼. The sourced half — the 14.4248 crossing — is
    # still UNTOUCHED, and this move is *away* from it.
    assert 13.5 < peak_w < 13.9, peak_w
    assert 0.945 < peak_w / 14.4248 < 0.962, (
        "the margin narrative is stale; re-measure it"
    )


def test_only_open_season_enters_the_declining_branch() -> None:
    """Six of the seven frozen scenarios never leave the plateau (they are
    carbon-limited chambers), which is why the plateau reading — not the declining
    branch — decided the form. Measured, because the plan doc's finding 9 rests on
    it.
    """
    cases = [
        (sc.SEALED_CHAMBER_SCENARIO, 3),
        (sc.PERENNIAL_CHAMBER_SCENARIO, 5),
        (sc.CONSUMER_CHAMBER_SCENARIO, 5),
    ]
    for scenario, years in cases:
        state, registry = build_season(scenario)
        weather = _weather(years)
        states, _, _ = run_season(
            EulerIntegrator(registry),
            state,
            weather_resolver(weather, scenario),
            BIO_DT,
            steps_for(len(weather)),
        )
        peak_w = max(
            _t_per_ha(
                s.stocks[LEAF_C].amount
                + s.stocks[STEM_C].amount
                + s.stocks[STORAGE_C].amount,
                scenario.ground_area,
            )
            for s in states
        )
        assert peak_w < 1.0, (scenario, peak_w)


# --- 3. one physical event, two currency legs ----------------------------------------
def test_shed_nitrogen_uses_the_same_carbon_flux_as_the_senescence_flow() -> None:
    """The N-shedding flow RECOMPUTES the senescence flux, because a flow may only read
    the step-entry snapshot and there is no channel to hand it across. That
    recomputation is the hazard: if the two ever disagree, litter C:N silently
    decouples from the physics.

    Pinned by comparing the shed-N leg against ``Senescence``'s own litter-carbon leg,
    through the ratio the coupled law asserts.
    """
    scenario = sc.SEALED_CHAMBER_SCENARIO
    state, registry = build_season(scenario)
    weather = _weather()
    states, _, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        BIO_DT,
        steps_for(60),
    )
    nitro = load_nitrogen_params()
    sen_flow = Senescence(
        FlowId("probe.senescence"),
        0,
        leaf_c=LEAF_C,
        stem_c=STEM_C,
        root_c=ROOT_C,
        litter_sink=LITTER_SINK,
        params=load_senescence_params(),
    )
    n_flow = next(
        f for f in registry.flows if str(f.id) == "biosphere.nitrogen_senescence"
    )
    checked = 0
    for snapshot in states[1:]:
        env = SourceResolver(forcings={}).bind(snapshot, 1.0)
        shed_c = next(
            leg.amount
            for leg in sen_flow.evaluate(snapshot, env, 1.0).legs
            if leg.stock == LITTER_SINK
        )
        shed_n = next(
            leg.amount
            for leg in n_flow.evaluate(snapshot, env, 1.0).legs
            if leg.stock == LITTER_N
        )
        if shed_c <= 0.0:
            continue
        biomass = sum(snapshot.stocks[o].amount for o in (LEAF_C, STEM_C, ROOT_C))
        conc = snapshot.stocks[PLANT_N].amount / biomass
        expected = min(conc, nitro.n_residual_per_mol_c) * shed_c
        assert math.isclose(shed_n, expected, rel_tol=1e-12), snapshot.n
        checked += 1
    assert checked > 30, checked  # non-vacuous


# --- 4. litter C:N, and where the residual error lives -------------------------------
def test_shed_material_has_a_straw_like_carbon_to_nitrogen_ratio() -> None:
    """The SHED material's C:N is ``carbon_fraction / n_residual`` = 0.45/0.005 = 90.

    Both terms are cited (Raimanova et al. 2024; Van Hecke et al. 2020), so the C:N of
    dying tissue is now a consequence of two measured concentrations rather than the
    ratio of two unrelated rate constants. Wheat straw is ~80, so this is the right
    quantity in the right place — which is the actual deliverable of the form change.
    """
    nitro = load_nitrogen_params()
    n_residual_kg_kg = nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c
    shed_cn = _CARBON_FRACTION / n_residual_kg_kg
    assert math.isclose(shed_cn, 90.0, rel_tol=1e-9)
    assert 60.0 < shed_cn < 120.0, "shed C:N left the real-residue band"


def _litter_rows(
    scenario: sc.SeasonScenario, years: int, *, resets: bool
) -> list[tuple[float, float]]:
    """``(litter_n, litter_carbon)`` per step, under the scenario's OWN golden driver.

    ⚠ ``resets`` is not a knob — it is a property of the scenario, and getting it wrong
    is what this module's correction was about. ``PERENNIAL_CHAMBER_SCENARIO`` and
    ``CONSUMER_CHAMBER_SCENARIO`` are driven by :func:`run_perennial` in
    ``test_regression_perennial_season.py`` / ``test_regression_consumer_season.py``;
    the annual reset IS what makes them perennial. Running them through
    :func:`run_season` produces a *different chamber* — see the caller's docstring.
    """
    season = _weather(1)
    weather = season * years
    state, registry = build_season(scenario)
    resolver = weather_resolver(weather, scenario)
    if resets:
        states, rationed, _ = run_perennial(
            EulerIntegrator(registry),
            state,
            scenario,
            resolver,
            BIO_DT,
            steps_for(len(weather)),
            year=steps_for(len(season)),
        )
    else:
        states, rationed, _ = run_season(
            EulerIntegrator(registry), state, resolver, BIO_DT, steps_for(len(weather))
        )
    assert rationed == 0
    return [(s.stocks[LITTER_N].amount, s.stocks[LITTER_CARBON].amount) for s in states]


def test_litter_pool_cn_is_TWO_regimes_set_by_which_event_fills_the_pool() -> None:
    """⚠ THE LAW CHANGED WHEN THE FORM DID - and the change is the payoff of (B).

    **The old law, and why it existed.** Under the retired direct ``Mineralization``,
    nitrogen left the litter pool at a FREE ``mineralization_rate`` (0.03/day) while
    carbon left it at ``decomposition_rate`` (0.011/day). Two independent rates on one
    pool push its ratio away from its input's, and the quasi-steady law for a pool under
    continuous input was::

        pool C:N  ->  (shed C:N) x (k_min / k_decomp)   =  90 x 2.727  =  245.5

    **The new law, measured.** The microbe-mediated legs carry nitrogen on the very
    carbon flux their decomposer siblings move, so BOTH currencies now leave the litter
    pool on the same first-order flux ``decomposition_rate``. The pushing factor is
    therefore exactly **1**, and the pool converges on the ratio of the material fed
    into
    it::

        pool C:N  ->  shed C:N  =  carbon_fraction / n_residual  =  90

    That is not a smaller error; it is a different KIND of quantity. The litter pool's
    C:N is no longer set by an accident of two unrelated rate constants - it is set by
    the composition of the material that fell into it, and both of the numbers that fix
    that composition are cited (``carbon_fraction`` 0.45, ``n_residual`` Van Hecke
    2020).

    ⚠ **THREE PREVIOUSLY-PINNED CLAIMS ARE RETIRED HERE, and none of them was
    WRONG** - each was a true measurement of a form that no longer exists. Recording the
    distinction matters, because this project's habit is to retire artefacts, and these
    are not artefacts:

    * the ``245.5`` quasi-steady law - its ``k_min`` no longer exists;
    * *"a shedding-fed pool runs N-poor at 0.71-0.78 of the law"* - it ran N-poor
      BECAUSE N drained 2.7x faster than C; with equal drains it does not;
    * *"the end-of-run snapshot is inflated, ``sealed_chamber`` ends at 2.4x its own
      peak-time value"* - the inflation was the differential drain showing up in the
      tail between pulses. With no differential there is no inflation:
      ``sealed_chamber``
      now ends at **90.6**, i.e. 0.90x its peak-time value and within 0.7 % of the shed
      ratio. **The horizon-dependence that correction 1 was written to stop is gone at
      its source**, which is a stronger outcome than continuing to guard against it.

    **What did NOT change: there are still two regimes, and the second is untouched.**
    A reset-driven chamber's pool is filled by the ANNUAL DUMP, not by shedding, and the
    dump's C:N is set by the dying plant rather than by any rate - so (B) barely moves
    it (10.9 -> 10.0, 9.9 -> 9.1). "Peak ``litter_n``" still silently names two
    different
    events: the seasonal senescence maximum in a shedding-fed chamber, versus the dump
    one step past a year boundary in a reset-driven one. That dumped material is N-rich
    (C:N ~5.6-6.1) because a senescing plant retains its N while its biomass denominator
    collapses - this work's own recorded limitation 5, which SETS this regime rather
    than
    footnoting it.

    ⚠ Each scenario is driven the way ITS OWN GOLDEN drives it. Correction 2 exists
    because a previous version drove the perennial chambers through :func:`run_season`,
    and the annual reset is what makes them perennial.
    """
    nitro = load_nitrogen_params()
    n_residual_kg_kg = nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c
    shed_cn = _CARBON_FRACTION / n_residual_kg_kg
    assert math.isclose(shed_cn, 90.0, rel_tol=1e-6), shed_cn

    def peak(rows: list[tuple[float, float]]) -> tuple[float, int]:
        peak_n = max(r[0] for r in rows)
        assert peak_n > 0.0
        i = next(k for k, r in enumerate(rows) if r[0] == peak_n)
        return rows[i][1] * _M_C / peak_n, i

    # --- regime 1: SHEDDING-FED (no annual reset) ------------------------------------
    shedding_ratios: list[float] = []
    for scenario, years in (
        (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS),
        (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS),
    ):
        rows = _litter_rows(scenario, years, resets=False)
        at_peak, _ = peak(rows)
        # AT the shed ratio now, not 2.7x pushed away from it. The residual above
        # it is NOT a "pulsed transient" (equal drains make the ratio exactly invariant
        # between pulses) -- it is the N-free `litter_carbon0` seed these scenarios
        # carry, measured in
        # test_the_pool_cn_IS_the_shed_ratio_and_the_deviation_is_the_N_FREE_SEED.
        assert 95.0 < at_peak < 110.0, (scenario, at_peak)
        assert 1.0 < at_peak / shed_cn < 1.2, (scenario, at_peak)
        # and therefore within ~1.25x of real wheat straw (~80), where the frozen
        # pre-(A) form gave 0.004 and the post-(A) direct form gave 173-192. ⚠ These
        # two bounds are SCENARIO facts, not model facts: adding a `litter_n0`
        # counterpart would drop them to the model's own 1.125x and SHOULD go red.
        assert 1.1 < at_peak / 80.0 < 1.4, (scenario, at_peak)
        shedding_ratios.append(at_peak / shed_cn)

    # --- regime 2: RESET-DUMP-DOMINATED ----------------------------------------------
    season_len = len(_weather(1))
    for scenario, years in (
        (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS),
        (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS),
    ):
        rows = _litter_rows(scenario, years, resets=True)
        at_peak, i_peak = peak(rows)
        # an order of magnitude BELOW the shedding-fed regime, and below real straw
        assert 5.0 < at_peak < 20.0, (scenario, at_peak)
        # the mechanism, asserted rather than described: the peak IS the annual dump -
        # it lands one step past a year boundary (the reset fires before that step).
        assert i_peak % season_len == 1, (scenario, i_peak, season_len)
        # and the dumped material itself is N-rich, which is what pulls the pool down
        dump_c = rows[i_peak][1] - rows[i_peak - 1][1]
        dump_n = rows[i_peak][0] - rows[i_peak - 1][0]
        assert dump_c > 0.0 and dump_n > 0.0, (scenario, dump_c, dump_n)
        # ⚠ RE-PINNED 2026-08-14 (was ``4.0 < x < 8.0``). The light path made the dumped
        # material ~12 % more carbon-rich — perennial 7.866 -> 8.843, consumer
        # 7.544 -> 7.876 — so the perennial crossed a bound the committed tree was
        # already sitting at the top of (7.866 of 8.0). ⚠ That is worth naming: the old
        # upper edge was never a claim about anything, it was one measurement's
        # neighbourhood rounded up, and a band with no headroom fails on the next change
        # whichever direction the science goes. The claim this line is *for* — the dump
        # is N-rich, which is what pulls the pool down — is unharmed at 8.8 against the
        # shedding-fed regime's ~100 and real straw's ~80.
        assert 7.0 < (dump_c * _M_C / dump_n) < 9.5, (scenario, dump_c * _M_C / dump_n)

    # --- the ANTI-REGRESSION pins ----------------------------------------------------
    # 1. The two regimes stay an order of magnitude apart. Kept from correction 2 (in
    #    the inverted direction it was rewritten to), because "the four sealed chambers
    #    share one litter C:N" is the claim that was false twice and must stay
    #    refutable.
    reset_ratios = [
        peak(_litter_rows(s, y, resets=True))[0] / shed_cn
        for s, y in (
            (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS),
            (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS),
        )
    ]
    assert min(shedding_ratios) / max(reset_ratios) > 5.0, (
        shedding_ratios,
        reset_ratios,
    )
    # 2. ⚠ THE INVERTED SUCCESSOR to correction 1's end-of-run pin. That pin
    #    asserted the tail was INFLATED (`end / peak > 2`); under (B) it must NOT be,
    #    because the inflation WAS the differential drain. Asserting the new
    #    direction is what would catch a regression re-introducing an independent N
    #    drain - including
    #    the specific one of collapsing a leg back to a bare rate.
    sealed = _litter_rows(
        sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, resets=False
    )
    sealed_peak, _ = peak(sealed)
    sealed_end = sealed[-1][1] * _M_C / sealed[-1][0]
    assert 0.8 < sealed_end / sealed_peak < 1.05, (sealed_end, sealed_peak)
    assert math.isclose(sealed_end, shed_cn, rel_tol=0.05), (sealed_end, shed_cn)


def test_the_free_mineralization_rate_no_longer_EXISTS_to_be_calibrated() -> None:
    """⚠ RETIRES the scope-B projection test - its premise is gone, not its maths.

    The retired test (``test_the_cited_mineralization_range_would_put_litter_cn_at_real_
    residue``) projected what the litter pool's C:N *would* be if
    ``mineralization_rate``
    were moved into Stanford & Smith's 39-soil range: 31-83, pooled mean 47, against
    ~184
    for our uncited 0.03/day. It was correct, and it was the second of two independent
    lines saying that rate was too fast.

    **It is retired because the rate it would calibrate no longer exists.** The
    microbe-mediated legs carry N on the carbon flux, and
    ``decomposed_C / litter_C == decomposition_rate`` identically, so there is no free N
    rate left to move into any range. The question was not answered - a value was not
    chosen from a cited band - it was **dissolved**: the quantity stopped being a free
    parameter of the model.

    That also disposes of the surviving objection. The decomposer calibration declined
    to
    move ``mineralization_rate`` on three grounds; the (A) work falsified two (it was
    not
    behaviorally inert, and N/C were no longer uncoupled), leaving only POOL IDENTITY -
    that S&S measured soil organic N0 while ours is fresh residue N. A parameter that
    does not exist cannot be mis-anchored to the wrong pool, so that objection is now
    moot rather than merely refused.

    ⚠ What this does NOT claim: the decomposer cluster's *carbon* rates are
    untouched and still run at the fast edge of their literature ranges
    (``decomposition_rate`` 4.0/yr, Olson's fastest ecosystem). Retiring an N rate says
    nothing about them, and the N pool's C:N now inherits whatever that carbon rate is.
    The honest statement is that the N cycle no longer contributes a *separate* uncited
    rate - not that the decomposer side is now fully cited.
    """
    import domains.biosphere.mineralization as mineralization_module

    # No mineralization rate survives anywhere in the module's surface.
    assert not hasattr(mineralization_module, "mineralization_flux")
    assert not hasattr(mineralization_module, "MineralizationParams")
    assert not hasattr(mineralization_module, "Mineralization")

    # And the pool C:N is now a function of CITED quantities only: the shed composition.
    nitro = load_nitrogen_params()
    shed_cn = _CARBON_FRACTION / (nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c)
    rows = _litter_rows(
        sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, resets=False
    )
    peak_n = max(r[0] for r in rows)
    i = next(k for k, r in enumerate(rows) if r[0] == peak_n)
    at_peak = rows[i][1] * _M_C / peak_n
    # At the cited shed ratio, up to this scenario's N-free `litter_carbon0` seed (see
    # the seed test below) - no free rate in sight either way.
    assert 1.0 < at_peak / shed_cn < 1.2, (at_peak, shed_cn)


def test_the_pool_cn_IS_the_shed_ratio_and_the_deviation_is_the_N_FREE_SEED() -> None:
    """⚠ CORRECTS this file's own first attribution — and the correction makes the
    result STRONGER, which is exactly why it was worth chasing rather than leaving.

    The test above measures the shedding-fed pool at 98.7-100.6 against a shed ratio of
    90, and the first write-up called that gap "the pulsed-input transient". **That
    explanation cannot be right, and the reason is one line of algebra.** With both
    currencies draining on the same first-order flux, ``dC/dt = -kC`` and
    ``dN/dt = -kN``, so ``d(C/N)/dt = 0``: the ratio is *exactly invariant* between
    pulses. Pulsing structurally cannot move it. That mechanism was real under the
    RETIRED form — where N drained 2.7x faster than C, so a tail between pulses really
    did inflate — and it got carried forward into a regime where it no longer exists.
    **The same shape as the three claims this work retired one function above**: an
    explanation outliving the mechanism that made it true.

    The pool can only sit above the shed ratio if something *entered* above it, and the
    chambers do exactly that: they seed ``litter_carbon0 = 3.0`` mol C with **no
    ``litter_n0`` counterpart**, i.e. C:N = infinity. That seam was already named in the
    (A) record; it turns out to be the whole of the deviation.

    Measured with the seed removed, the pool C:N is the shed ratio **to 1.4e-15
    relative, at every step of the run** — not a band, an identity. So:

    * the MODEL's litter pool C:N is ``carbon_fraction / n_residual`` exactly, i.e.
      **1.125x wheat straw's ~80**, not 1.25x;
    * the committed scenarios' deviation is a known **unphysical initial condition**,
      and it decays at ``decomposition_rate`` like anything else in the pool — which is
      why ``sealed_chamber`` (3 yr) ends at 90.6 while ``water_biting`` (1 yr) still
      reads 98.6 with ~0.10 mol of seed carbon left.

    ⚠ If anyone ever adds a ``litter_n0`` counterpart, the committed-scenario bounds in
    the test above should go red — and that would be a **good** reason: it would mean
    the scenarios stopped seeding N-free carbon.
    """
    nitro = load_nitrogen_params()
    shed_cn = _CARBON_FRACTION / (nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c)

    for scenario, years in (
        (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS),
        (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS),
    ):
        seedless = replace(scenario, litter_carbon0=0.0)
        rows = _litter_rows(seedless, years, resets=False)
        ratios = [c * _M_C / n for n, c in rows if n > 0.0]
        assert ratios, scenario
        # An IDENTITY at every step, not a band at the peak.
        worst = max(abs(r - shed_cn) / shed_cn for r in ratios)
        assert worst < 1e-12, (scenario, worst, min(ratios), max(ratios))

    # And the committed scenarios sit ABOVE it by exactly the seed's doing: the same
    # scenario with the seed present is measurably higher at its peak.
    seeded = _litter_rows(
        sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, resets=False
    )
    peak_n = max(r[0] for r in seeded)
    i = next(k for k, r in enumerate(seeded) if r[0] == peak_n)
    assert seeded[i][1] * _M_C / peak_n > shed_cn * 1.05
    assert sc.SEALED_CHAMBER_SCENARIO.litter_carbon0 > 0.0


def test_nitrogen_is_conserved_across_the_annual_reset() -> None:
    """The reset now moves NITROGEN too (it used to be carbon-only, leaving the seedling
    an N windfall). It must balance exactly: the seed keeps the parent's
    concentration and the remainder goes to litter, so total N is untouched by the
    transform.
    """
    scenario = sc.PERENNIAL_CHAMBER_SCENARIO
    state, registry = build_season(scenario)
    weather = _weather(5)
    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        BIO_DT,
        steps_for(len(weather)),
    )
    assert rationed == 0

    def total_n(s) -> float:  # noqa: ANN001 - State
        return sum(
            st.amount * st.composition.get(Quantity.NITROGEN, 0.0)
            for st in s.stocks.values()
        )

    start = total_n(states[0])
    for s in states:
        # abs_tol 1e-9, matching test_mineralization's own
        # test_sealed_conserves_nitrogen_exactly, whose comment explicitly rejects 1e-12
        # for this quantity at this scale ("total N ~ 100, soil_n-dominated"). This run
        # is 1825 steps; a tighter bound than the sibling's is how a Windows-green test
        # goes red on the Linux CI job.
        assert math.isclose(total_n(s), start, rel_tol=0.0, abs_tol=1e-9), s.n
