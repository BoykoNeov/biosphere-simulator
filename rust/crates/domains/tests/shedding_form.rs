//! The lab shedding form (`domains::lab::shedding`; `docs/plans/post-roadmap-leaf-shedding.md`
//! §7) — LAB-ONLY: nothing frozen builds it.
//!
//! Three claims, each one the others cannot stand in for:
//!
//! * **the control** — the lab flows at the frozen form reproduce the frozen run bit for bit, on
//!   the open field (carbon only) and in the sealed jar (carbon and its nitrogen twin). Without
//!   it, a measured difference could be the lab copy's arithmetic. ⚠ Since 2026-10-06 the frozen
//!   form is [`SheddingForm::BEFORE_ANTHESIS_ONLY`] (it was [`SheddingForm::FLAT`] until then);
//! * **the law** — the rates the sources' form returns before and after anthesis, by hand;
//! * **the reach** — the sources' form moves the run (a lab flow the seam silently ignored
//!   would pass the control too).

use domains::biosphere::params::{self, BiosphereParams};
use domains::biosphere::science;
use domains::biosphere::stocks::{LEAF_C, TEMP_VAR, THERMAL_TIME};
use domains::biosphere::system::sealed_chamber_scenario;
use domains::biosphere::{
    run_season, season_setup, steps_for_years, weather_resolver, SeasonScenario, BIO_DT,
    DEFAULT_SCENARIO,
};
use domains::lab::mechanism::Composition;
use domains::lab::shedding::{afgen, carbon_only, composition, SheddingForm, SheddingLaw, LLVT};
use simcore::environment::Environment;
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

const YEARS: usize = 1;

fn frozen() -> BiosphereParams {
    params::biosphere()
}

/// Every stock, every step: `(ids, amounts step-major)`.
fn series(
    state: State,
    integrator: &EulerIntegrator,
    scenario: &SeasonScenario,
) -> (Vec<String>, Vec<f64>) {
    let resolver = weather_resolver(scenario, YEARS).expect("resolver");
    let ids: Vec<String> = state.stocks.keys().cloned().collect();
    let mut out = Vec::new();
    {
        let mut observe = |s: &State| out.extend(s.stocks.values().map(|st| st.amount));
        let (_f, rationed, events) = run_season(
            integrator,
            state,
            &resolver,
            BIO_DT,
            steps_for_years(YEARS),
            None,
            &mut observe,
        )
        .expect("season");
        assert_eq!(rationed, 0);
        assert!(events.is_empty(), "{events:?}");
    }
    (ids, out)
}

fn frozen_series(scenario: &SeasonScenario) -> (Vec<String>, Vec<f64>) {
    let (state, integrator, _r) = season_setup(scenario, YEARS).expect("setup");
    series(state, &integrator, scenario)
}

fn composed_series(scenario: &SeasonScenario, comp: &Composition) -> (Vec<String>, Vec<f64>) {
    let (state, registry) = comp.apply(scenario, &frozen()).expect("compose");
    series(state, &EulerIntegrator::new(registry), scenario)
}

fn bits(s: &(Vec<String>, Vec<f64>)) -> (Vec<String>, Vec<u64>) {
    (s.0.clone(), s.1.iter().map(|x| x.to_bits()).collect())
}

/// ⚠ **The control moved on 2026-10-06** (the biosphere unfreeze, the leaf-shedding note §10):
/// the agreed half — no leaf or root shedding before anthesis — IS the frozen form now, so it is
/// the lab copy of it that must reproduce the frozen run bit for bit (prediction U1). `FLAT`
/// is the form before that day.
#[test]
fn the_agreed_half_through_the_lab_is_the_frozen_open_field_bit_for_bit() {
    let s = DEFAULT_SCENARIO;
    assert_eq!(
        bits(&composed_series(
            &s,
            &carbon_only(SheddingForm::BEFORE_ANTHESIS_ONLY)
        )),
        bits(&frozen_series(&s))
    );
}

#[test]
fn the_agreed_half_through_the_lab_is_the_frozen_sealed_jar_bit_for_bit() {
    let s = sealed_chamber_scenario();
    assert_eq!(
        bits(&composed_series(
            &s,
            &composition(SheddingForm::BEFORE_ANTHESIS_ONLY)
        )),
        bits(&frozen_series(&s))
    );
}

/// The form before 2026-10-06 (`FLAT`, shedding from day one) is no longer the frozen run.
#[test]
fn the_flat_form_is_no_longer_the_frozen_run() {
    let s = DEFAULT_SCENARIO;
    assert_ne!(
        bits(&composed_series(&s, &carbon_only(SheddingForm::FLAT))),
        bits(&frozen_series(&s))
    );
}

