//! The authored crop bed (slice 4 of `docs/plans/post-roadmap-authored-plants.md`): a `bed:`
//! builds the station's lamp-lit plant chamber into an authored habitat and runs it on the
//! station's day driver.
//!
//! The claims, each pinned below: the bed IS the reference chamber (its starting state equals
//! the sealed station's, stock for stock and value for value); a bed file runs clean and repeats;
//! the plants actually grow; every refusal the plan names fires; the re-sow calendar fires when
//! it says.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use authoring::schema::ScenarioSpec;
use authoring::yaml::parse_document;
use authoring::{
    interpret, load_scenario, run_scenario, run_scenario_allowing_rationing, BuiltScenario,
};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/scenarios/crop_bed.yaml")
}

fn fixture() -> String {
    std::fs::read_to_string(fixture_path()).expect("read crop_bed.yaml")
}

fn build(yaml: &str) -> Result<BuiltScenario, authoring::AuthoringError> {
    let doc = parse_document(yaml)?;
    let spec = ScenarioSpec::from_yaml(&doc)?;
    interpret(&spec, Path::new("."), &BTreeMap::new())
}

/// `fixture()` with `from` replaced by `to` — asserting the edit applied, so a moved line
/// cannot turn a refusal test into a test of the unedited fixture.
fn edited(from: &str, to: &str) -> String {
    let text = fixture();
    assert!(
        text.contains(from),
        "the fixture no longer contains {from:?}"
    );
    text.replacen(from, to, 1)
}

fn refused(yaml: &str, needle: &str) {
    match build(yaml) {
        Ok(_) => panic!("must be refused (expected {needle:?})"),
        Err(e) => assert!(
            e.message.contains(needle),
            "refused for the wrong reason: {} (expected {needle:?})",
            e.message
        ),
    }
}

#[test]
fn the_bed_starts_as_the_sealed_stations_own_chamber() {
    // Every stock the bed brings, and every starting value, equals the reference sealed
    // station's at step 0 — the bed is that chamber, not a likeness of it.
    let bed = load_scenario(&fixture_path(), &BTreeMap::new()).expect("the fixture builds");
    let scenario = station::scenario::sealed_station_scenario();
    let (station_state, _, _) = station::sealed::build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        &scenario,
        false,
        false,
    )
    .expect("the reference sealed station builds");
    let bed_ids = authoring::bed::bed_stock_ids(
        &authoring::bed::bed_scenario(&parse_bed(), 60.0, 4320)
            .unwrap()
            .0,
        station::chamber::ChamberSurroundings::Outdoor,
    )
    .unwrap();
    assert!(
        bed_ids.len() > 10,
        "the bed brings its plants, soil, air and heat store"
    );
    for id in bed_ids.keys() {
        let (b, s) = (&bed.state.stocks[id], &station_state.stocks[id]);
        assert_eq!(b, s, "bed stock {id:?} differs from the station's");
    }
    assert_eq!(
        bed.state.aux, station_state.aux,
        "the starting values differ from the station's"
    );
}

fn parse_bed() -> authoring::schema::BedSpec {
    let doc = parse_document(&fixture()).unwrap();
    ScenarioSpec::from_yaml(&doc).unwrap().bed.unwrap()
}

/// The final amounts by stock id, as bits.
fn final_bits(yaml: &str) -> Vec<(String, u64)> {
    let result = run_scenario(build(yaml).expect("builds")).expect("runs clean");
    assert_eq!(result.total_rationed, 0);
    assert!(result.first_reversal.is_none());
    result
        .final_state
        .stocks
        .iter()
        .map(|(id, s)| (id.clone(), s.amount.to_bits()))
        .collect()
}

#[test]
fn a_bed_file_runs_clean_repeats_and_its_plants_grow() {
    let first = final_bits(&fixture());
    assert_eq!(first, final_bits(&fixture()), "two runs ended differently");
    let built = build(&fixture()).unwrap();
    let start_leaf = built.state.stocks[domains::biosphere::stocks::LEAF_C].amount;
    let result = run_scenario(built).unwrap();
    assert_eq!(
        result.final_state.n,
        3 * 16,
        "three days of sixteen plant steps"
    );
    let end_leaf = result.final_state.stocks[domains::biosphere::stocks::LEAF_C].amount;
    assert_ne!(start_leaf, end_leaf, "the plant registry never ran");
    // The lamp drew on the author's battery: the port is wired, not decorative.
    let battery = result.final_state.stocks["power.battery"].amount;
    assert!(battery < 2.0e10, "the lamp never drew on power.battery");
}

