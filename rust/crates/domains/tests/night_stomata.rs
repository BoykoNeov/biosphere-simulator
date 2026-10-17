//! **Stomata that close in the dark** — the lab form's own checks.
//!
//! Plan: `docs/plans/post-roadmap-night-stomata.md`. `Closing` at McAusland et al. (2023)'s booting
//! ratio is the loader's since 2026-10-15 (§7); [`NightStomata::Open`] and the heading ratio are
//! lab-only. This file asserts the rule, the loader's choice, that the open field under the
//! daily-mean weather cannot see it (its net radiation never dark — §2a; the loader's weather until
//! 2026-10-17, named since — the loader's open field now has a night), and that it reaches
//! `Transpiration` on a dark step by exactly the rule.

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::science::{
    self, NightStomata, NIGHT_CONDUCTANCE_RATIO_BOOTING, NIGHT_CONDUCTANCE_RATIO_HEADING,
};
use domains::biosphere::stocks::{LEAF_C, RN_VAR, ROOTED_DEPTH, SOIL_WATER};
use domains::biosphere::system::{build_season_with, water_air_vars, SeasonScenario, DEFAULT_SCENARIO};
use domains::biosphere::{season_setup_composed, steps_for_years, BIO_DT, STEPS_PER_DAY};
use domains::biosphere::science::AirTemperatureRead;
use domains::biosphere::system::{weather_resolver_lab, WeatherAir, WeatherRadiation};
use domains::lab::{with_air_temperature, with_night_stomata};
use simcore::environment::{Environment, SourceResolver};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

const CLOSING: [NightStomata; 2] = [
    NightStomata::Closing(NIGHT_CONDUCTANCE_RATIO_BOOTING),
    NightStomata::Closing(NIGHT_CONDUCTANCE_RATIO_HEADING),
];

fn params_with(form: NightStomata) -> BiosphereParams {
    with_night_stomata(params::biosphere(), form)
}

fn setup(s: &SeasonScenario, p: &BiosphereParams) -> (State, EulerIntegrator, SourceResolver) {
    season_setup_composed(s, 1, p, &|s, p| build_season_with(s, p)).expect("setup")
}

/// The rule: dark (`Rn ≤ 0`) divides the day resistance by the ratio, light leaves it alone, and
/// `Open` never changes it.
#[test]
fn the_rule_divides_by_the_ratio_only_in_the_dark() {
    let day = 50.0;
    for f in [
        NIGHT_CONDUCTANCE_RATIO_BOOTING,
        NIGHT_CONDUCTANCE_RATIO_HEADING,
    ] {
        let closing = NightStomata::Closing(f);
        assert_eq!(
            science::night_surface_resistance(day, 0.0, closing),
            day / f
        );
        assert_eq!(
            science::night_surface_resistance(day, -1.0, closing),
            day / f
        );
        assert_eq!(science::night_surface_resistance(day, 1e-9, closing), day);
        assert_eq!(science::night_surface_resistance(day, 300.0, closing), day);
    }
    for rn in [-1.0, 0.0, 1e-9, 300.0] {
        assert_eq!(
            science::night_surface_resistance(day, rn, NightStomata::Open),
            day
        );
    }
}

/// The loader closes the stomata in the dark at the booting ratio — adopted 2026-10-15 on the
/// user's call (plan §7). Until then this test pinned `Open` (`the_loader_keeps_the_stomata_open`).
/// The adoption moved no param file, flow or aux set, so no manifest can see it: this is its pin.
#[test]
fn the_loader_closes_the_stomata_at_the_booting_ratio() {
    assert_eq!(
        params::biosphere().transp.night_stomata,
        NightStomata::Closing(NIGHT_CONDUCTANCE_RATIO_BOOTING)
    );
}

