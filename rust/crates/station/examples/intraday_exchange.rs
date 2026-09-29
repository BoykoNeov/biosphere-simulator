//! Plants and cabin trading gas within the day — the lab half of
//! `docs/plans/post-roadmap-intraday-gas-exchange.md`, as one command:
//!
//! ```text
//! cargo run --release -q -p station --example intraday_exchange
//! ```
//!
//! Runs every two-rate scenario that carries a plant in both day orders (the reference's
//! slow-first, and interleaved) through the lab-only [`TwoRate`] driver, and prints:
//!
//! * **control 1 at full length** — slow-first against the reference `run_sealed`, the whole
//!   4-year sealed station with its real re-sow hook, bit for bit;
//! * **slice 2** — every stock at the end of each frozen scenario, order against order;
//! * **control 4** — the area-scaling transform on the standalone sealed chamber;
//! * **slice 3** — the station at one crew member's crop (14.09 m²) and the whole crew's
//!   (187.45 m²), sized as the crew-loop record sized them;
//! * **slice 5** — the carbon books of each station run: what the crew exhaled, what the
//!   scrubber removed, what the plant side drew from the air, what harvest returned.
//!
//! Every plant step is re-evaluated at its entry state and handed to the backstop's own
//! [`arbitration::scale_factors`], so each rationing firing is NAMED by the stock it
//! overdraws. The probe's count is checked against the driver's plant-side count.
//!
//! It writes nothing and takes no decision.

use std::collections::BTreeMap;

use domains::biosphere::params::biosphere;
use domains::biosphere::readouts::withdrawal_demand;
use domains::biosphere::science::leaf_area_index;
use domains::biosphere::stocks::{
    CARBON_POOL, LEAF_C, O2_POOL, ROOT_C, STEM_C, STEM_RESERVE_C, STORAGE_C,
};
use domains::biosphere::system::{
    build_season, sealed_chamber_scenario, weather_resolver, SeasonScenario, SEALED_CHAMBER_YEARS,
};
use domains::biosphere::{steps_for_years, BIO_DT};
use domains::crew::FECAL_WASTE;
use domains::eclss::CO2_REMOVED;
use domains::params;
use simcore::arbitration;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::snapshot::from_engine;
use simcore::state::State;
use station::driver::{DayOrder, Side, SideTotals, TwoRate};
use station::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver};
use station::lighting::{build_lighting, lighting_bio_resolver, lighting_power_resolver};
use station::params as station_params;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, lighting_scenario, sealed_station_scenario,
    SealedStationScenario,
};
use station::sealed::{
    build_sealed_station, run_sealed, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};

const ORDERS: [DayOrder; 2] = [DayOrder::SlowFirst, DayOrder::Interleaved];

/// BVAD's crop area for one crew member, and for the sealed station's whole crew
/// (the crew-loop record, finding 3).
const ONE_CREW_M2: f64 = 14.0909;
const WHOLE_CREW_M2: f64 = 187.4523;

fn amount(s: &State, id: &str) -> f64 {
    s.stocks.get(id).map(|x| x.amount).unwrap_or(0.0)
}

fn crop_c(s: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C]
        .iter()
        .map(|id| amount(s, id))
        .sum()
}

// ------------------------------------------------------------------------------------
// The books a run keeps, by side.
// ------------------------------------------------------------------------------------

#[derive(Default)]
struct Books {
    /// The cabin CO₂ pool's net change from plant steps, cabin steps and re-sows.
    slow_dpool: f64,
    fast_dpool: f64,
    reset_dpool: f64,
    /// What the scrubber removed (the boundary sink's gain over cabin steps).
    scrubbed: f64,
    /// What cabin steps took out of the grain store (the harvest flow; 0 without it).
    harvested: f64,
    /// The probe's own count of plant-side firings, and the stocks each overdrew.
    probe_firings: u64,
    binding: BTreeMap<String, u64>,
    firing_days: std::collections::BTreeSet<u64>,
    /// Per stock: the worst plant step's withdrawal demand ÷ amount held, and its step.
    worst_draw: BTreeMap<String, (f64, u64)>,
    peak_lai: f64,
    peak_crop_c: f64,
    /// The cabin CO₂ each plant step starts from, by quarter of the day (index `n % 4`):
    /// sum and count, so the mean is what the crop saw. Quarters 1 and 2 carry 6 of the lamp's
    /// 16 hours each; 0 and 3 carry 2.
    pool_at_entry: [(f64, u64); 4],
    /// The same for the cabin O₂ pool — the other gas the two sides share.
    o2_at_entry: [(f64, u64); 4],
}

