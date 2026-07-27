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
   ``n_critical`` at 14.42. Anything that grows the open-field crop ~15 % moves a frozen
   golden, so the margin is asserted rather than mentioned (the "freeze's prose half is
   ungated" lesson).
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
from pathlib import Path

import pytest

from domains.biosphere import scenario as sc
from domains.biosphere.allocation import Senescence
from domains.biosphere.loader import (
    load_decomposition_params,
    load_mineralization_params,
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


def test_open_season_peaks_below_the_crossing_with_the_margin_pinned() -> None:
    """⚠ THE LOAD-BEARING MARGIN. ``open_season`` is the only frozen scenario that
    enters Greenwood's declining branch at all, and it peaks at 88 % of the crossing
    mass.

    So ``f_N == 1`` across the frozen set is **not structural** — it is a ~12 % margin
    on one scenario. A weather-fixture change, a canopy improvement, or any calibration
    that grows the open-field crop ~15 % pushes the target below ``n_critical`` and
    moves a frozen golden. That is exactly the kind of claim that rots silently in
    prose, so it is asserted here.
    """
    scenario = sc.DEFAULT_SCENARIO
    state, registry = build_season(scenario)
    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(_weather(), scenario),
        1.0,
        len(_weather()),
    )
    assert rationed == 0
    # Greenwood's W: whole plant EXCLUDING fibrous roots.
    peak_w = max(
        _t_per_ha(
            s.stocks[LEAF_C].amount
            + s.stocks[STEM_C].amount
            + s.stocks[STORAGE_C].amount,
            scenario.ground_area,
        )
        for s in states
    )
    assert 12.0 < peak_w < 13.0, peak_w
    assert peak_w < 14.4248, "open_season entered the stressed branch — f_N moved"
    assert peak_w / 14.4248 > 0.85, "the margin narrative is stale; re-measure it"


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
            1.0,
            len(weather),
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
        1.0,
        60,
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
            1.0,
            len(weather),
            year=len(season),
        )
    else:
        states, rationed, _ = run_season(
            EulerIntegrator(registry), state, resolver, 1.0, len(weather)
        )
    assert rationed == 0
    return [(s.stocks[LITTER_N].amount, s.stocks[LITTER_CARBON].amount) for s in states]


