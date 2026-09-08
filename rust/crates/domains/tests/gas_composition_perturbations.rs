//! **Gas-composition perturbations — the biosphere side of the perturbation suite.**
//!
//! Plan: `docs/plans/post-roadmap-perturbation-suite.md`, §3 (the probes) and §4 (the
//! predictions, written before any of these ran).
//!
//! # Why this file exists
//!
//! Every perturbation in this project was a **station cascade** — brownout, radiator
//! failure, single-pool leak, crew spike, lighting failure — and `grep -rln
//! "perturbations::"` returned no user anywhere under `domains/tests/`. Not one of them
//! perturbed the plant science. That mattered on 2026-09-07, when adopting the live-O₂ form
//! inverted a station perturbation's headline: `o2_leak_is_absorbed_by_makeup_effort` had
//! asserted the crop was untouched at a **10715×** contrast, which was true of a model whose
//! crop could not see oxygen and false the moment the pool acquired a second path. The
//! nominal roster moved 0.058 % under that adoption and 9.8 % under a leak on the same tree.
//!
//! So the question these probes ask is not "what can we break". It is: **does a verdict that
//! rests on the word *inert* survive a run that leaves the nominal roster?**
//!
//! # The axis: gas composition, and the atmosphere that is not there
//!
//! The biology reads mole fractions — `co2_mol / chamber_air_mol` and `o2_mol /
//! chamber_air_mol`. ⚠ **`chamber_air_mol` is a constant carried on the flow**
//! ([`crate::biosphere::flows`]'s `chamber_air_mol: Option<f64>`), not a stock. There is no
//! total-gas state, no nitrogen and no pressure anywhere in the tree, so **nothing in this
//! model can lose an atmosphere.** A hull breach has no representation.
//!
//! That gap splits the question in two, and the split is the whole design here:
//!
//! * **[`E1`](the_bookkeeping_correct_vent_holds_composition_and_shrinks_the_room) — scale
//!   all three.** Composition is unchanged by construction; the room is simply smaller. This
//!   is what venting *should* do to the bookkeeping.
//! * **[`E2`](the_partial_pressure_drop_is_what_the_leaf_would_actually_feel) — scale the two
//!   gases, hold the air.** Both fractions fall by `f`. At the leaf that is **arithmetically
//!   identical** to the total pressure falling by `f` at fixed composition, because partial
//!   pressure is fraction × total. So the model *can* express a depressurization's effect on
//!   photosynthesis; what it cannot do is **arrive** there, because nothing conserves total
//!   gas.
//!
//! ⚠ The tree already knew half of this and checked none of it: `consumer_chamber_scenario`
//! was enlarged 2× with the note *"All three gas quantities scale by the same factor so Ci0
//! (250) and x_O2 (0.21) both stay invariant."* That is E1, applied by hand as a sizing
//! choice, with its invariance asserted in prose and tested by nothing.
//!
//! # Diagnostics, no golden
//!
//! The precedent is the perturbation modules' own: these compose onto assembled inputs and no
//! golden pins a perturbed run. **Direction-only, perturbed-vs-baseline** — never a magnitude
//! pin, which is the rule the station file already follows and the reason its O₂ assert
//! survived adoption as a *sign* while its headline number did not.

use domains::biosphere::params;
use domains::biosphere::readouts::{min_ppm, peak_w, trajectory, Trajectory};
use domains::biosphere::science::{
    electron_transport_rate, light_limited_rate, o2_coupled, oxygen_limitation_factor,
    rubisco_limited_rate,
};
use domains::biosphere::system::{sealed_chamber_scenario, SeasonScenario, SEALED_CHAMBER_YEARS};

// ===================================================================================
// E0 — the arithmetic, with no run at all
// ===================================================================================
//
// The two FvCB branches answer a proportional change in BOTH gases differently, and the
// difference is exact rather than measured. This is the fingerprint of the whole question,
// and it is pinned here rather than left to a run whose co-limitation hides which branch
// spoke.

/// The scale factors tried. `1.0` is deliberately absent — it is the identity and would pass
/// every assert below by construction, which is the shape of an inert row.
const FACTORS: [f64; 4] = [0.25, 0.5, 0.75, 1.5];

