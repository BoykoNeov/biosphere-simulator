//! **The hull breach** — the habitat losing its atmosphere, which until 2026-09-08 it could
//! not do.
//!
//! Plan: `docs/plans/post-roadmap-atmosphere.md` §1.3. Companions: `tests/atmosphere.rs` (the
//! gas is conserved and pressure exists) and `tests/gas_composition_perturbations.rs` (the
//! probes that measured the gap).
//!
//! # What changed, and why it needed a mechanism rather than a scenario
//!
//! The predecessor's E2 probe reached "both partial pressures halved" by **editing the
//! scenario's charge**. That is a state, not an event: nothing in the model could *travel*
//! there, because there was no inert gas and therefore no total to lose. E2 could imitate a
//! depressurization; it could not perform one.
//!
//! [`with_hull_breach`] performs one. Every gas in the chamber vents to its own boundary sink,
//! first-order at a single rate — which is not a simplification but the physics: a well-mixed
//! volume venting to vacuum loses each species in proportion to its own partial pressure, so
//! one `k` serves all four and `n_i(t) = n_i(0)·e^{-kt}` for every species at once.
//!
//! Diagnostics, **no golden, no manifest, no unfreeze** — the perturbation precedent. Every
//! claim here is a direction or an invariance, never a magnitude pin.

use std::collections::BTreeMap;

use domains::biosphere::params;
use domains::breach::{with_hull_breach, BREACH_SINK_PREFIX};
use domains::biosphere::science::{
    light_limited_rate, o2_coupled, rubisco_limited_rate, N2_MOLAR_MASS_KG_PER_MOL,
};
use domains::biosphere::stocks::{CARBON_POOL, CHAMBER_INERT, O2_POOL, WATER_VAPOR};
use domains::biosphere::system::{
    build_season_with, perennial_chamber_scenario, run_season, weather_resolver, SeasonScenario,
};
use domains::biosphere::{steps_for_years, BIO_DT};
use simcore::integrator::EulerIntegrator;
use simcore::state::State;

/// Every per-step series a breach test reads, sampled from one run.
struct Breached {
    co2: Vec<f64>,
    o2: Vec<f64>,
    inert_kg: Vec<f64>,
    vapour_kg: Vec<f64>,
    /// Total moles vented into the four boundary sinks, per step.
    sinks_mol: Vec<f64>,
    /// The inert sink alone (kg) — the one ledger with no biology in it.
    inert_sink_kg: Vec<f64>,
    rationed: u64,
    events: usize,
    capacity: f64,
}

impl Breached {
    /// Live total gas (mol) at step `i` — the same fold `readouts::total_gas_mol` performs,
    /// recomputed here because this harness runs below `Trajectory`.
    fn total(&self, i: usize) -> f64 {
        self.co2[i]
            + self.o2[i]
            + self.inert_kg[i] / N2_MOLAR_MASS_KG_PER_MOL
            + self.vapour_kg[i] / domains::biosphere::science::H2O_MOLAR_MASS_KG_PER_MOL
    }
    fn pressure(&self, i: usize) -> f64 {
        self.total(i) / self.capacity
    }
    fn len(&self) -> usize {
        self.co2.len()
    }
}

/// Run `scenario` with a breach open over `[start, end)`; `k_leak == 0.0` is the baseline.
///
/// ⚠ The baseline is the **same composed registry** with the rate set to zero, not the
/// unperturbed build. That is deliberate: it holds the flow list, the sink stocks and the
/// forcing map identical between the two arms, so a contrast measures the breach and not the
/// composition. A `k` of zero makes every leak leg exactly `0.0`.
fn breached(scenario: SeasonScenario, years: usize, k_leak: f64, start: u64, end: u64) -> Breached {
    let p = params::biosphere();
    let (state, registry) = build_season_with(&scenario, &p).expect("build");
    let resolver = weather_resolver(&scenario, years).expect("weather");
    let (state, registry, resolver) =
        with_hull_breach(&state, registry, resolver, k_leak, start, end).expect("breach");

    let sink_ids: Vec<String> = [CARBON_POOL, O2_POOL, CHAMBER_INERT, WATER_VAPOR]
        .iter()
        .map(|gas| format!("{BREACH_SINK_PREFIX}{}", gas.rsplit('.').next().unwrap()))
        .collect();

    let mut out = Breached {
        co2: Vec::new(),
        o2: Vec::new(),
        inert_kg: Vec::new(),
        vapour_kg: Vec::new(),
        sinks_mol: Vec::new(),
        inert_sink_kg: Vec::new(),
        rationed: 0,
        events: 0,
        capacity: scenario.chamber_air_capacity_mol,
    };
    {
        let amt = |s: &State, id: &str| s.stocks.get(id).map_or(0.0, |st| st.amount);
        let mut observe = |s: &State| {
            out.co2.push(amt(s, CARBON_POOL));
            out.o2.push(amt(s, O2_POOL));
            out.inert_kg.push(amt(s, CHAMBER_INERT));
            out.vapour_kg.push(amt(s, WATER_VAPOR));
            // The sinks in MOLES, so the total is comparable with the interior's: the two
            // gas sinks are already mol, the inert and vapour sinks are kg.
            let m = |id: &str, molar: f64| amt(s, id) / molar;
            out.sinks_mol.push(
                amt(s, &sink_ids[0])
                    + amt(s, &sink_ids[1])
                    + m(&sink_ids[2], N2_MOLAR_MASS_KG_PER_MOL)
                    + m(&sink_ids[3], domains::biosphere::science::H2O_MOLAR_MASS_KG_PER_MOL),
            );
            out.inert_sink_kg.push(amt(s, &sink_ids[2]));
        };
        let integrator = EulerIntegrator::new(registry);
        let (_final, rationed, events) = run_season(
            &integrator,
            state,
            &resolver,
            BIO_DT,
            steps_for_years(years),
            None,
            &mut observe,
        )
        .expect("the breached run did not survive");
        out.rationed = rationed;
        out.events = events.len();
    }
    out
}

