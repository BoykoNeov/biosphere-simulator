//! The plant chamber's walls and heater, Step 3c slice 2b-iii
//! (`docs/plans/post-roadmap-room-temperature.md` §23i). The reference's walls face the outdoor
//! weather; the cabin, space and the station structure are lab options. Until slice 3a the lamp
//! outweighed the walls, so the heater never fired in the reference — this file is what made it
//! fire. Since slice 3a (§24) the reference heater fires at each warm-up after the cold period,
//! and the plants read the chamber, so L1's "the crop is identical with and without the heater"
//! INVERTED by design (§24h): it now asserts the direction the colder chamber moves the crop.

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, TEMP_VAR, THERMAL_TIME};
use domains::biosphere::system::weather_forcings;
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
    build_sealed_station_in, full_lamp_heat_input_w, sealed_bio_resolver, sealed_fast_resolver,
    sealed_reset_hook,
};

const PER_DAY: u64 = domains::biosphere::STEPS_PER_DAY as u64;
const HEATER_HEALTH: &str = "test.chamber_heater_health";
/// The coldest five days of the weather file (§23i, L1): days 113–117, mean 0.04 °C.
const COLD: (u64, u64) = (113, 118);
/// L1's heater-less run loses its heater from this day on: the lamp-failure window's first, so
/// both runs share everything before it and differ only from the window on. ⚠ Day 100 until
/// 2026-10-15, when the heater was idle between the warm-up and the window; since the room's lamp
/// heat follows the light the heater fires on warm nights, and a cut at day 100 moved the runs
/// apart 13 days early (`docs/plans/post-roadmap-room-lamp-heat.md` §8c).
const HEATER_CUT_FROM: u64 = COLD.0;

struct Run {
    states: Vec<State>,
}

/// Run the sealed station `days` master days with the walls facing `surroundings`; the lamp
/// failed over `lamp_off` (days) if given; the heater scaled by `heater` from day
/// [`HEATER_CUT_FROM`] on (whole at 1.0 before it).
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
        window_override(
            constant(1.0).unwrap(),
            // `+ 1`: the fast group after plant step `k` runs at `n = k + 1`, so the cut starts
            // with day HEATER_CUT_FROM's first group and the day-end states bracket it.
            HEATER_CUT_FROM * PER_DAY + 1,
            u64::MAX,
            heater,
        ),
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

fn veg(s: &State) -> f64 {
    amount(s, LEAF_C) + amount(s, STEM_C) + amount(s, ROOT_C)
}