struct Measured {
    last: State,
    totals: SideTotals,
    books: Books,
    pool0: f64,
    days: usize,
}

/// Run `two` in `order` with a probe on every plant step.
fn measure(two: &TwoRate, order: DayOrder, s0: State, days: usize, ground_area: f64) -> Measured {
    let sla = biosphere().canopy.sla_per_mol_c;
    let pool0 = amount(&s0, CARBON_POOL);
    let mut books = Books::default();
    let slow_per_day = two.slow_steps_per_day;
    let mut observe = |side: Side, before: &State, after: &State| {
        let dpool = amount(after, CARBON_POOL) - amount(before, CARBON_POOL);
        match side {
            Side::Reset => books.reset_dpool += dpool,
            Side::Fast => {
                books.fast_dpool += dpool;
                books.scrubbed += amount(after, CO2_REMOVED) - amount(before, CO2_REMOVED);
                books.harvested += amount(before, STORAGE_C) - amount(after, STORAGE_C);
            }
            Side::Slow => {
                books.slow_dpool += dpool;
                let bound = two.slow_resolver.bind(before, two.slow_dt);
                let results: Vec<FlowResult> = two
                    .slow
                    .registry()
                    .flows()
                    .iter()
                    .map(|f| f.evaluate(before, &bound, two.slow_dt).expect("evaluate"))
                    .collect();
                let factors =
                    arbitration::scale_factors(&results, &before.stocks).expect("factors");
                let demand = withdrawal_demand(&results, &before.stocks);
                for (stock, d) in &demand {
                    let held = amount(before, stock);
                    let ratio = if held > 0.0 { d / held } else { f64::INFINITY };
                    let slot = books.worst_draw.entry(stock.clone()).or_insert((0.0, 0));
                    if ratio > slot.0 {
                        *slot = (ratio, before.n);
                    }
                }
                let fired = factors.iter().filter(|f| **f < 1.0).count() as u64;
                if fired > 0 {
                    books.probe_firings += fired;
                    books.firing_days.insert(before.n / slow_per_day);
                    for (stock, d) in &demand {
                        if *d > amount(before, stock) {
                            *books.binding.entry(stock.clone()).or_insert(0) += 1;
                        }
                    }
                }
                books.peak_lai =
                    books
                        .peak_lai
                        .max(leaf_area_index(amount(after, LEAF_C), sla, ground_area));
                books.peak_crop_c = books.peak_crop_c.max(crop_c(after));
                let slot = &mut books.pool_at_entry[(before.n % slow_per_day) as usize % 4];
                slot.0 += amount(before, CARBON_POOL);
                slot.1 += 1;
                let slot = &mut books.o2_at_entry[(before.n % slow_per_day) as usize % 4];
                slot.0 += amount(before, O2_POOL);
                slot.1 += 1;
            }
        }
    };
    let (states, totals) = two.run(order, s0, days, &mut observe).expect("run");
    assert_eq!(
        books.probe_firings, totals.slow_rationed,
        "the probe disagrees with the backstop — it is not measuring the run"
    );
    Measured {
        last: states.last().expect("a day").clone(),
        totals,
        books,
        pool0,
        days,
    }
}

