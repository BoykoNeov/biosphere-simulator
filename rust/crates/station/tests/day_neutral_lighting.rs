//! The lamp-lit, warm, DAY-NEUTRAL habitat — the lamp-lit-`LightingScenario`-wiring
//! deferred by `docs/plans/post-roadmap-day-neutral-crop.md` (step 4), landed Rust-first
//! under the pivot (authored content, no golden moved → Rust; the pivot's dividing line).
//!
//! The product form the day-neutral crop was made for: a warm habitat (20 °C) lit by an
//! artificial grow lamp (no sun), growing a crop with BOTH phenology gates removed. The
//! payload is the **contrast** — under the identical warm + lamp habitat the frozen winter
//! wheat is *permanently arrested* (no cold cue ⇒ `verfun ≡ 0` ⇒ thermal time gated to 0),
//! while the day-neutral crop develops on thermal time and reaches maturity — plus the
//! Step-5 lighting guarantees (ENERGY closes, the biosphere loops close, `rationed == 0`,
//! deterministic, lamp-on-grows / lamp-off-declines).
//!
//! "Authored ≠ validated": conservation + determinism, NOT a frozen golden. The frozen
//! `lighting_scenario()` (`habitat_temp_c = None`) is byte-identical — its
//! `lighting_state.json` cross-port golden is untouched.

use domains::biosphere::science::development_stage;
use domains::biosphere::stocks::{
    CONDENSATE, LEAF_C, LITTER_CARBON, MICROBIAL_CARBON, ROOT_C, SOIL_WATER, STEM_C, STORAGE_C,
    THERMAL_TIME, WATER_VAPOR,
};
use domains::biosphere::{SeasonScenario, DEFAULT_SCENARIO};
use domains::power::{BATTERY, WASTE_HEAT};
use simcore::conservation::compute_ledger;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;
use station::flows::PAR_PHOTON_ENERGY_J_PER_UMOL;
use station::lighting::{
    build_lighting, lighting_bio_resolver, lighting_power_resolver, run_lighting, LIGHT_USED,
};
use station::scenario::{
    day_neutral_lighting_bio_scenario, day_neutral_lighting_scenario, LightingScenario,
};

const BIO_C: [&str; 6] = [
    LEAF_C,
    STEM_C,
    ROOT_C,
    STORAGE_C,
    LITTER_CARBON,
    MICROBIAL_CARBON,
];

fn run(scenario: &LightingScenario, with_lamp: bool) -> (Vec<State>, u64) {
    let lamp = station::params::lamp();
    let (state, bio_reg, power_reg) =
        build_lighting(&lamp, scenario, with_lamp).expect("build_lighting");
    let bio_resolver = lighting_bio_resolver(&lamp, scenario, with_lamp).expect("bio_resolver");
    let power_resolver = lighting_power_resolver(scenario).expect("power_resolver");
    let (states, rationed, events) = run_lighting(
        &EulerIntegrator::new(bio_reg),
        &EulerIntegrator::new(power_reg),
        state,
        &bio_resolver,
        &power_resolver,
        scenario,
    )
    .expect("run lighting");
    assert!(events.is_empty(), "lighting must be event-free");
    (states, rationed)
}

fn amt(state: &State, id: &str) -> f64 {
    state.stocks.get(id).expect("stock present").amount
}

fn bio_organic_c(state: &State) -> f64 {
    BIO_C.iter().map(|id| amt(state, id)).sum()
}

fn final_dvs(state: &State) -> f64 {
    let pheno = domains::biosphere::params::biosphere().pheno;
    let tt = state.aux.get(THERMAL_TIME).copied().unwrap_or(0.0);
    development_stage(tt, pheno.tsum_anthesis, pheno.tsum_maturity)
}

/// The frozen winter wheat (both gates ON) dropped into the SAME warm + lamp habitat — the
/// arrest arm of the contrast. Same sealed chamber, same lamp, same 20 °C, only the gates
/// differ.
fn winter_wheat_in_warm_habitat() -> LightingScenario {
    LightingScenario {
        bio: SeasonScenario {
            vernalization: true,
            photoperiod: true,
            ..day_neutral_lighting_bio_scenario()
        },
        ..day_neutral_lighting_scenario()
    }
}

#[test]
fn habitat_is_well_fed_lit_and_dark() {
    // The battery survives the 120-day lamp drain and the sealed chamber never over-draws
    // (litter_carbon0 = 3 feeds the decomposer CO₂ source) — both lit and dark.
    for with_lamp in [true, false] {
        let (_, rationed) = run(&day_neutral_lighting_scenario(), with_lamp);
        assert_eq!(
            rationed, 0,
            "day-neutral habitat must be well-fed (lamp={with_lamp})"
        );
    }
}

#[test]
fn every_day_boundary_conserves() {
    // The combined biosphere + Power ledger balances per quantity across each master day.
    let (states, _) = run(&day_neutral_lighting_scenario(), true);
    for (before, after) in states.iter().zip(states.iter().skip(1)) {
        for ql in compute_ledger(before, after).expect("ledger") {
            assert!(
                ql.residual.abs() <= 1e-6,
                "{:?} must close across each day (residual {:.2e})",
                ql.quantity,
                ql.residual
            );
        }
    }
}