/// L5 — the time base. The fast operator keeps the SLOW step count; the walls must read the
/// weather's day the plants' clock names for the same `n` (`floor(n·bio_dt)`), at the first and
/// the last plant step of a day. Since slice 3a the plants read the chamber, so the table is
/// read straight from the weather (`weather_forcings`), not through the plant resolver.
#[test]
fn the_walls_read_the_plants_day_on_the_fast_step() {
    let scenario = sealed_station_scenario();
    let fast = sealed_fast_resolver(&domains::params::charge(), &scenario).unwrap();
    let bio = SourceResolver::new(
        weather_forcings(&scenario.bio, scenario.years).unwrap(),
        std::collections::HashMap::new(),
    )
    .unwrap();
    assert!(
        sealed_bio_resolver(&station::params::lamp(), &scenario)
            .unwrap()
            .bind(
                &State::new(0, Default::default(), 0, Default::default()).unwrap(),
                1.0
            )
            .get(TEMP_VAR)
            .is_err(),
        "the plant resolver still carries a temperature"
    );
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
    // The nominal run with its heater cut from the window: what the nominal heater spends there.
    let nominal_unheated = run(days, ChamberSurroundings::Outdoor, None, 0.0);
    let heated = run(days, ChamberSurroundings::Outdoor, Some(COLD), 1.0);
    let cold = run(days, ChamberSurroundings::Outdoor, Some(COLD), 0.0);
    let end = days;
    let (n, h, c) = (&nominal.states[end], &heated.states[end], &cold.states[end]);

    // ⚠ The fast operator runs after its group's plant step has advanced `n`, so a lamp window
    // keyed `[113·16, 118·16)` goes dark in the LAST group of day 112 — 90 minutes before
    // `states[113]` (measured, §23j) — and ends 90 minutes before day 118's. Since the room's
    // lamp heat follows the light (2026-10-15, `docs/plans/post-roadmap-room-lamp-heat.md` §8c)
    // both of those groups are dark, so the lamp gives nothing in them either way: the failed
    // run is still the nominal run, bit for bit, at the end of day 112.
    let before = COLD.0 as usize;
    assert_eq!(
        heated.states[before], nominal.states[before],
        "the lamp window's first group (day 112's last) is no longer dark"
    );
    // The heater runs are cut from day 113's first group, so they share everything to its start.
    assert_eq!(
        amount(&heated.states[before], BATTERY).to_bits(),
        amount(&cold.states[before], BATTERY).to_bits()
    );
    // What it drew from the window on: the two failed runs differ by the heater alone. With the
    // lamp off the heater pays the walls' loss and whatever the room's store changed by — the
    // books of the heated run, from the last state before the window to the end. (A pin of
    // 1.4574e7 J stood here until 2026-10-15; the span now also carries the dark 90 minutes
    // after the window, which the averaged lamp used to heat.)
    let drawn = amount(c, BATTERY) - amount(h, BATTERY);
    let h0 = &heated.states[before];
    let walls = amount(h, CHAMBER_SURROUNDINGS) - amount(h0, CHAMBER_SURROUNDINGS);
    let stored = amount(h, CHAMBER) - amount(h0, CHAMBER);
    assert!(
        (drawn - (walls + stored)).abs() <= 1e-6 * drawn,
        "the heater drew {drawn} J; the walls took {walls} J and the room stored {stored} J"
    );
    // Against the nominal run: the lamp's energy not drawn, less the heater the failure added
    // over the nominal run's own — which fires on these nights too, the lamp being off at night.
    let nominal_heater = amount(&nominal_unheated.states[end], BATTERY) - amount(n, BATTERY);
    assert!(
        nominal_heater > 0.0,
        "the nominal heater never fired on the coldest nights"
    );
    let scenario = sealed_station_scenario();
    let lamp_saved = full_lamp_heat_input_w(&scenario) * 5.0 * 86_400.0;
    let gained = amount(h, BATTERY) - amount(n, BATTERY);
    assert!(
        (gained - (lamp_saved - (drawn - nominal_heater))).abs() <= 1e-6 * lamp_saved,
        "battery {gained} vs lamp {lamp_saved} − (heater {drawn} − nominal heater {nominal_heater})"
    );
    eprintln!(
        "L1: heater drew {drawn} J (walls {walls}, stored {stored}); nominal heater \
         {nominal_heater} J; battery vs nominal {gained} J"
    );

    // The chamber held: never more than one step's wall loss below the setpoint (≤ 0.0146 K).
    // Days 113–117 are in the warm phase, so the setpoint is the warm one.
    let set = station::params::chamber().warm_setpoint;
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
    // The plants read the chamber (slice 3a). Identical until the window opens — both runs
    // share the warm-up and every night's heater before day 113.
    for (a, b) in heated.states[..=before].iter().zip(&cold.states[..=before]) {
        crop_identical(a, b);
    }
    // Inverted, with directions (§24h): without the heater the plants read a colder chamber,
    // so they accrue LESS thermal time; and in the dark (no photosynthesis) they burn less in
    // maintenance, so they keep MORE leaf + stem + root than the heated run.
    let (tt_h, tt_c) = (h.aux[THERMAL_TIME], c.aux[THERMAL_TIME]);
    eprintln!(
        "L1: thermal time heated {tt_h}, unheated {tt_c}; veg {} vs {}",
        veg(h),
        veg(c)
    );
    assert!(
        tt_c < tt_h,
        "unheated thermal time {tt_c} not below heated {tt_h}"
    );
    assert!(
        veg(c) > veg(h),
        "unheated leaf + stem + root {} not above heated {}",
        veg(c),
        veg(h)
    );
}

/// L2 — the cabin, away from the zero-flow point, and the chamber held either way. Since slice
/// 3a the first three days are the COLD period (4 °C), so both cabins are warmer than the
/// chamber and both push heat IN — re-derived in closed form (§24h): the deadbeat cooler's
/// steady state `T = (T_set + (P + UA·T_cab)·τ/C) / (1 + UA·τ/C)`. With `P` the cold phase's
/// DIMMED lamp since slice 3b, 40 W × 8/24 = 13.33 W (§24l C1; 133.3 W in 3a, −21.409 /
/// −35.224 W), it gives 277.1639 K / −21.483 W against 18 °C and 277.1695 K / −35.298 W
/// against 27 °C.
#[test]
fn a_cabin_hotter_or_colder_than_the_chamber() {
    let days = 3;
    let cold_set = sealed_station_scenario().cold.setpoint;
    for (cabin_k, watts) in [(291.15, -21.483), (300.15, -35.298)] {
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
        assert!((t - cold_set).abs() < 0.1, "cabin {cabin_k} K: chamber {t}");
        eprintln!("L2: cabin {cabin_k} K: {mean_w} W out, chamber {t} K");
    }
}

