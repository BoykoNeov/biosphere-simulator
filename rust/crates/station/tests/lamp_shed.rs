//! **LAB-ONLY** — the lamp shed at a reserve, and a crop that feels it (Step 3a, option B).
//!
//! Plan: `docs/plans/post-roadmap-light-from-delivered-power.md`. WHAT-IF — would the crop
//! feel a power shortage if the lamp were shed at a reserve: 24 h of the life-support load,
//! a round number chosen, not sourced. The results here are about that assumption, not about
//! real stations.
//!
//! The controls come first: the lab wiring with nothing shed must be the plain sealed run,
//! bit for bit, or every difference below would be the wiring's.

use domains::biosphere::science::LATENT_HEAT_VAPORIZATION;
use station::latent_heat::{EVAPORATED_TOTAL_AUX, EVAPORATION_RATE_AUX};
use std::sync::OnceLock;

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, PAR_VAR, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C,
};
use domains::power::BATTERY;
use simcore::environment::{constant, SourceResolver};
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{chamber_temperature, CHAMBER, CHAMBER_HEATER, CHAMBER_SURROUNDINGS};
use station::driver::run_master_day;
use station::gas_exchange::window_key;
use station::lamp_shed::{
    rewire_for_shedding, run_shedding, what_if_reserve, with_lamp_power_cut, ShedLog,
    LAMP_DELIVERY_AUX,
};
use station::perturbations::{with_brownout, ScaledFlow};
use station::scenario::{sealed_station_scenario, ColdProgram, SealedStationScenario};
use station::sealed::{build_sealed_station, sealed_bio_resolver, sealed_fast_resolver};

const DAYS: usize = 8;
const STEPS: u64 = domains::biosphere::STEPS_PER_DAY as u64;
/// A battery small enough for a three-day blackout to reach the reserve, large enough that
/// the lamp alone does not reach it in eight days (it drains 1.152e7 J/day, unpaid — §6).
const SMALL_BATTERY: f64 = 1.5e8;
const BLACKOUT: (u64, u64) = (2 * STEPS, 5 * STEPS);

fn scenario(battery0: f64) -> SealedStationScenario {
    SealedStationScenario {
        years: 1,
        season_days: 305,
        battery0,
        ..sealed_station_scenario()
    }
}

/// [`scenario`] without the cold period (slice 3a, §24h): for the heater tests, whose subject is
/// the heater of a WARM chamber. In the cold period the walls bring heat in and the heater
/// never fires, so those tests would lose their subject (H1 trivially true, H4 drew 0 J on the
/// first run of 3a, H2's liveness compared against the warm setpoint and passed meaninglessly).
fn warm_scenario(battery0: f64) -> SealedStationScenario {
    let s = scenario(battery0);
    SealedStationScenario {
        cold: ColdProgram { days: 0, ..s.cold },
        ..s
    }
}

/// [`scenario`] with the cold period's lamp at FULL power (slice 3b, §24l): the cold setpoint
/// kept, the dimmed lamp and short day undone. For the reserve-crossing tests, whose subject is
/// shedding the full lamp — under the dim one (13 W) the small battery may never reach the
/// reserve. Not `cold.days = 0`: a warm chamber would wake the heater, which is shed with the
/// lamp (a second cause). This program is stage 1's, which reproduced 3a bit for bit, so these
/// runs are slice 3a's.
fn full_lamp_scenario(battery0: f64) -> SealedStationScenario {
    let s = scenario(battery0);
    SealedStationScenario {
        cold: ColdProgram {
            par: s.full_lamp_par(station::params::lamp().photon_efficacy),
            photoperiod_hours: s.photoperiod_hours,
            ..s.cold
        },
        ..s
    }
}

fn build(scn: &SealedStationScenario) -> (State, Registry, Registry) {
    build_sealed_station(
        &domains::params::charge(),
        &domains::params::thermal(),
        &domains::params::crew(),
        &domains::params::eclss(),
        &station::params::water_recovery(),
        &station::params::lamp(),
        &station::params::harvest(),
        scn,
        false,
        false,
    )
    .unwrap()
}

