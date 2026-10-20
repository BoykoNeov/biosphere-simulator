//! **What each cited cabin CO₂ level costs**: the sealed station at today's ~400 ppm and at
//! three levels BVAD Rev 2 Table 4-1 (printed p. 63) names. Lab-only, a price for the user's
//! choice (`docs/plans/post-roadmap-co2-level.md`).
//!
//! ```text
//! cargo run --release -q -p station --example co2_level_price
//! cargo run --release -q -p station --example co2_level_price -- above-target  # one row
//! ```
//!
//! Today the cabin's CO₂ is set by nothing anyone chose: the scrubber is first-order, so the
//! cabin settles at the crew's output over its rate, `P / k = 3.796e-3 / 1e-3 = 3.796 mol` —
//! 399.6 ppm of the 9500 mol cabin, Earth's outdoor air. This tool holds the same first-order
//! form and moves `k` to `P / n_target`, so each run settles at the level named, and starts the
//! cabin AT that level (`chamber_co2_mol0`), as the frozen scenario starts at its own — no
//! start-up transient in the price. The form that would carry a cited level in the reference
//! is a separate question the plan takes up; at steady state both hold the same level.
//!
//! ⚠ **It writes nothing and takes no decision.** A run that rations, reverses or fails is
//! printed as that result.
//!
//! ⚠ **Since 0.13 kPa was adopted (2026-10-19)** "today" means the pre-adoption reference: the
//! first-order rows run through `domains::lab::with_first_order_scrubber`, and the
//! above-target row is the loader's own form.

use domains::biosphere::stocks::{CARBON_POOL, LEAF_C, ROOT_C, STEM_C, STORAGE_C};
use domains::params;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::params as station_params;
use station::scenario::{sealed_station_scenario, SEALED_STATION_SEASON_DAYS};
use station::sealed::{build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver};

/// BVAD's reference pressure for the kPa → mole-fraction step (Table 4-1's 101 kPa nominal;
/// the model's `STANDARD_ATMOSPHERE_PA`).
const P_STD_KPA: f64 = 101.325;

/// The levels priced: (label, mole fraction, the lab's above-target form?, what names it).
fn levels() -> Vec<(&'static str, Option<f64>, bool, &'static str)> {
    vec![
        ("today", None, false, "P/k with the pre-adoption k = 1e-3 — nobody's choice"),
        ("plants 0.13 kPa", Some(0.13 / P_STD_KPA), false, "Table 4-1 p[CO2] for Plants, nominal ('accepted optimum')"),
        ("ISS cap 0.40 kPa", Some(0.40 / P_STD_KPA), false, "BVAD §4.1 text: ISS's lower operating maximum"),
        ("crew 0.507 kPa", Some(0.507 / P_STD_KPA), false, "Table 4-1 p[CO2] for Crew, nominal"),
        ("plants 0.13 kPa, above-target form", Some(0.13 / P_STD_KPA), true, "the adopted form (plan §9-10): k = 1e-2 on the excess over 130 Pa"),
    ]
}

/// Total plant organic carbon (mol) — leaf, stem, root and grain.
fn plant_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C].iter().map(|id| s.stocks[*id].amount).sum()
}

struct Price {
    rationed: u64,
    events: usize,
    ppm: Vec<(f64, f64, f64)>,
    plant_peak: Vec<f64>,
    grain_peak: Vec<f64>,
    scrubbed: f64,
}

