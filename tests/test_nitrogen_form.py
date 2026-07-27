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
   material is straw-like (C:N 90, from two cited params); the litter POOL sits ~5x
   higher because ``mineralization_rate`` drains N ~2.7x faster than
   ``decomposition_rate`` drains C. That ratio is now OBSERVABLE, which retires the
   decomposer calibration's finding that ``mineralization_rate`` is "behaviorally
   inert".
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


def test_litter_pool_cn_is_the_shed_ratio_times_the_decomposer_rate_RATIO() -> None:
    """⚠ THE RESIDUAL, AND ITS CAUSE — recorded as a test so it cannot quietly rot.

    The litter POOL's C:N is not the shed material's: nitrogen mineralizes OUT of litter
    (``mineralization_rate`` 0.03/day) faster than carbon decomposes out of it
    (``decomposition_rate`` 0.011/day), so the pool runs N-poor by that ratio::

        pool C:N  ~  (shed C:N) x (k_min / k_decomp) x 1.894

    where 1.894 is a measured geometry factor also recorded by the plan doc's finding 3.
    Measured here at ~465 for the sealed chamber against wheat straw's ~80.

    **This retires a recorded finding.** The decomposer calibration declined to move
    ``mineralization_rate`` and gave three reasons; two are now false. It is no longer
    "behaviorally inert" — it sets this ratio — and N and C are no longer "uncoupled",
    which was the other half of the argument that litter C:N was not a physical
    quantity. Only the pool-identity objection survives (S&S measured soil N0; ours is
    fresh residue N). Recalibrating it is scope B and a separate user decision, so the
    value is UNMOVED and the consequence is pinned instead.
    """
    nitro = load_nitrogen_params()
    mineral = load_mineralization_params()
    decomp = load_decomposition_params()
    n_residual_kg_kg = nitro.n_residual_per_mol_c / nitro.dm_kg_per_mol_c
    shed_cn = _CARBON_FRACTION / n_residual_kg_kg
    predicted = (
        shed_cn * (mineral.mineralization_rate / decomp.decomposition_rate) * 1.894
    )

    scenario = sc.SEALED_CHAMBER_SCENARIO
    state, registry = build_season(scenario)
    weather = _weather(3)
    states, rationed, _ = run_season(
        EulerIntegrator(registry),
        state,
        weather_resolver(weather, scenario),
        1.0,
        len(weather),
    )
    assert rationed == 0
    final = states[-1]
    litter_n = final.stocks[LITTER_N].amount
    assert litter_n > 0.0
    pool_cn = final.stocks[LITTER_CARBON].amount * _M_C / litter_n

    assert math.isclose(pool_cn, predicted, rel_tol=0.02), (pool_cn, predicted)
    # The honest framing: 4 orders better than the frozen form's 0.004, and still ~5x
    # straw.
    assert 400.0 < pool_cn < 550.0, pool_cn
    assert pool_cn / 80.0 < 10.0, "pool C:N drifted more than an order off real residue"


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
        assert math.isclose(total_n(s), start, rel_tol=0.0, abs_tol=1e-12), s.n