fn resolvers(scn: &SealedStationScenario, blackout: bool) -> (SourceResolver, SourceResolver) {
    let bio = sealed_bio_resolver(&station::params::lamp(), scn).unwrap();
    let mut fast = sealed_fast_resolver(&domains::params::charge(), scn).unwrap();
    if blackout {
        fast = with_brownout(fast, BLACKOUT.0, BLACKOUT.1, 0.0).unwrap();
    }
    (bio, fast)
}

struct Run {
    states: Vec<State>,
    rationed: u64,
    log: ShedLog,
}

fn plain(scn: &SealedStationScenario, fast: &SourceResolver, bio: &SourceResolver) -> Run {
    let (state, bio_reg, fast_reg) = build(scn);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio,
        fast,
        DAYS,
        scn.steps_per_day,
        scn.bio_steps_per_day,
        scn.bio_dt,
        scn.cabin_dt,
        None,
    )
    .unwrap();
    assert!(events.is_empty());
    Run {
        states,
        rationed,
        log: ShedLog::default(),
    }
}

fn lab(
    scn: &SealedStationScenario,
    fast: &SourceResolver,
    bio: &SourceResolver,
    reserve_j: f64,
) -> Run {
    let (state, bio_reg, fast_reg) = build(scn);
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(state, bio_reg, fast_reg, reserve_j).unwrap();
    let (states, rationed, events, log) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio,
        fast,
        scn,
        DAYS,
    )
    .unwrap();
    assert!(events.is_empty());
    Run {
        states,
        rationed,
        log,
    }
}

const HEATER_HEALTH: &str = "test.chamber_heater_health";

/// [`lab`] with the chamber heater scaled by `heater` (0 = a heater that gives nothing) over the
/// steps `[cut.0, cut.1)`, whole outside them. The wrapper keeps the heater's id, so the rewire
/// still finds and sheds it.
fn lab_with_heater(
    scn: &SealedStationScenario,
    fast: SourceResolver,
    bio: &SourceResolver,
    reserve_j: f64,
    heater: f64,
    cut: (u64, u64),
) -> Run {
    let (state, bio_reg, fast_reg) = build(scn);
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
    let fast = with_forcing(
        fast,
        HEATER_HEALTH,
        window_override(constant(1.0).unwrap(), cut.0, cut.1, heater),
    )
    .unwrap();
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(state, bio_reg, fast_reg, reserve_j).unwrap();
    let (states, rationed, events, log) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        bio,
        &fast,
        scn,
        DAYS,
    )
    .unwrap();
    assert!(events.is_empty());
    Run {
        states,
        rationed,
        log,
    }
}

fn chamber_t(state: &State) -> f64 {
    chamber_temperature(state.stocks[CHAMBER].amount, &station::params::chamber())
}

fn reserve(scn: &SealedStationScenario) -> f64 {
    what_if_reserve(&domains::params::charge(), scn)
}

fn crop(state: &State) -> f64 {
    [LEAF_C, STEM_C, ROOT_C, STORAGE_C]
        .iter()
        .map(|s| state.stocks[*s].amount)
        .sum()
}

/// The lab state with its bookkeeping slot removed — what the plain run carries.
fn without_slot(state: &State) -> State {
    let mut s = state.clone();
    s.aux.remove(LAMP_DELIVERY_AUX);
    s
}

/// The plain build's light is the lamp's nameplate: the PAR its plant step records is the same
/// in `a` and `b`, bit for bit, at every day's end.
fn assert_same_recorded_light(a: &Run, b: &Run) {
    let key = window_key(PAR_VAR);
    for (day, (x, y)) in a.states.iter().zip(&b.states).enumerate().skip(1) {
        assert_eq!(
            x.aux[&key].to_bits(),
            y.aux[&key].to_bits(),
            "day {day}: the plain crop's recorded light differs"
        );
    }
}

fn assert_same_run(lab: &Run, plain: &Run) {
    assert_eq!(lab.rationed, plain.rationed);
    assert_eq!(lab.states.len(), plain.states.len());
    for (day, (l, p)) in lab.states.iter().zip(&plain.states).enumerate() {
        assert_eq!(&without_slot(l), p, "day {day} differs");
    }
}