fn print_books(label: &str, m: &Measured, exhale_per_day: f64) {
    let b = &m.books;
    let dpool = amount(&m.last, CARBON_POOL) - m.pool0;
    let residual = dpool - (b.slow_dpool + b.fast_dpool + b.reset_dpool);
    let exhaled = b.fast_dpool + b.scrubbed;
    let exhaled_expected = exhale_per_day * m.days as f64;
    let crop_net_draw = -b.slow_dpool;
    println!("  {label}");
    println!(
        "    rationing: plant side {}, cabin side {}, events {}   (days with a plant-side firing: {})",
        m.totals.slow_rationed,
        m.totals.fast_rationed,
        m.totals.events.len(),
        b.firing_days.len()
    );
    if !b.binding.is_empty() {
        let names: Vec<String> = b.binding.iter().map(|(k, v)| format!("{k} ×{v}")).collect();
        println!("    overdrawn at a firing: {}", names.join(", "));
    }
    let mut worst: Vec<(&String, &(f64, u64))> = b.worst_draw.iter().collect();
    worst.sort_by(|a, b| b.1 .0.total_cmp(&a.1 .0));
    let top: Vec<String> = worst
        .iter()
        .take(4)
        .map(|(k, (r, n))| format!("{k} {r:.4} (step {n})"))
        .collect();
    println!("    worst plant-step draw ÷ held: {}", top.join("; "));
    println!(
        "    peak LAI {:.4}   peak crop C {:.4} mol   final crop C {:.4} mol",
        b.peak_lai,
        b.peak_crop_c,
        crop_c(&m.last)
    );
    let means: Vec<String> = b
        .pool_at_entry
        .iter()
        .enumerate()
        .map(|(q, (s, c))| format!("q{q} {:.4}", s / *c as f64))
        .collect();
    println!(
        "    mean cabin CO2 at plant-step entry, by quarter-day (mol): {}",
        means.join("  ")
    );
    let o2: Vec<String> = b
        .o2_at_entry
        .iter()
        .enumerate()
        .map(|(q, (s, c))| format!("q{q} {:.3}", s / *c as f64))
        .collect();
    println!(
        "    mean cabin O2 at plant-step entry, by quarter-day (mol): {}",
        o2.join("  ")
    );
    println!(
        "    carbon books (mol C): crew exhaled {exhaled:.4} (expected {exhaled_expected:.4}, \
         rel {:.2e}); scrubbed {:.4}; plant side drew net {crop_net_draw:.4}; re-sow {:.4}; \
         harvest returned {:.4}",
        (exhaled - exhaled_expected) / exhaled_expected,
        b.scrubbed,
        b.reset_dpool,
        b.harvested
    );
    println!(
        "    pool budget residual {residual:.3e} mol   closure share (plant net draw ÷ \
         exhaled) {:.4} %",
        100.0 * crop_net_draw / exhaled
    );
}

// ------------------------------------------------------------------------------------
// Scaling the growing area (control 4 and slice 3).
// ------------------------------------------------------------------------------------

/// Scale the ground area and every extensive starting amount by `a`. With `with_air`, the
/// chamber's gas and water-vapour fields scale too (a bigger room); without, they stay (the
/// station's cabin air, which the crew-loop record showed must NOT be scaled).
fn scale_area(s: &SeasonScenario, a: f64, with_air: bool) -> SeasonScenario {
    let mut t = *s;
    t.ground_area *= a;
    for f in [
        &mut t.leaf_c0,
        &mut t.stem_c0,
        &mut t.root_c0,
        &mut t.storage_c0,
        &mut t.co2_atmos0,
        &mut t.litter_carbon0,
        &mut t.consumer_c0,
        &mut t.soil_water0,
        &mut t.water_source0,
        &mut t.soil_n0,
        &mut t.n_source0,
        &mut t.plant_n0,
        &mut t.sn_residual,
        &mut t.sn_critical,
        &mut t.subsoil_water0,
    ] {
        *f *= a;
    }
    if with_air {
        for f in [
            &mut t.chamber_air_capacity_mol,
            &mut t.chamber_co2_mol0,
            &mut t.chamber_o2_mol0,
            &mut t.water_vapor0,
            &mut t.condensate0,
        ] {
            *f *= a;
        }
    }
    t
}

