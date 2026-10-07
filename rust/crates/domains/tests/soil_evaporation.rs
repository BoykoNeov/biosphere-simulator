//! **Bare-soil evaporation, with the leaf-area canopy resistance and watering in events** — LAB-ONLY.
//!
//! Plan: `docs/plans/post-roadmap-soil-evaporation.md` (§4b the design, §6 the predictions, §7 the
//! watering). Asserts only the instrument — the frozen path through the three new switches is the
//! frozen run, the floor meets its daily total, the soil and crop shares stay inside the net
//! radiation, every re-sow resets the new values — and prints the measurement (`--ignored`).

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::perturbations::window_override;
use domains::biosphere::science::{
    self, SoilEvaporationForm as Soil, SurfaceResistanceForm as Rs, WateringForm as Water,
};
use domains::biosphere::stocks::{
    IRRIGATION_VAR, LEAF_C, RN_VAR, ROOTED_DEPTH, ROOT_C, SOIL_DRY_DAYS, SOIL_EVAP_TODAY,
    SOIL_POTENTIAL_TODAY, SOIL_SHADE_LAI, SOIL_WATER, STEM_C, STORAGE_C, TEMP_VAR, THERMAL_TIME,
    TOP_SOIL_WATER, WATER_VAPOR,
};
use domains::biosphere::system::{
    annual_reset, annual_reset_with, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, DEFAULT_SCENARIO,
};
use domains::biosphere::{
    season_setup_composed, season_setup_with, season_steps, steps_for_years, BIO_DT,
    STEPS_PER_DAY,
};
use domains::lab::biosphere_with_soil_evaporation;
use simcore::environment::{Environment, SourceResolver};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

fn params(rs: Rs, soil: Soil, water: Water) -> BiosphereParams {
    biosphere_with_soil_evaporation(&[], rs, soil, water).expect("frozen params load")
}

/// One run's states and, per step, the water leaving the root zone through `Transpiration`.
struct Run {
    states: Vec<State>,
    out: Vec<f64>,
}

fn run(
    s: &SeasonScenario,
    p: &BiosphereParams,
    years: usize,
    perennial: bool,
    cut: Option<(usize, usize)>,
) -> Run {
    let build = |s: &SeasonScenario, p: &BiosphereParams| {
        domains::biosphere::system::build_season_with(s, p)
    };
    let (state0, integ, res) = season_setup_composed(s, years, p, &build).expect("setup");
    let res = match cut {
        None => res,
        Some((a, b)) => {
            let (mut f, sh) = res.into_parts();
            let base = f.remove(IRRIGATION_VAR).expect("watered");
            let w = |d: usize| (d * STEPS_PER_DAY) as u64;
            f.insert(IRRIGATION_VAR.to_string(), window_override(base, w(a), w(b), 0.0));
            SourceResolver::new(f, sh).expect("rewired")
        }
    };
    step(state0, &integ, &res, s, p, years, perennial)
}

fn step(
    state0: State,
    integ: &EulerIntegrator,
    res: &SourceResolver,
    s: &SeasonScenario,
    p: &BiosphereParams,
    years: usize,
    perennial: bool,
) -> Run {
    let mut state = state0;
    let mut r = Run { states: vec![state.clone()], out: vec![] };
    let year = season_steps();
    for n in 0..steps_for_years(years) {
        if perennial && n > 0 && n % year == 0 {
            state = annual_reset_with(&state, s, p).expect("re-sow");
        }
        let env = res.bind(&state, BIO_DT);
        let mut out = 0.0;
        for f in integ.registry().flows() {
            if f.type_name() == "Transpiration" {
                for l in f.evaluate(&state, &env, BIO_DT).expect("flow").legs {
                    if l.stock == SOIL_WATER {
                        out -= l.amount;
                    }
                }
            }
        }
        let rep = integ.step_report(&state, res, BIO_DT).expect("step");
        assert_eq!(rep.rationed, 0, "the backstop must stay out of a lab measurement");
        assert!(rep.events.is_empty(), "unexpected events: {:?}", rep.events);
        r.out.push(out);
        state = rep.state;
        r.states.push(state.clone());
    }
    r
}

fn ftsw(st: &State, s: &SeasonScenario) -> f64 {
    let cap = science::transpirable_capacity(
        st.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0),
        s.soil_extractable_water,
        s.ground_area,
    );
    science::fraction_transpirable(st.stocks[SOIL_WATER].amount, cap)
}

fn lai(st: &State, s: &SeasonScenario) -> f64 {
    science::leaf_area_index(st.stocks[LEAF_C].amount, params::biosphere().canopy.sla_per_mol_c, s.ground_area)
}

/// The soil's evaporation over each whole day (kg): the day accumulator as the NEXT day starts.
fn soil_by_day(r: &Run) -> Vec<f64> {
    (1..r.states.len() / STEPS_PER_DAY)
        .map(|d| r.states[d * STEPS_PER_DAY].aux[SOIL_EVAP_TODAY])
        .collect()
}