/// ⚠ **`Aj` is homogeneous of degree ZERO in `(Ci, Γ*)`** — scale both and the numerator
/// `J·(Ci − Γ*)` and the denominator `4Ci + 8Γ*` scale by the same `f`, which cancels.
///
/// So the light-limited branch is **exactly blind to a proportional change in both gases**,
/// and therefore blind to a pressure change at fixed composition. Any crop that is
/// light-limited throughout its season would lose an arbitrary fraction of its atmosphere
/// with no effect on assimilation at all — which is a fact about FvCB, not about this tree.
///
/// ⚠ Not asserted bit-identical. Numerator and denominator each round separately, so the
/// invariance is algebraic and the comparison carries a real (tiny) tolerance.
#[test]
fn the_light_limited_branch_is_exactly_blind_to_a_proportional_gas_change() {
    let p = params::biosphere().photo;
    let j = electron_transport_rate(500.0, &p);
    let ci = 250.0;
    let base = light_limited_rate(ci, j, p.gamma_star);
    assert!(base > 0.0, "the baseline must be a live rate: {base}");

    for f in FACTORS {
        let scaled = light_limited_rate(f * ci, j, f * p.gamma_star);
        let rel = ((scaled - base) / base).abs();
        assert!(
            rel <= 1e-12,
            "Aj moved under a proportional scaling by f={f}: {base} -> {scaled} (rel {rel:e})"
        );
    }
}

/// ⚠ **`Ac` is NOT homogeneous** — the numerator scales by `f`, the denominator does not,
/// because `Kc` is a constant *term* in `Ci + Kc·(1 + O/Ko)`. So the Rubisco-limited branch
/// **loses** as the atmosphere thins, and gains as it thickens.
///
/// Asserted as strict **monotonicity in `f`**, not merely "different": a form that moved the
/// rate in some unordered way would satisfy a `!=` and would not be this claim. The
/// monotonicity is what makes the branch a *direction* the whole-run probes can inherit.
#[test]
fn the_rubisco_branch_is_strictly_monotone_in_a_proportional_gas_change() {
    let p = params::biosphere().photo;
    let ci = 250.0;

    // Ordered ascending so the sequence of rates must be ascending too.
    let mut ordered: Vec<f64> = FACTORS.to_vec();
    ordered.push(1.0);
    ordered.sort_by(|a, b| a.partial_cmp(b).expect("no NaN in FACTORS"));

    let rates: Vec<f64> = ordered
        .iter()
        .map(|&f| {
            // Both gases scale: Ci by f, and O by f *through the adopted live form*, which
            // carries Γ* with it (`o2_coupled` sets Γ* ∝ O). Poking `p.o2` alone would leave
            // Γ* frozen and would be a different — and unphysical — experiment.
            let q = o2_coupled(&p, f * p.o2);
            rubisco_limited_rate(f * ci, &q)
        })
        .collect();

    for w in rates.windows(2) {
        assert!(
            w[1] > w[0],
            "Ac is not strictly increasing in the gas scale: {rates:?} over {ordered:?}"
        );
    }

    // ⚠ The control that makes the test above a claim about `Kc` rather than about scaling in
    // general: with Kc removed from the denominator, Ac becomes homogeneous too and the
    // monotonicity vanishes. This is the falsifier, run rather than asserted in prose.
    let mut no_kc = p;
    no_kc.kc = 0.0;
    let flat: Vec<f64> = ordered
        .iter()
        .map(|&f| {
            let mut q = o2_coupled(&no_kc, f * no_kc.o2);
            q.kc = 0.0;
            rubisco_limited_rate(f * ci, &q)
        })
        .collect();
    let spread = (flat.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
        - flat.iter().cloned().fold(f64::INFINITY, f64::min))
        / flat[0].abs();
    assert!(
        spread <= 1e-12,
        "with Kc = 0 the Rubisco branch should scale-invariant too, but spread is {spread:e}: \
         {flat:?}"
    );
}

// ===================================================================================
// E1 / E2 — the whole-run probes on the jar
// ===================================================================================

