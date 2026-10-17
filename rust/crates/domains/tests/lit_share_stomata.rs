//! **Stomata that close for the dark part of a partly lit step** — the lab form's own checks.
//!
//! Plan: `docs/plans/post-roadmap-lit-share-stomata.md`. [`NightRule::LitShare`] splits a partly
//! lit step into its lit and dark time — the loader's since 2026-10-16 (§6);
//! [`NightRule::NetRadiation`] (a partly lit step counts as day) is lab-only. This file asserts the
//! form (§1), that it reaches `Transpiration` and moves only a partly lit step, that a build without
//! the `lit_fraction` forcing fails rather than falling back, that the weather's builds are today's
//! rule to the bit, and the loader's choice.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::science::{
    self, NightRule, NightStomata, NIGHT_CONDUCTANCE_RATIO_BOOTING,
    NIGHT_CONDUCTANCE_RATIO_HEADING,
};
use domains::biosphere::stocks::{
    LEAF_C, LIT_FRACTION_VAR, RN_VAR, ROOTED_DEPTH, SOIL_WATER,
};
use domains::biosphere::system::{build_season_with, water_air_vars, SeasonScenario, DEFAULT_SCENARIO};
use domains::biosphere::{season_setup_composed, BIO_DT, STEPS_PER_DAY};
use domains::biosphere::science::AirTemperatureRead;
use domains::biosphere::system::{weather_resolver_lab, WeatherAir, WeatherRadiation};
use domains::lab::{with_air_temperature, with_night_rule, with_night_stomata};
use simcore::environment::{Environment, SourceResolver};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

const CLOSING: [NightStomata; 2] = [
    NightStomata::Closing(NIGHT_CONDUCTANCE_RATIO_BOOTING),
    NightStomata::Closing(NIGHT_CONDUCTANCE_RATIO_HEADING),
];

/// A chamber-like step: 22 °C, a 0.66 kPa deficit, the sealed room's 7.5 s/m, a grown canopy's
/// 50 s/m day resistance.
const VPD: f64 = 660.0;
const TEMP: f64 = 22.0;
const RA: f64 = 7.5;
const DAY_RS: f64 = 50.0;

fn binary(rn: f64, share: f64, night: NightStomata) -> f64 {
    science::step_crop_transpiration(rn, share, None, VPD, TEMP, RA, DAY_RS, night).unwrap()
}

fn split(rn: f64, share: f64, phi: f64, night: NightStomata) -> f64 {
    science::step_crop_transpiration(rn, share, Some(phi), VPD, TEMP, RA, DAY_RS, night).unwrap()
}

/// The form's edges (§1): a fully lit step, a dark step whatever its lit share says, and `Open`
/// are the binary rule's to the bit; and the binary rule is today's `PM(s·Rn, night rs)`.
#[test]
fn only_a_partly_lit_step_is_split() {
    for night in CLOSING.into_iter().chain([NightStomata::Open]) {
        for share in [1.0, 0.63] {
            for rn in [-1.0, 0.0, 1e-9, 84.2, 300.0] {
                let today = science::penman_monteith_transpiration(
                    rn * share,
                    VPD,
                    TEMP,
                    RA,
                    science::night_surface_resistance(DAY_RS, rn, night),
                );
                assert_eq!(binary(rn, share, night).to_bits(), today.to_bits());
                if rn > 0.0 {
                    assert_eq!(split(rn, share, 1.0, night).to_bits(), today.to_bits());
                } else {
                    // A shed or failed lamp darkens the light, not the lit share: still dark.
                    for phi in [0.0, 1.0 / 3.0, 1.0] {
                        assert_eq!(split(rn, share, phi, night).to_bits(), today.to_bits());
                    }
                }
            }
            if night == NightStomata::Open {
                assert_eq!(
                    split(84.2, share, 1.0 / 3.0, night).to_bits(),
                    binary(84.2, share, night).to_bits(),
                    "Open has no dark resistance to split by"
                );
            }
        }
    }
}