/// Control 4: does area-scaling the standalone sealed chamber reproduce `a ×` the unscaled
/// run, every stock at every step? Prints the worst deviation.
fn similarity(a: f64) {
    let base = sealed_chamber_scenario();
    let big = scale_area(&base, a, true);
    let (mut s1, r1) = build_season(&base).expect("build");
    let (mut sa, ra) = build_season(&big).expect("build scaled");
    let e1 = weather_resolver(&base, SEALED_CHAMBER_YEARS).expect("resolver");
    let ea = weather_resolver(&big, SEALED_CHAMBER_YEARS).expect("resolver");
    let (i1, ia) = (EulerIntegrator::new(r1), EulerIntegrator::new(ra));
    let mut worst_rel = (0.0f64, String::new(), 0u64);
    let mut worst_aux = (0.0f64, String::new(), 0u64);
    let steps = steps_for_years(SEALED_CHAMBER_YEARS);
    let mut rationed = (0u64, 0u64);
    for _ in 0..steps {
        let r = i1.step_report(&s1, &e1, BIO_DT).expect("step");
        let q = ia.step_report(&sa, &ea, BIO_DT).expect("step scaled");
        rationed.0 += r.rationed;
        rationed.1 += q.rationed;
        s1 = r.state;
        sa = q.state;
        for (id, st) in &s1.stocks {
            let want = a * st.amount;
            let got = amount(&sa, id);
            // Relative where the stock holds anything; a stock at exactly zero must stay so.
            let dev = if want != 0.0 {
                ((got - want) / want).abs()
            } else {
                got.abs()
            };
            if dev > worst_rel.0 {
                worst_rel = (dev, id.clone(), s1.n);
            }
        }
        for (k, v) in &s1.aux {
            let got = sa.aux.get(k).copied().unwrap_or(f64::NAN);
            let dev = if *v != 0.0 {
                ((got - v) / v).abs()
            } else {
                got.abs()
            };
            if dev > worst_aux.0 || dev.is_nan() {
                worst_aux = (dev, k.clone(), s1.n);
            }
        }
    }
    println!(
        "  A = {a}: {steps} steps; worst stock deviation from A× {:.3e} ({} at step {}); worst \
         aux deviation {:.3e} ({} at step {}); rationed {} vs {}",
        worst_rel.0,
        worst_rel.1,
        worst_rel.2,
        worst_aux.0,
        worst_aux.1,
        worst_aux.2,
        rationed.0,
        rationed.1
    );
}

// ------------------------------------------------------------------------------------
// The sealed station.
// ------------------------------------------------------------------------------------

/// The sealed station sized for `a` m² of crop, as the crew-loop record did it: the plot
/// and every extensive plant and soil amount × a, NOT the air; lamp, solar and battery × a;
/// one season.
fn crew_sized(a: f64) -> SealedStationScenario {
    let mut sc = sealed_station_scenario();
    sc.bio = scale_area(&sc.bio, a, false);
    sc.lamp_power_w *= a;
    sc.power.solar_peak_w *= a;
    sc.power.battery0 *= a;
    sc.battery0 *= a;
    sc.years = 1;
    sc
}

fn sealed_measure(sc: &SealedStationScenario, with_harvest: bool, order: DayOrder) -> Measured {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let hp = station_params::harvest();
    let (s0, bio, fast) = build_sealed_station(
        &charge,
        &thermal,
        &crew,
        &eclss,
        &recovery,
        &lamp,
        &hp,
        sc,
        with_harvest,
        false,
    )
    .expect("build sealed");
    let bio_res = sealed_bio_resolver(&lamp, sc).expect("bio resolver");
    let fast_res = sealed_fast_resolver(&charge, sc).expect("fast resolver");
    let reset = sealed_reset_hook(sc);
    let (slow, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let two = TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: &bio_res,
        fast_resolver: &fast_res,
        steps_per_day: sc.steps_per_day,
        slow_steps_per_day: sc.bio_steps_per_day,
        slow_dt: sc.bio_dt,
        fast_dt: sc.cabin_dt,
        slow_reset: Some(&*reset),
    };
    measure(&two, order, s0, sc.days(), sc.bio.ground_area)
}

/// Control 1 at full length: the reference `run_sealed` against `TwoRate` slow-first, with the
/// real re-sow hook, over the whole frozen horizon.
fn control_full_sealed(slow_first: &State) {
    let charge = params::charge();
    let thermal = params::thermal();
    let crew = params::crew();
    let eclss = params::eclss();
    let recovery = station_params::water_recovery();
    let lamp = station_params::lamp();
    let hp = station_params::harvest();
    let sc = sealed_station_scenario();
    let (s0, bio, fast) = build_sealed_station(
        &charge, &thermal, &crew, &eclss, &recovery, &lamp, &hp, &sc, false, false,
    )
    .expect("build sealed");
    let (states, _, _) = run_sealed(
        &EulerIntegrator::new(bio),
        &EulerIntegrator::new(fast),
        s0,
        &sealed_bio_resolver(&lamp, &sc).expect("bio"),
        &sealed_fast_resolver(&charge, &sc).expect("fast"),
        &sc,
    )
    .expect("reference run");
    let same =
        from_engine(states.last().expect("day")).to_json() == from_engine(slow_first).to_json();
    println!(
        "  control 1, full 4-year sealed station with the real re-sow: slow-first == reference \
         bit for bit: {same}"
    );
    assert!(
        same,
        "control 1 failed at full length — the lab is not measuring the reference"
    );
}