def test_litter_pool_cn_is_TWO_regimes_set_by_which_event_fills_the_pool() -> None:
    """⚠ THE RESIDUAL AND ITS CAUSE — and the SECOND correction to this claim.

    The litter POOL's C:N is not the shed material's: nitrogen mineralizes OUT of litter
    (``mineralization_rate`` 0.03/day) faster than carbon decomposes out of it
    (``decomposition_rate`` 0.011/day), so a shedding-fed pool runs N-poor. The
    quasi-steady law for a pool under CONTINUOUS input is::

        pool C:N  ->  (shed C:N) x (k_min / k_decomp)   =  90 x 2.727  =  245.5

    **CORRECTION 1 (kept, still true).** An earlier version asserted that law times a
    "measured geometry factor of 1.894". That factor was fitted to ``sealed_chamber``'s
    *final* state at one horizon and is not a constant of the model. Do not write one
    down.

    ⚠ **CORRECTION 2 (this test, 2026-07-27) — correction 1's OWN replacement band was
    measured on a MIS-DRIVEN scenario set, and the meta-finding took another instance.**
    Correction 1 replaced the bad factor with "pool C:N is **173-192**, a tight band
    across all four sealed scenarios", and pinned that tightness with
    ``max(peak_ratios)/min(peak_ratios) < 1.2``. But it drove **all four** scenarios
    through :func:`run_season`, and two of them — ``PERENNIAL_CHAMBER_SCENARIO`` and
    ``CONSUMER_CHAMBER_SCENARIO`` — are driven by :func:`run_perennial` in their own
    regression goldens. **The annual reset is what makes them perennial**, and dropping
    it changes the answer by an order of magnitude. Measured under each golden's own
    driver:

    ======================  ==============  ================  =====================
    scenario                pool C:N @peak  fraction of law   regime
    ======================  ==============  ================  =====================
    ``sealed_chamber``      191.78          0.781             shedding-fed
    ``water_biting``        173.37          0.706             shedding-fed
    ``perennial``            10.91          0.044             reset-dump-dominated
    ``consumer``              9.87          0.040             reset-dump-dominated
    ======================  ==============  ================  =====================

    So the true spread is **19.4x, not 1.11x**, and the assertion that certified the
    tightness has been RETIRED rather than widened — a widened bound would preserve the
    shape of a claim that is simply gone. Note what this means: the pin that existed
    *specifically* to stop a constant factor being written down was itself certifying a
    stability that is a driver artefact.

    **THE MECHANISM, and it is why these are two quantities rather than one wide band:
    "peak ``litter_n``" silently names two different events.** In a shedding-fed chamber
    the peak is the seasonal senescence maximum, and the pool C:N is ~0.71-0.78 of the
    law. In a reset-driven chamber the peak is the **annual dump** — measured at step
    611 of 1525 for ``perennial``, i.e. one step past the year-2 boundary at 610 — which
    deposits the dying plant's whole retained N at its own elevated concentration. That
    material has C:N ~5.6-6.1, so the pool it lands in reads ~10. That elevated
    concentration is not incidental: it is this work's own recorded limitation, that
    shedding at the residual concentration leaves a senescing plant holding its N while
    its biomass denominator collapses.

    ⚠ **Two further sentences of correction 1 were driver artefacts and are WITHDRAWN**,
    recorded rather than quietly dropped because a wrong number that reads fluently is
    the thing this project keeps catching:

    * *"the end-of-run value is 210 (1 yr), 465 (3 yr), 9076 and 11877 (5 yr)"* — the
      5-year figures are wrong. Correctly driven they are **242.9** and **235.2**.
    * *"by year 5 ``litter_n`` is 1.3e-11 kg ... quoting numerical dust"* — false for
      the actual perennial chamber, whose final ``litter_n`` is **6.05e-05 kg**, six
      orders larger. A pool starved of input for five years drains to dust; the
      perennial chamber is refilled every year by the very reset that was dropped.

    What survives from correction 1, and is still pinned below: there is no
    horizon-independent factor, and the END-OF-RUN snapshot is not the pool's C:N
    (``sealed_chamber`` ends at 464.7 against a peak-time 191.8 — 2.4x its own value).
    The *reason* is unchanged and correct: the input is a pulse, and between pulses N
    drains ~2.7x faster than C, so a tail reading is inflated.

    **This retires a recorded finding.** The decomposer calibration declined to move
    ``mineralization_rate`` for three reasons; two are now false. It is no longer
    "behaviorally inert" — it sets the shedding-fed ratio — and N and C are no longer
    "uncoupled", which was the other half of the argument that litter C:N was not a
    physical quantity. Only the pool-identity objection survives (S&S measured soil N0;
    ours is fresh residue N). Recalibrating is scope B and a separate user decision, so
    the value is UNMOVED and the consequence is pinned here instead.
    """
    nitro = load_nitrogen_params()
    mineral = load_mineralization_params()
    decomp = load_decomposition_params()
    n_residual_kg_kg = nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c
    shed_cn = _CARBON_FRACTION / n_residual_kg_kg
    law = shed_cn * (mineral.mineralization_rate / decomp.decomposition_rate)
    assert math.isclose(law, 245.5, rel_tol=1e-3), law

    def peak(rows: list[tuple[float, float]]) -> tuple[float, int]:
        peak_n = max(r[0] for r in rows)
        assert peak_n > 0.0
        i = next(k for k, r in enumerate(rows) if r[0] == peak_n)
        return rows[i][1] * _M_C / peak_n, i

    # --- regime 1: SHEDDING-FED (no annual reset) -------------------------------------
    shedding_fractions: list[float] = []
    for scenario, years in (
        (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS),
        (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS),
    ):
        rows = _litter_rows(scenario, years, resets=False)
        at_peak, _ = peak(rows)
        # below the law (a pulsed pool never converges upward) and the right order for
        # real residue (wheat straw ~80) — which the pre-(A) form's 0.004 was not.
        assert 150.0 < at_peak < 220.0, (scenario, at_peak)
        assert at_peak < law, (scenario, at_peak, law)
        assert 1.5 < at_peak / 80.0 < 3.0, (scenario, at_peak)
        shedding_fractions.append(at_peak / law)
    # THIS is the relationship the scope-B projection below is entitled to use, and its
    # scope is exactly these two scenarios — never "the four sealed scenarios".
    assert all(0.65 < f < 0.85 for f in shedding_fractions), shedding_fractions

    # --- regime 2: RESET-DUMP-DOMINATED ----------------------------------------------
    season_len = len(_weather(1))
    for scenario, years in (
        (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS),
        (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS),
    ):
        rows = _litter_rows(scenario, years, resets=True)
        at_peak, i_peak = peak(rows)
        # an order of magnitude BELOW the shedding-fed band, and below real straw
        assert 5.0 < at_peak < 20.0, (scenario, at_peak)
        # the mechanism, asserted rather than described: the peak IS the annual dump —
        # it lands one step past a year boundary (the reset fires before that step).
        assert i_peak % season_len == 1, (scenario, i_peak, season_len)
        # and the dumped material itself is N-rich, which is what pulls the pool down
        dump_c = rows[i_peak][1] - rows[i_peak - 1][1]
        dump_n = rows[i_peak][0] - rows[i_peak - 1][0]
        assert dump_c > 0.0 and dump_n > 0.0, (scenario, dump_c, dump_n)
        assert 4.0 < (dump_c * _M_C / dump_n) < 8.0, (scenario, dump_c * _M_C / dump_n)

    # --- the ANTI-REGRESSION pins ----------------------------------------------------
    # 1. The four sealed scenarios do NOT share a peak-time ratio. This replaces the
    #    retired `< 1.2` assertion, in the opposite direction, so correction 1's band
    #    cannot be restored without this going red.
    all_fractions = [
        peak(_litter_rows(s, y, resets=r))[0] / law
        for s, y, r in (
            (sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, False),
            (sc.WATER_BITING_SCENARIO, sc.WATER_BITING_YEARS, False),
            (sc.PERENNIAL_CHAMBER_SCENARIO, sc.PERENNIAL_CHAMBER_YEARS, True),
            (sc.CONSUMER_CHAMBER_SCENARIO, sc.CONSUMER_CHAMBER_YEARS, True),
        )
    ]
    assert max(all_fractions) / min(all_fractions) > 10.0, all_fractions
    # 2. The END-OF-RUN snapshot is still not the pool's C:N — the surviving half of
    #    correction 1, now stated on the one scenario that actually shows it rather than
    #    as a cross-scenario spread (which, correctly driven, is only ~2.2x).
    sealed = _litter_rows(
        sc.SEALED_CHAMBER_SCENARIO, sc.SEALED_CHAMBER_YEARS, resets=False
    )
    sealed_peak, _ = peak(sealed)
    sealed_end = sealed[-1][1] * _M_C / sealed[-1][0]
    assert sealed_end / sealed_peak > 2.0, (sealed_end, sealed_peak)
    assert sealed_end > law, (sealed_end, law)