/// The direction gate judges every state a bed's day commits, not its end (plan §13). An O₂
/// regulator on the bed's air with no crew: the plants' daytime O₂ lifts the pool past the
/// regulator's target, so it runs backwards inside the day — and by midnight, eight hours into the
/// dark (the cold program's lamp is on 08:00–16:00), the pool is back under it. Until 2026-10-18 the
/// gate looked only at midnight and passed.
#[test]
fn the_direction_gate_sees_a_midday_excursion_a_day_end_check_misses() {
    let yaml = format!(
        "{}{}",
        edited(
            "  - id: boundary.space\n",
            "  - id: boundary.o2_supply\n    domain: boundary\n    quantity: oxygen\n    kind: \
             boundary\n    amount: 0.0\n    unclamped: true\n    composition:\n      oxygen: \
             2.0\n  - id: boundary.space\n",
        ),
        "  - id: eclss.o2_makeup\n    type: eclss.o2_makeup\n    wiring:\n      o2_supply: \
         boundary.o2_supply\n      cabin_o2: biosphere.o2_pool\n    params: eclss\n    \
         cabin_air_mol: 9500.0\n",
    );
    let err = match run_scenario(build(&yaml).unwrap()) {
        Ok(_) => panic!("the regulator ran backwards and the gate did not see it"),
        Err(e) => e,
    };
    assert_eq!(err.kind, authoring::ErrorKind::Reversed, "{}", err.message);
    let result = run_scenario_allowing_rationing(build(&yaml).unwrap()).unwrap();
    let seen = result.first_reversal.expect("a reversal is recorded");
    assert_eq!(seen.stock_id, "biosphere.o2_pool");
    let minute_of_day = seen.step % 1440;
    assert!(
        (480..960).contains(&minute_of_day),
        "seen at fast step {} — inside the cold program's lit hours (08:00–16:00), not at a day's end",
        seen.step
    );
    // A day-end check would have passed: the pool is under the target at the last midnight.
    let end = result.final_state.stocks["biosphere.o2_pool"].amount;
    assert!(
        end <= seen.setpoint,
        "{end} at the last midnight vs {}",
        seen.setpoint
    );
}

#[test]
fn the_resow_calendar_fires_when_it_says() {
    // A re-sow draws its seed from the old crop's grain (`annual_reset`), so a calendar that
    // re-sows a crop with no grain yet is refused by the science, at the re-sow — not papered
    // over. Three days in, the fixture's crop has none.
    let yaml = edited("resow: maturity", "resow:\n    every_days: 1");
    let err = match run_scenario(build(&yaml).unwrap()) {
        Ok(_) => panic!("a re-sow with no seed must be refused"),
        Err(e) => e,
    };
    assert!(err.message.contains("seed bank"), "{}", err.message);
    // With a seed bank, every two days: the crop is re-sown once, at the start of day 2 — plant
    // step 2 x 16 = 32. (Every day would need seed again the next morning, and a re-sow
    // harvests the grain it draws from.)
    let yaml = edited("resow: maturity", "resow:\n    every_days: 2");
    let mut built = build(&yaml).unwrap();
    built
        .state
        .stocks
        .get_mut(domains::biosphere::stocks::STORAGE_C)
        .expect("the bed carries its grain")
        .amount = 50.0;
    let result = run_scenario(built).unwrap();
    assert_eq!(result.final_state.aux[station::sowing::SOWN_STEP], 32.0);
    // `never` leaves the first sowing standing.
    let yaml = edited("resow: maturity", "resow: never");
    let result = run_scenario(build(&yaml).unwrap()).unwrap();
    assert_eq!(result.final_state.aux[station::sowing::SOWN_STEP], 0.0);
}

/// The author declares the plant step (the user's answer): powers of two and not — 8, 12, 24, 32
/// and 48 pinned beside the fixture's 16. Not validated: the science is measured at 16 only. ⚠ 24
/// was refused until 2026-10-18 — the lamp's lit share over a 1/24-day step rounded to
/// 1.0000000000000004 and the station's guard stopped the run at the first lit step (plan §12).
/// The wholly lit window itself is pinned in `light_path`'s own tests; these three days run on the
/// cold program, whose day length the bed's `photoperiod_hours` does not set.
/// 0 is refused; a count sharing no factor with the day's 1440 fast steps is the driver's refusal.
#[test]
fn any_plant_step_the_driver_can_interleave_runs() {
    for k in [8u64, 12, 24, 32, 48] {
        let yaml = edited(
            "plant_steps_per_day: 16",
            &format!("plant_steps_per_day: {k}"),
        );
        let result = run_scenario(build(&yaml).unwrap()).expect("runs clean");
        assert_eq!(result.final_state.n, 3 * k, "three days of {k} plant steps");
        assert_eq!(result.total_rationed, 0);
    }
    refused(
        &edited("plant_steps_per_day: 16", "plant_steps_per_day: 0"),
        "at least 1",
    );
    let coprime = edited("plant_steps_per_day: 16", "plant_steps_per_day: 7");
    assert!(
        build(&coprime).and_then(run_scenario).is_err(),
        "7 plant steps share no factor with 1440 fast steps"
    );
}

