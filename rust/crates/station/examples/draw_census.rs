//! The draw census — slice 1 of the review's Step 2 ("small air volumes against a big step"),
//! `docs/plans/post-roadmap-draw-census.md`, as one command:
//!
//! ```text
//! cargo run --release -q -p station --example draw_census
//! cargo run --release -q -p station --example draw_census -- --only jar   # a subset
//! ```
//!
//! For every frozen run (every golden file on disk but the hand-written `state_snapshot.json`),
//! and for every store the Euler backstop guards, the most any single step takes out of the
//! store, as a fraction of what it holds when the step starts. `rationed == 0` is yes/no; this
//! is how far from yes.
//!
//! One probe, three drivers: the biosphere season loop (re-sow, THEN probe, then step —
//! `run_season`'s own order), the single-rate loop, and the station's two-rate day through the
//! lab [`TwoRate`] observer, both sides probed. The probe evaluates every flow on the step's own
//! starting state and sums each store's withdrawals the way arbitration does.
//!
//! Controls, printed with every run and asserted where they can be:
//!
//! 1. the run's end state byte-matches its committed golden (or, for the two folded goldens,
//!    the reference runner's own end state) — so the census loop IS the reference run;
//! 2. the sealed jar's CO₂ row reads the pinned 0.165230 at step 3108 (a full run that does not
//!    reproduce it fails the verdict);
//! 3. on every step, `before + Σ legs (after the backstop's scaling) == after` bit for bit for
//!    every store — the check that can fail on a run that never rations;
//! 4. the probe's firing count equals the integrator's, and on one case that really rations
//!    (the jar shrunk to a tenth, in the explicit CO₂ form) it is not zero. An unfiltered run
//!    that does not reach that case fails the verdict too.
//!
//! It writes nothing and takes no decision.

use std::collections::BTreeMap;

use domains::biosphere::params::{self as bio_params, BiosphereParams};
use domains::biosphere::readouts::withdrawal_demand;
use domains::biosphere::science::Co2Read;
use domains::biosphere::stocks::{CARBON_POOL, WATER_VAPOR};
use domains::biosphere::system::{
    annual_reset_with, build_season_with, consumer_chamber_scenario, perennial_chamber_scenario,
    sealed_chamber_scenario, SeasonScenario, DEFAULT_SCENARIO,
};
use domains::biosphere::weather::{season_forcing, weather_facts};
use domains::biosphere::{
    season_setup_composed, season_steps, steps_for_years, BIO_DT, CONSUMER_CHAMBER_YEARS,
    LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};
use domains::crew::{build_crew, crew_resolver, MISSION_DAYS, MISSION_SCENARIO};
use domains::eclss::{build_eclss, eclss_resolver, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS};
use domains::goldens::{committed, compare, Verdict};
use domains::lab::biosphere_with_co2_read;
use domains::params;
use domains::power::{
    build_power, power_resolver, BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
};
use domains::thermal::{build_thermal, thermal_resolver, EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS};
use simcore::arbitration;
use simcore::conservation::assert_conserved_default;
use simcore::environment::SourceResolver;
use simcore::flow::FlowResult;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::cabin::{build_cabin, cabin_resolver};
use station::driver::{DayOrder, Side, TwoRate};
use station::greenhouse::{build_greenhouse, greenhouse_bio_resolver, greenhouse_cabin_resolver};
use station::harvest::{build_harvest, harvest_bio_resolver, harvest_cabin_resolver};
use station::lighting::{build_lighting, lighting_bio_resolver, lighting_power_resolver};
use station::params as station_params;
use station::run_station;
use station::scenario::{
    greenhouse_scenario, harvest_scenario, lighting_scenario, sealed_station_scenario,
    CABIN_GAS_SCENARIO, CABIN_GAS_STEPS, HEAT_CLOSURE_DAYS, HEAT_CLOSURE_SCENARIO,
    SEALED_ENERGY_DAYS, WATER_RECOVERY_SCENARIO, WATER_RECOVERY_STEPS,
};
use station::sealed::{
    build_sealed_station, sealed_bio_resolver, sealed_fast_resolver, sealed_reset_hook,
};
use station::system::{build_station, station_resolver};
use station::water::{build_water_recovery, water_recovery_resolver};