fn small_lab_baseline() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let scn = full_lamp_scenario(SMALL_BATTERY);
        let (bio, fast) = resolvers(&scn, false);
        lab(&scn, &fast, &bio, reserve(&scn))
    })
}

#[test]
fn a_run_resumed_from_a_saved_day_is_the_continuous_run_bit_for_bit() {
    // The share lives in `State` so that a saved day carries it. Split an 8-day run whose
    // lamp is cut over days 2–5 at day 5 — the last group of day 5 is the first lit one
    // after the cut, so a day-end state that held a stale share would resume dark.
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, _) = resolvers(&scn, false);
    let cut = || {
        with_lamp_power_cut(
            sealed_fast_resolver(&domains::params::charge(), &scn).unwrap(),
            BLACKOUT.0,
            BLACKOUT.1,
        )
        .unwrap()
    };
    let whole = lab(&scn, &cut(), &bio, reserve(&scn));
    // ⚠ Comparing the resumed crop alone cannot see a stale share: a day starts at midnight,
    // inside the lamp's dark hours, so the first plant step multiplies zero light. So each
    // saved day must carry the share of its own last group.
    let groups = whole.log.delivery.len() / DAYS;
    for day in 1..=DAYS {
        assert_eq!(
            whole.states[day].aux[LAMP_DELIVERY_AUX],
            whole.log.delivery[day * groups - 1],
            "the day-{day} state carries a stale share"
        );
    }
    let (_, bio_reg, fast_reg) = build(&scn);
    let saved = whole.states[5].clone();
    let (state, bio_reg, fast_reg) =
        rewire_for_shedding(saved, bio_reg, fast_reg, reserve(&scn)).unwrap();
    let (resumed, _, _, _) = run_shedding(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio,
        &cut(),
        &scn,
        DAYS - 5,
    )
    .unwrap();
    assert_eq!(resumed.last().unwrap(), whole.states.last().unwrap());
}

fn small_lab_blackout() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let scn = full_lamp_scenario(SMALL_BATTERY);
        let (bio, fast) = resolvers(&scn, true);
        lab(&scn, &fast, &bio, reserve(&scn))
    })
}

// --- controls -------------------------------------------------------------------------

#[test]
fn rule_off_the_lab_wiring_is_the_plain_sealed_run_bit_for_bit() {
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let lab = lab(&scn, &fast, &bio, f64::NEG_INFINITY);
    assert!(lab.log.delivery.iter().all(|&d| d == 1.0));
    assert_same_run(&lab, &plain(&scn, &fast, &bio));
}

#[test]
fn rule_on_with_the_frozen_battery_nothing_sheds_and_nothing_moves() {
    // The promotion prediction: the frozen battery never reaches the reserve inside this
    // horizon, so the share is the literal 1.0 throughout and the run is the plain one.
    let scn = scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let lab = lab(&scn, &fast, &bio, reserve(&scn));
    assert!(lab.log.delivery.iter().all(|&d| d == 1.0));
    assert_same_run(&lab, &plain(&scn, &fast, &bio));
}

#[test]
fn the_small_battery_alone_does_not_reach_the_reserve() {
    let run = small_lab_baseline();
    assert!(run.log.delivery.iter().all(|&d| d == 1.0));
    assert_eq!(run.rationed, 0);
    let scn = full_lamp_scenario(SMALL_BATTERY);
    assert!(run.states.last().unwrap().stocks[BATTERY].amount > reserve(&scn));
}

// --- the defect, as the reference has it --------------------------------------------------