#[test]
fn every_named_refusal_fires() {
    refused(&edited("crop: wheat", "crop: potato"), "not a crop");
    refused(&edited("dt: 60.0", "dt: 7.0"), "whole steps");
    refused(&edited("steps: 4320", "steps: 4000"), "whole days");
    refused(
        &edited("integrator: euler", "integrator: rk4"),
        "Euler only",
    );
    refused(&edited("steps: 4320", "steps: 4320\nn_sub: 2"), "n_sub");
    refused(
        &edited("power: power.battery", "power: no.such"),
        "no stock",
    );
    refused(
        &edited("heat_sink: thermal.node", "heat_sink: power.batteryx"),
        "no stock",
    );
    refused(
        &edited("surroundings: outdoor", "surroundings:\n    space: 0.03"),
        "radiative sink",
    );
    refused(
        &edited(
            "heat_sink: thermal.node",
            "heat_sink: thermal.node\n  space: boundary.space",
        ),
        "only by surroundings",
    );
    refused(
        &edited("resow: maturity", "resow:\n    every_days: 0"),
        ">= 1",
    );
    // A stock the bed brings may be wired to, never declared.
    refused(
        &edited(
            "  - id: boundary.space\n",
            "  - id: biosphere.carbon_pool\n    domain: x\n    quantity: carbon\n    kind: pool\n    amount: 1.0\n  - id: boundary.space\n",
        ),
        "crop bed's own",
    );
    // An author forcing may not take a name the chamber reads.
    refused(
        &format!(
            "{}\nforcings:\n  chamber_outdoor_temp:\n    const: 1.0\n",
            fixture()
        ),
        "chamber reads",
    );
}

#[test]
fn a_regulator_on_the_beds_air_must_declare_the_beds_air() {
    let with_regulator = |air: &str| {
        format!(
            "{}\n  - id: eclss.o2_makeup\n    type: eclss.o2_makeup\n    wiring:\n      \
             o2_supply: boundary.o2_supply\n      cabin_o2: biosphere.o2_pool\n    params: eclss\n    \
             cabin_air_mol: {air}\n",
            fixture().replacen(
                "  - id: boundary.space\n",
                "  - id: boundary.o2_supply\n    domain: boundary\n    quantity: oxygen\n    kind: boundary\n    amount: 0.0\n    unclamped: true\n    composition:\n      oxygen: 2.0\n  - id: boundary.space\n",
                1,
            )
        )
    };
    refused(&with_regulator("100.0"), "the bed's air");
    let built = build(&with_regulator("9500.0")).expect("the bed's own air builds");
    assert_eq!(built.demand_controlled.len(), 1);
}

/// The example greenhouse (`scenarios/lamp_greenhouse.yaml`) over 300 days: the authored bed
/// reaches maturity and re-sows itself, clean. Slow (~50 s), so `--ignored`; the content test
/// runs the file's own 14 days on every suite.
///
/// Measured 2026-10-18: rationed 0, no reversal, last sowing at plant step 4480 (day 280) — the
/// sealed station's own re-sows fall on days 139 and 278.
#[test]
#[ignore = "slow: a 300-day bed run"]
fn the_example_greenhouse_matures_and_resows_over_a_season() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../scenarios/lamp_greenhouse.yaml");
    let text = std::fs::read_to_string(path).expect("read the example");
    assert!(
        text.contains("steps: 20160 "),
        "the example's horizon line moved"
    );
    let result = run_scenario(build(&text.replace("steps: 20160 ", "steps: 432000")).unwrap())
        .expect("a season runs clean");
    assert_eq!(result.total_rationed, 0);
    assert!(result.first_reversal.is_none());
    let sown = result.final_state.aux[station::sowing::SOWN_STEP];
    assert!(
        sown >= 139.0 * 16.0,
        "the crop was never re-sown at maturity (last sowing at step {sown})"
    );
}