// ------------------------------------------------------------------------------------
// Slice 2's comparison of end states.
// ------------------------------------------------------------------------------------

fn compare(label: &str, a: &State, b: &State) {
    let mut moved: Vec<(f64, String, f64, f64)> = Vec::new();
    let mut same = 0;
    for (id, st) in &a.stocks {
        let (x, y) = (st.amount, amount(b, id));
        if x.to_bits() == y.to_bits() {
            same += 1;
        } else {
            let rel = if x != 0.0 { (y - x) / x.abs() } else { y };
            moved.push((rel, id.clone(), x, y));
        }
    }
    moved.sort_by(|p, q| q.0.abs().total_cmp(&p.0.abs()));
    println!(
        "  {label}: {} stocks bit-identical, {} moved; crop C {:.6} → {:.6} ({:+.4} %)",
        same,
        moved.len(),
        crop_c(a),
        crop_c(b),
        100.0 * (crop_c(b) - crop_c(a)) / crop_c(a)
    );
    for (rel, id, x, y) in moved.iter().take(12) {
        println!("      {id:<34} {x:.9e} → {y:.9e}   rel {rel:+.3e}");
    }
    if moved.len() > 12 {
        println!("      … {} more, all smaller", moved.len() - 12);
    }
}

fn two_rate_final(
    build: &dyn Fn() -> (State, EulerIntegrator, EulerIntegrator),
    bio_res: &simcore::environment::SourceResolver,
    fast_res: &simcore::environment::SourceResolver,
    shape: (u64, u64, f64, f64, usize),
    order: DayOrder,
) -> (State, SideTotals) {
    let (s0, slow, fast) = build();
    let (steps_per_day, slow_steps_per_day, slow_dt, fast_dt, days) = shape;
    let two = TwoRate {
        slow: &slow,
        fast: &fast,
        slow_resolver: bio_res,
        fast_resolver: fast_res,
        steps_per_day,
        slow_steps_per_day,
        slow_dt,
        fast_dt,
        slow_reset: None,
    };
    let (states, totals) = two.run(order, s0, days, &mut |_, _, _| {}).expect("run");
    (states.last().expect("day").clone(), totals)
}