/// L4 — the station structure (120 days): the walls lose more than the lamp gives over a day,
/// so the heater runs all the time; the node settles near 179.15 K and the heater near 44.8 W.
///
/// ⚠ **Re-pinned 2026-10-15** (`docs/plans/post-roadmap-room-lamp-heat.md` §8c; the room's lamp
/// heat follows the light), measured by this tree: the node **180.3016 K**, the heater
/// **51.785 W**. Direction from the change: the lit lamp (200 W) now outruns the walls by day,
/// the cooler hands that excess to the node, and the heater makes it up by night — more heater
/// energy, all of it ending in the node. Its size was not predicted.
#[test]
fn walls_onto_the_station_structure_run_the_heater_continuously() {
    let days = 120;
    let r = run(days, ChamberSurroundings::Structure, None, 1.0);
    let reference = run(days, ChamberSurroundings::Outdoor, None, 1.0);
    let t = node_t(&r.states[days]);
    // Over the last 30 days the battery falls faster than the reference's by the heater alone.
    let span = 30;
    let fall =
        |run: &Run| amount(&run.states[days - span], BATTERY) - amount(&run.states[days], BATTERY);
    let heater_w = (fall(&r) - fall(&reference)) / (span as f64 * 86_400.0);
    eprintln!("L4: node {t} K, heater {heater_w} W");
    assert!((t - 180.3016).abs() < 0.01, "node {t} K, measured 180.3016");
    assert!(
        (heater_w - 51.785).abs() < 0.05,
        "heater {heater_w} W, measured 51.785"
    );
    // A day end is dark, so the heater holds the room against the walls' whole loss to the node
    // and the deadbeat controller leaves it one step's loss below the setpoint — derived since
    // 2026-10-15 (0.0706 K here). Until then a fixed 0.05 K stood: the averaged lamp's 133 W
    // covered most of the loss in the dark, so the step's deficit was 0.018 K.
    let ch = station::params::chamber();
    let set = ch.warm_setpoint;
    let tc = chamber_t(&r.states[days]);
    let step_loss =
        ch.wall_conductance * ch.wall_area * (set - t) * sealed_station_scenario().cabin_dt
            / ch.heat_capacity;
    assert!(
        (step_loss - 0.0706).abs() < 1e-3,
        "the hand step loss (1.536 W/K × 114.85 K × 60 s / 1.5e5 J/K): {step_loss}"
    );
    assert!(
        tc < set && set - tc <= step_loss * (1.0 + 1e-3),
        "chamber {tc}, {} K below the setpoint; one step's loss is {step_loss} K",
        set - tc
    );
}

/// L4, the full horizon — slow (~1 min release). With the heater on the structure the battery
/// ends above zero, so no rationing (asserted by `run`). Re-derived for slice 3a with the
/// independent re-simulation extended to the structure (`W:\temp\claude\slice3a\
/// resim_structure.py`, run BEFORE this test): the cold chamber faces the node across 98 K
/// instead of 116 K, so the heater gives 4.2689e9 J over the horizon, not 4.7293e9, and the
/// battery ends near 5.9456e9 − 4.2689e9 = 1.6767e9 J. (Control: the same instrument without the
/// cold period gives 1.2163e9, against this test's earlier measured 1.219e9.) Slice 3b (§24l
/// C3, `W:\temp\claude\slice3b\resim.py`): the dimmed lamp saves 2.3224e9 J and the heater
/// spends it again against the cold node, so the battery ends near **1.6808e9 J** (net +4.1 MJ).
///
/// ⚠ **Re-pinned by slice 4 stage 2** (§25g; re-sow on maturity): **2.2602e9 J**, measured by
/// this tree — NOT by an independent re-simulation, as every earlier figure here was. Its
/// direction was predicted (§25f Q5: 504 dim cold days instead of 224, so less lamp drawn); its
/// size was not, and it is a pin against regression, not a check of the physics.
///
/// ⚠ **Re-pinned 2026-10-15** (`docs/plans/post-roadmap-room-lamp-heat.md` §8c; the room's lamp
/// heat follows the light), measured by this tree: **1.4834e9 J** — the heater now also runs
/// every night the lamp is off (L4 above: +6.9 W on the structure). Direction predicted, size not.
#[test]
#[ignore = "the full 1220-day sealed horizon; run with --ignored"]
fn the_structures_heater_does_not_empty_the_battery_within_the_horizon() {
    let days = sealed_station_scenario().days();
    let r = run(days, ChamberSurroundings::Structure, None, 1.0);
    let b = amount(r.states.last().unwrap(), BATTERY);
    eprintln!("L4 full horizon: battery ends at {b} J");
    assert!(
        b > 0.0 && (b / 1.4834e9 - 1.0).abs() < 0.02,
        "battery ends at {b} J"
    );
}