/// The jar, driven exactly as its own golden drives it (`run_season`, no re-sow).
fn jar(scenario: SeasonScenario) -> Trajectory {
    trajectory(scenario, SEALED_CHAMBER_YEARS, false, &params::biosphere())
}

/// **E1** — every gas quantity *and* the air scaled together: same composition, smaller room.
fn vented(f: f64) -> SeasonScenario {
    let base = sealed_chamber_scenario();
    SeasonScenario {
        chamber_air_mol: base.chamber_air_mol * f,
        chamber_co2_mol0: base.chamber_co2_mol0 * f,
        chamber_o2_mol0: base.chamber_o2_mol0 * f,
        ..base
    }
}

/// **E2** — the two gases scaled, the air held: the partial-pressure drop as the leaf feels
/// it.
fn depressurized(f: f64) -> SeasonScenario {
    let base = sealed_chamber_scenario();
    SeasonScenario {
        chamber_co2_mol0: base.chamber_co2_mol0 * f,
        chamber_o2_mol0: base.chamber_o2_mol0 * f,
        ..base
    }
}

/// A run that leaves the nominal roster still has to be a *run*: no rationing, no events.
/// ⚠ Without this, a probe whose chamber collapses reports a small number and reads as a
/// small effect — the arbitration backstop firing is not a finding about pressure.
fn well_posed(t: &Trajectory, what: &str) {
    assert_eq!(t.rationed, 0, "{what}: the backstop fired");
    assert_eq!(t.events, 0, "{what}: the run raised events");
    assert!(!t.carbon_pool.is_empty(), "{what}: no CO2 series");
    assert!(!t.o2_pool.is_empty(), "{what}: no O2 series");
}

/// **E1 — the bookkeeping-correct vent.** Composition is identical by construction, so the
/// crop starts on exactly the same air; what changes is that the same plant now draws on a
/// proportionally smaller pool.
///
/// The claim is therefore about **depletion, not about kinetics**: the season-low CO₂ (ppm)
/// falls as the room shrinks. ⚠ And the *first* step must be composition-invariant, which is
/// the half that says this is E1 and not E2 — checked as the initial mole fraction rather
/// than assumed from the construction.
///
/// ⚠⚠ **Bounded to the well-posed regime, and that bound is a MEASUREMENT, not caution.**
/// The plan predicted this direction with no cliff in it. Below `f ≈ 0.8` the arbitration
/// backstop starts firing (2 firings at 0.75, 28 at 0.5) and the direction **reverses** —
/// pinned separately in
/// [`the_gated_observable_reads_HEALTHIER_once_the_backstop_fires`], because a reversal that
/// this test merely avoided would be a fact nothing in the tree recorded.
#[test]
fn the_bookkeeping_correct_vent_holds_composition_and_shrinks_the_room() {
    let base = sealed_chamber_scenario();
    let baseline = jar(base);
    well_posed(&baseline, "E1 baseline");
    let base_low = min_ppm(&baseline);

    for f in [0.8, 0.9] {
        let s = vented(f);

        // The composition half: initial mole fractions are unchanged, exactly.
        let x_co2 = |s: &SeasonScenario| s.chamber_co2_mol0 / s.chamber_air_mol;
        let x_o2 = |s: &SeasonScenario| s.chamber_o2_mol0 / s.chamber_air_mol;
        assert!(
            (x_co2(&s) - x_co2(&base)).abs() <= 1e-15 * x_co2(&base),
            "E1 changed the CO2 fraction at f={f}"
        );
        assert!(
            (x_o2(&s) - x_o2(&base)).abs() <= 1e-15 * x_o2(&base),
            "E1 changed the O2 fraction at f={f}"
        );

        let t = jar(s);
        well_posed(&t, "E1");
        let low = min_ppm(&t);
        assert!(
            low < base_low,
            "a smaller room did not deplete further at f={f}: {base_low} -> {low} ppm"
        );
    }

    // ⚠ The falsifier, run in the same test: enlarging the room must move it the OTHER way.
    // An assert that only ever sees one direction cannot distinguish "shrinking depletes" from
    // "any change depletes".
    let bigger = jar(vented(1.5));
    well_posed(&bigger, "E1 enlarged");
    assert!(
        min_ppm(&bigger) > base_low,
        "a larger room did not deplete less: {base_low} -> {} ppm",
        min_ppm(&bigger)
    );
}