/// P2's open-field half, under the DAILY-MEAN weather: the open season never has a dark step (its net
/// radiation is the weather's daily mean, and the file's lowest IRRAD is above zero), so closing moves
/// nothing, to the bit, on every step. ⚠ The loader's weather until 2026-10-17; since the open field's
/// night was adopted (`docs/plans/post-roadmap-open-field-night.md` §9) it is a lab form, named here.
#[test]
fn the_daily_mean_open_field_never_sees_a_dark_step() {
    let s = DEFAULT_SCENARIO;
    let daily = |p: BiosphereParams| {
        let (state0, integrator, _) =
            setup(&s, &with_air_temperature(p, AirTemperatureRead::DailyMean));
        let resolver =
            weather_resolver_lab(&s, 1, WeatherRadiation::DailyMean, WeatherAir::DailyMean)
                .expect("the daily-mean weather");
        (state0, integrator, resolver)
    };
    // The baseline is named OPEN, not taken from the loader, so the check survives an adoption.
    let (state0, integrator, resolver) = daily(params_with(NightStomata::Open));
    let mut frozen = vec![state0.clone()];
    let mut state = state0;
    for _ in 0..steps_for_years(1) {
        let env = resolver.bind(&state, BIO_DT);
        assert!(
            env.get(RN_VAR).expect("rn") > 0.0,
            "an open-field step was dark"
        );
        state = integrator
            .step_report(&state, &resolver, BIO_DT)
            .expect("step")
            .state;
        frozen.push(state.clone());
    }
    for form in CLOSING {
        let (state0, integrator, resolver) = daily(params_with(form));
        let mut state = state0;
        assert_eq!(state, frozen[0]);
        for (n, want) in frozen.iter().enumerate().skip(1) {
            state = integrator
                .step_report(&state, &resolver, BIO_DT)
                .expect("step")
                .state;
            assert_eq!(&state, want, "{form:?} moved the open field at step {n}");
        }
    }
}

/// The reach: on a dark step the flow's water is Penman–Monteith at the day resistance `/ f`, and
/// on a lit step it is the frozen flow's. The open season is grown 200 days for a canopy, then its
/// `Transpiration` is evaluated with the net radiation forced to 0 (at `Rn = 0` the soil's
/// evaporation potential is 0, so the flow's root-zone leg is the crop's alone).
#[test]
fn closing_reaches_the_flow_on_a_dark_step_by_exactly_the_rule() {
    let s = DEFAULT_SCENARIO;
    let (state0, integrator, resolver) = setup(&s, &params::biosphere());
    let mut state = state0;
    // Grown to NOON of day 200: under a weather with a real night step 0 of a day is midnight, which
    // is not the lit step the "moved a lit step" check needs (open-field-night plan §7e).
    for _ in 0..200 * STEPS_PER_DAY + STEPS_PER_DAY / 2 {
        state = integrator
            .step_report(&state, &resolver, BIO_DT)
            .expect("step")
            .state;
    }
    let root_zone_draw = |p: &BiosphereParams, dark: bool| -> f64 {
        let (_, integrator, resolver) = setup(&s, p);
        let (mut forcings, shared) = resolver.into_parts();
        if dark {
            forcings.insert(RN_VAR.to_string(), Box::new(|_, _| 0.0));
        }
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

    // The baseline is named OPEN, not taken from the loader, so the check survives an adoption.
    let frozen = params_with(NightStomata::Open);
    let tp = frozen.transp;
    let env = resolver.bind(&state, BIO_DT);
    // The names the water reads (the daily mean, or the within-day air).
    let (temp_var, vpd_var) = water_air_vars(&frozen);
    let (vpd, temp) = (env.get(vpd_var).unwrap(), env.get(temp_var).unwrap());
    let lai = science::leaf_area_index(
        state.stocks[LEAF_C].amount,
        frozen.canopy.sla_per_mol_c,
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
    let open_dark = root_zone_draw(&frozen, true);
    assert!(
        open_dark > 0.0,
        "an open-stomata crop transpires in the dark"
    );
    for form in CLOSING {
        let NightStomata::Closing(f) = form else {
            unreachable!()
        };
        let p = params_with(form);
        let want = science::penman_monteith_transpiration(0.0, vpd, temp, ra, day_rs / f)
            * f_water
            * s.ground_area
            * BIO_DT;
        let got = root_zone_draw(&p, true);
        assert!(
            ((got - want) / want).abs() < 1e-12,
            "{form:?}: dark draw {got} against the rule's {want}"
        );
        assert!(got < open_dark, "{form:?} did not lower the dark draw");
        assert_eq!(
            root_zone_draw(&p, false),
            root_zone_draw(&frozen, false),
            "{form:?} moved a lit step"
        );
        eprintln!(
            "{form:?}: LAI {lai:.2}, day rs {day_rs:.1} s/m; dark draw {:.3} of open",
            got / open_dark
        );
    }
}