#[test]
fn the_plain_crop_does_not_see_a_blackout() {
    // The blackout drains the battery, and the plain crop's LIGHT is the lamp's nameplate: the
    // light its plant step records is bit-identical to the run without it.
    //
    // ⚠ Until slice 3a the whole crop was bit-identical. Since the plants read the chamber
    // (§24) the plain crop FEELS the blackout through the room: the unshed lamp empties the
    // small battery on day 6, the backstop then rations the lamp's draw (5 140 times, measured on
    // the first run of 3a), less lamp heat reaches the chamber, and the crop reads a colder room
    // from day 7. That is the plants reading the chamber, not the light defect this pins.
    //
    // ⚠ Since the room's lamp heat follows the light (2026-10-15, `docs/plans/post-roadmap-room-
    // lamp-heat.md` §8c) a day-end state is dark, and the room is back at its setpoint in both
    // runs there (the last day end bit-identical, measured), so the colder room is read where it acts: in the lit
    // hours, where this full lamp and the walls outrun the cooler and warm the room by up to
    // ~4 K (measured), and the crop that grows less in the rationed room.
    let scn = full_lamp_scenario(SMALL_BATTERY);
    let (bio, calm) = resolvers(&scn, false);
    let (_, dark) = resolvers(&scn, true);
    let calm = plain(&scn, &calm, &bio);
    let dark = plain(&scn, &dark, &bio);
    let (c, d) = (calm.states.last().unwrap(), dark.states.last().unwrap());
    assert!(d.stocks[BATTERY].amount < c.stocks[BATTERY].amount);
    assert_same_recorded_light(&calm, &dark);
    assert!(dark.rationed > 0 && calm.rationed == 0);
    assert!(
        crop(d) < crop(c),
        "the crop in the rationed lamp's room grew no less: {} vs {}",
        crop(d),
        crop(c)
    );
}

// --- the lab answer ---------------------------------------------------------------------

#[test]
fn a_blackout_sheds_the_lamp_and_the_crop_feels_it() {
    let calm = small_lab_baseline();
    let dark = small_lab_blackout();
    let (c, d) = (calm.states.last().unwrap(), dark.states.last().unwrap());
    // The lamp was shed: some group delivered less than its nominal power…
    assert!(dark.log.delivery.iter().any(|&x| x < 1.0));
    // …and over the run it delivered less in all (until 2026-10-05 this read the light that
    // reached `boundary.light_used`; the sealed station's light now heats the plant chamber).
    let total = |log: &[f64]| log.iter().sum::<f64>();
    assert!(total(&dark.log.delivery) < total(&calm.log.delivery));
    // …life support was never short (the reserve held, the backstop never fired)…
    assert_eq!(dark.rationed, 0);
    // …and the crop grew less.
    assert!(crop(d) < crop(c));
}

#[test]
fn a_shed_lamp_on_the_sealed_station_never_comes_back() {
    // §6: with the lamp off, this station's solar pays exactly its life support, so the
    // battery never charges back over the reserve once the blackout has passed.
    let dark = small_lab_blackout();
    let first_off = dark.log.delivery.iter().position(|&x| x < 1.0).unwrap();
    assert!(dark.log.delivery[first_off + 1..].iter().all(|&x| x == 0.0));
}

#[test]
fn cutting_only_the_lamps_power_darkens_the_lab_crop() {
    // The lighting-failure perturbation cuts the crop's light and the lamp's draw by hand,
    // together. The lab crop reads what the lamp drew, so the draw alone is enough.
    let scn = full_lamp_scenario(sealed_station_scenario().battery0);
    let (bio, fast) = resolvers(&scn, false);
    let cut = with_lamp_power_cut(
        sealed_fast_resolver(&domains::params::charge(), &scn).unwrap(),
        BLACKOUT.0,
        BLACKOUT.1,
    )
    .unwrap();
    let lit = lab(&scn, &fast, &bio, reserve(&scn));
    let off = lab(&scn, &cut, &bio, reserve(&scn));
    assert!(off.log.delivery.contains(&0.0));
    assert!(crop(off.states.last().unwrap()) < crop(lit.states.last().unwrap()));
    // The plain build cannot see it: its crop is lit by the nameplate. (Since slice 3a it does
    // feel the cut lamp's missing HEAT, through the chamber — so the pin is on the light it
    // records, and the room was shown colder inside the cut, on day 4. Since 2026-10-15 the
    // lamp heats the room only while lit, and a day-end state is dark: both rooms sat at the
    // same temperature there, bit for bit, measured. The same day the plants' evaporation began
    // taking heat from the room (`docs/plans/post-roadmap-latent-heat-books.md` §8): the deadbeat
    // controller leaves each room one step's net input from its setpoint, and the evaporation is
    // part of that input, at each crop's own recorded rate — so the two rooms now differ by
    // exactly the difference of their evaporation steps. The colder room is in the cut's lit
    // hours, where this full lamp and the walls outrun the cooler; the crop that reads it grows
    // less.)
    let plain_cut = plain(&scn, &cut, &bio);
    let plain_lit = plain(&scn, &fast, &bio);
    assert_same_recorded_light(&plain_cut, &plain_lit);
    let ch = station::params::chamber();
    let step = |r: &Run| {
        LATENT_HEAT_VAPORIZATION * r.states[4].aux[EVAPORATION_RATE_AUX] * scn.cabin_dt
            / ch.heat_capacity
    };
    let (dt_room, dt_step) = (
        chamber_t(&plain_cut.states[4]) - chamber_t(&plain_lit.states[4]),
        step(&plain_lit) - step(&plain_cut),
    );
    assert!(
        dt_step != 0.0 && (dt_room - dt_step).abs() <= 1e-8,
        "a dark day end no longer separates the rooms by their evaporation steps: rooms {dt_room}          K apart, steps {dt_step} K"
    );
    assert!(crop(plain_cut.states.last().unwrap()) < crop(plain_lit.states.last().unwrap()));
}