/// ⚠⚠ **THE FINDING THIS BATCH DID NOT PREDICT: past the rationing cliff, the gated
/// observable improves as the chamber gets worse.**
///
/// `season-low chamber CO₂ (ppm)` is the quantity the biosphere contract gates the jar on.
/// Shrink the room and it falls, correctly — until the arbitration backstop starts firing, at
/// which point the throttled withdrawal stops drawing the pool down and the *minimum comes
/// back up*. At `f = 0.5` the jar rations 28 times and reads **above** its own healthy
/// baseline.
///
/// So a chamber broken badly enough to need the backstop reads *healthier* on the gated
/// observable than one merely stressed. Anything ranking chambers by this number — and the
/// compensation-point band does exactly that — is non-monotone across that boundary.
///
/// ⚠ The backstop is not misbehaving; it is doing its job, and the golden runs assert its
/// firing count is 0 precisely so the reference never sits here. What is new is that the
/// observable's **direction** is not safe outside that assumption, which is a property of the
/// metric and not of the run.
#[test]
fn the_gated_observable_reads_healthier_once_the_backstop_fires() {
    let baseline = jar(sealed_chamber_scenario());
    well_posed(&baseline, "cliff baseline");
    let base_low = min_ppm(&baseline);

    let stressed = jar(vented(0.8));
    let broken = jar(vented(0.5));

    // The stressed room is well-posed and reads lower, as E1 says it must.
    assert_eq!(
        stressed.rationed, 0,
        "f=0.8 was expected to stay well-posed"
    );
    assert!(min_ppm(&stressed) < base_low);

    // The broken room rations — and reads HIGHER than the healthy baseline.
    assert!(
        broken.rationed > 0,
        "f=0.5 was expected to ration; the cliff has moved and this finding needs re-measuring"
    );
    assert!(
        min_ppm(&broken) > base_low,
        "the reversal is gone: broken {} vs baseline {base_low} ppm",
        min_ppm(&broken)
    );
    assert!(
        min_ppm(&broken) > min_ppm(&stressed),
        "the broken room does not read better than the merely stressed one"
    );
}

/// The companion to the reversal: **peak biomass stays monotone where season-low CO₂ does
/// not.** Across the same four rooms, including the two that ration, the crop orders
/// correctly every time.
///
/// That makes it the honest observable off the nominal roster, and it is worth having pinned
/// next to the reversal — the pair is what says "the metric misled", rather than "the run
/// went strange".
#[test]
fn peak_biomass_stays_monotone_across_the_cliff_that_reverses_the_gated_one() {
    let w = |f: f64| peak_w(&jar(vented(f)));
    let (half, four_fifths, one, half_again) = (w(0.5), w(0.8), w(1.0), w(1.5));
    assert!(
        half < four_fifths && four_fifths < one && one < half_again,
        "peak W is not monotone in room size: {half} {four_fifths} {one} {half_again}"
    );
}

/// ⚠ **Why E1 breaks where E2 does not, and it is not about how much carbon is left.**
///
/// E1 rations at `f = 0.75`. E2 does **not** ration at `f = 0.25`, with a third as much CO₂
/// in the chamber in absolute moles. The difference is the *feedback*: E2 lowers the mole
/// fraction, so `Ci` falls, so the plant's own demand falls with the supply — it self-limits.
/// E1 holds the fraction and therefore holds the appetite, while removing the buffer that
/// appetite was drawing on.
///
/// This is the same mechanism as the chamber-controller result recorded on the CO₂ setpoint
/// decision — *holding* the chamber at its starting CO₂ measured four times worse than
/// letting it deplete, because the controller removes the self-limiting feedback. Reached
/// here by a different route (shrink the room instead of regulating it), which is why it is
/// pinned rather than cross-referenced.
#[test]
fn the_self_limiting_feedback_is_what_keeps_the_jar_well_posed() {
    let e1 = jar(vented(0.75));
    let e2 = jar(depressurized(0.25));

    let base = sealed_chamber_scenario();
    assert!(
        base.chamber_co2_mol0 * 0.25 < base.chamber_co2_mol0 * 0.75,
        "the comparison is only interesting if E2 holds strictly LESS carbon"
    );

    assert!(
        e1.rationed > 0,
        "E1 at f=0.75 no longer rations; the cliff has moved"
    );
    assert_eq!(
        e2.rationed, 0,
        "E2 at f=0.25 rationed — the self-limiting reading of the contrast is wrong"
    );
}

