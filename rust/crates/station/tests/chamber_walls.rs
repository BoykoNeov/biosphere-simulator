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
use station::driver::{DayOrder, Side, TwoRate};
use station::latent_heat::CHAMBER_LATENT_HEAT;
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
    /// The heat the plants' evaporation took from the chamber up to each day-end state (J,
    /// cumulative, `latent[d]` beside `states[d]`) — the room's third outflow since 2026-10-15
    /// (`docs/plans/post-roadmap-latent-heat-books.md` §8), summed minute by minute from the
    /// latent flow's own legs.
    latent: Vec<f64>,
    /// Fast-side backstop firings (0 in every run but the structure's full horizon).
    fast_rationed: u64,
}

/// Run the sealed station `days` master days with the walls facing `surroundings`; the lamp
/// failed over `lamp_off` (days) if given; the heater scaled by `heater` from day
/// [`HEATER_CUT_FROM`] on (whole at 1.0 before it). Asserts nothing rationed.
fn run(
    days: usize,
    surroundings: ChamberSurroundings,
    lamp_off: Option<(u64, u64)>,
    heater: f64,
) -> Run {
    let r = run_rationing(days, surroundings, lamp_off, heater);
    assert_eq!(r.fast_rationed, 0);
    r
}

/// [`run`] without its rationing assertion: the structure's full horizon empties its battery
/// (§8 of the latent-heat plan) and the fast side rations from then on.
fn run_rationing(
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
        sealed_fast_resolver(&charge, &domains::params::crew(), &scenario).unwrap(),
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
    let (slow_i, fast_i) = (EulerIntegrator::new(bio_reg), EulerIntegrator::new(fast_reg));
    // The reference day, operation for operation (`driver::TwoRate`, `DayOrder::Interleaved`),
    // with an observer for the latent flow's legs.
    let two = TwoRate {
        slow: &slow_i,
        fast: &fast_i,
        slow_resolver: &bio,
        fast_resolver: &fast,
        steps_per_day: scenario.steps_per_day,
        slow_steps_per_day: scenario.bio_steps_per_day,
        slow_dt: scenario.bio_dt,
        fast_dt: scenario.cabin_dt,
        slow_reset: Some(&*reset),
    };
    let mut by_day = vec![0.0; days];
    let mut observe = |side: Side, before: &State, _after: &State| {
        if side != Side::Fast {
            return;
        }
        let flow = two
            .fast
            .registry()
            .flows()
            .iter()
            .find(|f| f.id() == CHAMBER_LATENT_HEAT)
            .expect("the latent flow");
        let r = flow
            .evaluate(before, &two.fast_resolver.bind(before, two.fast_dt), two.fast_dt)
            .unwrap();
        let heat: f64 = -r
            .legs
            .iter()
            .filter(|l| l.stock == CHAMBER)
            .map(|l| l.amount)
            .sum::<f64>();
        // The minutes after plant step `k` run at `n = k + 1`: they lie in window `k`.
        by_day[((before.n - 1) / PER_DAY) as usize] += heat;
    };
    let (states, totals) = two
        .run(DayOrder::Interleaved, state, days, &mut observe)
        .unwrap();
    assert_eq!(totals.slow_rationed, 0);
    assert!(totals.events.is_empty());
    let mut latent = vec![0.0];
    for d in by_day {
        latent.push(latent.last().unwrap() + d);
    }
    Run {
        states,
        latent,
        fast_rationed: totals.fast_rationed,
    }
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
    let fast = sealed_fast_resolver(&domains::params::charge(), &domains::params::crew(), &scenario).unwrap();
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
    // lamp off the heater pays the walls' loss, the plants' evaporation (since 2026-10-15,
    // `docs/plans/post-roadmap-latent-heat-books.md` §8) and whatever the room's store changed by
    // — the books of the heated run, from the last state before the window to the end. (A pin of
    // 1.4574e7 J stood here until 2026-10-15; the span now also carries the dark 90 minutes
    // after the window, which the averaged lamp used to heat.)
    let h0 = &heated.states[before];
    let walls = amount(h, CHAMBER_SURROUNDINGS) - amount(h0, CHAMBER_SURROUNDINGS);
    let stored = amount(h, CHAMBER) - amount(h0, CHAMBER);
    let latent = heated.latent[end] - heated.latent[before];
    let drawn = amount(c, BATTERY) - amount(h, BATTERY);
    eprintln!("L1: battery gap {drawn} J; walls {walls} J, latent {latent} J, stored {stored} J");
    assert!(
        latent > 0.0,
        "the plants evaporated nothing over five dark days"
    );
    assert!(
        (drawn - (walls + latent + stored)).abs() <= 1e-6 * drawn,
        "the heater drew {drawn} J; the walls took {walls} J, the evaporation {latent} J and the          room stored {stored} J"
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

    // The chamber held: never more than one step's loss below the setpoint — the walls' (≤ 0.0146
    // K) and, since 2026-10-15, one step of the window's evaporation at its recorded rate.
    // Days 113–117 are in the warm phase, so the setpoint is the warm one.
    let ch = station::params::chamber();
    let set = ch.warm_setpoint;
    // Day-end states inside the window: `states[113..118)` (the 118th is 90 lit minutes past it).
    for s in &heated.states[before..end] {
        let t = chamber_t(s);
        let latent_step = domains::biosphere::science::LATENT_HEAT_VAPORIZATION
            * s.aux[station::latent_heat::EVAPORATION_RATE_AUX]
            * scenario.cabin_dt
            / ch.heat_capacity;
        assert!(
            t <= set && t >= set - 0.015 - latent_step,
            "day {}: chamber {t} (one step's evaporation {latent_step} K)",
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
        // Since 2026-10-15 the plants' evaporation takes heat from the room too
        // (`docs/plans/post-roadmap-latent-heat-books.md` §8): the closed form's input `P` is the
        // lamp less the run's mean latent power. The form is linear in `P`, so the mean serves.
        let ch = station::params::chamber();
        let (ua, k) = (
            ch.wall_conductance * ch.wall_area,
            ch.response_time / ch.heat_capacity,
        );
        let latent_w = r.latent[days] / (days as f64 * 86_400.0);
        let p = 40.0 * 8.0 / 24.0 - latent_w;
        let t_pred = (cold_set + (p + ua * cabin_k) * k) / (1.0 + ua * k);
        let pred_w = ua * (t_pred - cabin_k);
        // The hand values stand for the lamp alone.
        let t_lamp = (cold_set + (40.0 * 8.0 / 24.0 + ua * cabin_k) * k) / (1.0 + ua * k);
        assert!(
            (ua * (t_lamp - cabin_k) - watts).abs() < 1e-3,
            "the closed form no longer gives the hand value {watts} W"
        );
        eprintln!("L2: cabin {cabin_k} K: latent {latent_w} W; predicted {pred_w} W out");
        assert!(
            (mean_w - pred_w).abs() < 2e-3,
            "cabin {cabin_k} K: {mean_w} W out, predicted {pred_w} (lamp alone {watts})"
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
///
/// ⚠ **Re-pinned 2026-10-15 again** (`docs/plans/post-roadmap-latent-heat-books.md` §8; the plants'
/// evaporation takes heat from the room), measured by this tree: the node **189.2987 K**, the heater
/// **136.919 W**. Direction from the change: the heater also pays the evaporation (the lamp's daytime
/// surplus now goes to it), and that heat too ends in the node by the coil. Size not predicted. The
/// day-end band gains one step of the window's evaporation.
///
/// ⚠ **Re-pinned 2026-10-16** (`docs/plans/post-roadmap-lit-share-stomata.md` §6d; stomata close for
/// the dark part of a partly lit step), measured by this tree: the node **188.7373 K** (was
/// 189.2987), the heater **139.788 W** (was 136.919). Neither size was predicted, nor the heater's
/// sign: the plants evaporate less, which lowers what the coil hands the node, but this heater
/// figure is the battery's fall LESS the outdoor reference's, so it moves with both runs — not
/// traced.
///
/// ⚠ **Re-pinned 2026-10-19** (`docs/plans/post-roadmap-co2-level.md` §10; the cabin holds 0.13 kPa
/// CO₂, 1283 ppm, where it sat at 400), measured by this tree: the node **188.7014 K** (was
/// 188.7373), the heater **139.193 W** (was 139.788). Direction: the crop grows faster at the
/// higher CO₂ and evaporates more, which moves the chamber's heat books; the lab run before the
/// adoption already showed the thermal node moving (+0.04 % on the sealed station's final day).
/// Size not predicted; the chain was not traced.
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
    assert!((t - 188.7014).abs() < 0.01, "node {t} K, measured 188.7014");
    assert!(
        (heater_w - 139.193).abs() < 0.05,
        "heater {heater_w} W, measured 139.193"
    );
    // A day end is dark, so the heater holds the room against the walls' whole loss to the node
    // and the deadbeat controller leaves it one step's loss below the setpoint — derived since
    // 2026-10-15 (0.0706 K here, then 0.0650 K once the node warmed with the evaporation's heat,
    // plus one step of the evaporation itself). Until then a fixed 0.05 K stood: the averaged
    // lamp's 133 W covered most of the loss in the dark, so the step's deficit was 0.018 K.
    let ch = station::params::chamber();
    let set = ch.warm_setpoint;
    let tc = chamber_t(&r.states[days]);
    let dt = sealed_station_scenario().cabin_dt;
    let wall_step = ch.wall_conductance * ch.wall_area * (set - t) * dt / ch.heat_capacity;
    assert!(
        (wall_step - 0.0650).abs() < 1e-3,
        "the hand step loss (1.536 W/K × 105.85 K × 60 s / 1.5e5 J/K): {wall_step}"
    );
    let step_loss = wall_step
        + domains::biosphere::science::LATENT_HEAT_VAPORIZATION
            * r.states[days].aux[station::latent_heat::EVAPORATION_RATE_AUX]
            * dt
            / ch.heat_capacity;
    // ⚠ Since the plants' evaporation takes heat from the room (2026-10-15, latent-heat plan §8)
    // the room is NOT held here: walls and evaporation together outrun the 200 W heater earlier
    // in the night (the partly lit windows' and the first dark windows' evaporation), the room
    // falls, and by the day end the heater is still climbing back on its few watts of margin.
    // Measured: 0.9526 K below the setpoint, against one step's loss of 0.0771 K.
    // ⚠ Re-stated 2026-10-16 (lit-share plan §6d): the partly lit windows' dark minutes now keep
    // their stomata shut, which is most of the evaporation that outran the heater, so at day 120's
    // end the room IS held again — measured 0.07642 K below the setpoint, inside one step's loss
    // (0.07647 K), the deadbeat offset. Pinned as measured; the horizon test below still counts
    // the nights the heater saturates.
    eprintln!("L4: chamber {} K below the setpoint; one step's loss {step_loss} K", set - tc);
    assert!(
        set - tc <= step_loss,
        "the room is no longer held at the day end: {} K below, one step's loss {step_loss} K",
        set - tc
    );
    assert!(
        ((set - tc) - 0.07642).abs() < 1e-3,
        "chamber {tc}, {} K below the setpoint; measured 0.07642 K",
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
///
/// ⚠ **RESTATED 2026-10-15 to the new truth** (`docs/plans/post-roadmap-latent-heat-books.md` §8, the
/// user's answer): since the plants' evaporation takes heat from the room, this lab room **cannot
/// afford its own transpiration** — the walls lose ~184 W to the node around the clock, the lamp's
/// daytime surplus that paid part of it now goes to evaporation, and the heater must find the
/// evaporation's heat as well (~6.3 GJ more over the horizon against a 1.48 GJ margin). The battery
/// empties inside the horizon and the fast side rations from then on. The test was
/// `the_structures_heater_does_not_empty_the_battery_within_the_horizon`; it now pins the day the
/// battery runs out, and holds the room to its band only before it.
///
/// ⚠ **Re-pinned 2026-10-16** (`docs/plans/post-roadmap-lit-share-stomata.md` §6d; a partly lit step
/// closes for its dark time), measured by this tree: the battery runs out by **day 1051** (was
/// 1025), the fast side rations 672 003 times (was 763 338), and only **8** day ends fall off the
/// held band (was 410). Direction from the change: less evaporation for the heater to pay, and
/// the partly lit windows' was most of what saturated it at night (L4 above: the room is held
/// again at day 120's end). Not predicted.
#[test]
#[ignore = "the full 1220-day sealed horizon; run with --ignored"]
fn the_structures_heater_empties_the_battery_within_the_horizon() {
    // Measured 2026-10-16: the battery runs out by day 1051, the fast side rations 672 003 times,
    // and before that 8 day ends fall off the held band — the nights the heater saturates.
    // ⚠ Re-pinned 2026-10-19 (`docs/plans/post-roadmap-co2-level.md` §10; the cabin holds 0.13 kPa
    // CO₂, where it sat at 400 ppm), measured by this tree: the battery runs out by day 1023, 774 543
    // fast-side rationings, 7 day ends off the band. Earlier, presumably because the larger crop
    // evaporates more and the heater pays that heat (the latent-heat books) — a reading, not traced.
    // Not predicted.
    const EMPTY_DAY_LO: usize = 1013;
    const EMPTY_DAY_HI: usize = 1033;
    const OFF_MAX: usize = 18;
    let days = sealed_station_scenario().days();
    let r = run_rationing(days, ChamberSurroundings::Structure, None, 1.0);
    let empty_day = r
        .states
        .iter()
        .position(|s| amount(s, BATTERY) < 1.0e6)
        .expect("the battery never ran out");
    let b = amount(r.states.last().unwrap(), BATTERY);
    eprintln!(
        "L4 full horizon: the battery runs out by day {empty_day}; ends at {b} J; fast side          rationed {} times",
        r.fast_rationed
    );
    let scenario = sealed_station_scenario();
    let ch = station::params::chamber();
    let ua = ch.wall_conductance * ch.wall_area;
    let (mut off, mut node_min) = (0, [f64::MAX; 2]);
    for s in r.states.iter().skip(1).take(empty_day.saturating_sub(1)) {
        let phase = scenario.phase(s).unwrap();
        let set = scenario.chamber_setpoint(phase, ch.warm_setpoint);
        let (tc, tn) = (chamber_t(s), node_t(s));
        let i = usize::from(phase == station::scenario::Phase::Warm);
        node_min[i] = node_min[i].min(tn);
        let latent_step = domains::biosphere::science::LATENT_HEAT_VAPORIZATION
            * s.aux[station::latent_heat::EVAPORATION_RATE_AUX]
            * scenario.cabin_dt
            / ch.heat_capacity;
        let step_loss = ua * (set - tn) * scenario.cabin_dt / ch.heat_capacity + latent_step;
        if !(tc <= set && set - tc <= step_loss * (1.0 + 1e-3)) {
            off += 1;
        }
    }
    eprintln!(
        "L4 full horizon, before the battery ran out: node min cold {} / warm {} K; {off} day ends          off the held band",
        node_min[0], node_min[1]
    );
    assert!(r.fast_rationed > 0, "the battery ran out and nothing rationed");
    assert!(
        (EMPTY_DAY_LO..=EMPTY_DAY_HI).contains(&empty_day),
        "the battery runs out by day {empty_day}"
    );
    assert!(off <= OFF_MAX, "{off} day ends off the held band before day {empty_day}");
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
///
/// ⚠ **Re-pinned 2026-10-15 again** (`docs/plans/post-roadmap-latent-heat-books.md` §8; the plants'
/// evaporation takes heat from the room), measured by this tree: the node **162.4912 / 176.3676 /
/// 170.4416 K** (warmer: the heater's extra ~1 GJ, paying the evaporation, ends in the node by the
/// coil). The band gains one step of each day end's evaporation (409 day ends fell outside the
/// walls-only band); the run ends at **295.1354351 K**, 8 days earlier in its season (each crop
/// matures a day later).
///
/// ⚠ **Re-pinned 2026-10-16** (`docs/plans/post-roadmap-lit-share-stomata.md` §6d; a partly lit step
/// closes for its dark time), measured by this tree: the node **162.4715 / 175.3180 / 170.0109 K**
/// (colder: 8.5 % less evaporation, so less of its heat reaches the node by the coil); the run
/// ends at **295.1355095 K**; 9 day ends between the bands, as before. Direction from the change,
/// size not predicted.
///
/// ⚠ **Re-pinned 2026-10-19** (`docs/plans/post-roadmap-co2-level.md` §10; the cabin holds 0.13 kPa
/// CO₂, where it sat at 400 ppm), measured by this tree: the node **162.4702 / 175.3236 / 170.0607 K**
/// (warmer on the mean by 0.05 K: the larger crop evaporates more, and that heat reaches the node by
/// the coil, the 2026-10-15 direction); the run still ends at **295.1355085 K** (was 295.1355095);
/// 9 day ends between the bands, as before. Size not predicted.
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
    // Since 2026-10-15 a day end also carries one step of the window's evaporation below the
    // setpoint (latent-heat plan §8): the band is the walls' 0.015 K plus that step, per state.
    let chamber = station::params::chamber();
    let band = |s: &State| {
        0.015
            + domains::biosphere::science::LATENT_HEAT_VAPORIZATION
                * s.aux[station::latent_heat::EVAPORATION_RATE_AUX]
                * sealed_station_scenario().cabin_dt
                / chamber.heat_capacity
    };
    let outside = r
        .states
        .iter()
        .skip(1)
        .filter(|s| {
            let t = chamber_t(s);
            (t - cold_set).abs() > band(s) && (t - warm_set).abs() > band(s)
        })
        .count();
    let end = chamber_t(r.states.last().unwrap());
    eprintln!(
        "chamber daily min {clo} max {chi}; {outside} day-end states between the bands; ends {end} K"
    );
    assert!(
        (lo - 162.4702).abs() < 0.01 && (hi - 175.3236).abs() < 0.01,
        "{lo} {hi}"
    );
    assert!((mean - 170.0607).abs() < 0.01, "{mean}");
    assert!(
        clo >= cold_set - 0.015 && chi <= warm_set + 0.015,
        "{clo} {chi}"
    );
    // One mid-warm-up and one mid-cool-down day-end state per crop at most: 9 crops (§25g).
    assert!(outside <= 2 * 9, "{outside}");
    assert!((end - 295.1355085).abs() < 1e-6, "chamber ends {end} K");
}