#[test]
fn day_neutral_develops_where_winter_wheat_arrests() {
    // The payload: identical warm + lamp habitat, opposite phenology outcomes.
    let (day_neutral, _) = run(&day_neutral_lighting_scenario(), true);
    let (winter, _) = run(&winter_wheat_in_warm_habitat(), true);

    let final_day_neutral = day_neutral.last().expect("day boundaries");
    let final_winter = winter.last().expect("day boundaries");

    // Winter wheat: no cold cue at 20 °C ⇒ verfun ≡ 0 ⇒ thermal time never accrues ⇒ DVS
    // pinned at 0. Permanent arrest, not a slowdown (a real deployment failure).
    assert_eq!(
        final_winter.aux.get(THERMAL_TIME).copied().unwrap_or(0.0),
        0.0,
        "winter wheat must accrue NO thermal time in a warm habitat"
    );
    assert_eq!(
        final_dvs(final_winter),
        0.0,
        "winter wheat must stay at DVS 0"
    );

    // Day-neutral: thermal time advances at the plain degree-day rate ⇒ the crop reaches
    // maturity within the horizon.
    assert!(
        final_dvs(final_day_neutral) >= 2.0,
        "day-neutral crop must mature (DVS {} >= 2.0)",
        final_dvs(final_day_neutral)
    );
}

#[test]
fn lamp_on_grows_lamp_off_declines() {
    // The signed "it bit" gate: the lamp carries the ENERGY that drives carbon fixation.
    // Lit ⇒ the seedling net-assimilates (biosphere organic carbon rises); dark (PAR = 0)
    // ⇒ it only respires (organic carbon falls). Same crop, same warmth — only PAR differs.
    let scenario = day_neutral_lighting_scenario();
    let (lit, _) = run(&scenario, true);
    let (dark, _) = run(&scenario, false);

    let lit0 = bio_organic_c(lit.first().unwrap());
    let litf = bio_organic_c(lit.last().unwrap());
    let dark0 = bio_organic_c(dark.first().unwrap());
    let darkf = bio_organic_c(dark.last().unwrap());

    assert!(
        litf > lit0,
        "the lit crop must fix net carbon ({lit0} -> {litf})"
    );
    assert!(
        darkf < dark0,
        "the unlit crop must lose carbon ({dark0} -> {darkf})"
    );
    assert!(
        litf > darkf,
        "the lamp must make the crop a net sink vs the dark baseline"
    );
}

#[test]
fn energy_closes_and_battery_stays_well_fed() {
    // ENERGY closure, quantitatively: the battery loses exactly the lamp's daily energy ×
    // days; light_used / waste_heat accumulate the η-split; and the battery stays well
    // above 0 (rationed == 0 is the structural proof, this is the magnitude).
    let scenario = day_neutral_lighting_scenario();
    let lamp = station::params::lamp();
    let (states, _) = run(&scenario, true);
    let (initial, final_) = (states.first().unwrap(), states.last().unwrap());

    let daily = scenario.lamp_power_w * scenario.photoperiod_hours as f64 * 3600.0;
    let drawn = daily * scenario.days as f64;
    let drained = amt(initial, BATTERY) - amt(final_, BATTERY);
    assert!(
        (drained - drawn).abs() <= drawn * 1e-9,
        "battery must drain by the lamp energy"
    );

    let eta = lamp.photon_efficacy * PAR_PHOTON_ENERGY_J_PER_UMOL;
    assert!((amt(final_, LIGHT_USED) - eta * drawn).abs() <= drawn * 1e-9);
    assert!((amt(final_, WASTE_HEAT) - (1.0 - eta) * drawn).abs() <= drawn * 1e-9);
    // Well-fed with real margin (the Python precedent's bar), not merely non-zero: the
    // battery ends above half its initial charge over the 120-day development horizon.
    assert!(
        amt(final_, BATTERY) > 0.5 * amt(initial, BATTERY),
        "the battery must stay well-fed (> half initial)"
    );
}

#[test]
fn biosphere_internal_water_loop_closed() {
    // The lamp couples Power to the biosphere's PAR only — the internal water ring
    // (soil_water -> water_vapor -> condensate -> soil_water) stays closed across the run.
    let (states, _) = run(&day_neutral_lighting_scenario(), true);
    let loop_ids = [SOIL_WATER, WATER_VAPOR, CONDENSATE];
    let total = |s: &State| loop_ids.iter().map(|id| amt(s, id)).sum::<f64>();
    let drift = total(states.last().unwrap()) - total(states.first().unwrap());
    assert!(
        drift.abs() <= 1e-9,
        "internal water loop must stay closed (drift {drift:.2e})"
    );
}

#[test]
fn deterministic() {
    // Bit-identical re-run: the authored habitat is deterministic (every stock, to bits).
    let (a, _) = run(&day_neutral_lighting_scenario(), true);
    let (b, _) = run(&day_neutral_lighting_scenario(), true);
    let (af, bf) = (a.last().unwrap(), b.last().unwrap());
    assert_eq!(af.stocks.len(), bf.stocks.len());
    for (id, stock) in &af.stocks {
        assert_eq!(
            stock.amount.to_bits(),
            bf.stocks.get(id).unwrap().amount.to_bits(),
            "stock {id} must be bit-identical across runs"
        );
    }
}

#[test]
fn frozen_lighting_scenario_keeps_weather_temperature() {
    // The additive-ness guard: the frozen lighting scenario opts OUT of the temperature
    // override (None), so its weather-driven temperature — and its cross-port golden — are
    // untouched. Only the authored day-neutral habitat sets a constant habitat temp.
    assert_eq!(station::scenario::lighting_scenario().habitat_temp_c, None);
    assert_eq!(day_neutral_lighting_scenario().habitat_temp_c, Some(20.0));
    // The day-neutral bio really has the gates removed; the default keeps them (the latter
    // a compile-time guard — the default-preserving invariant that keeps every golden green).
    let dn = day_neutral_lighting_bio_scenario();
    assert!(!dn.vernalization && !dn.photoperiod);
    const { assert!(DEFAULT_SCENARIO.vernalization && DEFAULT_SCENARIO.photoperiod) };
}
