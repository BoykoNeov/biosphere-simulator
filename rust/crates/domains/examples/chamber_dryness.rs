//! **Measurement for Step 3b** — a sealed chamber's crop transpiring against the chamber's own
//! air instead of the weather's (`docs/plans/post-roadmap-chamber-dryness.md`).
//!
//! Each sealed biosphere chamber runs twice — the weather's deficit (the reading before
//! 2026-10-01) and the chamber's own — and every step's state is read for: the potential
//! transpiration Penman–Monteith gives, the water-stress factor, the subsoil water, and the
//! chamber's relative humidity at the start of the step.
//!
//! From `rust/`: `cargo run --release -q -p domains --example chamber_dryness`

use domains::biosphere::science::{self, VpdRead};
use domains::biosphere::stocks::{RN_VAR, TEMP_VAR, VPD_VAR};
use domains::biosphere::stocks::{ROOTED_DEPTH, SOIL_WATER, SUBSOIL_WATER, WATER_VAPOR};
use domains::biosphere::system::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial_with, run_season,
    sealed_chamber_scenario, SeasonScenario, CONSUMER_CHAMBER_YEARS, PERENNIAL_CHAMBER_YEARS,
    SEALED_CHAMBER_YEARS,
};
use domains::biosphere::{season_setup_with, steps_for_years, BIO_DT};
use simcore::environment::Environment;
use simcore::state::State;

#[derive(Default)]
struct Tally {
    steps: u64,
    potential_mm: f64,
    actual_kg: f64,
    min_fwater: f64,
    stressed_steps: u64,
    min_subsoil: f64,
    below_target_steps: u64,
    min_rh: f64,
    min_rh_after_day_one: f64,
    extra_kg: f64,
    extra_kg_below_target: f64,
}

fn run(scn: &SeasonScenario, years: usize, perennial: bool, read: VpdRead) -> Tally {
    let p = domains::lab::biosphere_with_vpd_read(&[], read).expect("params");
    let (state, integrator, resolver) = season_setup_with(scn, years, &p).expect("setup");
    let mut t = Tally {
        min_fwater: f64::INFINITY,
        min_subsoil: f64::INFINITY,
        min_rh: f64::INFINITY,
        min_rh_after_day_one: f64::INFINITY,
        ..Tally::default()
    };
    let n_ref = scn.chamber_air_capacity_mol;
    let setpoint = p.water.humidity_setpoint;
    // The observer sees each step's starting state — what that step's flows read.
    let mut observe = |s: &State| {
        let env = resolver.bind(s, BIO_DT);
        let (rn, temp, weather_vpd) = (
            env.get(RN_VAR).unwrap(),
            env.get(TEMP_VAR).unwrap(),
            env.get(VPD_VAR).unwrap(),
        );
        let vapour = s.stocks[WATER_VAPOR].amount;
        let chamber_vpd = science::chamber_vpd_pa(temp, vapour, n_ref);
        let pm = |vpd| {
            science::penman_monteith_transpiration(
                rn,
                vpd,
                temp,
                p.transp.aerodynamic_resistance,
                p.transp.surface_resistance,
            )
        };
        let used = match read {
            VpdRead::Chamber => chamber_vpd,
            VpdRead::Weather => weather_vpd,
        };
        let fw = science::soil_water_stress(
            s.stocks[SOIL_WATER].amount,
            s.aux.get(ROOTED_DEPTH).copied().unwrap_or(0.0),
            scn.soil_extractable_water,
            scn.ground_area,
            scn.wssg,
        );
        let area_dt = scn.ground_area * BIO_DT;
        t.steps += 1;
        t.potential_mm += pm(used) * BIO_DT;
        t.actual_kg += pm(used) * fw * area_dt;
        t.min_fwater = t.min_fwater.min(fw);
        if fw < 1.0 {
            t.stressed_steps += 1;
        }
        t.min_subsoil = t.min_subsoil.min(s.stocks[SUBSOIL_WATER].amount);
        let rh = vapour / science::saturation_vapour_kg(temp, n_ref);
        t.min_rh = t.min_rh.min(rh);
        if t.steps > 16 {
            // The chambers start with empty air (`water_vapor0 = 0`).
            t.min_rh_after_day_one = t.min_rh_after_day_one.min(rh);
        }
        let extra = (pm(chamber_vpd) - pm(weather_vpd)) * fw * area_dt;
        t.extra_kg += extra;
        if vapour < science::humidity_target_kg(temp, n_ref, setpoint) * (1.0 - 1e-9) {
            t.below_target_steps += 1;
            t.extra_kg_below_target += extra;
        }
    };
    let steps = steps_for_years(years);
    if perennial {
        run_perennial_with(
            &integrator,
            state,
            scn,
            &p,
            &resolver,
            BIO_DT,
            steps,
            0,
            &mut observe,
        )
        .expect("run");
    } else {
        run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps,
            None,
            &mut observe,
        )
        .expect("run");
    }
    t
}

fn main() {
    println!("Step 3b — sealed transpiration: weather deficit vs the chamber's own air\n");
    for (name, scn, years, perennial) in [
        (
            "sealed_chamber",
            sealed_chamber_scenario(),
            SEALED_CHAMBER_YEARS,
            false,
        ),
        (
            "perennial_chamber",
            perennial_chamber_scenario(),
            PERENNIAL_CHAMBER_YEARS,
            true,
        ),
        (
            "consumer_chamber",
            consumer_chamber_scenario(),
            CONSUMER_CHAMBER_YEARS,
            true,
        ),
    ] {
        let w = run(&scn, years, perennial, VpdRead::Weather);
        let c = run(&scn, years, perennial, VpdRead::Chamber);
        println!("== {name} ({years} yr, {} steps)", c.steps);
        println!(
            "   potential transpiration  weather {:.1} mm  chamber {:.1} mm  ratio {:.4}",
            w.potential_mm,
            c.potential_mm,
            c.potential_mm / w.potential_mm
        );
        println!(
            "   actual transpiration     weather {:.1} kg  chamber {:.1} kg  ratio {:.4}",
            w.actual_kg,
            c.actual_kg,
            c.actual_kg / w.actual_kg
        );
        for (label, t) in [("weather", &w), ("chamber", &c)] {
            println!(
                "   {label:<8} lowest stress factor {:.6}  steps below 1: {}  lowest subsoil {:.3} kg",
                t.min_fwater, t.stressed_steps, t.min_subsoil
            );
        }
        println!(
            "   chamber run: steps starting below the 75 % target {} of {}  lowest RH {:.4} (after day 1: {:.4})",
            c.below_target_steps, c.steps, c.min_rh, c.min_rh_after_day_one
        );
        println!(
            "   chamber run: extra transpiration {:.2} kg, of which on below-target steps {:.2} kg ({:.1} %)\n",
            c.extra_kg,
            c.extra_kg_below_target,
            100.0 * c.extra_kg_below_target / c.extra_kg
        );
    }
}
