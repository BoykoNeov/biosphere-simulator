//! The LAB leaf-area form (`LeafAreaForm::NodeEnvelope`) — what keeps it lab-only, and what
//! makes its re-measurement trustworthy.
//!
//! Plan: `docs/plans/post-roadmap-leaf-rust-remeasure.md`. The mechanism is the parked leaf
//! mechanism (`docs/log/leaf-expansion.md`), re-implemented from the deleted Python branch so it
//! can be re-measured; it is **not** adopted.
//!
//! ## Why this form needs gates its two siblings did not
//!
//! `KineticsForm` and `O2Form` change arithmetic inside existing flows. This one changes the
//! build's **shape**: an aux process type (`LeafAreaExpansion`) and an aux key
//! (`leaf_area_index`) exist only under it. So two failures are new:
//!
//! * the type or the key reaching a canonical build. The committed manifest's `aux_set` byte
//!   comparison catches the type — *until someone regenerates the manifest*, which is the hole
//!   `lab_only_mechanisms.rs` names for lab flow types. That scan does not see this type (it
//!   lives in the spine, like the forms' branches do), so the manifests are read here as text;
//! * a perennial run of the form re-sown by the frozen `annual_reset`, which cannot reset the
//!   stored area. On the Python branch that rationed 85 times. It is an error now, pinned below.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::readouts::{leaf_thickness_ratio, peak_lai, step_draws, trajectory};
use domains::biosphere::science::{self, LeafAreaForm};
use domains::biosphere::stocks::{CARBON_POOL, LEAF_AREA_INDEX, LEAF_C};
use domains::biosphere::system::{
    annual_reset, annual_reset_with, build_season, build_season_with, consumer_chamber_scenario,
    perennial_chamber_scenario, sealed_chamber_scenario, SeasonScenario, DEFAULT_SCENARIO,
};
use domains::lab::biosphere_with_leaf_form;

const BIOSPHERE_MANIFEST: &str = include_str!("../../../../docs/biosphere-reference.manifest.json");
const STATION_MANIFEST: &str = include_str!("../../../../docs/station-reference.manifest.json");

fn canonical() -> [SeasonScenario; 4] {
    [
        DEFAULT_SCENARIO,
        sealed_chamber_scenario(),
        perennial_chamber_scenario(),
        consumer_chamber_scenario(),
    ]
}

fn lab() -> BiosphereParams {
    biosphere_with_leaf_form(&[], LeafAreaForm::NodeEnvelope).expect("frozen params load")
}

fn derived_seedling_lai(s: &SeasonScenario) -> f64 {
    science::leaf_area_index(s.leaf_c0, params::biosphere().canopy.sla_per_mol_c, s.ground_area)
}

/// Both loaders set the frozen form. The form is not a param-file field, so this is the only
/// place a flipped default would show before a run did.
#[test]
fn the_loaders_set_the_derived_form() {
    assert_eq!(params::biosphere().canopy.leaf_form, LeafAreaForm::Derived);
    assert_eq!(params::potato().canopy.leaf_form, LeafAreaForm::Derived);
}

/// No canonical build carries the key or the type — checked on the BUILD, not inferred from the
/// loader, because the three wiring sites share one predicate and this is where a fourth,
/// unshared one would show.
#[test]
fn no_canonical_build_stores_leaf_area() {
    for s in canonical() {
        let (state, registry) = build_season(&s).expect("build");
        assert!(!state.aux.contains_key(LEAF_AREA_INDEX), "aux key leaked into a canonical build");
        let types: Vec<&str> = registry.aux_processes().iter().map(|a| a.type_name()).collect();
        assert!(!types.contains(&"LeafAreaExpansion"), "{types:?}");
    }
}

/// The committed manifests do not name the lab type — the regenerate-and-go-green hole.
#[test]
fn the_committed_manifests_do_not_name_the_lab_type() {
    for (name, text) in [("biosphere", BIOSPHERE_MANIFEST), ("station", STATION_MANIFEST)] {
        assert!(!text.contains("LeafAreaExpansion"), "{name} manifest names the lab leaf type");
    }
    // Anti-vacuity: the manifests DO name the aux types that are frozen, so a search for a
    // type name in them is a search that can succeed.
    assert!(BIOSPHERE_MANIFEST.contains("RootDepthExtension"));
}

/// Under the form the build DOES carry both, and step 0 is the frozen seedling's area exactly
/// — the whole difference between the forms is in how the canopy evolves.
#[test]
fn the_form_seeds_the_seedlings_derived_area_and_wires_the_process() {
    let p = lab();
    for s in canonical() {
        let (state, registry) = build_season_with(&s, &p).expect("build");
        assert_eq!(
            state.aux[LEAF_AREA_INDEX].to_bits(),
            derived_seedling_lai(&s).to_bits(),
            "seed"
        );
        let types: Vec<&str> = registry.aux_processes().iter().map(|a| a.type_name()).collect();
        assert!(types.contains(&"LeafAreaExpansion"), "{types:?}");
    }
}