#[test]
fn a_blackout_also_cuts_the_crops_water_loss() {
    // §21 of the room-temperature plan (2026-10-05): the crop's net radiation is the lamp's, so a
    // shed lamp darkens its water loss too. The plants' water is a closed loop, so less
    // transpired leaves more in the soil and less in the condensate. Before §21 the water books
    // of the two runs were bit-identical (nothing that moves water reads the canopy).
    let calm = small_lab_baseline();
    let dark = small_lab_blackout();
    let (c, d) = (calm.states.last().unwrap(), dark.states.last().unwrap());
    assert!(
        dark.log.delivery.iter().any(|&x| x < 1.0),
        "nothing was shed"
    );
    assert!(
        d.stocks[SOIL_WATER].amount > c.stocks[SOIL_WATER].amount,
        "the dark crop transpired no less: soil {} vs {}",
        d.stocks[SOIL_WATER].amount,
        c.stocks[SOIL_WATER].amount
    );
    assert!(d.stocks[CONDENSATE].amount < c.stocks[CONDENSATE].amount);
}

// --- the heater is shed with the lamp (room-temperature plan §23l) ---------------------------

#[test]
fn a_shed_heater_is_the_heaterless_run_bit_for_bit() {
    // H1: lit, the lamp holds the chamber above its setpoint, so the heater gives nothing; shed,
    // the heater goes with the lamp on the same reading; and this station never restores. So
    // the blackout run is the run whose heater is gone from the shed's group on, in every stock
    // on every day. ⚠ Until 2026-10-15 it was the run with no heater at all: the lamp's heat was
    // its daily average, so the warm chamber never needed the heater. Since the lamp heats the
    // room only while lit, the heater fires on every warm night before the shed
    // (`docs/plans/post-roadmap-room-lamp-heat.md` §8c). The shed falls inside a lit group,
    // where the heater gives nothing either way, so the cut can start at that group's step.
    let scn = warm_scenario(SMALL_BATTERY);
    let (bio, fast) = resolvers(&scn, true);
    let dark = &lab(&scn, &fast, &bio, reserve(&scn));
    let first_off = dark
        .log
        .delivery
        .iter()
        .position(|&x| x < 1.0)
        .expect("nothing was shed");
    // Group `g` follows plant step `g` (one plant step a group here), so it runs at `n = g + 1`.
    assert_eq!(dark.log.delivery.len(), DAYS * STEPS as usize);
    let shed_n = first_off as u64 + 1;
    let heaterless = lab_with_heater(&scn, fast, &bio, reserve(&scn), 0.0, (shed_n, u64::MAX));
    assert_eq!(dark.log, heaterless.log);
    assert_eq!(dark.rationed, heaterless.rationed);
    assert_eq!(dark.states.len(), heaterless.states.len());
    for (day, (d, h)) in dark.states.iter().zip(&heaterless.states).enumerate() {
        assert_eq!(d, h, "day {day} differs");
    }
    // H2, the liveness: the chamber did fall well below its setpoint, so a heater left on would
    // have drawn (3.9 MJ before §23l).
    let setpoint = station::params::chamber().warm_setpoint;
    let end = chamber_t(dark.states.last().unwrap());
    eprintln!(
        "H1/H2: dark chamber ends {end} K, {} K below the setpoint",
        setpoint - end
    );
    assert!(
        end < setpoint - 3.0,
        "chamber {end} K, setpoint {setpoint} K"
    );
}