/// The chamber this file breaches: the perennial jar, which unlike `sealed_chamber` is charged
/// at a realistic 21 % O₂ and so has a composition worth preserving.
fn jar() -> SeasonScenario {
    perennial_chamber_scenario()
}

/// A breach over the first season, gentle enough that `k·dt < 1` by three orders.
const K: f64 = 0.02;
const YEARS: usize = 1;
const START: u64 = 100;
const END: u64 = 500;

/// A breached run still has to be a **run**: no rationing, no extinction events.
///
/// ⚠ Its own test, and first, for the predecessor's finding 4 — halving the jar's O₂ already
/// took it anoxic with no backstop firing and no event, and a breach scales *every* species,
/// so that path is easier to reach here, not harder. Anything read as science below is read
/// off a run that passed this.
#[test]
fn the_breached_run_is_well_posed() {
    let t = breached(jar(), YEARS, K, START, END);
    assert_eq!(t.rationed, 0, "the arbitration backstop fired");
    assert_eq!(t.events, 0, "the run raised extinction events");
    assert!(t.co2.iter().all(|c| *c > 0.0), "the CO2 pool emptied");
    assert!(t.o2.iter().all(|o| *o > 0.0), "the O2 pool emptied");
}

/// **The claim the single rate constant buys: every species vents at the same fraction.**
///
/// Measured where it can be measured *exactly* — the **first active step**. Before `START` the
/// breach is closed, so the breached and baseline arms are in bit-identical states; over that
/// one step the only difference between them is the vent. So
///
/// ```text
/// (baseline_i(START+1) - breached_i(START+1)) / n_i(START)  ==  k·dt   for every species i
/// ```
///
/// and it holds to 1e-13 for all four. That is composition preservation stated as the property
/// that *causes* it, rather than as a ratio read off a run.
///
/// ⚠ **This is deliberately NOT "the CO₂:O₂ ratio holds across the window", which was the
/// first version of this test and was WRONG.** It failed at 1.5e-2, and the failure was the
/// test's, not the breach's: photosynthesis and respiration write CO₂ and O₂ throughout the
/// window, so their ratio moves for reasons that have nothing to do with venting. A vent's
/// composition-preservation is only observable on the vent's own contribution — which means
/// against a baseline, over an interval short enough that the biology is identical in both
/// arms. One step is the only such interval.
#[test]
fn one_rate_on_every_species_vents_the_same_fraction() {
    let base = breached(jar(), YEARS, 0.0, START, END);
    let b = breached(jar(), YEARS, K, START, END);
    let s = START as usize;
    let expected = K * BIO_DT;

    let species: [(&str, f64, f64, f64); 4] = [
        ("CO2", base.co2[s + 1], b.co2[s + 1], base.co2[s]),
        ("O2", base.o2[s + 1], b.o2[s + 1], base.o2[s]),
        (
            "inert",
            base.inert_kg[s + 1],
            b.inert_kg[s + 1],
            base.inert_kg[s],
        ),
        (
            "vapour",
            base.vapour_kg[s + 1],
            b.vapour_kg[s + 1],
            base.vapour_kg[s],
        ),
    ];
    for (name, unbreached, vented, before) in species {
        assert!(
            before > 0.0,
            "{name} is empty at the breach, so this species is untested here"
        );
        let fraction = (unbreached - vented) / before;
        assert!(
            (fraction - expected).abs() <= 1.0e-13,
            "{name} vented {fraction:e} of itself in one step, not k*dt = {expected:e} — the              four species are NOT leaving at one rate, so the vent changes the composition"
        );
    }
    // Anti-vacuity: the arms must actually be identical *before* the window, or "the only
    // difference is the vent" is an assumption rather than a fact.
    assert_eq!(base.co2[s], b.co2[s], "the arms differ before the breach opens");
    assert_eq!(base.o2[s], b.o2[s], "the arms differ before the breach opens");
}