fn main() {
    let crew = params::crew();
    let eclss = params::eclss();
    let exhale_per_day = {
        let sc = sealed_station_scenario();
        crew.respired_carbon_fraction * sc.cabin.food_intake_rate * 86400.0
    };
    // Sections by name (`roster`, `sealed`, `similarity`, `crew`); none given runs all.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let want = |s: &str| args.is_empty() || args.iter().any(|a| a == s);

    if want("roster") {
        println!("== slice 2: the frozen roster, slow-first → interleaved ==");
        {
            let sc = greenhouse_scenario();
            let bio_res = greenhouse_bio_resolver(&sc).expect("bio");
            let cabin_res = greenhouse_cabin_resolver(&sc).expect("cabin");
            let build = || {
                let (s, b, c) = build_greenhouse(&crew, &eclss, &sc, true, FECAL_WASTE).expect("b");
                (s, EulerIntegrator::new(b), EulerIntegrator::new(c))
            };
            let shape = (
                sc.steps_per_day,
                sc.bio_steps_per_day,
                sc.bio_dt,
                sc.cabin_dt,
                sc.days,
            );
            let (a, ta) = two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::SlowFirst);
            let (b, tb) =
                two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::Interleaved);
            compare("greenhouse (7 d)", &a, &b);
            println!("      rationed {ta:?} → {tb:?}");
        }
        {
            let lamp = station_params::lamp();
            let sc = lighting_scenario();
            let bio_res = lighting_bio_resolver(&lamp, &sc, true).expect("bio");
            let power_res = lighting_power_resolver(&sc).expect("power");
            let build = || {
                let (s, b, p) = build_lighting(&lamp, &sc, true).expect("b");
                (s, EulerIntegrator::new(b), EulerIntegrator::new(p))
            };
            let shape = (
                sc.steps_per_day,
                sc.bio_steps_per_day,
                sc.bio_dt,
                sc.power_dt,
                sc.days,
            );
            let (a, _) = two_rate_final(&build, &bio_res, &power_res, shape, DayOrder::SlowFirst);
            let (b, _) = two_rate_final(&build, &bio_res, &power_res, shape, DayOrder::Interleaved);
            compare("lighting (7 d)", &a, &b);
        }
        {
            let hp = station_params::harvest();
            let sc = harvest_scenario();
            let gh = sc.greenhouse;
            let bio_res = harvest_bio_resolver(&sc).expect("bio");
            let cabin_res = harvest_cabin_resolver(&sc).expect("cabin");
            let build = || {
                let (s, b, c) = build_harvest(&crew, &eclss, &hp, &sc, true, true).expect("b");
                (s, EulerIntegrator::new(b), EulerIntegrator::new(c))
            };
            let shape = (
                gh.steps_per_day,
                gh.bio_steps_per_day,
                gh.bio_dt,
                gh.cabin_dt,
                gh.days,
            );
            let (a, ta) = two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::SlowFirst);
            let (b, tb) =
                two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::Interleaved);
            compare("harvest (7 d)", &a, &b);
            println!("      rationed {ta:?} → {tb:?}");

            // Isolation for the harvest ring's movement: which of its two seams makes the order
            // matter — harvest draining the grain store, or feces landing in the soil litter?
            println!("    harvest isolation, slow-first → interleaved, relative change:");
            for (with_harvest, close_feces) in
                [(false, false), (true, false), (false, true), (true, true)]
            {
                let build = || {
                    let (s, b, c) =
                        build_harvest(&crew, &eclss, &hp, &sc, with_harvest, close_feces)
                            .expect("b");
                    (s, EulerIntegrator::new(b), EulerIntegrator::new(c))
                };
                let (a, _) =
                    two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::SlowFirst);
                let (b, _) =
                    two_rate_final(&build, &bio_res, &cabin_res, shape, DayOrder::Interleaved);
                let rel = |id: &str| (amount(&b, id) - amount(&a, id)) / amount(&a, id).abs();
                println!(
                "      harvest {with_harvest:<5} feces→litter {close_feces:<5}: crop C {:+.3e}  \
                 storage_c {:+.3e}  microbial C {:+.3e}  humus C {:+.3e}  litter C {:+.3e}",
                (crop_c(&b) - crop_c(&a)) / crop_c(&a),
                rel(STORAGE_C),
                rel(domains::biosphere::stocks::MICROBIAL_CARBON),
                rel(domains::biosphere::stocks::HUMUS_CARBON),
                rel(domains::biosphere::stocks::LITTER_CARBON),
            );
            }
        }
    }
    if want("sealed") {
        let sealed = sealed_station_scenario();
        let runs: Vec<Measured> = ORDERS
            .iter()
            .map(|o| sealed_measure(&sealed, false, *o))
            .collect();
        control_full_sealed(&runs[0].last);
        compare("sealed_station (4 y)", &runs[0].last, &runs[1].last);

        println!("\n== slice 5: the carbon books, the frozen 1 m² sealed station (4 y) ==");
        print_books("slow-first (the reference order)", &runs[0], exhale_per_day);
        print_books("interleaved", &runs[1], exhale_per_day);
    }
    if want("similarity") {
        println!("\n== control 4: area scaling on the standalone sealed chamber (air included) ==");
        similarity(ONE_CREW_M2);
        similarity(WHOLE_CREW_M2);
    }
    if want("crew") {
        println!(
            "\n== slice 3 + 5: the crew-sized station, one season, harvest on, air NOT scaled =="
        );
        // 1 m² is the like-for-like baseline: the frozen station's plot, one season, harvest on.
        for a in [1.0, ONE_CREW_M2, WHOLE_CREW_M2] {
            let sc = crew_sized(a);
            println!(
                "-- {a} m²: lamp {:.1} W, solar peak {:.1} W, battery {:.3e} J; unscaled air: \
             capacity {} mol, CO2 {} mol, O2 {} mol, vapour {} kg, condensate {} kg",
                sc.lamp_power_w,
                sc.power.solar_peak_w,
                sc.battery0,
                sc.bio.chamber_air_capacity_mol,
                sc.bio.chamber_co2_mol0,
                sc.bio.chamber_o2_mol0,
                sc.bio.water_vapor0,
                sc.bio.condensate0
            );
            for order in ORDERS {
                let m = sealed_measure(&sc, true, order);
                print_books(&format!("{order:?}"), &m, exhale_per_day);
            }
        }
    }
}