const ON: Soil = Soil::TwoStage { floor: true };
const OFF_FLOOR: Soil = Soil::TwoStage { floor: false };

/// S1 — with the soil off, the constant resistance and continuous watering, the new path is the
/// frozen run to the bit.
#[test]
fn the_frozen_forms_through_the_switches_are_the_frozen_run() {
    for s in [DEFAULT_SCENARIO, sealed_chamber_scenario()] {
        let frozen = run(&s, &params::biosphere(), 1, false, None);
        let switched = run(&s, &params(Rs::Constant, Soil::Off, Water::Continuous), 1, false, None);
        let bits = |st: &State| -> Vec<u64> {
            st.stocks
                .values()
                .map(|k| k.amount.to_bits())
                .chain(st.aux.values().map(|v| v.to_bits()))
                .collect()
        };
        assert!(frozen.states.iter().zip(&switched.states).all(|(a, b)| bits(a) == bits(b)));
    }
}

/// S2 — the floor is a daily total: on every day whose bare-soil potential exceeded 1.5 mm and
/// whose top layer could pay it, the soil evaporated at least 1.5 mm. And the Stage II factor is
/// constant within a day.
#[test]
fn the_floor_meets_its_daily_total_and_stage_two_is_a_daily_factor() {
    let s = DEFAULT_SCENARIO;
    let r = run(&s, &params(Rs::SzeiczLong, ON, Water::Fao56Trigger), 1, false, None);
    let by_day = soil_by_day(&r);
    let floor = science::SOIL_EVAPORATION_FLOOR_MM_DAY * s.ground_area;
    let (mut checked, mut short) = (0usize, 0usize);
    for (d, evap) in by_day.iter().enumerate() {
        let last = &r.states[(d + 1) * STEPS_PER_DAY - 1];
        let potential = r.states[(d + 1) * STEPS_PER_DAY].aux[SOIL_POTENTIAL_TODAY];
        let room = last.aux[TOP_SOIL_WATER] > floor + science::TOP_LAYER_WET_MM * s.ground_area;
        if potential > floor && room && ftsw(last, &s) > science::STAGE_ONE_FTSW {
            checked += 1;
            if *evap < floor - 1e-9 {
                short += 1;
            }
        }
    }
    println!("S2: {checked} floor-eligible days; {short} below the floor");
    assert!(checked > 0, "no day exercised the floor — the check is inert");
    assert_eq!(short, 0);
    for d in [0.0, 1.0, 7.0] {
        let f = science::stage_two_factor(d);
        for k in 1..STEPS_PER_DAY {
            assert_eq!(science::stage_two_factor(d + k as f64 * BIO_DT), f);
        }
        assert!((f - ((d + 1.0_f64).sqrt() - d.sqrt())).abs() < 1e-15);
    }
}

/// S3 — energy: on every step the soil's potential and the crop's radiation-driven potential,
/// as latent heat, together stay inside the net radiation (the floor is the book's own excess,
/// left out).
#[test]
fn soil_and_crop_together_stay_inside_the_net_radiation() {
    let s = DEFAULT_SCENARIO;
    let p = params(Rs::SzeiczLong, OFF_FLOOR, Water::Fao56Trigger);
    let (_, _, res) = season_setup_with(&s, 1, &p).expect("setup");
    let r = run(&s, &p, 1, false, None);
    let to_w = |kg_m2_day: f64| kg_m2_day * 2.45e6 / 86_400.0;
    let mut worst: f64 = 0.0;
    for st in r.states.iter().step_by(7) {
        let env = res.bind(st, BIO_DT);
        let rn = env.get(RN_VAR).unwrap();
        if rn <= 0.0 {
            continue;
        }
        let t = env.get(TEMP_VAR).unwrap();
        let soil = science::soil_evaporation_potential(rn, 0.23, t, st.aux[SOIL_SHADE_LAI]);
        let crop = science::penman_monteith_transpiration(
            rn * science::crop_radiation_share(lai(st, &s)),
            0.0,
            t,
            p.transp.aerodynamic_resistance,
            science::canopy_surface_resistance(Rs::SzeiczLong, 70.0, lai(st, &s)),
        );
        worst = worst.max((to_w(soil) + to_w(crop)) / rn);
    }
    println!("S3: worst (soil + crop radiation term) / net radiation = {worst:.4}");
    assert!(worst <= 1.0 + 1e-9, "energy counted twice: {worst}");
}

