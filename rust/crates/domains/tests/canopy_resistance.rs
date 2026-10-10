//! **A canopy surface resistance that reads leaf area** — the measurement that priced it.
//!
//! Plan: `docs/plans/post-roadmap-canopy-resistance.md` (§3 the forms, §5 the predictions). Two
//! cited forms, switched in through [`domains::lab::biosphere_with_rs_form`]. ⚠ **Szeicz–Long is
//! the loader's form since the water-forms adoption (2026-10-10,
//! `docs/plans/post-roadmap-soil-evaporation.md` §16)**, with the soil's evaporation on; the
//! constant 70 s/m and FAO's full-cover form are the lab's. This file asserts only its instrument —
//! the loader's form through the switch is the reference run, and each leaf-area form reaches
//! `Transpiration` exactly — and PRINTS the measurement (`--nocapture`), whose rows now carry the
//! reference's soil evaporation and event watering. No ratio is pinned.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::perturbations::window_override;
use domains::biosphere::science::{self, SurfaceResistanceForm};
use domains::biosphere::stocks::{
    IRRIGATION_VAR, LEAF_C, RN_VAR, ROOTED_DEPTH, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C, TEMP_VAR,
    VPD_VAR,
};
use domains::biosphere::system::{
    build_season_with, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, DEFAULT_SCENARIO,
};
use domains::biosphere::{season_setup_composed, steps_for_years, BIO_DT, STEPS_PER_DAY};
use domains::lab::biosphere_with_rs_form;
use domains::lab::mechanism::build_season_without;
use simcore::environment::{Environment, SourceResolver};
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;

const FORMS: [SurfaceResistanceForm; 3] = [
    SurfaceResistanceForm::Constant,
    SurfaceResistanceForm::FaoFullCover,
    SurfaceResistanceForm::SzeiczLong,
];

fn params_for(form: SurfaceResistanceForm) -> BiosphereParams {
    biosphere_with_rs_form(&[], form).expect("frozen params load")
}

/// One season's books: every state, and per step the water leaving the root zone by
/// transpiration and by drainage (kg).
struct Run {
    states: Vec<State>,
    transpired: Vec<f64>,
    drained: Vec<f64>,
    /// Water the irrigation put INTO the root zone per step (kg).
    irrigated: Vec<f64>,
}

fn run(
    s: &SeasonScenario,
    p: &BiosphereParams,
    drop: &[&str],
    drought: Option<(usize, usize)>,
) -> Run {
    let build = |s: &SeasonScenario, p: &BiosphereParams| build_season_without(s, p, drop);
    let (state0, integrator, resolver) = season_setup_composed(s, 1, p, &build).expect("setup");
    let resolver = match drought {
        None => resolver,
        Some((from, to)) => {
            let (mut forcings, shared) = resolver.into_parts();
            let base = forcings.remove(IRRIGATION_VAR).expect("watered");
            let (a, b) = ((from * STEPS_PER_DAY) as u64, (to * STEPS_PER_DAY) as u64);
            forcings.insert(IRRIGATION_VAR.to_string(), window_override(base, a, b, 0.0));
            SourceResolver::new(forcings, shared).expect("rewired")
        }
    };
    step_through(state0, &integrator, &resolver)
}

fn step_through(state0: State, integrator: &EulerIntegrator, resolver: &SourceResolver) -> Run {
    let mut state = state0;
    let mut out = Run {
        states: vec![state.clone()],
        transpired: vec![],
        drained: vec![],
        irrigated: vec![],
    };
    for _ in 0..steps_for_years(1) {
        let env = resolver.bind(&state, BIO_DT);
        let (mut t, mut d, mut i) = (0.0, 0.0, 0.0);
        for flow in integrator.registry().flows() {
            let kind = flow.type_name();
            if kind != "Transpiration" && kind != "Drainage" && kind != "Irrigation" {
                continue;
            }
            for leg in flow.evaluate(&state, &env, BIO_DT).expect("flow").legs {
                if leg.stock == SOIL_WATER {
                    match kind {
                        "Transpiration" => t -= leg.amount,
                        "Drainage" => d -= leg.amount,
                        _ => i += leg.amount,
                    }
                }
            }
        }
        let report = integrator.step_report(&state, resolver, BIO_DT).expect("step");
        assert_eq!(report.rationed, 0, "the backstop must stay out of a lab measurement");
        assert!(report.events.is_empty(), "unexpected events: {:?}", report.events);
        out.transpired.push(t);
        out.drained.push(d);
        out.irrigated.push(i);
        state = report.state;
        out.states.push(state.clone());
    }
    out
}

