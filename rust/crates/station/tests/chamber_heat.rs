//! The plant chamber's heat store, Step 3c slice 2b (`docs/plans/post-roadmap-room-temperature.md`
//! §23e–§23g). The lamp's whole draw — its waste heat (2b-i) and its light (2b-ii) — heats
//! `thermal.chamber`, whose cooler hands it on to the node.
//!
//! Nominally the chamber holds its steady state. This file checks the side the reference run
//! cannot show: the room **failing** (2b-i's P9, 2b-ii's Q7). With the cooler dead, the chamber
//! warms at the lamp's draw over its heat capacity and the node — losing that input — relaxes
//! toward the colder equilibrium of the remaining dissipation.
//!
//! ⚠ **Since slice 3a the plants read the chamber** (`docs/plans/post-roadmap-room-temperature.md`
//! §24h), so P9's "the crop is untouched" INVERTED, by design: it is now this file's liveness
//! for the plants reading the room — the overheated crop's leaf, stem and root fall below the
//! nominal crop's over day 1. With the walls off a dead cooler drives the chamber without bound
//! (thousands of kelvin by day 60, outside every plant formula's range), so the failed run's
//! rationing is recorded as found, not asserted zero. **The cold period is opted out**
//! (`cold.days = 0`): this file's subject is the cooler of a warm chamber.
//!
//! ⚠ **The walls (2b-iii) are switched off in every run here** (scaled by 0), so this file keeps
//! checking the cooler alone: with walls the nominal chamber follows the weather and a dead
//! cooler's chamber climbs toward its walls' equilibrium, not linearly (§23j). The walls and
//! the heater have their own file, `tests/chamber_walls.rs`.

use domains::biosphere::perturbations::{window_override, with_forcing};
use domains::biosphere::stocks::{LEAF_C, ROOT_C, STEM_C, STEM_RESERVE_C, STORAGE_C};
use domains::thermal::{equilibrium_temperature, ThermalParams, NODE};
use simcore::environment::constant;
use simcore::flow::Flow;
use simcore::integrator::EulerIntegrator;
use simcore::quantities::Quantity;
use simcore::registry::Registry;
use simcore::state::State;
use station::chamber::{chamber_temperature, CHAMBER, CHAMBER_COOLING, CHAMBER_WALL};
use station::driver::run_master_day;
use station::perturbations::ScaledFlow;
use station::scenario::{sealed_station_scenario, ColdProgram, Phase, SealedStationScenario};
use station::sealed::{
    build_sealed_station, full_lamp_heat_input_w, sealed_bio_resolver, sealed_fast_resolver,
    sealed_node_heat, sealed_reset_hook,
};
use station::water_reserve::WaterReserve;

const DAYS: usize = 60;
const COOLER_HEALTH: &str = "test.chamber_cooler_health";
/// The chamber's walls (2b-iii), switched OFF in every run here (see the module doc).
const WALL_HEALTH: &str = "test.chamber_wall_health";

/// The sealed scenario without the cold period: the chamber held warm from day 0.
///
/// ⚠ And without the station's water reserve (2026-10-20,
/// `docs/plans/post-roadmap-shared-room-humidity.md` §13): with it, the crew's tank keeps watering
/// the dead cooler's soil, so the hot room never stops evaporating within the run, and this file's
/// node bound is derived from the day evaporation stops. The reserve's side of a dead cooler is
/// its own test, [`a_dead_coolers_room_keeps_drawing_on_the_reserve`].
fn warm_scenario() -> SealedStationScenario {
    let s = sealed_station_scenario();
    SealedStationScenario {
        cold: ColdProgram { days: 0, ..s.cold },
        water_reserve: WaterReserve::Off,
        ..s
    }
}

/// Run the sealed station `DAYS` master days, the chamber's cooler at `health` throughout:
/// `(states, rationed)`.
fn run(health: f64) -> (Vec<State>, u64) {
    run_with(health, true)
}

/// [`run`], with the chamber's latent-heat books on (the reference since 2026-10-15) or off (the
/// lab switch, `station::latent_heat::without_latent_heat`).
fn run_with(health: f64, books: bool) -> (Vec<State>, u64) {
    run_in(&warm_scenario(), health, books)
}