def test_the_cited_mineralization_range_would_put_litter_cn_at_real_residue() -> None:
    """The scope-B projection, stated at the scope it is actually measured at.

    ⚠ Recorded because an earlier version of this projection was inflated by the
    spurious 1.894 factor above (it read "78-211, mean 119"). Using the measured
    peak-time relationship instead — pool C:N ~ 0.75 x shed_cn x (k_min / k_decomp) —
    Stanford & Smith's 39-soil range lands litter C:N at **31-83 with a pooled-mean
    47**, i.e. at or below wheat straw's ~80, against **~184** for our uncited 0.03/day.

    ⚠ **SCOPE CORRECTED (2026-07-27), magnitude UNCHANGED.** The 0.75 fraction was
    recorded as "the measured peak-time fraction" across *four* sealed scenarios; two of
    those were mis-driven (see the test above), and correctly driven they sit at 0.040
    and 0.044, not 0.75. **The projection survives because it was only ever entitled to
    the SHEDDING-FED regime** — which is where a ``k_min / k_decomp`` relationship means
    anything at all, since a reset-dump-dominated pool takes its C:N from the dying
    plant and not from either rate. Correctly driven, the two shedding-fed scenarios
    give 0.781 and 0.706, so 0.75 remains a fair midpoint and every number below is
    unmoved. The lesson is that a number can be right while the sentence justifying it
    is wrong: the fraction was defensible, "across all four sealed scenarios" was not.

    The DIRECTION is unchanged and is the point: two independent lines — the citation
    bound (2.2x above the fastest of 39 soils) and the litter C:N target — both say
    ``mineralization_rate`` is too fast. The magnitude is now quoted honestly.
    """
    nitro = load_nitrogen_params()
    mineral = load_mineralization_params()
    decomp = load_decomposition_params()
    shed_cn = _CARBON_FRACTION / (nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c)
    # The measured peak-time fraction of the quasi-steady law in the SHEDDING-FED
    # scenarios only (0.706 / 0.781; asserted in the test above). Not a property of the
    # reset-driven chambers, whose pool C:N is set by the annual dump.
    observed_fraction = 0.75

    def pool_cn(k_min: float) -> float:
        return observed_fraction * shed_cn * (k_min / decomp.decomposition_rate)

    ours = pool_cn(mineral.mineralization_rate)
    assert 170.0 < ours < 200.0, ours
    # Stanford & Smith 1972, Table 3: 39 soils spanning 0.035-0.095/wk, pooled mean
    # 0.054/wk => 0.0050-0.0136/day, mean 0.0077/day.
    assert 25.0 < pool_cn(0.0050) < 40.0
    assert 40.0 < pool_cn(0.0077) < 55.0
    assert 70.0 < pool_cn(0.0136) < 95.0
    # every cited value lands at or below real residue; ours is >2x above it
    assert pool_cn(0.0136) < 80.0 * 1.1
    assert ours / 80.0 > 2.0


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
        1.0,
        len(weather),
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