fn ftsw(s: &State, sc: &SeasonScenario) -> f64 {
    let depth = s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0);
    let cap = science::transpirable_capacity(depth, sc.soil_extractable_water, sc.ground_area);
    science::fraction_transpirable(s.stocks[SOIL_WATER].amount, cap)
}

fn lai(s: &State, sc: &SeasonScenario) -> f64 {
    science::leaf_area_index(
        s.stocks[LEAF_C].amount,
        params::biosphere().canopy.sla_per_mol_c,
        sc.ground_area,
    )
}

fn carbon_bits(s: &State) -> Vec<u64> {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|k| s.stocks[*k].amount.to_bits())
        .collect()
}

fn deep_water() -> SeasonScenario {
    SeasonScenario { irrigation_mm_day: 1.0, ..DEFAULT_SCENARIO }
}

/// P1 — the loader's form, routed through the switch, is the reference run to the bit. (Until
/// 2026-10-10 the loader's form was the constant, and this pinned that one; restated at the
/// water-forms adoption, plan §16.)
#[test]
fn the_loaders_form_through_the_switch_is_the_reference_run() {
    assert_eq!(params::biosphere().transp.rs_form, SurfaceResistanceForm::SzeiczLong);
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let frozen = run(&s, &params::biosphere(), &[], None);
        let switched = run(&s, &params_for(SurfaceResistanceForm::SzeiczLong), &[], None);
        let bits = |st: &State| -> Vec<u64> {
            st.stocks.values().map(|k| k.amount.to_bits()).collect()
        };
        assert!(frozen.states.iter().zip(&switched.states).all(|(a, b)| bits(a) == bits(b)));
    }
}

/// P2 — each leaf-area form reaches `Transpiration`: on every state of a reference run, the flow's
/// water leg equals Penman–Monteith at that form's resistance for the state's LAI, times the
/// soil-water factor, to rounding. Open field and a sealed chamber (whose leg splits into air +
/// condensate, but whose root-zone leg is the same flux). ⚠ With the soil's evaporation switched
/// OFF for the flow under test: the by-hand formula has no soil–crop energy split, and the claim is
/// the resistance's reach, not the split's (that one is `soil_evaporation.rs`'s S3). Restated at the
/// water-forms adoption (2026-10-10, plan §16), when the loader turned the soil on.
#[test]
fn each_lab_form_reaches_the_water_flow_exactly() {
    let p0 = params::biosphere();
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let frozen = run(&s, &p0, &[], None);
        for form in [SurfaceResistanceForm::FaoFullCover, SurfaceResistanceForm::SzeiczLong] {
            let mut p = params_for(form);
            p.transp.soil_evap = science::SoilEvaporationForm::Off;
            let (_, registry): (State, Registry) = build_season_with(&s, &p).expect("build");
            let flow = registry
                .flows()
                .iter()
                .find(|f| f.type_name() == "Transpiration")
                .expect("the season transpires");
            let (_, _, resolver) =
                season_setup_composed(&s, 1, &p, &|s, p| build_season_with(s, p)).expect("setup");
            let mut moved = 0usize;
            for st in frozen.states.iter().step_by(37) {
                let env = resolver.bind(st, BIO_DT);
                let leg = flow.evaluate(st, &env, BIO_DT).expect("flow").legs;
                let got = -leg.iter().find(|l| l.stock == SOIL_WATER).map_or(0.0, |l| l.amount);
                let temp = env.get(TEMP_VAR).unwrap();
                let vpd = if s.sealed {
                    science::chamber_vpd_pa(
                        temp,
                        st.stocks[domains::biosphere::stocks::WATER_VAPOR].amount,
                        s.chamber_air_capacity_mol,
                    )
                } else {
                    env.get(VPD_VAR).unwrap()
                };
                let rs = science::canopy_surface_resistance(
                    form,
                    p.transp.surface_resistance,
                    lai(st, &s),
                    p.transp.leaf_stomatal_resistance,
                    p.transp.threshold_lai,
                );
                let f_water = science::soil_water_stress(
                    st.stocks[SOIL_WATER].amount,
                    st.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0),
                    s.soil_extractable_water,
                    s.ground_area,
                    s.wssg,
                );
                let want = science::penman_monteith_transpiration(
                    env.get(RN_VAR).unwrap(),
                    vpd,
                    temp,
                    p.transp.aerodynamic_resistance,
                    rs,
                ) * f_water
                    * s.ground_area
                    * BIO_DT;
                assert!(
                    (got - want).abs() <= 1e-12 * want.abs().max(1e-12),
                    "{form:?} step {}: flow {got}, by hand {want}",
                    st.n
                );
                if (rs - p.transp.surface_resistance).abs() > 1.0 {
                    moved += 1;
                }
            }
            assert!(moved > 0, "{form:?} never changed the resistance — the check is inert");
        }
    }
}