/// **E2 — the depressurization, as the leaf would actually feel it.**
///
/// Both mole fractions fall by `f` with the air held, which at the leaf is the same thing as
/// the total pressure falling by `f` at fixed composition. E0 says the answer depends
/// entirely on which branch binds: exactly nothing if the season is light-limited throughout,
/// a loss wherever it is Rubisco-limited.
///
/// ⚠ The assert is a **sign on the crop**, and the sign is the load-bearing part: an
/// atmosphere with less CO₂ *and* less O₂ pits the assimilation loss against the
/// photorespiration relief that made an O₂-only leak raise the crop by 9.8 % on the station.
/// The two must not come out the same way, or the CO₂ path is weaker than the O₂ path at
/// these levels — which would contradict `carbon_leak_lowers_biomass_and_scrubber_effort`'s
/// ordering and is a thing to chase, not to record.
#[test]
fn the_partial_pressure_drop_is_what_the_leaf_would_actually_feel() {
    let baseline = jar(sealed_chamber_scenario());
    well_posed(&baseline, "E2 baseline");
    let base_w = peak_w(&baseline);
    assert!(base_w > 0.0, "the jar must grow something: {base_w}");

    let thin = jar(depressurized(0.5));
    well_posed(&thin, "E2 thinned");
    let thin_w = peak_w(&thin);

    let signed = (thin_w - base_w) / base_w;
    assert!(
        signed < 0.0,
        "halving BOTH gases raised the crop ({signed:+.3e}); the CO2 loss must outweigh the \
         photorespiration relief, or the O2 path is stronger than the carbon path and the \
         station's carbon-vs-O2 leak ordering is wrong"
    );

    // The falsifier: a thicker atmosphere must move it the other way. Same reason as E1's.
    let thick = jar(depressurized(1.5));
    well_posed(&thick, "E2 thickened");
    let thick_signed = (peak_w(&thick) - base_w) / base_w;
    assert!(
        thick_signed > 0.0,
        "enriching BOTH gases did not raise the crop ({thick_signed:+.3e})"
    );
}