/// A partly lit step is `φ · PM(s·Rn/φ, rs_day) + (1 − φ) · PM(0, rs_day/f)`, and since
/// Penman–Monteith is linear in its radiation at a fixed resistance, the split differs from the
/// binary rule by exactly the dark time's drop in the drying-air term:
/// `(1 − φ) · (PM(0, rs_day) − PM(0, rs_day/f))`. It lies between the binary rule and full closure.
#[test]
fn a_partly_lit_step_closes_for_its_dark_time() {
    for night in CLOSING {
        let NightStomata::Closing(f) = night else {
            unreachable!()
        };
        for (rn, phi) in [(84.2 / 3.0, 1.0 / 3.0), (16.8 * 2.0 / 3.0, 2.0 / 3.0)] {
            for share in [1.0, 0.63] {
                let got = split(rn, share, phi, night);
                let want = phi
                    * science::penman_monteith_transpiration(
                        rn * share / phi,
                        VPD,
                        TEMP,
                        RA,
                        DAY_RS,
                    )
                    + (1.0 - phi)
                        * science::penman_monteith_transpiration(0.0, VPD, TEMP, RA, DAY_RS / f);
                assert_eq!(got.to_bits(), want.to_bits());
                let open_dark = science::penman_monteith_transpiration(0.0, VPD, TEMP, RA, DAY_RS);
                let shut_dark =
                    science::penman_monteith_transpiration(0.0, VPD, TEMP, RA, DAY_RS / f);
                let drop = binary(rn, share, night) - got;
                let by_hand = (1.0 - phi) * (open_dark - shut_dark);
                assert!(
                    ((drop - by_hand) / by_hand).abs() < 1e-12,
                    "{night:?} φ {phi}: drop {drop} against {by_hand}"
                );
                assert!(drop > 0.0, "the dark time did not close");
            }
        }
    }
}

/// A lit share that disagrees with the light is a wiring fault: refused, never read.
#[test]
fn a_lit_share_that_disagrees_with_the_light_is_refused() {
    let night = CLOSING[0];
    for (rn, phi) in [(84.2, 0.0), (84.2, -0.1), (84.2, 1.5), (0.0, 1.5), (84.2, f64::NAN)] {
        assert!(
            science::step_crop_transpiration(rn, 1.0, Some(phi), VPD, TEMP, RA, DAY_RS, night)
                .is_err(),
            "Rn {rn}, φ {phi} was accepted"
        );
    }
}

/// The loader splits a partly lit step (plan §6, adopted 2026-10-16); until then it counted one as
/// day (§1a), which this test pinned as `the_loader_keeps_the_binary_rule`.
#[test]
fn the_loader_splits_a_partly_lit_step() {
    assert_eq!(params::biosphere().transp.night_rule, NightRule::LitShare);
}

fn setup(s: &SeasonScenario, p: &BiosphereParams) -> (State, EulerIntegrator, SourceResolver) {
    season_setup_composed(s, 1, p, &|s, p| build_season_with(s, p)).expect("setup")
}

/// A build without the lit share fails at its first step rather than quietly reading the binary
/// rule. Until the adoption the open field was such a build; the weather now supplies φ = 1 (plan
/// §6), so the forcing is taken out of its resolver here to keep the claim.
#[test]
fn a_build_without_the_lit_share_fails_under_the_split() {
    let s = DEFAULT_SCENARIO;
    let p = with_night_rule(params::biosphere(), NightRule::LitShare);
    let (state, integrator, resolver) = setup(&s, &p);
    let (mut forcings, shared) = resolver.into_parts();
    assert!(
        forcings.remove(LIT_FRACTION_VAR).is_some(),
        "the weather supplies no lit share, so nothing was taken out"
    );
    let resolver = SourceResolver::new(forcings, shared).expect("rewired");
    let err = integrator
        .step_report(&state, &resolver, BIO_DT)
        .expect_err("the open field ran without a lit share");
    assert!(
        format!("{err:?}").contains(LIT_FRACTION_VAR),
        "the failure does not name the forcing: {err:?}"
    );
}

/// The DAILY-MEAN weather's φ = 1 is the binary rule to the bit (plan §6, the staging step): the open
/// field run 30 days under the split and under the binary rule ends in the same state, every stock and
/// aux. (Its net radiation is a daily mean, > 0 on every step, so neither rule ever sees a dark step.)
/// ⚠ The loader's weather until 2026-10-17; since the open field's night was adopted it is a lab form,
/// named here (`docs/plans/post-roadmap-open-field-night.md` §9) — the loader's own is pinned in
/// `open_field_air.rs`.
#[test]
fn the_daily_mean_weather_is_wholly_lit_and_moves_nothing() {
    let s = DEFAULT_SCENARIO;
    let run = |rule: NightRule| -> State {
        let p = with_air_temperature(
            with_night_rule(params::biosphere(), rule),
            AirTemperatureRead::DailyMean,
        );
        let (mut state, integrator, _) = setup(&s, &p);
        let resolver =
            weather_resolver_lab(&s, 1, WeatherRadiation::DailyMean, WeatherAir::DailyMean)
                .expect("the daily-mean weather");
        for _ in 0..30 * STEPS_PER_DAY {
            state = integrator
                .step_report(&state, &resolver, BIO_DT)
                .expect("step")
                .state;
        }
        state
    };
    let (split, binary) = (run(NightRule::LitShare), run(NightRule::NetRadiation));
    assert!(split.stocks[SOIL_WATER].amount != run_start_water(&s), "no water moved");
    for (id, stock) in &binary.stocks {
        assert_eq!(split.stocks[id].amount.to_bits(), stock.amount.to_bits(), "{id}");
    }
    for (key, value) in &binary.aux {
        assert_eq!(split.aux[key].to_bits(), value.to_bits(), "{key}");
    }
}