/// R3 / R4 — the reference trajectory: the node's daily range and mean, the chamber's range.
/// Slice 3b's §24g B4 / §24l C4 (3a's A3/A4 before): the node 163.1421 / 174.3095 / 171.44761 K
/// (min / max / mean of the day-end states; the dim lamp sends 120 W less through the cooler in
/// the cold phase); the chamber in the cold band (277.1562–277.1628 K) or the warm one (to
/// 295.2034 K) except at the transitions, and ending mid-cool-down at 288.3622829 K.
///
/// ⚠ **Re-pinned by slice 4 stage 2** (§25g; re-sow on maturity, 9 crops), measured by this tree
/// (no independent re-simulation): the node **162.2719 / 174.2496 / 169.1984 K** (colder, as §25f
/// Q6 predicted — the dim lamp of 504 cold days instead of 224); the chamber's cold band WIDER,
/// **277.1517–277.1673 K** (152 cold day-ends fell within 0.0043 K outside the old band: the cold
/// weeks now fall in every part of the weather year, where they used to start on the same day
/// of it, so the walls face different weather — not predicted); at most two transition day-ends
/// per crop (measured 9 over 9 crops); the run ends WARM, its last crop 108 days old, at
/// **295.1999296 K**.
///
/// ⚠ **Re-pinned 2026-10-15** (`docs/plans/post-roadmap-room-lamp-heat.md` §8c; the room's lamp
/// heat follows the light), measured by this tree: the node **162.3792 / 174.4180 / 169.4745 K**
/// (warmer on the mean: the cooler hands the lit lamp's excess to the node by day, and the heater
/// replaces it by night). The chamber's bands are now DERIVED, not measured: a day end is dark,
/// so the room is held at its setpoint by the heater (below, by at most one step's wall loss,
/// 0.0146 K — L1's bound) or by the cooler (above, the walls bringing heat in): measured
/// −0.0036 / +0.0112 K at the cold setpoint and −0.0146 / +0.0001 K at the warm, so ±0.015 K of
/// either setpoint. Nine day ends fall outside (the warm-ups); the run ends at **295.1466290 K**.
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
    let ch: Vec<f64> = r.states.iter().skip(1).map(chamber_t).collect();
    let (clo, chi) = ch
        .iter()
        .fold((f64::MAX, f64::MIN), |(l, h), &x| (l.min(x), h.max(x)));
    let (cold_set, warm_set) = (
        sealed_station_scenario().cold.setpoint,
        station::params::chamber().warm_setpoint,
    );
    let in_band = |t: f64| (t - cold_set).abs() <= 0.015 || (t - warm_set).abs() <= 0.015;
    let outside = ch.iter().filter(|&&t| !in_band(t)).count();
    let end = chamber_t(r.states.last().unwrap());
    eprintln!(
        "chamber daily min {clo} max {chi}; {outside} day-end states between the bands; ends {end} K"
    );
    assert!(
        (lo - 162.3792).abs() < 0.01 && (hi - 174.4180).abs() < 0.01,
        "{lo} {hi}"
    );
    assert!((mean - 169.4745).abs() < 0.01, "{mean}");
    assert!(
        clo >= cold_set - 0.015 && chi <= warm_set + 0.015,
        "{clo} {chi}"
    );
    // One mid-warm-up and one mid-cool-down day-end state per crop at most: 9 crops (§25g).
    assert!(outside <= 2 * 9, "{outside}");
    assert!((end - 295.1466290).abs() < 1e-6, "chamber ends {end} K");
}