#[test]
fn a_lamp_failure_with_a_healthy_battery_keeps_its_heater() {
    // H4: the heater is cut by the battery rule, not by the lamp being dark. With only the lamp's
    // power cut and the frozen battery (never near the reserve), the heater holds the chamber.
    let scn = warm_scenario(sealed_station_scenario().battery0);
    let (bio, _) = resolvers(&scn, false);
    let cut = || {
        with_lamp_power_cut(
            sealed_fast_resolver(&domains::params::charge(), &scn).unwrap(),
            BLACKOUT.0,
            BLACKOUT.1,
        )
        .unwrap()
    };
    // The cold run's heater is cut over days 2–4 exactly — one group later than the lamp's
    // window, which (keyed on `n`, run after its plant step) starts in day 1's last group and
    // ends a group before day 5's end. Both groups are dark, so the lamp gives nothing in them
    // either way. ⚠ Until 2026-10-15 the cold run had no heater at all; since the lamp heats the
    // room only while lit, the heater fires on every warm night, so a run-long cut would count
    // the nights too (`docs/plans/post-roadmap-room-lamp-heat.md` §8c).
    let days = (2_usize, 5_usize);
    let heater_cut = (BLACKOUT.0 + 1, BLACKOUT.1 + 1);
    let heated = lab_with_heater(&scn, cut(), &bio, reserve(&scn), 1.0, heater_cut);
    let cold = lab_with_heater(&scn, cut(), &bio, reserve(&scn), 0.0, heater_cut);
    assert!(
        heated.log.delivery.contains(&0.0),
        "the lamp never went dark"
    );
    assert!(heated
        .states
        .iter()
        .all(|s| s.stocks[BATTERY].amount > reserve(&scn)));
    let at = |r: &Run, day: usize, id: &str| r.states[day].stocks[id].amount;
    assert_eq!(
        at(&heated, days.0, BATTERY).to_bits(),
        at(&cold, days.0, BATTERY).to_bits()
    );
    let drawn = at(&cold, days.1, BATTERY) - at(&heated, days.1, BATTERY);
    // With the lamp off the heater pays the walls' loss, the plants' evaporation (since
    // 2026-10-15, `docs/plans/post-roadmap-latent-heat-books.md` §8: λ × the water the windows
    // recorded, every recorded window's heat drawn by a day end) and the room's change of store.
    let walls =
        at(&heated, days.1, CHAMBER_SURROUNDINGS) - at(&heated, days.0, CHAMBER_SURROUNDINGS);
    let stored = at(&heated, days.1, CHAMBER) - at(&heated, days.0, CHAMBER);
    let evaporated = |day: usize| heated.states[day].aux[EVAPORATED_TOTAL_AUX];
    let latent = LATENT_HEAT_VAPORIZATION * (evaporated(days.1) - evaporated(days.0));
    eprintln!(
        "H4: the heater drew {drawn} J with the lamp's power cut (walls {walls}, latent {latent},          stored {stored})"
    );
    assert!(latent > 0.0, "the plants evaporated nothing over three days");
    assert!(
        (drawn - (walls + latent + stored)).abs() <= 1e-6 * drawn,
        "the heater drew {drawn} J; the walls took {walls} J, the evaporation {latent} J and the          room stored {stored} J"
    );
    assert!(
        (drawn / 3.5e6 - 1.0).abs() < 0.2,
        "the heater drew {drawn} J, predicted 3.5e6 ±20 %"
    );
}