fn run_start_water(s: &SeasonScenario) -> f64 {
    setup(s, &params::biosphere()).0.stocks[SOIL_WATER].amount
}

/// The reach: the open season grown 200 days for a canopy, then its `Transpiration` evaluated with
/// the light and lit share forced. A fully lit and a dark step draw what the binary rule draws, to
/// the bit; a partly lit step draws less by exactly the crop's dark-time drop (the soil's
/// evaporation reads the step's mean either way, so it cancels).
#[test]
fn the_switch_reaches_the_flow_and_moves_only_a_partly_lit_step() {
    let s = DEFAULT_SCENARIO;
    let (state0, integrator, resolver) = setup(&s, &params::biosphere());
    let mut state = state0;
    // Grown to NOON of day 200: once the weather has a night, the midnight air can sit at its dew
    // point, with no deficit for the dark-time drop to show (open-field-night plan §7e).
    for _ in 0..200 * STEPS_PER_DAY + STEPS_PER_DAY / 2 {
        state = integrator
            .step_report(&state, &resolver, BIO_DT)
            .expect("step")
            .state;
    }
    let draw = |p: &BiosphereParams, rn: f64, phi: f64| -> f64 {
        let (_, integrator, resolver) = setup(&s, p);
        let (mut forcings, shared) = resolver.into_parts();
        forcings.insert(RN_VAR.to_string(), Box::new(move |_, _| rn));
        forcings.insert(LIT_FRACTION_VAR.to_string(), Box::new(move |_, _| phi));
        let resolver = SourceResolver::new(forcings, shared).expect("rewired");
        let env = resolver.bind(&state, BIO_DT);
        let flow = integrator
            .registry()
            .flows()
            .iter()
            .find(|f| f.type_name() == "Transpiration")
            .expect("one transpiration flow");
        -flow
            .evaluate(&state, &env, BIO_DT)
            .expect("flow")
            .legs
            .iter()
            .filter(|l| l.stock == SOIL_WATER)
            .map(|l| l.amount)
            .sum::<f64>()
    };

    let env = resolver.bind(&state, BIO_DT);
    // The names the water reads (the daily mean, or the within-day air).
    let (temp_var, vpd_var) = water_air_vars(&params::biosphere());
    let (vpd, temp) = (env.get(vpd_var).unwrap(), env.get(temp_var).unwrap());
    for night in CLOSING {
        let NightStomata::Closing(f) = night else {
            unreachable!()
        };
        // Both rules named, never the loader's: once the loader takes the split, a baseline read
        // from it would compare the split with itself and pass inertly.
        let binary_p = with_night_rule(
            with_night_stomata(params::biosphere(), night),
            NightRule::NetRadiation,
        );
        let split_p = with_night_rule(binary_p.clone(), NightRule::LitShare);
        let tp = &split_p.transp;
        let lai = science::leaf_area_index(
            state.stocks[LEAF_C].amount,
            split_p.canopy.sla_per_mol_c,
            s.ground_area,
        );
        assert!(lai > 0.5, "the canopy is grown (LAI {lai})");
        let day_rs = science::canopy_surface_resistance(
            tp.rs_form,
            tp.surface_resistance,
            lai,
            tp.leaf_stomatal_resistance,
            tp.threshold_lai,
        );
        let ra = tp.aerodynamic_resistance_for(s.sealed);
        let f_water = science::soil_water_stress(
            state.stocks[SOIL_WATER].amount,
            state.aux[ROOTED_DEPTH],
            s.soil_extractable_water,
            s.ground_area,
            s.wssg,
        );
        // Fully lit, and dark (the binary rule ignores the lit share, so any value serves it).
        for (rn, phi) in [(120.0, 1.0), (0.0, 0.0)] {
            assert_eq!(
                draw(&split_p, rn, phi).to_bits(),
                draw(&binary_p, rn, phi).to_bits(),
                "{night:?}: the switch moved a step with Rn {rn}, φ {phi}"
            );
        }
        let phi = 1.0 / 3.0;
        let drop = draw(&binary_p, 40.0, phi) - draw(&split_p, 40.0, phi);
        let by_hand = (1.0 - phi)
            * (science::penman_monteith_transpiration(0.0, vpd, temp, ra, day_rs)
                - science::penman_monteith_transpiration(0.0, vpd, temp, ra, day_rs / f))
            * f_water
            * s.ground_area
            * BIO_DT;
        assert!(
            ((drop - by_hand) / by_hand).abs() < 1e-9,
            "{night:?}: the partly lit step dropped {drop}, the rule {by_hand}"
        );
    }
}