/// The frozen re-sow refuses a stored leaf area rather than silently skip resetting it.
#[test]
fn the_frozen_resow_refuses_a_stored_leaf_area() {
    let s = perennial_chamber_scenario();
    let (mut state, _) = build_season_with(&s, &lab()).expect("build");
    // A crop worth re-sowing: enough grain for the seed bank.
    let grown = state.stocks["biosphere.storage_c"].with_amount(1.0).expect("amount");
    state.stocks.insert("biosphere.storage_c".to_string(), grown);
    state.aux.insert(LEAF_AREA_INDEX.to_string(), 4.0);
    let err = annual_reset(&state, &s).expect_err("must refuse");
    assert!(err.to_string().contains("annual_reset_with"), "{err}");

    let resown = annual_reset_with(&state, &s, &lab()).expect("re-sow");
    assert_eq!(resown.aux[LEAF_AREA_INDEX].to_bits(), derived_seedling_lai(&s).to_bits());
    assert_eq!(resown.stocks[LEAF_C].amount, s.leaf_c0);
}

/// On a state that stores no leaf area the two re-sows are the same function.
#[test]
fn the_params_aware_resow_is_the_frozen_one_on_a_frozen_state() {
    let s = perennial_chamber_scenario();
    let (mut state, _) = build_season(&s).expect("build");
    let grown = state.stocks["biosphere.storage_c"].with_amount(1.0).expect("amount");
    state.stocks.insert("biosphere.storage_c".to_string(), grown);
    let a = annual_reset(&state, &s).expect("frozen");
    let b = annual_reset_with(&state, &s, &params::biosphere()).expect("with");
    assert_eq!(a.aux, b.aux);
    for (id, stock) in &a.stocks {
        assert_eq!(stock.amount.to_bits(), b.stocks[id].amount.to_bits(), "{id}");
    }
}

/// The node rate, from its closed form: at emergence `MSNN = 1`, so the rate is
/// `PLACON·PLAPOW·DTU/PHYL · PDEN/10⁴ · WSFL`, linear in both the thermal rate and `WSFL`.
#[test]
fn the_node_rate_at_emergence_is_its_closed_form() {
    assert_eq!(science::leaf_main_stem_nodes(0.0), 1.0);
    assert_eq!(science::leaf_main_stem_nodes(science::LEAF_PHYLLOCHRON), 2.0);
    let want = 1.0 * 2.464 * 10.0 / 112.0 * 300.0 / 10_000.0;
    let got = science::leaf_node_area_rate(0.0, 10.0, 1.0);
    assert!((got - want).abs() < 1e-15, "{got} vs {want}");
    assert_eq!(science::leaf_node_area_rate(0.0, 10.0, 0.5), got * 0.5);
    // The power law makes it ACCELERATE with node number — a constant-rate mutation is red.
    assert!(science::leaf_node_area_rate(500.0, 10.0, 1.0) > 2.0 * got);
}

/// The envelope's orientation: the THINNEST leaf (min specific weight) is the CEILING.
#[test]
fn the_envelope_is_oriented_thinnest_leaf_on_top() {
    let (floor, ceiling) = science::leaf_thickness_envelope(2.0, 0.5, 1.0);
    let derived = science::leaf_area_index(2.0, 0.5, 1.0);
    assert_eq!(ceiling, derived / 0.85);
    assert_eq!(floor, derived / 1.50);
    assert!(floor < derived && derived < ceiling);
}

/// The envelope as IMPLEMENTED: each step's stored area lies inside [E]'s band around the leaf
/// carbon the step ENTERED with. That is the whole claim — the SAMPLED ratio (state over the
/// same instant's carbon) leaves the band by one step's leaf change, measured 0.628–0.643 at
/// day 10.75 (floor 0.667, leaf carbon +3.6–6.1 % that step) and 1.185–1.187 late-season
/// (ceiling 1.176). The plan's P6 was written against the sampled ratio and is FALSIFIED as
/// written; this is the version that holds, and it reddens if the clamp reads the wrong instant.
#[test]
fn the_envelope_holds_against_step_entry_leaf_carbon() {
    let s = perennial_chamber_scenario();
    let p = lab();
    let t = trajectory(s, 1, true, &p);
    assert!(leaf_thickness_ratio(&t).is_some());
    let sla = p.canopy.sla_per_mol_c;
    let mut bound = 0;
    for i in 1..t.leaf_area_state.len() {
        let entry = science::leaf_area_index(t.leaf_c[i - 1], sla, s.ground_area);
        let (floor, ceiling) = (entry / 1.50, entry / 0.85);
        let lai = t.leaf_area_state[i];
        let ulp = 8.0 * f64::EPSILON * lai.abs().max(1.0);
        assert!(lai >= floor - ulp && lai <= ceiling + ulp, "step {i}: {lai} outside [{floor}, {ceiling}]");
        if (lai - floor).abs() <= ulp || (lai - ceiling).abs() <= ulp {
            bound += 1;
        }
    }
    // Anti-vacuity: the band BINDS, so this is a test of the projection and not of a canopy
    // that happens to sit mid-band.
    assert!(bound > 0, "the envelope never bound — the test cannot see the clamp");
}