/// The pinned jar figure (`science_gates::margins::JAR_CO2_STEP_DRAW`) and its step. ⚠ A COPY:
/// the gate was re-pinned for the 1/16 step and again for option C (2026-09-30), and this stayed
/// at the quarter-day step's 0.756662 at step 777 — printed "NOT reproduced" while the verdict
/// read "all held", until control 2 joined the verdict (2026-10-04).
const JAR_CO2_DRAW: f64 = 0.165230;
const JAR_CO2_STEP: u64 = 3108;
/// The roster key of control 4's rationing case — one name, so the verdict cannot look up a
/// case the roster no longer carries.
const SQUEEZED_JAR: &str = "lab_squeezed_jar";

const SECONDS_PER_DAY: f64 = 86400.0;

// ------------------------------------------------------------------------------------
// The probe.
// ------------------------------------------------------------------------------------

/// One store's draws over a run, on one side.
#[derive(Clone, Debug)]
struct Row {
    worst: f64,
    worst_n: u64,
    worst_day: f64,
    /// The flow with the largest withdrawal from this store on the worst step.
    worst_flow: String,
    /// Σ draw and the number of steps that drew at all — the mean.
    sum: f64,
    drawn: u64,
    over_tenth: u64,
    over_half: u64,
}

/// Every store's draws over a run, on one side, plus the probe's own counts.
#[derive(Default)]
struct Census {
    rows: BTreeMap<String, Row>,
    steps: u64,
    /// The probe's count of flows the backstop scaled (arbitration's own `fired`).
    firings: u64,
    /// Steps where `before + Σ legs != after` for some store (control 3).
    mismatched: u64,
    first_mismatch: Option<String>,
}

impl Census {
    /// Probe one step: `before` → `after`, at time `day` (days since the run began).
    fn probe(
        &mut self,
        registry: &Registry,
        resolver: &SourceResolver,
        dt: f64,
        before: &State,
        after: &State,
        day: f64,
    ) {
        let bound = resolver.bind(before, dt);
        let flows = registry.flows();
        let results: Vec<FlowResult> = flows
            .iter()
            .map(|f| f.evaluate(before, &bound, dt).expect("evaluate"))
            .collect();
        let demand = withdrawal_demand(&results, &before.stocks);
        for (stock, d) in &demand {
            let held = before.stocks[stock].amount;
            let ratio = if held > 0.0 { d / held } else { f64::INFINITY };
            let row = self.rows.entry(stock.clone()).or_insert(Row {
                worst: f64::NEG_INFINITY,
                worst_n: 0,
                worst_day: 0.0,
                worst_flow: String::new(),
                sum: 0.0,
                drawn: 0,
                over_tenth: 0,
                over_half: 0,
            });
            row.sum += ratio;
            row.drawn += 1;
            row.over_tenth += u64::from(ratio > 0.1);
            row.over_half += u64::from(ratio > 0.5);
            if ratio > row.worst {
                row.worst = ratio;
                row.worst_n = before.n;
                row.worst_day = day;
                row.worst_flow = largest_withdrawal(flows, &results, stock);
            }
        }
        // Control 3: re-apply the step the way the integrator does — the backstop's own
        // scaling, legs summed in flow order × leg order from 0.0, then `amount + 1.0 * delta`.
        let (scaled, fired) =
            arbitration::min_scaling(&results, &before.stocks).expect("min_scaling");
        self.firings += fired;
        let mut delta: BTreeMap<&str, f64> = BTreeMap::new();
        for r in &scaled {
            for leg in &r.legs {
                *delta.entry(leg.stock.as_str()).or_insert(0.0) += leg.amount;
            }
        }
        for (id, stock) in &before.stocks {
            let expected = match delta.get(id.as_str()) {
                Some(d) => stock.amount + 1.0 * d,
                None => stock.amount,
            };
            let got = after.stocks[id].amount;
            if expected.to_bits() != got.to_bits() {
                self.mismatched += 1;
                self.first_mismatch.get_or_insert_with(|| {
                    format!(
                        "step {} ({id}): expected {expected:e}, got {got:e}",
                        before.n
                    )
                });
                break;
            }
        }
        self.steps += 1;
    }
}