/// **The habitat loses its atmosphere** — the sentence that was false about this model until
/// this work, stated as a test.
///
/// Every claim is a **contrast against the k = 0 arm**, not an absolute direction, and the
/// reason is a measurement: across this window the unbreached jar's CO₂ *rises* (0.399 →
/// 1.349) because respiration outruns the crop. An absolute "CO₂ fell" assertion was the first
/// version of this test and it failed on a perfectly correct breach. What the breach owns is
/// the **difference** between the arms.
#[test]
fn the_breach_takes_the_whole_atmosphere_and_pressure_falls_with_it() {
    let base = breached(jar(), YEARS, 0.0, START, END);
    let b = breached(jar(), YEARS, K, START, END);
    let e = END as usize;
    assert_eq!(b.pressure(0), 1.0, "the jar did not start at reference");
    assert!(
        b.pressure(e) < base.pressure(e),
        "pressure did not fall against the unbreached arm: {} vs {}",
        b.pressure(e),
        base.pressure(e)
    );
    // Every species, not just the total — a vent that took only the inert fill would drop the
    // pressure while RAISING the reactive partial pressures, which is the opposite mechanism
    // and would leave the crop better off.
    assert!(b.co2[e] < base.co2[e], "CO2 was not vented");
    assert!(b.o2[e] < base.o2[e], "O2 was not vented");
    assert!(b.inert_kg[e] < base.inert_kg[e], "the inert fill was not vented");
    assert!(b.vapour_kg[e] < base.vapour_kg[e], "the vapour was not vented");
}

/// The vent **moves** mass rather than destroying it.
///
/// ⚠ The general proof is not here and could not be: `simcore` asserts conservation of every
/// quantity on **every step**, so a breach that destroyed mass would abort the run rather than
/// reach an assertion in this file. `the_breached_run_is_well_posed` completing is that proof.
///
/// What this test adds is the readable witness — the inert fill is written by *nothing* except
/// the breach, so `inert + its own sink` is constant with no biology in the way.
///
/// ⚠ **Not to the bit, and the reason is worth stating rather than absorbing.** The pool
/// accumulates a sequence of *subtractions* and the sink the matching *additions*, at
/// different magnitudes — 22.12 against 0.0 and rising — so the two round differently. Measured
/// drift is 4 ULP (`22.120585216200006` against `...002`). The tolerance below is therefore a
/// real one at the rounding floor; a leak large enough to matter is many orders above it.
///
/// ⚠ It is deliberately not the four-species ledger, which was this test's first version and
/// failed at 6.7e-1. That failure was the test's: water vapour enters the gas phase from soil
/// water and leaves it to condensate, so the *total gas* is not a closed quantity even before a
/// breach. Interior + sinks was never going to be constant, and asserting it would have meant
/// weakening the tolerance until a real leak could hide under it.
#[test]
fn the_inert_fill_the_breach_removes_is_exactly_what_its_sink_gains() {
    let t = breached(jar(), YEARS, K, START, END);
    let closed = |i: usize| t.inert_kg[i] + t.inert_sink_kg[i];
    let charge = closed(0);
    let worst = (0..t.len())
        .map(|i| (closed(i) - charge).abs() / charge)
        .fold(0.0_f64, f64::max);
    assert!(
        worst <= 1.0e-15,
        "inert + sink drifted {worst:e} from the charge — that is far above the rounding          floor, so the breach is destroying the inert fill rather than moving it"
    );
    assert!(
        *t.inert_sink_kg.last().unwrap() > 0.0,
        "the inert sink is empty, so this test passed on a run where nothing leaked"
    );
}

/// The baseline arm is genuinely inert: `k = 0` reproduces a chamber that never breaches.
///
/// ⚠ Anti-vacuity for every contrast above. Without it, "pressure fell" could be measured
/// against a composition that was already leaking, and "the sinks filled" could be an artefact
/// of composing the flows at all.
#[test]
fn a_zero_rate_breach_moves_nothing() {
    let t = breached(jar(), YEARS, 0.0, START, END);
    assert_eq!(*t.sinks_mol.last().unwrap(), 0.0, "a k=0 breach still vented");
    for i in 0..t.len() {
        assert_eq!(
            t.inert_kg[i], t.inert_kg[0],
            "a k=0 breach moved the inert fill at step {i}"
        );
    }
}

// ===================================================================================
// The fingerprint: which FvCB branch feels a depressurization, on a RUN's numbers
// ===================================================================================