fn run_at(target: Option<f64>, above_target: bool) -> Result<Price, String> {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let lamp = station_params::lamp();
    // Since the adoption (2026-10-19) the loader's form is the above-target one; every row but
    // that one runs the pre-adoption first-order form through the lab's switch.
    let mut eclss = domains::lab::with_first_order_scrubber(
        params::eclss(),
        domains::lab::FIRST_ORDER_SCRUB_RATE,
    );
    let mut scenario = sealed_station_scenario();
    let air = scenario.bio.chamber_air_capacity_mol;
    let production = crew.respired_carbon_fraction * scenario.cabin.food_intake_rate(&crew);
    // The pre-adoption start: the first-order form's own fixed point.
    scenario.bio.chamber_co2_mol0 = 3.796;
    if let Some(x) = target {
        let n = x * air;
        eclss.co2_scrub_rate = production / n;
        scenario.bio.chamber_co2_mol0 = n;
        if above_target {
            // The adopted form: remove only the excess over the cited partial pressure.
            eclss = params::eclss();
            scenario.bio.chamber_co2_mol0 =
                eclss.co2_target_mol(air).expect("an above-target scrubber has a target");
        }
    }
    let (state, bio_reg, fast_reg) = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        &eclss,
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .map_err(|e| format!("{e:?}"))?;
    let bio = sealed_bio_resolver(&lamp, &scenario).map_err(|e| format!("{e:?}"))?;
    let fast = sealed_fast_resolver(&charge, &crew, &scenario).map_err(|e| format!("{e:?}"))?;
    let removed0 = state.stocks[domains::eclss::CO2_REMOVED].amount;
    let (states, rationed, events) = run_sealed(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        &scenario,
    )
    .map_err(|e| format!("{e:?}"))?;
    let days = SEALED_STATION_SEASON_DAYS;
    let mut ppm = Vec::new();
    let mut plant_peak = Vec::new();
    let mut grain_peak = Vec::new();
    for year in states[1..].chunks(days) {
        let c: Vec<f64> = year.iter().map(|s| 1e6 * s.stocks[CARBON_POOL].amount / air).collect();
        let lo = c.iter().fold(f64::INFINITY, |a, &v| a.min(v));
        let hi = c.iter().fold(f64::NEG_INFINITY, |a, &v| a.max(v));
        ppm.push((lo, c.iter().sum::<f64>() / c.len() as f64, hi));
        plant_peak.push(year.iter().map(plant_c).fold(0.0, f64::max));
        grain_peak.push(year.iter().map(|s| s.stocks[STORAGE_C].amount).fold(0.0, f64::max));
    }
    let last = states.last().expect("a final state");
    let scrubbed = last.stocks[domains::eclss::CO2_REMOVED].amount - removed0;
    Ok(Price { rationed, events: events.len(), ppm, plant_peak, grain_peak, scrubbed })
}

fn main() {
    println!("sealed station, 4 years, CO2 held by a first-order scrubber at k = P / n_target");
    println!("(day-boundary readings; plant C and grain are each year's peak, mol)\n");
    let mut base: Option<Price> = None;
    let only: Vec<String> = std::env::args().skip(1).collect();
    for (label, x, above, why) in levels() {
        // The baseline always runs: every other row is read against it.
        if !only.is_empty() && x.is_some() && !only.iter().any(|o| label.contains(o.as_str())) {
            continue;
        }
        println!("== {label}  ({why})");
        match run_at(x, above) {
            Err(e) => println!("   FAILED: {e}\n"),
            Ok(p) => {
                println!("   rationed {}  events {}  scrubbed {:.1} mol C", p.rationed, p.events, p.scrubbed);
                for (y, ((lo, mean, hi), (pc, g))) in
                    p.ppm.iter().zip(p.plant_peak.iter().zip(&p.grain_peak)).enumerate()
                {
                    let vs = base.as_ref().map_or(String::new(), |b| {
                        format!(
                            "   vs today: plant {:+.1} %, grain {:+.1} %",
                            100.0 * (pc / b.plant_peak[y] - 1.0),
                            100.0 * (g / b.grain_peak[y] - 1.0)
                        )
                    });
                    println!(
                        "   year {}: CO2 {lo:.0}/{mean:.0}/{hi:.0} ppm (min/mean/max)  plant {pc:.3}  grain {g:.3}{vs}",
                        y + 1
                    );
                }
                println!();
                if base.is_none() {
                    base = Some(p);
                }
            }
        }
    }
}