/// **E2 halves the two gases together. This splits the pair — and the split inverts the
/// station's sign.**
///
/// ⚠⚠ **THE FINDING OF THIS BATCH.** On the station, an O₂ leak *raises* the crop by 9.8 %:
/// less oxygen is less photorespiration. In the **unregulated** chambers, halving O₂ alone
/// **lowers** it — 1.45 % in the jar, 0.86 % in the big perennial chamber. Same code, same
/// params, opposite sign.
///
/// Oxygen enters this model **twice**, and which entry dominates is a property of the
/// scenario rather than of the science:
///
/// * at the **leaf**, through `Γ*` and the Rubisco denominator — the photorespiration relief;
/// * at the **soil**, through `oxygen_limitation_factor`, a Michaelis–Menten `x/(K + x)` with
///   `K = 1e-4 mol/mol`, applied to decomposition and microbial respiration. In a sealed
///   chamber the decomposers **are** the CO₂ supply.
///
/// ⚠ **What distinguishes the station is that its O₂ level is DEFENDED.** `O2Makeup` holds
/// the cabin at 209.8 mmol/mol, so the supply path never bites there and only the leaf path
/// is left to answer. Both chambers let oxygen deplete, and the supply path wins in both. The
/// contrast is **regulated vs unregulated**, not big vs small — measured, after the first
/// reading of this file guessed *"the jar's low charge"* and the big chamber refuted it.
///
/// ⚠ **One route is verified and one is NOT, and they are separated here on purpose.** The
/// jar's is measured — it goes anoxic, pinned below. The big chamber's soil factor barely
/// moves (0.999521 → 0.999040 at its trajectory minimum) yet its crop loses 0.86 %, which is
/// 18× the throttle; **that route is open** and is not claimed here. Compounding over its
/// perennial years is the obvious candidate and has not been run.
///
/// ⚠ And in the jar the two cuts do **not** fight, they compound: −4.95 % (CO₂ alone) and
/// −1.45 % (O₂ alone) against −6.70 % together, near-additive. The intuition that a
/// depressurization trades assimilation against photorespiration is a *station* intuition.
#[test]
fn oxygen_is_a_carbon_supply_nutrient_where_it_is_not_defended() {
    let base = sealed_chamber_scenario();
    let baseline = jar(base);
    well_posed(&baseline, "O2-split baseline");
    let w0 = peak_w(&baseline);

    let o2_only = |f: f64| SeasonScenario {
        chamber_o2_mol0: base.chamber_o2_mol0 * f,
        ..base
    };
    let co2_only = |f: f64| SeasonScenario {
        chamber_co2_mol0: base.chamber_co2_mol0 * f,
        ..base
    };

    let less_o2 = jar(o2_only(0.5));
    well_posed(&less_o2, "O2 halved");
    let d_o2 = (peak_w(&less_o2) - w0) / w0;

    // THE sign, and the one that inverts the station's. A sign is what a mis-scoped mechanism
    // breaks, and no conservation or arbitration check can see it.
    assert!(
        d_o2 < 0.0,
        "halving the JAR's oxygen raised the crop ({d_o2:+.3e}); that is the STATION's sign, \
         and it belongs to a cabin whose O2 level is defended"
    );

    // The falsifier: more oxygen must move it the other way.
    let more_o2 = jar(o2_only(2.0));
    well_posed(&more_o2, "O2 doubled");
    assert!(
        (peak_w(&more_o2) - w0) / w0 > 0.0,
        "doubling the jar's oxygen did not raise the crop"
    );

    // The mechanism at the arithmetic level: the decomposers are INSIDE the limiting region at
    // the jar's charge, which is what makes this a supply story and not a leaf story.
    let f_at = |mol: f64| oxygen_limitation_factor(mol, base.chamber_air_mol, SOIL_K_O2);
    let full = f_at(base.chamber_o2_mol0);
    let half = f_at(base.chamber_o2_mol0 * 0.5);
    assert!(
        full < 1.0 && half < full,
        "the jar's decomposers are not oxygen-limited at charge: {full} -> {half}"
    );

    // The compounding: the two cuts add rather than cancel. If they fought, the pair would be
    // milder than the CO₂ cut alone.
    let d_co2 = (peak_w(&jar(co2_only(0.5))) - w0) / w0;
    let d_both = (peak_w(&jar(depressurized(0.5))) - w0) / w0;
    assert!(
        d_both < d_co2,
        "the pair ({d_both:+.3e}) is milder than the CO2 cut alone ({d_co2:+.3e}) — the O2 \
         half is relieving, not compounding, and the jar reading is wrong"
    );
}

/// The cited soil O₂ half-saturation (`microbial_respiration.yaml`, 1e-4 mol/mol).
///
/// ⚠ A literal, not a read of the params: the constant lives on each flow as
/// `o2_half_saturation` and there is no loaded struct field to reach from here. It is used
/// only to locate the *regime* — whether the decomposers sit inside the limiting region —
/// never to reproduce a rate, so a drift in the file would change which regime these tests
/// describe and not whether their whole-run asserts hold.
const SOIL_K_O2: f64 = 1.0e-4;