/// The measurement (§5), printed, never asserted. Twenty-four seasons, so it is run on demand:
/// `cargo test --release -p domains --test canopy_resistance -- --ignored --nocapture`.
#[test]
#[ignore = "a printed lab measurement, 24 seasons; run with --ignored --release"]
fn measurement() {
    let named: [(&str, SeasonScenario); 5] = [
        ("default", DEFAULT_SCENARIO),
        ("deep water 1 mm/d", deep_water()),
        ("sealed chamber", sealed_chamber_scenario()),
        ("perennial chamber", perennial_chamber_scenario()),
        ("consumer chamber", consumer_chamber_scenario()),
    ];
    println!("\nseason water (kg): transpired | of it at LAI<0.5 | at LAI>=4 | drained | irrigated | lowest FTSW | carbon == Constant");
    for (name, s) in &named {
        let base = run(s, &params::biosphere(), &[], None);
        for form in FORMS {
            let r = run(s, &params_for(form), &[], None);
            let total: f64 = r.transpired.iter().sum();
            let by = |pred: &dyn Fn(f64) -> bool| -> f64 {
                r.transpired
                    .iter()
                    .zip(&r.states)
                    .filter(|(_, st)| pred(lai(st, s)))
                    .map(|(t, _)| t)
                    .sum()
            };
            let low = by(&|l| l < 0.5);
            let high = by(&|l| l >= 4.0);
            let drained: f64 = r.drained.iter().sum();
            let irrigated: f64 = r.irrigated.iter().sum();
            let min_ftsw = r.states.iter().map(|st| ftsw(st, s)).fold(f64::MAX, f64::min);
            let same = carbon_bits(r.states.last().unwrap()) == carbon_bits(base.states.last().unwrap());
            println!(
                "  {name:<18} {form:<13?} {total:>9.2} | {low:>8.2} | {high:>8.2} | {drained:>8.2} | {irrigated:>8.2} | {min_ftsw:.4} | {same}"
            );
        }
    }

    println!("\ndeep-water rescue (with the below-root store / without):");
    for form in FORMS {
        let p = params_for(form);
        let with = run(&deep_water(), &p, &[], None);
        let without = run(&deep_water(), &p, &["biosphere.root_zone_capture"], None);
        let peak = |r: &Run| r.states.iter().map(|s| s.stocks[LEAF_C].amount).fold(f64::MIN, f64::max);
        let grain = |r: &Run| r.states.last().unwrap().stocks[STORAGE_C].amount;
        println!(
            "  {form:<13?} canopy {:.4}x ({:.3} / {:.3}); grain at season end {:.4}x ({:.3} / {:.3})",
            peak(&with) / peak(&without),
            peak(&with),
            peak(&without),
            grain(&with) / grain(&without),
            grain(&with),
            grain(&without)
        );
    }

    println!("\nthe drought window (watering cut days 220-260), first day below FTSW 0.30 / lowest:");
    for form in FORMS {
        let r = run(&DEFAULT_SCENARIO, &params_for(form), &[], Some((220, 260)));
        let first = r
            .states
            .iter()
            .position(|st| ftsw(st, &DEFAULT_SCENARIO) < DEFAULT_SCENARIO.wssg)
            .map(|n| n / STEPS_PER_DAY);
        let lowest = r.states.iter().map(|st| ftsw(st, &DEFAULT_SCENARIO)).fold(f64::MAX, f64::min);
        println!("  {form:<13?} first stressed day {first:?}; lowest FTSW {lowest:.4}");
    }
}
