//! The plant chamber's walls and heater, Step 3c slice 2b-iii
//! (`docs/plans/post-roadmap-room-temperature.md` §23i). The reference's walls face the outdoor
//! weather; the cabin, space and the station structure are lab options. In the reference the
//! lamp outweighs the walls, so the heater never fires there — this file is what makes it fire.

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::biosphere::stocks::TEMP_VAR;
use domains::power::BATTERY;
use domains::thermal::NODE;
use simcore::environment::{constant, Environment, SourceResolver};
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{
    chamber_temperature, ChamberSurroundings, CHAMBER, CHAMBER_HEATER, CHAMBER_SURROUNDINGS,
    OUTDOOR_TEMP_VAR,
};
use station::driver::run_master_day;
use station::gas_exchange::GasExchangeStep;
use station::perturbations::{with_lighting_failure, ScaledFlow};
use station::scenario::sealed_station_scenario;
use station::sealed::{
    build_sealed_station_in, chamber_heat_input_w, sealed_bio_resolver, sealed_fast_resolver,
    sealed_reset_hook,
};

const PER_DAY: u64 = domains::biosphere::STEPS_PER_DAY as u64;
const HEATER_HEALTH: &str = "test.chamber_heater_health";
/// The coldest five days of the weather file (§23i, L1): days 113–117, mean 0.04 °C.
const COLD: (u64, u64) = (113, 118);

struct Run {
    states: Vec<State>,
}

/// Run the sealed station `days` master days with the walls facing `surroundings`; the lamp
/// failed over `lamp_off` (days) if given; the heater scaled by `heater`.
fn run(
    days: usize,
    surroundings: ChamberSurroundings,
    lamp_off: Option<(u64, u64)>,
    heater: f64,
) -> Run {
    let scenario = sealed_station_scenario();
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let (state, bio_reg, fast_reg) = build_sealed_station_in(
        &charge,
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &lamp,
        &station::params::harvest(),
        &scenario,
        false,
        false,
        GasExchangeStep::Minute,
        surroundings,
    )
    .unwrap();
    let (flows, aux) = fast_reg.into_parts();
    let flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| {
            if f.id() == CHAMBER_HEATER {
                Box::new(ScaledFlow::new(f, HEATER_HEALTH.to_string())) as Box<dyn Flow>
            } else {
                f
            }
        })
        .collect();
    let fast_reg = Registry::new(flows, &state.stocks, aux).unwrap();
    let mut bio = sealed_bio_resolver(&lamp, &scenario).unwrap();
    let mut fast = with_forcing(
        sealed_fast_resolver(&charge, &scenario).unwrap(),
        HEATER_HEALTH,
        window_override(constant(1.0).unwrap(), 0, u64::MAX, heater),
    )
    .unwrap();
    if let Some((a, b)) = lamp_off {
        (bio, fast) = with_lighting_failure(bio, fast, a * PER_DAY, b * PER_DAY).unwrap();
    }
    let reset = sealed_reset_hook(&scenario);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &fast,
        days,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*reset),
    )
    .unwrap();
    assert_eq!(rationed, 0);
    assert!(events.is_empty());
    Run { states }
}

fn amount(s: &State, id: &str) -> f64 {
    s.stocks[id].amount
}

fn chamber_t(s: &State) -> f64 {
    chamber_temperature(amount(s, CHAMBER), &station::params::chamber())
}

fn node_t(s: &State) -> f64 {
    let th = domains::params::thermal();
    th.space_temperature + amount(s, NODE) / th.heat_capacity
}

fn crop_identical(a: &State, b: &State) {
    assert_eq!(a.aux, b.aux, "day {}: aux differs", a.n);
    for (id, sa) in &a.stocks {
        if sa.quantity != Quantity::Energy {
            assert_eq!(
                sa.amount.to_bits(),
                b.stocks[id].amount.to_bits(),
                "day {}: {id} moved with the heater",
                a.n
            );
        }
    }
}