/// [`run_with`] on `scenario`.
fn run_in(scenario: &SealedStationScenario, health: f64, books: bool) -> (Vec<State>, u64) {
    let scenario = *scenario;
    let charge = domains::params::charge();
    let lamp = station::params::lamp();
    let (state, bio_reg, fast_reg) = build_sealed_station(
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
    )
    .unwrap();
    let (state, bio_reg, fast_reg) = if books {
        (state, bio_reg, fast_reg)
    } else {
        station::latent_heat::without_latent_heat(state, bio_reg, fast_reg).unwrap()
    };
    let (flows, aux) = fast_reg.into_parts();
    let mut wrapped = 0;
    let flows: Vec<Box<dyn Flow>> = flows
        .into_iter()
        .map(|f| {
            if f.id() == CHAMBER_COOLING {
                wrapped += 1;
                Box::new(ScaledFlow::new(f, COOLER_HEALTH.to_string())) as Box<dyn Flow>
            } else if f.id() == CHAMBER_WALL {
                wrapped += 10;
                Box::new(ScaledFlow::new(f, WALL_HEALTH.to_string())) as Box<dyn Flow>
            } else {
                f
            }
        })
        .collect();
    assert_eq!(
        wrapped, 11,
        "the sealed build wires exactly one chamber cooler and one set of walls"
    );
    let fast_reg = Registry::new(flows, &state.stocks, aux).unwrap();
    let fast_resolver = with_forcing(
        sealed_fast_resolver(&charge, &domains::params::crew(), &scenario).unwrap(),
        COOLER_HEALTH,
        window_override(constant(1.0).unwrap(), 0, u64::MAX, health),
    )
    .unwrap();
    let fast_resolver = with_forcing(fast_resolver, WALL_HEALTH, constant(0.0).unwrap()).unwrap();
    let bio_resolver = sealed_bio_resolver(&lamp, &scenario).unwrap();
    let reset = sealed_reset_hook(&scenario);
    let (states, rationed, events) = run_master_day(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(fast_reg),
        state,
        &bio_resolver,
        &fast_resolver,
        DAYS,
        scenario.steps_per_day,
        scenario.bio_steps_per_day,
        scenario.bio_dt,
        scenario.cabin_dt,
        Some(&*reset),
    )
    .unwrap();
    // A working cooler's run has no extinction. A dead one's runs outside every plant formula's
    // range (module doc), and since the lit-share split's adoption (2026-10-16,
    // docs/plans/post-roadmap-lit-share-stomata.md §6d) its collapsing leaf reaches the
    // extinction snap there (day 6): recorded as found, as its rationing is — but only a crop
    // carbon pool may go, so any other extinction is still red.
    let crop = [LEAF_C, STEM_C, ROOT_C, STORAGE_C, STEM_RESERVE_C];
    if health > 0.0 {
        assert!(events.is_empty(), "health {health}, books {books}: {events:?}");
    } else {
        for e in &events {
            assert!(
                crop.contains(&e.stock.as_str()) && e.quantity == Quantity::Carbon,
                "the dead cooler's run lost something other than crop carbon: {e:?}"
            );
        }
        eprintln!("dead cooler, books {books}: {} extinctions {events:?}", events.len());
    }
    (states, rationed)
}

/// Leaf + stem + root carbon (mol).
fn veg(s: &State) -> f64 {
    s.stocks[LEAF_C].amount + s.stocks[STEM_C].amount + s.stocks[ROOT_C].amount
}

fn node_t(s: &State, th: &ThermalParams) -> f64 {
    th.space_temperature + s.stocks[NODE].amount / th.heat_capacity
}