/// S10 — every re-sow resets the new values, and the plain reset refuses a state carrying them.
#[test]
fn a_re_sow_resets_the_soil_values_and_the_plain_reset_refuses_them() {
    let s = perennial_chamber_scenario();
    let p = params(Rs::SzeiczLong, ON, Water::Fao56Trigger);
    let r = run(&s, &p, 1, false, None);
    let end = r.states.last().unwrap();
    assert!(annual_reset(end, &s).is_err(), "the plain reset must refuse the lab state");
    let sown = annual_reset_with(end, &s, &p).expect("re-sow");
    let seedling = science::leaf_area_index(s.leaf_c0, p.canopy.sla_per_mol_c, s.ground_area);
    assert_eq!(sown.aux[SOIL_DRY_DAYS], 0.0);
    assert_eq!(sown.aux[SOIL_EVAP_TODAY], 0.0);
    assert_eq!(sown.aux[SOIL_POTENTIAL_TODAY], 0.0);
    assert_eq!(sown.aux[SOIL_SHADE_LAI], seedling);
    let share = (science::TOP_LAYER_DEPTH_M / sown.aux[ROOTED_DEPTH]).min(1.0);
    assert_eq!(sown.aux[TOP_SOIL_WATER], sown.stocks[SOIL_WATER].amount * share);
}

/// The measurement (§6), printed, never asserted:
/// `cargo test --release -p domains --test soil_evaporation -- --ignored --nocapture`.
#[test]
#[ignore = "a printed lab measurement; run with --ignored --release"]
fn measurement() {
    let configs: [(&str, Rs, Soil, Water); 5] = [
        ("frozen", Rs::Constant, Soil::Off, Water::Continuous),
        ("S-L alone", Rs::SzeiczLong, Soil::Off, Water::Continuous),
        ("S-L+soil, daily", Rs::SzeiczLong, OFF_FLOOR, Water::Continuous),
        ("S-L+soil, events", Rs::SzeiczLong, OFF_FLOOR, Water::Fao56Trigger),
        ("S-L+soil+floor, ev", Rs::SzeiczLong, ON, Water::Fao56Trigger),
    ];
    let named: [(&str, SeasonScenario, usize, bool); 4] = [
        ("default", DEFAULT_SCENARIO, 1, false),
        ("sealed chamber", sealed_chamber_scenario(), 3, false),
        ("perennial chamber", perennial_chamber_scenario(), 5, true),
        ("consumer chamber", consumer_chamber_scenario(), 5, true),
    ];
    for (name, s, years, perennial) in &named {
        let frozen = run(s, &params::biosphere(), *years, *perennial, None);
        println!("\n{name} ({years} y): water out of the root zone (kg/yr) | of it soil | at LAI<0.5 | Stage I share | events | lowest FTSW | carbon == frozen | dead-crop vapour / frozen");
        for (label, rs, soil, water) in configs {
            let p = params(rs, soil, water);
            let r = run(s, &p, *years, *perennial, None);
            let total: f64 = r.out.iter().sum::<f64>() / *years as f64;
            let soil_total: f64 = if soil == Soil::Off { 0.0 } else { soil_by_day(&r).iter().sum::<f64>() / *years as f64 };
            let winter: f64 = r.out.iter().zip(&r.states).filter(|(_, st)| lai(st, s) < 0.5).map(|(o, _)| o).sum::<f64>() / *years as f64;
            let stage1 = if soil == Soil::Off {
                f64::NAN
            } else {
                r.states.iter().filter(|st| st.aux[SOIL_DRY_DAYS] == 0.0).count() as f64 / r.states.len() as f64
            };
            let events = r.states.windows(2).filter(|w| w[1].stocks[SOIL_WATER].amount - w[0].stocks[SOIL_WATER].amount > 5.0 * s.ground_area).count();
            let low = r.states.iter().map(|st| ftsw(st, s)).fold(f64::MAX, f64::min);
            let carbon = |st: &State| [LEAF_C, STEM_C, ROOT_C, STORAGE_C].map(|k| st.stocks[k].amount.to_bits());
            let same = carbon(r.states.last().unwrap()) == carbon(frozen.states.last().unwrap());
            let dead: Vec<usize> = (0..r.states.len()).filter(|&i| lai(&r.states[i], s) < 0.1).collect();
            let vap = if s.sealed && !dead.is_empty() {
                let m = |x: &Run| dead.iter().map(|&i| x.states[i].stocks[WATER_VAPOR].amount).sum::<f64>();
                m(&r) / m(&frozen)
            } else {
                f64::NAN
            };
            println!("  {label:<20} {total:>9.2} | {soil_total:>8.2} | {winter:>8.2} | {stage1:>6.3} | {events:>5} | {low:.4} | {same} | {vap:.3}");
        }
    }
    println!("\ndrought window (watering cut days 220-260), first day below FTSW 0.30:");
    for (label, rs, soil, water) in configs {
        let r = run(&DEFAULT_SCENARIO, &params(rs, soil, water), 1, false, Some((220, 260)));
        let first = r.states.iter().position(|st| ftsw(st, &DEFAULT_SCENARIO) < DEFAULT_SCENARIO.wssg).map(|n| n / STEPS_PER_DAY);
        println!("  {label:<20} {first:?}");
    }
    let _ = THERMAL_TIME;
}