/// L5 — the time base. The fast operator keeps the SLOW step count; the walls must read the
/// plants' day for the same `n`, at the first and the last plant step of a day.
#[test]
fn the_walls_read_the_plants_day_on_the_fast_step() {
    let scenario = sealed_station_scenario();
    let fast = sealed_fast_resolver(&domains::params::charge(), &scenario).unwrap();
    let bio = sealed_bio_resolver(&station::params::lamp(), &scenario).unwrap();
    let blank = |n: u64| {
        State::new(
            n,
            std::collections::BTreeMap::new(),
            0,
            std::collections::BTreeMap::new(),
        )
        .unwrap()
    };
    let read = |r: &SourceResolver, var: &str, n: u64, dt: f64| -> f64 {
        r.bind(&blank(n), dt).get(var).unwrap()
    };
    let mut changes = 0;
    for day in [0_u64, 1, 112, 113, 304, 305, 1219] {
        for n in [day * PER_DAY, day * PER_DAY + PER_DAY - 1] {
            let outdoor = read(&fast, OUTDOOR_TEMP_VAR, n, scenario.cabin_dt);
            let plants = read(&bio, TEMP_VAR, n, scenario.bio_dt);
            assert_eq!(outdoor.to_bits(), plants.to_bits(), "day {day}, n {n}");
        }
        if day > 0 {
            let before = read(
                &fast,
                OUTDOOR_TEMP_VAR,
                day * PER_DAY - 1,
                scenario.cabin_dt,
            );
            let after = read(&fast, OUTDOOR_TEMP_VAR, day * PER_DAY, scenario.cabin_dt);
            changes += u32::from(before != after);
        }
    }
    assert!(
        changes > 0,
        "no day boundary changed the temperature: the pin proves nothing"
    );
}

/// L1 — the heater fires. The lamp fails over the coldest five days; the walls draw the chamber
/// down and the heater holds it, on the battery. Control: the same failure with no heater.
#[test]
fn a_dead_lamp_on_cold_days_fires_the_heater() {
    let days = COLD.1 as usize;
    let nominal = run(days, ChamberSurroundings::Outdoor, None, 1.0);
    let heated = run(days, ChamberSurroundings::Outdoor, Some(COLD), 1.0);
    let cold = run(days, ChamberSurroundings::Outdoor, Some(COLD), 0.0);
    let end = days;
    let (n, h, c) = (&nominal.states[end], &heated.states[end], &cold.states[end]);

    // Before the window the heater never fired: the battery matches the run without a heater.
    // ⚠ The fast operator runs after its group's plant step has advanced `n`, so a lamp window
    // keyed `[113·16, 118·16)` goes dark in the LAST group of day 112 — 90 minutes before
    // `states[113]` (measured: the two runs first differ there, §23j). The last state wholly
    // before the window is `states[112]`; the day-112 end state already carries 90 heated
    // minutes, and the window ends 90 minutes before day 118's.
    let before = COLD.0 as usize;
    assert_eq!(
        amount(&heated.states[before - 1], BATTERY).to_bits(),
        amount(&cold.states[before - 1], BATTERY).to_bits()
    );
    assert_ne!(
        amount(&heated.states[before], BATTERY).to_bits(),
        amount(&cold.states[before], BATTERY).to_bits(),
        "the window no longer begins inside day 112: the lag this test documents moved"
    );
    // What it drew over the window: the two failed runs differ by the heater alone.
    let drawn = amount(c, BATTERY) - amount(h, BATTERY);
    assert!(
        (drawn / 1.4574e7 - 1.0).abs() < 1e-3,
        "the heater drew {drawn} J, predicted 1.4574e7"
    );
    // Against the nominal run: the lamp's energy not drawn, less the heater.
    let scenario = sealed_station_scenario();
    let lamp_saved = chamber_heat_input_w(&scenario) * 5.0 * 86_400.0;
    let gained = amount(h, BATTERY) - amount(n, BATTERY);
    assert!(
        (gained - (lamp_saved - drawn)).abs() <= 1e-6 * lamp_saved,
        "battery {gained} vs lamp {lamp_saved} − heater {drawn}"
    );
    assert!((gained / 4.3026e7 - 1.0).abs() < 1e-3, "{gained}");
    eprintln!("L1: heater drew {drawn} J; battery vs nominal {gained} J");

    // The chamber held: never more than one step's wall loss below the setpoint (≤ 0.0146 K).
    let set = station::params::chamber().setpoint;
    // Day-end states inside the window: `states[113..118)` (the 118th is 90 lit minutes past it).
    for s in &heated.states[before..end] {
        let t = chamber_t(s);
        assert!(
            t <= set && t >= set - 0.015,
            "day {}: chamber {t}",
            s.n / PER_DAY
        );
    }
    // Liveness: without the heater the chamber falls toward outdoors.
    assert!(
        chamber_t(c) < 280.0,
        "the unheated chamber is at {} K",
        chamber_t(c)
    );
    // The plants read no chamber temperature.
    for (a, b) in heated.states.iter().zip(&cold.states) {
        crop_identical(a, b);
    }
}