#[test]
fn a_dead_cooler_heats_the_chamber_cools_the_node_and_overheats_the_crop() {
    let th = domains::params::thermal();
    let ch = station::params::chamber();
    let scenario = warm_scenario();
    let lamp = station::params::lamp();
    let (nominal, nominal_rationed) = run(1.0);
    let (failed, failed_rationed) = run(0.0);
    assert_eq!(nominal_rationed, 0, "the nominal run rationed");
    // §24h's prediction: maintenance doubles every 10 K and soon asks for more than the organs
    // hold, so the backstop fires. Recorded, not a target.
    eprintln!("the failed run rationed {failed_rationed} times over {DAYS} days");
    assert!(
        failed_rationed > 0,
        "a crop in a chamber thousands of kelvin hot never rationed"
    );

    // Nominal: the chamber holds its steady state on every day (P2). Since 2026-10-15 the plants'
    // evaporation takes heat from the room (`docs/plans/post-roadmap-latent-heat-books.md` §8): a
    // day end is dark and the walls are off, so the heater holds the room one step's evaporation
    // below its start (the deadbeat offset), at the rate the last window recorded.
    let t0 = chamber_temperature(nominal[0].stocks[CHAMBER].amount, &ch);
    let mut below = 0;
    for s in &nominal {
        let t = chamber_temperature(s.stocks[CHAMBER].amount, &ch);
        let step = domains::biosphere::science::LATENT_HEAT_VAPORIZATION
            * s.aux[station::latent_heat::EVAPORATION_RATE_AUX]
            * scenario.cabin_dt
            / ch.heat_capacity;
        assert!(
            (t - (t0 - step)).abs() <= 1e-9,
            "nominal chamber drifted: {t} vs {t0} less one step's evaporation {step} K"
        );
        below += usize::from(step > 1e-9);
    }
    assert!(below > 0, "no day end carried any evaporation: the offset is untested");

    // Failed: it warms at w / C_ch a day, linear from day end to day end (a day of the switched
    // draw is the average's energy; within the day it climbs only while the lamp is lit).
    let w = full_lamp_heat_input_w(&scenario);
    let per_day = w * 86_400.0 / ch.heat_capacity;
    assert!(
        (per_day - 76.8).abs() < 1e-9,
        "the hand rate 3.2 K/h (133.33 W over 1.5e5 J/K): {per_day}"
    );
    // ⚠ Since 2026-10-15 the plants' evaporation takes heat from the room too
    // (`docs/plans/post-roadmap-latent-heat-books.md` §8), and a hot room's crop evaporates hard:
    // measured, day 1 rises 28.8 K, not 76.8. The latent path is the dehumidifier's coil, which
    // the cooler's failure does not touch (the model's condenser runs on its own). So a day's rise
    // is the lamp's energy less the evaporation's — read off the ledger, λ × the water the day's
    // windows recorded — over C_ch: an energy identity, no longer a straight line. It is taken on
    // DAY 2: day 1 opens at the setpoint in the dark, where the night's evaporation draws the room
    // below it and the (healthy) heater tops it up, ~20 kJ measured; from day 1's end the room is
    // tens of kelvin above the setpoint and the heater cannot fire.
    let t1 = chamber_temperature(failed[1].stocks[CHAMBER].amount, &ch);
    let t2 = chamber_temperature(failed[2].stocks[CHAMBER].amount, &ch);
    let total = |d: usize| failed[d].aux[station::latent_heat::EVAPORATED_TOTAL_AUX];
    let latent = domains::biosphere::science::LATENT_HEAT_VAPORIZATION * (total(2) - total(1));
    let want = (w * 86_400.0 - latent) / ch.heat_capacity;
    eprintln!(
        "dead cooler: day-1 rise {} K; day 2 rise {} K, lamp {per_day} K less evaporation {} K",
        t1 - t0,
        t2 - t1,
        latent / ch.heat_capacity
    );
    assert!(t1 - t0 > 10.0, "the dead cooler's room barely warmed on day 1");
    assert!(latent > 0.0, "the hot room's crop evaporated nothing");
    assert!(
        ((t2 - t1) - want).abs() <= 1e-9 * per_day,
        "day-2 rise {} vs the lamp less the evaporation, {want}",
        t2 - t1
    );

    // The node, losing the lamp's heat, relaxes toward the colder closed-form equilibrium.
    let t_eq_nominal = node_t(&nominal[0], &th);
    let heat_now = sealed_node_heat(&domains::params::charge(), &th, &lamp, &scenario)
        / th.heat_capacity
        + th.space_temperature;
    assert_eq!(t_eq_nominal, heat_now, "the node starts at its equilibrium");
    let dissipation = th.emissivity
        * domains::thermal::STEFAN_BOLTZMANN
        * th.radiator_area
        * (t_eq_nominal.powf(4.0) - th.space_temperature.powf(4.0));
    let t_eq_failed = equilibrium_temperature(&th, dissipation - w);
    assert!((t_eq_failed - 160.308).abs() < 1e-3, "{t_eq_failed}");
    let mut last = t_eq_nominal;
    for s in failed.iter().skip(1) {
        let t = node_t(s, &th);
        assert!(
            t < last,
            "the node must fall monotonically: {t} after {last}"
        );
        last = t;
    }
    // How close, derived rather than banded: for T ≥ T_eq, T⁴ − T_eq⁴ ≥ 4·T_eq³·(T − T_eq), so
    // the gap shrinks at least as fast as `exp(−t/τ_eq)`, τ_eq = C / (4εσA·T_eq³) — the
    // relaxation time AT the target, the slowest on the way down. And it never overshoots.
    // ⚠ A fixed 0.15 K band stood here in 2b-i, sized for that slice's 7.1 K starting gap; 2b-ii
    // starts the node 14.65 K above the target and the band was not re-derived (§23h, Q7).
    let tau_eq = th.heat_capacity
        / (4.0
            * th.emissivity
            * domains::thermal::STEFAN_BOLTZMANN
            * th.radiator_area
            * t_eq_failed.powf(3.0));
    // ⚠ Since 2026-10-15 (`docs/plans/post-roadmap-latent-heat-books.md` §8) the hot room's crop
    // evaporates hard for its first days (~120 W on day 2) and that heat reaches the node by the
    // dehumidifier's coil, so the node's input is not the lamp-less constant from t = 0; the crop
    // dies and the evaporation stops (measured: none after day ~15). The bound is taken from the
    // first day after which nothing more evaporates, at the node's temperature on that day — the
    // same derivation, started where its premise (a constant input) begins to hold.
    let total = |d: usize| failed[d].aux[station::latent_heat::EVAPORATED_TOTAL_AUX];
    let dry_from = (0..DAYS)
        .find(|&d| total(d) == total(DAYS))
        .expect("the hot room's crop never stopped evaporating");
    assert!(dry_from > 1 && dry_from < DAYS / 2, "evaporation stopped at day {dry_from}");
    let gap0 = node_t(&failed[dry_from], &th) - t_eq_failed;
    let bound = gap0 * (-((DAYS - dry_from) as f64) * 86_400.0 / tau_eq).exp();
    eprintln!("dead cooler: evaporation stops by day {dry_from}; the node's gap then {gap0} K");
    assert!(
        last > t_eq_failed && last - t_eq_failed <= bound,
        "after {DAYS} days the node is {} K above {t_eq_failed}; the bound from day {dry_from} is          {bound} K",
        last - t_eq_failed
    );
    // ⚠ Since 2026-10-15 (`docs/plans/post-roadmap-latent-heat-books.md` §8) the heater pays the
    // night's evaporation and that heat reaches the node by the coil, so the node's daily input
    // is the lamp PLUS the heater, and its equilibrium creeps up as the canopy grows (measured:
    // 0.393 K off the lamp's equilibrium by day 25, past the 0.384 K swing). The derivation below
    // assumes the lamp alone, so it is held on the run with the books OFF, where that premise
    // holds; the run with them on is held to the direction and size the heater allows after it.
    let (nominal_on, nominal) = (nominal, run_with(1.0, false).0);
    // Nominal node holds — to the lamp's within-day swing, since the room's lamp heat follows
    // the light (2026-10-15, `docs/plans/post-roadmap-room-lamp-heat.md` §8c; it held to 1e-6
    // while the lamp drew its daily average). With the walls off the cooler passes the lamp's
    // heat to the node as it comes, so the node's energy strays from its daily-mean path by at
    // most the lit hours' excess, (P_on − P_avg)·T_lit, and the radiator only damps that. The
    // start (the mean equilibrium) is off that daily cycle by no more than the same, and the
    // offset decays at least as fast as `exp(−t/τ)`, τ at the nominal node — so the last two
    // day ends agree to within the bound left after DAYS − 1 days.
    let p_on = scenario.lamp_power(Phase::Warm, lamp.photon_efficacy);
    let lit_s = scenario.photoperiod_hours(Phase::Warm) as f64 * 3600.0;
    let swing = (p_on - w) * lit_s / th.heat_capacity;
    assert!(
        (swing - 0.384).abs() < 1e-3,
        "the hand swing (66.7 W × 16 h / 1e7 J/K): {swing}"
    );
    for s in &nominal {
        let t = node_t(s, &th);
        assert!(
            (t - t_eq_nominal).abs() <= swing,
            "day {}: node {t}, {} K off its equilibrium; the swing is {swing} K",
            s.n / scenario.bio_steps_per_day,
            t - t_eq_nominal
        );
    }
    let tau_nominal = th.heat_capacity
        / (4.0
            * th.emissivity
            * domains::thermal::STEFAN_BOLTZMANN
            * th.radiator_area
            * t_eq_nominal.powf(3.0));
    let settled = swing * (-((DAYS - 1) as f64) * 86_400.0 / tau_nominal).exp();
    let (last2, last1) = (node_t(&nominal[DAYS - 1], &th), node_t(&nominal[DAYS], &th));
    eprintln!(
        "nominal node: ends {last1} K ({} off equilibrium), last day moved {} K, bound {settled}",
        last1 - t_eq_nominal,
        last1 - last2
    );
    assert!(
        (last1 - last2).abs() <= settled,
        "the nominal node's last day moved {} K; the decayed bound is {settled} K",
        last1 - last2
    );
    // With the books on, the node sits above the books-off node, by no more than the night's
    // evaporation can pay for: the heater gives at most all the evaporation's heat, so the
    // equilibrium moves by at most the run's largest daily mean evaporation over the radiator's
    // conductance at the node, 4εσA·T³.
    let conductance = th.heat_capacity / tau_nominal;
    let total = |d: usize| nominal_on[d].aux[station::latent_heat::EVAPORATED_TOTAL_AUX];
    let l_max = (0..DAYS)
        .map(|d| domains::biosphere::science::LATENT_HEAT_VAPORIZATION * (total(d + 1) - total(d)))
        .fold(0.0, f64::max)
        / 86_400.0;
    let lift = node_t(&nominal_on[DAYS], &th) - last1;
    eprintln!(
        "nominal node with the books on: {lift} K above the books-off node; the largest daily          evaporation {l_max} W allows {} K",
        l_max / conductance
    );
    assert!(
        lift > 0.0 && lift <= l_max / conductance,
        "the books moved the node {lift} K; the evaporation allows (0, {}] K",
        l_max / conductance
    );

    // The crop reads the chamber (slice 3a): identical at the start, and over day 1 — the
    // chamber 76.8 K above nominal by its end — the overheated crop burns more than it fixes,
    // so its leaf + stem + root fall below the nominal crop's.
    for (id, sa) in &nominal[0].stocks {
        if sa.quantity != Quantity::Energy {
            assert_eq!(
                sa.amount.to_bits(),
                failed[0].stocks[id].amount.to_bits(),
                "{id}"
            );
        }
    }
    let (v_nominal, v_failed) = (veg(&nominal[1]), veg(&failed[1]));
    eprintln!("day 1: leaf + stem + root nominal {v_nominal}, overheated {v_failed}");
    assert!(
        v_failed < v_nominal,
        "the overheated crop's leaf + stem + root {v_failed} is not below the nominal {v_nominal}"
    );
}