/// The flow with the largest withdrawal from `stock` in `results`.
fn largest_withdrawal(
    flows: &[Box<dyn simcore::flow::Flow>],
    results: &[FlowResult],
    stock: &str,
) -> String {
    let mut best = (0.0, String::new());
    for (f, r) in flows.iter().zip(results) {
        let out: f64 = r
            .legs
            .iter()
            .filter(|l| l.stock == stock && l.amount < 0.0)
            .map(|l| -l.amount)
            .sum();
        if out > best.0 {
            best = (out, f.id().to_string());
        }
    }
    best.1
}

// ------------------------------------------------------------------------------------
// A measured run.
// ------------------------------------------------------------------------------------

struct Measured {
    name: String,
    /// Per side ("plant"; "fast" — the cabin, or Power under `lighting`; "all" for a single-rate run).
    sides: Vec<(&'static str, Census, u64)>,
    events: usize,
    control1: String,
    control1_ok: bool,
}

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// Control 1 against a committed golden.
fn against_golden(final_state: &State, golden: &str) -> (String, bool) {
    let numerics = station::goldens::all()
        .into_iter()
        .find(|g| g.name == golden)
        .unwrap_or_else(|| panic!("{golden} is not on the roster"))
        .numerics;
    match compare(&snapshot(final_state), &committed(golden), numerics) {
        Verdict::ByteExact => (format!("{golden}: byte-exact"), true),
        Verdict::StructurallyEqual => {
            (format!("{golden}: structurally equal (off-platform)"), true)
        }
        Verdict::Differs(why) => (format!("{golden}: DIFFERS — {why}"), false),
    }
}

/// The biosphere season loop: re-sow (if perennial), then step, then probe the step.
fn season(
    name: &str,
    scenario: SeasonScenario,
    years: usize,
    p: &BiosphereParams,
    resow: bool,
    golden: Option<&str>,
) -> Measured {
    let (mut state, integrator, resolver) =
        season_setup_composed(&scenario, years, p, &build_season_with).expect("setup");
    let year = season_steps() as u64;
    let mut census = Census::default();
    let (mut rationed, mut events) = (0u64, 0usize);
    for _ in 0..steps_for_years(years) {
        if resow && state.n > 0 && state.n.is_multiple_of(year) {
            let sown = annual_reset_with(&state, &scenario, p).expect("re-sow");
            assert_conserved_default(&state, &sown).expect("re-sow conserves");
            state = sown;
        }
        let report = integrator
            .step_report(&state, &resolver, BIO_DT)
            .expect("euler step");
        let day = state.n as f64 * BIO_DT;
        census.probe(
            integrator.registry(),
            &resolver,
            BIO_DT,
            &state,
            &report.state,
            day,
        );
        rationed += report.rationed;
        events += report.events.len();
        state = report.state;
    }
    let (control1, control1_ok) = match golden {
        Some(g) => against_golden(&state, g),
        None => ("(no golden: a lab case)".into(), true),
    };
    Measured {
        name: name.into(),
        sides: vec![("plant", census, rationed)],
        events,
        control1,
        control1_ok,
    }
}

/// The single-rate loop. `reference` re-runs the reference runner for a folded golden.
fn single(
    name: &str,
    integrator: &EulerIntegrator,
    initial: State,
    resolver: &SourceResolver,
    dt_seconds: f64,
    steps: u64,
    golden: Result<&str, &str>,
) -> Measured {
    let mut state = initial.clone();
    let mut census = Census::default();
    let (mut rationed, mut events) = (0u64, 0usize);
    for _ in 0..steps {
        let report = integrator
            .step_report(&state, resolver, dt_seconds)
            .expect("euler step");
        let day = state.n as f64 * dt_seconds / SECONDS_PER_DAY;
        census.probe(
            integrator.registry(),
            resolver,
            dt_seconds,
            &state,
            &report.state,
            day,
        );
        rationed += report.rationed;
        events += report.events.len();
        state = report.state;
    }
    let (control1, control1_ok) = match golden {
        Ok(g) => against_golden(&state, g),
        Err(folded) => {
            let mut noop = |_: &State| {};
            let (reference, _, _) =
                run_station(integrator, initial, resolver, dt_seconds, steps, &mut noop)
                    .expect("reference run");
            let same = snapshot(&reference) == snapshot(&state);
            (
                format!(
                    "{folded} (folded): end state vs the reference runner's — {}",
                    if same { "bit-identical" } else { "DIFFERS" }
                ),
                same,
            )
        }
    };
    Measured {
        name: name.into(),
        sides: vec![("all", census, rationed)],
        events,
        control1,
        control1_ok,
    }
}

/// The station's two-rate day, both sides probed.
fn two_rate(name: &str, two: &TwoRate, s0: State, days: usize, golden: &str) -> Measured {
    let mut plant = Census::default();
    let mut cabin = Census::default();
    let mut fast_k: u64 = 0;
    let mut observe = |side: Side, before: &State, after: &State| match side {
        Side::Reset => {}
        Side::Slow => plant.probe(
            two.slow.registry(),
            two.slow_resolver,
            two.slow_dt,
            before,
            after,
            before.n as f64 * two.slow_dt,
        ),
        Side::Fast => {
            cabin.probe(
                two.fast.registry(),
                two.fast_resolver,
                two.fast_dt,
                before,
                after,
                fast_k as f64 * two.fast_dt / SECONDS_PER_DAY,
            );
            fast_k += 1;
        }
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, s0, days, &mut observe)
        .expect("two-rate run");
    let (control1, control1_ok) = against_golden(states.last().expect("a day"), golden);
    Measured {
        name: name.into(),
        sides: vec![
            ("plant", plant, totals.slow_rationed),
            ("fast", cabin, totals.fast_rationed),
        ],
        events: totals.events.len(),
        control1,
        control1_ok,
    }
}

// ------------------------------------------------------------------------------------
// The roster.
// ------------------------------------------------------------------------------------

type Job = (&'static str, Box<dyn Fn() -> Measured>);

fn roster() -> Vec<Job> {
    let frozen = || bio_params::biosphere();
    vec![
        (
            "open_season",
            Box::new(move || {
                season(
                    "open_season",
                    DEFAULT_SCENARIO,
                    1,
                    &frozen(),
                    false,
                    Some("season_euler_state.json"),
                )
            }),
        ),
        (
            "sealed_jar",
            Box::new(move || {
                season(
                    "sealed_jar",
                    sealed_chamber_scenario(),
                    SEALED_CHAMBER_YEARS,
                    &frozen(),
                    false,
                    Some("sealed_chamber_state.json"),
                )
            }),
        ),
        (
            "perennial_5",
            Box::new(move || {
                season(
                    "perennial_5",
                    perennial_chamber_scenario(),
                    PERENNIAL_CHAMBER_YEARS,
                    &frozen(),
                    true,
                    Some("perennial_chamber_state.json"),
                )
            }),
        ),
        (
            "perennial_15",
            Box::new(move || {
                season(
                    "perennial_15 (+ drift_summary)",
                    perennial_chamber_scenario(),
                    LONG_HORIZON_YEARS,
                    &frozen(),
                    true,
                    Some("perennial_long_horizon_state.json"),
                )
            }),
        ),
        (
            "consumer_5",
            Box::new(move || {
                season(
                    "consumer_5",
                    consumer_chamber_scenario(),
                    CONSUMER_CHAMBER_YEARS,
                    &frozen(),
                    true,
                    Some("consumer_chamber_state.json"),
                )
            }),
        ),
        (
            "consumer_15",
            Box::new(move || {
                season(
                    "consumer_15 (+ drift_summary)",
                    consumer_chamber_scenario(),
                    LONG_HORIZON_YEARS,
                    &frozen(),
                    true,
                    Some("consumer_long_horizon_state.json"),
                )
            }),
        ),
        (
            SQUEEZED_JAR,
            Box::new(|| {
                // Control 4's rationing case. Until 2026-10-04 it was the lab leaf form in the
                // jar, which stopped rationing at the 1/16 step (2026-09-30). Now the one
                // `tests/leaf_form.rs` measures for the same purpose: the frozen jar with its
                // room shrunk to a tenth (air and both gases), in the EXPLICIT CO₂ form — under
                // the reference's option C it does not ration, so it could prove nothing.
                let s = sealed_chamber_scenario();
                let squeezed = SeasonScenario {
                    chamber_air_capacity_mol: s.chamber_air_capacity_mol * 0.1,
                    chamber_co2_mol0: s.chamber_co2_mol0 * 0.1,
                    chamber_o2_mol0: s.chamber_o2_mol0 * 0.1,
                    ..s
                };
                let explicit = biosphere_with_co2_read(&[], Co2Read::StartOfStep)
                    .expect("the frozen params load");
                season(
                    "lab_squeezed_jar (control 4, NOT frozen)",
                    squeezed,
                    1,
                    &explicit,
                    false,
                    None,
                )
            }),
        ),
        ("crew", Box::new(crew)),
        ("eclss", Box::new(eclss)),
        ("power", Box::new(|| power(false))),
        ("power_self_discharge", Box::new(|| power(true))),
        ("thermal", Box::new(thermal)),
        ("cabin_gas", Box::new(cabin_gas)),
        ("water_recovery", Box::new(water_recovery)),
        ("station_heat_closure", Box::new(|| heat_closure(false))),
        ("sealed_energy_drift", Box::new(|| heat_closure(true))),
        ("greenhouse", Box::new(greenhouse)),
        ("lighting", Box::new(lighting)),
        ("harvest", Box::new(harvest)),
        ("sealed_station", Box::new(sealed_station)),
    ]
}

fn crew() -> Measured {
    let p = params::crew();
    let scenario = MISSION_SCENARIO;
    let (state, registry) = build_crew(&p, &scenario).expect("build_crew");
    let resolver = crew_resolver(&scenario).expect("crew_resolver");
    let steps = MISSION_DAYS * scenario.steps_per_day;
    single(
        "crew",
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        steps,
        Ok("crew_state.json"),
    )
}

fn eclss() -> Measured {
    let p = params::eclss();
    let scenario = STEADY_STATE_SCENARIO;
    let (state, registry) = build_eclss(&p, &scenario).expect("build_eclss");
    let resolver = eclss_resolver(&scenario).expect("eclss_resolver");
    single(
        "eclss",
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        STEADY_STATE_STEPS,
        Ok("eclss_state.json"),
    )
}

fn power(self_discharge: bool) -> Measured {
    let charge = params::charge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let sd = self_discharge.then(params::self_discharge);
    let (state, registry) = build_power(&charge, &scenario, sd).expect("build_power");
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let (name, days, golden) = if self_discharge {
        (
            "power_self_discharge",
            SELF_DISCHARGE_DAYS,
            "power_self_discharge_state.json",
        )
    } else {
        ("power", BOUNDED_SOC_DAYS, "power_state.json")
    };
    single(
        name,
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        days * scenario.steps_per_day,
        Ok(golden),
    )
}

fn thermal() -> Measured {
    let p = params::thermal();
    let scenario = EQUILIBRIUM_SCENARIO;
    let (state, registry) = build_thermal(&p, &scenario).expect("build_thermal");
    let resolver = thermal_resolver(&scenario).expect("thermal_resolver");
    single(
        "thermal",
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        EQUILIBRIUM_STEPS,
        Ok("thermal_state.json"),
    )
}

fn cabin_gas() -> Measured {
    let scenario = CABIN_GAS_SCENARIO;
    let (state, registry) =
        build_cabin(&params::crew(), &params::eclss(), &scenario).expect("build_cabin");
    let resolver = cabin_resolver(&scenario).expect("cabin_resolver");
    single(
        "cabin_gas",
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        CABIN_GAS_STEPS,
        Ok("cabin_gas_state.json"),
    )
}

fn water_recovery() -> Measured {
    let scenario = WATER_RECOVERY_SCENARIO;
    let (state, registry) = build_water_recovery(
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &scenario,
    )
    .expect("build_water_recovery");
    let resolver = water_recovery_resolver(&scenario).expect("water_recovery_resolver");
    single(
        "water_recovery",
        &EulerIntegrator::new(registry),
        state,
        &resolver,
        scenario.dt_seconds,
        WATER_RECOVERY_STEPS,
        Ok("water_recovery_state.json"),
    )
}

fn heat_closure(fifteen_years: bool) -> Measured {
    let charge = params::charge();
    let scenario = HEAT_CLOSURE_SCENARIO;
    let (state, registry) =
        build_station(&charge, &params::thermal(), &scenario, None).expect("build_station");
    let resolver = station_resolver(&charge, &scenario).expect("station_resolver");
    let integrator = EulerIntegrator::new(registry);
    if fifteen_years {
        single(
            "sealed_energy_drift",
            &integrator,
            state,
            &resolver,
            scenario.power.dt_seconds,
            SEALED_ENERGY_DAYS * scenario.power.steps_per_day,
            Err("sealed_energy_drift_summary.json"),
        )
    } else {
        single(
            "station_heat_closure",
            &integrator,
            state,
            &resolver,
            scenario.power.dt_seconds,
            HEAT_CLOSURE_DAYS * scenario.power.steps_per_day,
            Ok("station_state.json"),
        )
    }
}

fn greenhouse() -> Measured {
    let scenario = greenhouse_scenario();
    let (state, bio, cabin) = build_greenhouse(
        &params::crew(),
        &params::eclss(),
        &scenario,
        true,
        domains::crew::FECAL_WASTE,
    )
    .expect("build_greenhouse");
    let (bio, cabin) = (EulerIntegrator::new(bio), EulerIntegrator::new(cabin));
    let bio_r = greenhouse_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_r = greenhouse_cabin_resolver(&scenario).expect("cabin_resolver");
    let two = TwoRate {
        slow: &bio,
        fast: &cabin,
        slow_resolver: &bio_r,
        fast_resolver: &cabin_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: None,
    };
    two_rate(
        "greenhouse",
        &two,
        state,
        scenario.days,
        "greenhouse_state.json",
    )
}

fn lighting() -> Measured {
    let lamp = station_params::lamp();
    let scenario = lighting_scenario();
    let (state, bio, power) = build_lighting(&lamp, &scenario, true).expect("build_lighting");
    let (bio, power) = (EulerIntegrator::new(bio), EulerIntegrator::new(power));
    let bio_r = lighting_bio_resolver(&lamp, &scenario, true).expect("bio_resolver");
    let power_r = lighting_power_resolver(&scenario).expect("power_resolver");
    let two = TwoRate {
        slow: &bio,
        fast: &power,
        slow_resolver: &bio_r,
        fast_resolver: &power_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.power_dt,
        slow_reset: None,
    };
    two_rate(
        "lighting",
        &two,
        state,
        scenario.days,
        "lighting_state.json",
    )
}

fn harvest() -> Measured {
    let scenario = harvest_scenario();
    let (state, bio, cabin) = build_harvest(
        &params::crew(),
        &params::eclss(),
        &station_params::harvest(),
        &scenario,
        true,
        true,
    )
    .expect("build_harvest");
    let (bio, cabin) = (EulerIntegrator::new(bio), EulerIntegrator::new(cabin));
    let bio_r = harvest_bio_resolver(&scenario).expect("bio_resolver");
    let cabin_r = harvest_cabin_resolver(&scenario).expect("cabin_resolver");
    let gh = &scenario.greenhouse;
    let two = TwoRate {
        slow: &bio,
        fast: &cabin,
        slow_resolver: &bio_r,
        fast_resolver: &cabin_r,
        steps_per_day: gh.steps_per_day,
        slow_steps_per_day: gh.bio_steps_per_day,
        slow_dt: gh.bio_dt,
        fast_dt: gh.cabin_dt,
        slow_reset: None,
    };
    two_rate("harvest", &two, state, gh.days, "harvest_state.json")
}

fn sealed_station() -> Measured {
    let charge = params::charge();
    let lamp = station_params::lamp();
    let scenario = sealed_station_scenario();
    let (state, bio, fast) = build_sealed_station(
        &charge,
        &params::thermal(),
        &params::crew(),
        &params::eclss(),
        &station_params::water_recovery(),
        &lamp,
        &station_params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("build_sealed_station");
    let (bio, fast) = (EulerIntegrator::new(bio), EulerIntegrator::new(fast));
    let bio_r = sealed_bio_resolver(&lamp, &scenario).expect("sealed_bio_resolver");
    let fast_r = sealed_fast_resolver(&charge, &scenario).expect("sealed_fast_resolver");
    let reset = sealed_reset_hook(&scenario);
    let two = TwoRate {
        slow: &bio,
        fast: &fast,
        slow_resolver: &bio_r,
        fast_resolver: &fast_r,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&*reset),
    };
    two_rate(
        "sealed_station",
        &two,
        state,
        scenario.days(),
        "sealed_station_state.json",
    )
}

// ------------------------------------------------------------------------------------
// Report.
// ------------------------------------------------------------------------------------

fn print_run(m: &Measured) {
    println!("\n== {}", m.name);
    println!(
        "   control 1: {}{}",
        m.control1,
        if m.control1_ok { "" } else { "   ⚠ FAILED" }
    );
    for (side, c, rationed) in &m.sides {
        let c3 = if c.mismatched == 0 {
            "every step re-applies bit for bit".to_string()
        } else {
            format!(
                "{} steps DO NOT re-apply (first: {}); events {}",
                c.mismatched,
                c.first_mismatch.as_deref().unwrap_or("?"),
                m.events
            )
        };
        let c4 = if c.firings == *rationed {
            "agree"
        } else {
            "⚠ DISAGREE"
        };
        println!(
            "   [{side}] {} steps; control 3: {c3}; control 4: probe firings {} vs integrator {} ({c4})",
            c.steps, c.firings, rationed
        );
        let mut rows: Vec<(&String, &Row)> = c.rows.iter().collect();
        rows.sort_by(|a, b| b.1.worst.total_cmp(&a.1.worst));
        println!(
            "     {:<34} {:>9} {:>9} {:>8} {:>9} {:>8} {:>8}  worst flow",
            "store", "worst", "mean", "step", "day", ">0.1", ">0.5"
        );
        for (stock, r) in rows {
            println!(
                "     {:<34} {:>9.6} {:>9.6} {:>8} {:>9.3} {:>8} {:>8}  {}",
                stock,
                r.worst,
                r.sum / r.drawn as f64,
                r.worst_n,
                r.worst_day,
                r.over_tenth,
                r.over_half,
                r.worst_flow
            );
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let only = args
        .iter()
        .position(|a| a == "--only")
        .map(|i| args[i + 1].clone());
    let mut measured = Vec::new();
    for (key, job) in roster() {
        if only.as_deref().is_some_and(|o| !key.contains(o)) {
            continue;
        }
        let t = std::time::Instant::now();
        let m = job();
        print_run(&m);
        println!("   ({:.1} s)", t.elapsed().as_secs_f64());
        measured.push((key, m));
    }

    // Control 2: the pinned jar figure.
    let mut control2_ok = None;
    if let Some((_, jar)) = measured.iter().find(|(k, _)| *k == "sealed_jar") {
        let row = &jar.sides[0].1.rows[CARBON_POOL];
        let ok = row.worst_n == JAR_CO2_STEP && (row.worst - JAR_CO2_DRAW).abs() < 5e-7;
        control2_ok = Some(ok);
        println!(
            "\ncontrol 2: jar CO₂ worst {:.6} at step {} (pinned {JAR_CO2_DRAW} at step {JAR_CO2_STEP}) — {}",
            row.worst,
            row.worst_n,
            if ok { "reproduced" } else { "⚠ NOT reproduced" }
        );
        // The vapour store's worst step: the condenser draws the whole excess above the
        // humidity target, and the target follows the day's temperature. So a cold day in the
        // weather file, not the crop, should set it — printed, not assumed.
        let vapour = &jar.sides[0].1.rows[WATER_VAPOR];
        let (latitude, rows) = weather_facts();
        let temp = season_forcing(latitude, &rows, 1).temp;
        let d = vapour.worst_day as usize;
        println!(
            "vapour: jar worst {:.6} on day {:.2}; weather temperature (°C) days {}..={}: {:?}",
            vapour.worst,
            vapour.worst_day,
            d - 2,
            d + 1,
            &temp[d - 2..=d + 1]
        );
    }

    // The roster-wide ranking: every (run, side, store) by worst draw.
    let mut all: Vec<(&str, &str, &String, &Row)> = Vec::new();
    for (key, m) in &measured {
        for (side, c, _) in &m.sides {
            for (stock, r) in &c.rows {
                all.push((key, side, stock, r));
            }
        }
    }
    all.sort_by(|a, b| b.3.worst.total_cmp(&a.3.worst));
    println!("\n== ranking (every run, side and store; worst draw first; top 40)");
    for (key, side, stock, r) in all.iter().take(40) {
        println!(
            "   {:<22} {:<6} {:<34} worst {:>9.6}  mean {:>9.6}  day {:>9.3}  >0.5 {:>6}  {}",
            key,
            side,
            stock,
            r.worst,
            r.sum / r.drawn as f64,
            r.worst_day,
            r.over_half,
            r.worst_flow
        );
    }

    // The controls, as a verdict.
    let mut failed = Vec::new();
    for (key, m) in &measured {
        if !m.control1_ok {
            failed.push(format!("{key}: control 1"));
        }
        for (side, c, rationed) in &m.sides {
            if c.mismatched > 0 && m.events == 0 {
                failed.push(format!(
                    "{key}/{side}: control 3 with no extinction to explain it"
                ));
            }
            if c.firings != *rationed {
                failed.push(format!("{key}/{side}: control 4"));
            }
        }
    }
    match control2_ok {
        Some(false) => failed.push("sealed_jar: control 2 (the pinned jar figure)".into()),
        None if only.is_none() => {
            failed.push("sealed_jar: control 2 case is not in the roster".into())
        }
        _ => {}
    }
    match measured.iter().find(|(k, _)| *k == SQUEEZED_JAR) {
        Some((_, lab)) if lab.sides[0].2 == 0 => {
            failed.push(format!(
                "{SQUEEZED_JAR}: control 4 case does not ration — it proves nothing"
            ));
        }
        Some(_) => {}
        // A `--only` subset may leave it out; a full run that does not reach it is a failure.
        None if only.is_none() => {
            failed.push(format!(
                "{SQUEEZED_JAR}: control 4 case is not in the roster"
            ));
        }
        None => {}
    }
    println!(
        "\ncontrols: {}",
        if failed.is_empty() {
            "all held".to_string()
        } else {
            format!("⚠ FAILED — {failed:?}")
        }
    );
}