/// L2 — the cabin, away from the zero-flow point: 18 °C pulls heat out, 27 °C pushes it in, and
/// the chamber stays held either way.
#[test]
fn a_cabin_hotter_or_colder_than_the_chamber() {
    let days = 3;
    for (cabin_k, watts) in [(291.15, 6.222), (300.15, -7.593)] {
        let r = run(
            days,
            ChamberSurroundings::Cabin {
                temperature_k: cabin_k,
            },
            None,
            1.0,
        );
        let out = amount(&r.states[days], CHAMBER_SURROUNDINGS);
        let mean_w = out / (days as f64 * 86_400.0);
        assert!(
            (mean_w - watts).abs() < 2e-3,
            "cabin {cabin_k} K: {mean_w} W out, predicted {watts}"
        );
        let t = chamber_t(&r.states[days]);
        assert!((t - 295.15).abs() < 0.1, "cabin {cabin_k} K: chamber {t}");
        eprintln!("L2: cabin {cabin_k} K: {mean_w} W out, chamber {t} K");
    }
}

/// L4 — the station structure (120 days): the walls lose more than the lamp gives, so the
/// heater runs all the time; the node settles near 179.15 K and the heater near 44.8 W.
#[test]
fn walls_onto_the_station_structure_run_the_heater_continuously() {
    let days = 120;
    let r = run(days, ChamberSurroundings::Structure, None, 1.0);
    let reference = run(days, ChamberSurroundings::Outdoor, None, 1.0);
    let t = node_t(&r.states[days]);
    assert!((t - 179.151).abs() < 0.01, "node {t} K, predicted 179.151");
    // Over the last 30 days the battery falls faster than the reference's by the heater alone.
    let span = 30;
    let fall =
        |run: &Run| amount(&run.states[days - span], BATTERY) - amount(&run.states[days], BATTERY);
    let heater_w = (fall(&r) - fall(&reference)) / (span as f64 * 86_400.0);
    assert!(
        (heater_w - 44.84).abs() < 0.05,
        "heater {heater_w} W, predicted 44.84"
    );
    eprintln!("L4: node {t} K, heater {heater_w} W");
    let set = station::params::chamber().setpoint;
    let tc = chamber_t(&r.states[days]);
    assert!(tc < set && tc > set - 0.05, "chamber {tc}");
}

/// L4, the full horizon — slow (~1 min release). The battery, which nominally falls to 5.946e9 J
/// over the 1220 days, ends near 1.22e9 J with the heater on the structure: above zero, so no
/// rationing (asserted by `run`).
#[test]
#[ignore = "the full 1220-day sealed horizon; run with --ignored"]
fn the_structures_heater_does_not_empty_the_battery_within_the_horizon() {
    let days = sealed_station_scenario().days();
    let r = run(days, ChamberSurroundings::Structure, None, 1.0);
    let b = amount(r.states.last().unwrap(), BATTERY);
    eprintln!("L4 full horizon: battery ends at {b} J");
    assert!(
        b > 0.0 && (b / 1.219e9 - 1.0).abs() < 0.02,
        "battery ends at {b} J"
    );
}

/// R3 / R4 — the reference trajectory: the node's daily range and mean, the chamber's range.
#[test]
#[ignore = "the full 1220-day sealed horizon; run with --ignored"]
fn the_reference_node_and_chamber_follow_the_weather() {
    let days = sealed_station_scenario().days();
    let r = run(days, ChamberSurroundings::Outdoor, None, 1.0);
    let nodes: Vec<f64> = r.states.iter().skip(1).map(node_t).collect();
    let (lo, hi) = nodes
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    let mean = nodes.iter().sum::<f64>() / nodes.len() as f64;
    eprintln!("node daily min {lo} max {hi} mean {mean}");
    assert!(
        (lo - 172.169).abs() < 0.01 && (hi - 174.889).abs() < 0.01,
        "{lo} {hi}"
    );
    assert!((mean - 173.247).abs() < 0.01, "{mean}");
    let ch: Vec<f64> = r.states.iter().skip(1).map(chamber_t).collect();
    let (clo, chi) = ch
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    eprintln!("chamber daily min {clo} max {chi}");
    assert!(clo >= 295.1886 && chi <= 295.2035, "{clo} {chi}");
}