/// The reserve's side of a dead cooler (2026-10-20, `docs/plans/post-roadmap-shared-room-humidity.md`
/// §13): the station keeps watering the hot room from the crew's tank, as a watering system on a
/// soil-dryness trigger would, so the room is still evaporating on its last day — where without the
/// reserve its soil dries and evaporation stops (measured: by day ~15) — and the crew's tank pays.
#[test]
fn a_dead_coolers_room_keeps_drawing_on_the_reserve() {
    let scenario = SealedStationScenario {
        water_reserve: WaterReserve::CrewStore,
        ..warm_scenario()
    };
    let (failed, _) = run_in(&scenario, 0.0, true);
    let total = |d: usize| failed[d].aux[station::latent_heat::EVAPORATED_TOTAL_AUX];
    let store = |d: usize| failed[d].stocks[domains::crew::WATER_STORE].amount;
    eprintln!(
        "dead cooler with the reserve: evaporated {} kg over {DAYS} days, {} kg on the last; \
         crew store {} → {} kg",
        total(DAYS),
        total(DAYS) - total(DAYS - 1),
        store(0),
        store(DAYS)
    );
    assert!(
        total(DAYS) > total(DAYS - 1),
        "the reserve's hot room stopped evaporating by day {DAYS}"
    );
}