/// ⚠⚠ **Halving the jar's oxygen charge drives it ANOXIC — and the model absorbs that
/// without a single defect.**
///
/// O₂ bottoms at `5.1e-15 mol`, a factor of `3e13` below its charge, against `1.5e-1` on the
/// healthy baseline. The soil factor goes to zero with it: decomposition, the sealed jar's
/// entire CO₂ supply, stops.
///
/// Two halves, and both are worth having pinned:
///
/// * **The fragility.** The jar is *one halving* away from total oxygen exhaustion. Its charge
///   is not a comfortable margin; it is the edge.
/// * **The robustness, which is the better news.** Across 3661 steps the stock is **never
///   negative**, the arbitration backstop **never fires**, and no event is raised — first-order
///   donor control means the draw vanishes with the pool, so the model reaches zero smoothly
///   instead of overshooting into a clamp. That is the positivity contract behaving exactly as
///   its docstring claims, at the one place in this batch that actually tested it.
#[test]
fn halving_the_jars_oxygen_takes_it_to_anoxia_without_going_negative() {
    let base = sealed_chamber_scenario();
    let t = jar(SeasonScenario {
        chamber_o2_mol0: base.chamber_o2_mol0 * 0.5,
        ..base
    });
    well_posed(&t, "anoxia probe");

    let min_o2 = t.o2_pool.iter().cloned().fold(f64::INFINITY, f64::min);
    let base_min = jar(base)
        .o2_pool
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    assert!(
        t.o2_pool.iter().all(|v| *v >= 0.0),
        "the oxygen stock went NEGATIVE; structural positivity is broken"
    );
    assert!(
        min_o2 < base_min / 1.0e6,
        "the halved jar did not go anoxic: {min_o2} against a baseline minimum of {base_min}"
    );
    assert!(
        oxygen_limitation_factor(min_o2, base.chamber_air_mol, SOIL_K_O2) < 1.0e-6,
        "the soil oxygen factor did not collapse with the pool"
    );
    // The baseline is already deep in the limiting region — the jar is not comfortable, it is
    // one halving from the edge. This is the line that makes the pin above a margin statement.
    assert!(
        oxygen_limitation_factor(base_min, base.chamber_air_mol, SOIL_K_O2) < 0.7,
        "the healthy jar is not oxygen-limited at its trough, so 'one halving from the edge' \
         is the wrong reading"
    );
}

/// ⚠ **The two probes are different experiments, and this is what says so.**
///
/// E1 and E2 apply the same factor to the same two gas stocks and differ only in whether the
/// air goes with them. If the biology were reading absolute moles rather than fractions they
/// would be the *same* run, every assert above would still pass, and the distinction this
/// whole file is built on would be fictional. Measured as a contrast rather than argued from
/// the source.
#[test]
fn holding_the_air_is_what_makes_e2_a_different_experiment_from_e1() {
    let base_w = peak_w(&jar(sealed_chamber_scenario()));
    let e1 = peak_w(&jar(vented(0.5)));
    let e2 = peak_w(&jar(depressurized(0.5)));

    let d1 = (e1 - base_w) / base_w;
    let d2 = (e2 - base_w) / base_w;
    assert!(
        (d1 - d2).abs() > 1e-9,
        "E1 and E2 are indistinguishable ({d1:+.3e} vs {d2:+.3e}) — the run is not reading \
         the air constant at all"
    );
}

/// Determinism is the no-golden insurance, and a new substrate owes its own re-run — the
/// station file's `matter_perturbation_is_deterministic` covers the station's matter cascades
/// and says nothing about a re-charged jar.
#[test]
fn the_gas_perturbations_are_deterministic() {
    for s in [vented(0.5), depressurized(0.5)] {
        let a = jar(s);
        let b = jar(s);
        assert_eq!(
            a.carbon_pool.len(),
            b.carbon_pool.len(),
            "two runs of one scenario differ in length"
        );
        for (i, (x, y)) in a.carbon_pool.iter().zip(b.carbon_pool.iter()).enumerate() {
            assert_eq!(
                x.to_bits(),
                y.to_bits(),
                "CO2 differs at step {i}: {x} vs {y}"
            );
        }
        for (i, (x, y)) in a.o2_pool.iter().zip(b.o2_pool.iter()).enumerate() {
            assert_eq!(
                x.to_bits(),
                y.to_bits(),
                "O2 differs at step {i}: {x} vs {y}"
            );
        }
    }
}