#[test]
fn the_sources_form_reaches_the_run() {
    let s = DEFAULT_SCENARIO;
    let frozen = frozen_series(&s);
    let all = composed_series(&s, &carbon_only(SheddingForm::ALL));
    assert_eq!(all.0, frozen.0);
    let leaf = all.0.iter().position(|k| k == LEAF_C).unwrap();
    let n = all.0.len();
    let last = |v: &[f64]| v[v.len() - n + leaf];
    assert_ne!(last(&all.1).to_bits(), last(&frozen.1).to_bits());
}

/// The rates by hand: FLAT is the three frozen constants; the sources' form sheds nothing
/// before anthesis (a seedling's LAI is far below the shading threshold) and, after it, leaf at
/// `DVR / (2 − DVS)` and root at the frozen `rdr_root`, stem never.
#[test]
fn the_law_before_and_after_anthesis() {
    let s = DEFAULT_SCENARIO;
    let p = frozen();
    let (mut state, _i, _r) = season_setup(&s, YEARS).expect("setup");
    let resolver = weather_resolver(&s, YEARS).expect("resolver");
    let rates = |form: SheddingForm, tt: f64, state: &mut State| {
        state.aux.insert(THERMAL_TIME.to_string(), tt);
        let env = resolver.bind(state, BIO_DT);
        SheddingLaw::new(form, &s, &p).rates(state, &env).unwrap()
    };
    let flat = rates(SheddingForm::FLAT, 500.0, &mut state);
    assert_eq!(
        flat,
        (p.senesc.rdr_leaf, p.senesc.rdr_stem, p.senesc.rdr_root)
    );
    assert_eq!(rates(SheddingForm::ALL, 500.0, &mut state), (0.0, 0.0, 0.0));
    let tt = p.pheno.tsum_anthesis + 0.5 * p.pheno.tsum_maturity; // DVS 1.5
    let (leaf, stem, root) = rates(SheddingForm::ALL, tt, &mut state);
    let temp = resolver.bind(&state, BIO_DT).get(TEMP_VAR).unwrap();
    let dvr =
        science::daily_thermal_time(temp, p.pheno.t_base, p.pheno.t_cap) / p.pheno.tsum_maturity;
    assert!(dvr > 0.0, "day 0 must be warm enough to develop: {temp} °C");
    assert_eq!(leaf, dvr / 0.5);
    assert_eq!((stem, root), (0.0, p.senesc.rdr_root));
    // The leaf alone: root and stem keep the frozen constants.
    let (_, stem, root) = rates(SheddingForm::LEAF, 500.0, &mut state);
    assert_eq!((stem, root), (p.senesc.rdr_stem, p.senesc.rdr_root));
}

/// Penning de Vries' table, by hand: 0 to anthesis, 0.007 at DVS 1.3, 0.012 from 1.8, linear in
/// between — and the second cited form sheds nothing before anthesis either.
#[test]
fn the_penning_de_vries_leaf_table() {
    assert_eq!(afgen(&LLVT, 0.5), 0.0);
    assert_eq!(afgen(&LLVT, 1.0), 0.0);
    assert!((afgen(&LLVT, 1.15) - 0.0035).abs() < 1e-15);
    assert_eq!(afgen(&LLVT, 1.3), 0.007);
    assert!((afgen(&LLVT, 1.55) - 0.0095).abs() < 1e-15);
    assert_eq!(afgen(&LLVT, 2.0), 0.012);
    let s = DEFAULT_SCENARIO;
    let p = frozen();
    let (mut state, _i, _r) = season_setup(&s, YEARS).expect("setup");
    let resolver = weather_resolver(&s, YEARS).expect("resolver");
    state.aux.insert(THERMAL_TIME.to_string(), 500.0);
    let env = resolver.bind(&state, BIO_DT);
    let law = SheddingLaw::new(SheddingForm::ALL_PDV, &s, &p);
    assert_eq!(law.rates(&state, &env).unwrap(), (0.0, 0.0, 0.0));
}

/// The agreed half alone: nothing before anthesis, the frozen rates after it.
#[test]
fn the_agreed_half_alone() {
    let s = DEFAULT_SCENARIO;
    let p = frozen();
    let (mut state, _i, _r) = season_setup(&s, YEARS).expect("setup");
    let resolver = weather_resolver(&s, YEARS).expect("resolver");
    let law = SheddingLaw::new(SheddingForm::BEFORE_ANTHESIS_ONLY, &s, &p);
    state.aux.insert(THERMAL_TIME.to_string(), 500.0);
    let env = resolver.bind(&state, BIO_DT);
    assert_eq!(
        law.rates(&state, &env).unwrap(),
        (0.0, p.senesc.rdr_stem, 0.0)
    );
    state
        .aux
        .insert(THERMAL_TIME.to_string(), p.pheno.tsum_anthesis + 100.0);
    let env = resolver.bind(&state, BIO_DT);
    assert_eq!(
        law.rates(&state, &env).unwrap(),
        (p.senesc.rdr_leaf, p.senesc.rdr_stem, p.senesc.rdr_root)
    );
}