/// **E0's arithmetic fingerprint, evaluated at pressures the breach actually produced.**
///
/// As the chamber depressurizes at fixed composition, `Ci` and `Γ*` (which is proportional to
/// O₂) fall by the same factor. Then:
///
/// * `light_limited_rate = J(Ci−Γ*)/(4Ci+8Γ*)` is **homogeneous of degree zero** — numerator
///   and denominator scale alike, so it is *exactly* blind to the pressure change;
/// * `rubisco_limited_rate = Vcmax(Ci−Γ*)/(Ci+Kc(1+O/Ko))` is **not** — `Kc` is a constant
///   term in the denominator, so the rate falls.
///
/// ⚠ **Written against the reference basis explicitly**, which is the whole reason this test
/// can tell "correct" from "blind": the scale factors below come from the *pressure ratio* the
/// run reached, i.e. from `n_i / n_ref`. A version written against the live mole fraction
/// would compute a scale factor of exactly 1.0 at every step and pass no matter what the leaf
/// did — it would be a test of the arithmetic's own tautology.
#[test]
fn the_light_limited_branch_is_blind_to_the_breach_and_the_rubisco_branch_is_not() {
    let t = breached(jar(), YEARS, K, START, END);
    let p = params::biosphere().photo;
    let f = t.pressure(END as usize) / t.pressure(START as usize);
    assert!(
        f < 0.999,
        "the breach barely moved the pressure (factor {f}), so this test has no subject"
    );

    let (ci, j) = (250.0_f64, 200.0_f64);
    let light_full = light_limited_rate(ci, j, p.gamma_star);
    let light_thin = light_limited_rate(f * ci, j, f * p.gamma_star);
    assert!(
        (light_thin - light_full).abs() <= 1.0e-12 * light_full,
        "the light-limited branch moved under a proportional gas change: {light_full} -> \
         {light_thin}"
    );

    // ⚠ O₂ scales *through* `o2_coupled`, which carries Γ* with it (Γ* ∝ O). Poking `p.o2`
    // alone would leave Γ* frozen — a different and unphysical experiment, and the same
    // reason E0 gives in `gas_composition_perturbations.rs`.
    let rub_full = rubisco_limited_rate(ci, &p);
    let rub_thin = rubisco_limited_rate(f * ci, &o2_coupled(&p, f * p.o2));
    assert!(
        rub_thin < rub_full,
        "the Rubisco-limited branch did NOT fall under depressurization: {rub_full} -> \
         {rub_thin}"
    );
}

/// The breach builds one sink per gas, and they are distinct stocks.
///
/// ⚠ The composer's whole reason for existing over `biosphere::perturbations`'s
/// single-sink `LeakFlow`: four legs into one sink would need that sink to carry four
/// compositions, and the predecessor deferred a two-gas leak on exactly that collision.
#[test]
fn the_breach_gives_every_gas_its_own_sink() {
    let p = params::biosphere();
    let scenario = jar();
    let (state, registry) = build_season_with(&scenario, &p).unwrap();
    let resolver = weather_resolver(&scenario, YEARS).unwrap();
    let (state, _r, _res) = with_hull_breach(&state, registry, resolver, K, START, END).unwrap();
    let sinks: BTreeMap<&String, &simcore::state::Stock> = state
        .stocks
        .iter()
        .filter(|(id, _)| id.starts_with(BREACH_SINK_PREFIX))
        .collect();
    assert_eq!(sinks.len(), 4, "expected one sink per gas, got {sinks:?}");
    // Each mirrors its pool's composition, which is what makes the legs per-quantity
    // balanced — the CO₂ sink must carry OXYGEN as well as CARBON, or the vent would break
    // conservation on oxygen alone.
    let co2_sink = state
        .stocks
        .get(&format!("{BREACH_SINK_PREFIX}carbon_pool"))
        .expect("the CO2 sink");
    assert_eq!(
        co2_sink.composition,
        state.stocks[CARBON_POOL].composition,
        "the CO2 sink does not mirror the CO2 pool's composition"
    );
}

/// An open field has no atmosphere to lose, and says so rather than composing a no-op.
///
/// ⚠ A silent success here would be the worst outcome: a breach composed onto the open field
/// would produce a perfectly normal run with four flows that can never fire, and every
/// contrast drawn from it would read as "the breach did nothing".
#[test]
fn a_breach_on_the_open_field_is_rejected() {
    let p = params::biosphere();
    let open = domains::biosphere::DEFAULT_SCENARIO;
    let (state, registry) = build_season_with(&open, &p).unwrap();
    let resolver = weather_resolver(&open, YEARS).unwrap();
    assert!(with_hull_breach(&state, registry, resolver, K, START, END).is_err());
}