/// The form MOVES the canopy it exists to move (anti-vacuity for every gate above), in the
/// chamber where the recorded case rests — with no rationing there.
#[test]
fn the_form_moves_the_perennial_chamber_without_rationing() {
    let s = perennial_chamber_scenario();
    let frozen = trajectory(s, 1, true, &params::biosphere());
    let lab = trajectory(s, 1, true, &lab());
    assert!(peak_lai(&lab) > peak_lai(&frozen), "the form did not enlarge the chamber canopy");
    assert_eq!(lab.rationed, 0);
    assert!(leaf_thickness_ratio(&frozen).is_none());
}

/// ⚠ **P5's falsification, and the finer step retired it: the lab form no longer rations the
/// sealed jar.**
///
/// At the quarter-day step (measured 2026-09-29) the lab form rationed the jar 5 times in
/// season 1 under Euler (RK4 raised at step 773), drawing 1.15 of the CO₂ pool in one step,
/// and this test pinned that as `lab.rationed > 0`. That was the "jar breaks" reading holding
/// the leaf form back. **At the 1/16-day step (2026-09-30) it does not ration at all**: its
/// tightest step draws 0.3128 of the pool (the frozen form's 0.2009), both on step 3108, in the
/// explicit CO₂ form; under C, 0.2324. So the
/// jar-breaks reading was taken at a step the reference no longer uses; whether the leaf form
/// should be reconsidered is the user's call, not this test's.
///
/// ⚠ Still the **control for the jar's step-draw pin** (`science_gates::margins::
/// the_jars_tightest_co2_step_is_pinned_by_its_headroom`): `step_draws` must read a CO₂ pool
/// past 1 on a run that rations, or that pin could be a probe that never sees a squeeze. With
/// the lab jar no longer rationing, the control is the frozen jar with its room shrunk to a
/// tenth (air and both gases scaled): measured 205 firings and a tightest draw of 1.488.
#[test]
fn the_form_no_longer_rations_the_sealed_jar_and_the_draw_probe_still_sees_a_squeeze() {
    let s = sealed_chamber_scenario();
    // The finding was measured in the EXPLICIT CO₂ form (the reference until C, 2026-09-30).
    let explicit_of = |mut p: BiosphereParams| {
        p.photo.co2_read = science::Co2Read::StartOfStep;
        p
    };
    let frozen = step_draws(s, 1, &explicit_of(params::biosphere()));
    let lab_explicit = step_draws(s, 1, &explicit_of(lab()));
    assert_eq!((frozen.rationed, lab_explicit.rationed), (0, 0));
    let (f, l) = (frozen.of(CARBON_POOL).ratio, lab_explicit.of(CARBON_POOL).ratio);
    assert!(f < l, "the lab form must still draw harder than the frozen one: {f} vs {l}");
    assert!(
        (0.29..0.34).contains(&l),
        "the lab jar's tightest step moved (measured 0.3128): {:?}",
        lab_explicit.of(CARBON_POOL)
    );
    // Under C the lab form's tightest step reads 0.2324 (step 3155): the draw is solved against
    // the air the step leaves, so the hardest step asks for less.
    let lab_c = step_draws(s, 1, &lab());
    assert_eq!(lab_c.rationed, 0);
    assert!(
        (0.21..0.25).contains(&lab_c.of(CARBON_POOL).ratio),
        "the lab jar's tightest step under C moved (measured 0.2324): {:?}",
        lab_c.of(CARBON_POOL)
    );

    let squeezed = SeasonScenario {
        chamber_air_capacity_mol: s.chamber_air_capacity_mol * 0.1,
        chamber_co2_mol0: s.chamber_co2_mol0 * 0.1,
        chamber_o2_mol0: s.chamber_o2_mol0 * 0.1,
        ..s
    };
    // ⚠ In the EXPLICIT CO₂ form: under the reference's option C (2026-09-30) the crop cannot
    // take more CO₂ than the air holds, so no reference run can show the probe a squeeze. The
    // pair below says C removed it, not that the squeeze stopped being one.
    let explicit =
        domains::lab::biosphere_with_co2_read(&[], science::Co2Read::StartOfStep).expect("params");
    let control = step_draws(squeezed, 1, &explicit);
    assert!(control.rationed > 0, "the tenth-size jar was measured to ration");
    assert!(
        control.of(CARBON_POOL).ratio > 1.0,
        "the squeezed jar rations but step_draws does not see its CO₂ pool overdrawn: {:?}",
        control.of(CARBON_POOL)
    );
    let under_c = step_draws(squeezed, 1, &params::biosphere());
    assert_eq!(under_c.rationed, 0, "the tenth-size jar rations under C");
    assert!(
        under_c.of(CARBON_POOL).ratio <= 1.0,
        "under C the probe reads the pool overdrawn: {:?}",
        under_c.of(CARBON_POOL)
    );
}
