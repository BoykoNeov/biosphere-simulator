//! The runs that produce the committed regression goldens, and the policy for comparing
//! against them — Stage-3 slice **S2** of the reference flip.
//!
//! # ⚠⚠ Why this module exists: FINDING 3
//!
//! `docs/plans/post-roadmap-reference-flip.md` §5q measured it and the sentence is worth
//! keeping verbatim: **"No Rust test compares a run against a committed golden."** The
//! reference *emitted* the goldens (the `emit_*` examples) and **Python alone compared
//! them** — 17 `tests/test_regression_*.py` modules, `tests/golden_platform.py`'s policy,
//! and `tests/crossport/test_golden_provenance.py`'s byte census. Every one of those sits
//! in the tree S6 deletes, so the regression contract had no owner that survives the flip.
//!
//! An `examples/` program is a binary target: no integration test can call into it. That
//! is *why* the comparison ended up in Python — shelling out to `cargo run` was the only
//! way to reach the run. So S2's first act is not writing a test, it is **moving the run
//! out of the binary and into the library**, where both the example and a test can call
//! it. The `emit_*` examples are now one-line wrappers around the functions below.
//!
//! ⚠ The relocation is byte-neutral **by construction** (one code path, two callers) and
//! was nevertheless *measured*: all 19 emitters' stdout was captured before the move and
//! diffed after. "By construction" is the claim; the diff is the control.
//!
//! # The platform policy, ported rather than reinvented
//!
//! `tests/golden_platform.py` carries the rule and its reason: a hex-float golden is
//! byte-exact only *within a single build on the platform that generated it* — here
//! Windows/UCRT. A run whose graph touches a transcendental (`exp` / `sqrt` / `sin` /
//! `powf`) lands last-ULP-different on another libm, so a byte compare of a *re-run*
//! against the Windows golden fails off the generation platform. That is physically
//! meaningless noise, not a regression.
//!
//! ⚠⚠ **And `cargo test` runs on `ubuntu-latest`.** Python's answer is
//! `windows_golden_only`, a pytest skip. Rust has no skip, and the obvious translation —
//! `#[cfg(windows)]` — is the shape this repo has been bitten by twice: a gate that
//! compiles out is a gate nobody can see is gone. So the translation here is
//! *classification*, not exclusion:
//!
//! * [`Numerics::PureArithmetic`] — byte-compared on **every** platform;
//! * [`Numerics::Transcendental`] — byte-compared on the generation platform, and
//!   **structurally** compared everywhere else ([`compare_structural`]): identical JSON
//!   shape, identical stock-id/aux key sets, identical discrete fields, every hex-float
//!   leaf present and finite on both sides. Exact, and not a tolerance.
//!
//! ⚠ **A band was deliberately NOT invented.** The measured cross-libm bands live in
//! `tests/crossport/tiers.json`, which **no Rust program reads** (grepped: the only
//! mention in `rust/` is a doc-comment pointer in `domains/src/lib.rs`). So `tiers.json`
//! is a *fifth* item for the plan's FINDING 2 — a contract artifact stranded in the dying
//! tree — recorded there rather than papered over here with a number nobody measured.
//! `golden_platform.py`'s C3 comment refuses exactly that move and the refusal still
//! holds: *"writing a band nobody measured is the derived-not-measured move this contract
//! exists to refuse."*

use std::path::PathBuf;

use simcore::integrator::EulerIntegrator;
use simcore::json::{self, JsonValue};
use simcore::state::State;

use crate::biosphere::{
    consumer_chamber_scenario, perennial_chamber_scenario, run_perennial_final, run_season,
    sealed_chamber_scenario, season_setup, steps_for_years, BIO_DT, CONSUMER_CHAMBER_YEARS,
    DEFAULT_SCENARIO, LONG_HORIZON_YEARS, PERENNIAL_CHAMBER_YEARS, SEALED_CHAMBER_YEARS,
};
use crate::crew::{build_crew, crew_resolver, MISSION_DAYS, MISSION_SCENARIO};
use crate::eclss::{build_eclss, eclss_resolver, STEADY_STATE_SCENARIO, STEADY_STATE_STEPS};
use crate::power::{
    build_power, power_resolver, BOUNDED_SOC_DAYS, BOUNDED_SOC_SCENARIO, SELF_DISCHARGE_DAYS,
};
use crate::thermal::{build_thermal, thermal_resolver, EQUILIBRIUM_SCENARIO, EQUILIBRIUM_STEPS};
use crate::{params, run};

// --------------------------------------------------------------------------- //
// The policy types — shared with `station`, which depends on this crate         //
// --------------------------------------------------------------------------- //

/// How a golden's bytes behave across libm implementations.
///
/// ⚠ The classification is by **what the evaluation graph touches**, not by the set that
/// happens to diverge on one glibc build. `golden_platform.py` states why and the reason
/// is inherited unchanged: a few contracting/regulator-erased finals coincidentally match
/// on some libms, and relying on that coincidence makes CI brittle across glibc versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Numerics {
    /// `*` `/` `+` `-` only — IEEE-correctly-rounded, so byte-exact on every conformant
    /// platform. The four ungated goldens on the Python side are the same four here.
    PureArithmetic,
    /// Touches `exp` / `sin` / `acos` / `powf` / `sqrt`, which IEEE-754 does **not**
    /// mandate correctly-rounded. Byte-exact within a build on its generation platform.
    Transcendental,
}

/// What it costs to produce the golden, because one of them costs two minutes.
///
/// ⚠⚠ **This is a roster field and not a `#[ignore]` attribute for a reason.** The sealed
/// station is ~1.3 M sub-steps over five domains and takes ~100 s at *any* optimization
/// level (measured 2026-08-19: 378 s at the stock dev profile, 116 s at `opt-level = 2`,
/// 93 s in release — the cost is the run, not the build). It therefore runs off by default
/// and explicitly in CI. A bare `#[ignore]` would make that invisible; naming it here lets
/// `the_ignored_set_is_exactly_the_expensive_roster` assert that the set of skipped
/// goldens is *exactly* this one — so a second golden quietly joining the skipped set is
/// red, which is the failure mode `#[ignore]` alone cannot see.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cost {
    /// Under ~4 s. Runs in every `cargo test`.
    Cheap,
    /// Minutes. `#[ignore]`d, run by CI with `--ignored`.
    Expensive,
}

/// What KIND of artifact the golden is — the axis S6 build item 2 had to add before a
/// regeneration tool could validate anything.
///
/// # ⚠⚠ Why this exists, measured rather than assumed
///
/// The Python regeneration tool validated every candidate before it could reach the disk:
/// `sim_io.snapshot.loads(produced)`, i.e. *"a golden that does not round-trip must never
/// be written"*. S6 recorded losing that check as a stated loss when `src/sim_io` was
/// deleted. Measured while porting it: **that check had been unrunnable for one of the
/// nineteen since slice C5.** `sealed_energy_drift_summary.json` is a folded summary with
/// no `version` key, and `state_from_dict` *raises* on a missing version — so `--write`
/// would have died part-way through, after rewriting whichever earlier goldens had moved.
/// Nobody had run it.
///
/// So the successor cannot blanket-validate through [`simcore::snapshot::from_json`]. The
/// shape is declared per golden and the validator dispatches on it — and because this is a
/// struct field and not a lookup table, a golden added without a declared shape is a
/// **compile error** rather than a silent skip. The alternative (special-casing the one
/// file that does not fit) is the widen-the-gate-from-inside move this repo refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shape {
    /// A full engine state snapshot: reconstructible through
    /// [`simcore::snapshot::from_json`], so every core invariant re-fires on load.
    StateSnapshot,
    /// A folded summary — a hand-serialized JSON document of scalars and hex-float
    /// arrays, with no `version` and no stocks. ⚠ **Its validation is genuinely weaker
    /// and that is stated rather than hidden**: there is no reader to round-trip through,
    /// because the reference writes this artifact and never reads it. What is checked is
    /// that it parses and that every hex-float leaf parses — enough to refuse a truncated
    /// or malformed write, not enough to re-fire an invariant.
    FoldedSummary,
}

/// One committed golden and the run that produces its exact bytes.
pub struct Golden {
    /// The file name in `rust/data/golden/`.
    pub name: &'static str,
    /// The run. Returns the golden's bytes as `String` — the same value the matching
    /// `emit_*` example prints, because the example calls this.
    pub run: fn() -> String,
    pub numerics: Numerics,
    pub cost: Cost,
    pub shape: Shape,
}

/// `rust/data/golden/` — S1's home for the goldens, resolved from this crate.
///
/// ⚠ `env!("CARGO_MANIFEST_DIR")` expands at *this crate's* compile time, so `station`
/// calling this helper still gets the one directory rather than a second spelling of the
/// path. S1's whole point was that the reference stops climbing out of its own tree; two
/// crates each spelling `../../data/golden` would be two chances to climb wrong.
pub fn golden_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data/golden")
}

/// Read a committed golden's bytes.
pub fn committed(name: &str) -> String {
    let path = golden_dir().join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read committed golden {}: {e}", path.display()))
}

/// Every `*.json` in `rust/data/golden/`, enumerated **from the directory**.
///
/// ⚠ Never hand-listed. `docs/log/coverage-roster-is-not-the-manifest.md` records this
/// repo believing a hand-maintained list was the census; the Python census
/// (`regen_goldens_from_rust.committed_goldens`) enumerates for the same reason.
pub fn committed_goldens() -> Vec<String> {
    let dir = golden_dir();
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot list {}: {e}", dir.display()))
        .map(|entry| {
            entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .filter(|name: &String| name.ends_with(".json"))
        .collect();
    names.sort();
    names
}

// --------------------------------------------------------------------------- //
// The comparison                                                               //
// --------------------------------------------------------------------------- //

/// The verdict of comparing a fresh run against its committed golden.
#[derive(Debug)]
pub enum Verdict {
    /// The bytes are equal. The strongest answer, and the only one accepted on the
    /// generation platform.
    ByteExact,
    /// The bytes differ but the structure is identical — the expected state for a
    /// [`Numerics::Transcendental`] golden off its generation platform.
    StructurallyEqual,
    /// A real difference, with the first divergence named.
    Differs(String),
}

/// Compare a fresh run against a committed golden under the platform policy.
pub fn compare(produced: &str, expected: &str, numerics: Numerics) -> Verdict {
    if produced == expected {
        return Verdict::ByteExact;
    }
    if numerics == Numerics::PureArithmetic || cfg!(windows) {
        return Verdict::Differs(first_difference(produced, expected));
    }
    match compare_structural(produced, expected) {
        Ok(()) => Verdict::StructurallyEqual,
        Err(why) => Verdict::Differs(why),
    }
}

/// Structural (Tier-0) equality: everything except the *value* of a hex-float leaf.
///
/// Identical tree shape, identical object keys **in order** (the goldens are
/// `sort_keys=True`, so an order change is a real change), identical array lengths,
/// identical non-hex-float leaves. A hex-float leaf must parse to a finite value on both
/// sides — a `NaN`/`inf` appearing where a number was is a regression on any platform.
///
/// ⚠ **A hex-float leaf is identified by parsing, not by key name.** The snapshot codec
/// spells floats as strings (`"0x1.87e...p+3"`), so JSON's own types cannot tell a float
/// from a unit label. `hexfloat::parse` is the discriminator: `"kg"` and `"boundary"` fail
/// it, `"0x0.0p+0"` does not. That means a float leaf that turned into a *label* is caught
/// (the parse succeeds on one side and not the other), which a key-name rule would miss.
pub fn compare_structural(produced: &str, expected: &str) -> Result<(), String> {
    let p = json::parse(produced).map_err(|e| format!("produced output is not JSON: {e:?}"))?;
    let e = json::parse(expected).map_err(|e| format!("committed golden is not JSON: {e:?}"))?;
    walk(&p, &e, "$")
}

fn walk(produced: &JsonValue, expected: &JsonValue, path: &str) -> Result<(), String> {
    match (produced, expected) {
        (JsonValue::Object(p), JsonValue::Object(e)) => {
            if p.len() != e.len() {
                let pk: Vec<&str> = p.iter().map(|(k, _)| k.as_str()).collect();
                let ek: Vec<&str> = e.iter().map(|(k, _)| k.as_str()).collect();
                return Err(format!(
                    "{path}: key count {} vs {} (produced {pk:?}, golden {ek:?})",
                    p.len(),
                    e.len()
                ));
            }
            for ((pk, pv), (ek, ev)) in p.iter().zip(e.iter()) {
                if pk != ek {
                    return Err(format!("{path}: key {pk:?} where the golden has {ek:?}"));
                }
                walk(pv, ev, &format!("{path}.{pk}"))?;
            }
            Ok(())
        }
        (JsonValue::Array(p), JsonValue::Array(e)) => {
            if p.len() != e.len() {
                return Err(format!("{path}: {} elements vs {}", p.len(), e.len()));
            }
            for (i, (pv, ev)) in p.iter().zip(e.iter()).enumerate() {
                walk(pv, ev, &format!("{path}[{i}]"))?;
            }
            Ok(())
        }
        (JsonValue::Str(p), JsonValue::Str(e)) => {
            match (
                simcore::hexfloat::parse(p).ok(),
                simcore::hexfloat::parse(e).ok(),
            ) {
                (Some(pf), Some(ef)) => {
                    if pf.is_finite() && ef.is_finite() {
                        Ok(())
                    } else {
                        Err(format!("{path}: non-finite hex-float {p:?} vs {e:?}"))
                    }
                }
                (None, None) => {
                    if p == e {
                        Ok(())
                    } else {
                        Err(format!("{path}: {p:?} vs {e:?}"))
                    }
                }
                _ => Err(format!(
                    "{path}: one side is a hex-float and the other is not — {p:?} vs {e:?}"
                )),
            }
        }
        _ => {
            if produced == expected {
                Ok(())
            } else {
                Err(format!("{path}: {produced:?} vs {expected:?}"))
            }
        }
    }
}

/// The first differing line, so a failure message points somewhere.
fn first_difference(produced: &str, expected: &str) -> String {
    for (i, (p, e)) in produced.lines().zip(expected.lines()).enumerate() {
        if p != e {
            return format!("line {}:\n  produced: {p}\n  golden:   {e}", i + 1);
        }
    }
    format!(
        "identical for {} lines, then the lengths differ ({} vs {} lines)",
        produced.lines().count().min(expected.lines().count()),
        produced.lines().count(),
        expected.lines().count()
    )
}

// --------------------------------------------------------------------------- //
// The runs — the bodies the `emit_*` examples used to hold                     //
// --------------------------------------------------------------------------- //
// The validator — S6 build item 2's half of the regeneration path              //
// --------------------------------------------------------------------------- //

/// Refuse a golden that is not a well-formed artifact of its declared [`Shape`].
///
/// # ⚠⚠ The gate this restores, and the way it was already broken
///
/// `regen_goldens_from_rust.py` ran `sim_io.snapshot.loads(produced)` on every candidate
/// before it could reach the disk — *"a golden that does not round-trip must never be
/// written"*. S6 deleted `src/sim_io`, disabled `--write` and recorded the loss.
///
/// Measured while porting it: **that check had been unrunnable for one of the nineteen
/// since slice C5.** `sealed_energy_drift_summary.json` has no `version` key and the
/// validator raised on a missing one, so a real `--write` would have died part-way
/// through — after rewriting whichever earlier goldens had moved. The reason nobody saw it
/// is that nobody ran `--write`; the report path, which is what gets run, never validated.
///
/// So the check comes back **dispatched on the declared shape** rather than applied
/// blanket. See [`Shape`] for why that is a struct field and not a lookup table.
///
/// ⚠ The two arms are not equally strong, and the asymmetry is the point rather than a
/// gap to paper over: a snapshot is reconstructed through the engine's own constructors,
/// so every core invariant re-fires; a folded summary has no reader to round-trip through
/// — the reference writes it and never reads it — so what is checked is that it parses and
/// that its float leaves parse. Enough to refuse a truncated or malformed write; not
/// enough to re-fire an invariant, and it does not claim to be.
pub fn validate(name: &str, text: &str, shape: Shape) -> Result<(), String> {
    match shape {
        Shape::StateSnapshot => simcore::snapshot::from_json(text)
            .map(|_| ())
            .map_err(|e| format!("{name} does not reconstruct as an engine state: {e:?}")),
        Shape::FoldedSummary => {
            let value = json::parse(text)
                .map_err(|e| format!("{name} is not well-formed JSON: {e:?}"))?;
            let fields = value.as_object().ok_or_else(|| {
                format!("{name} is not a JSON object, so it is not a folded summary")
            })?;
            if fields.is_empty() {
                return Err(format!("{name} is an empty object"));
            }
            let mut floats = 0usize;
            check_float_leaves(&value, "$", &mut floats)?;
            if floats == 0 {
                return Err(format!(
                    "{name} carries no hex-float leaf at all — a folded summary that \
                     folded nothing is a truncated write, not a summary"
                ));
            }
            Ok(())
        }
    }
}

/// Every string leaf that *looks* like a hex-float must parse to a finite number.
///
/// ⚠ The discriminator is `hexfloat::parse` and not a key-name rule, exactly as in
/// [`compare_structural`]: `"kg"` fails it and `"0x1.0p+0"` does not, so a float leaf
/// that turned into a label is caught rather than classified away.
fn check_float_leaves(value: &JsonValue, path: &str, floats: &mut usize) -> Result<(), String> {
    match value {
        JsonValue::Object(fields) => {
            for (key, v) in fields {
                check_float_leaves(v, &format!("{path}.{key}"), floats)?;
            }
            Ok(())
        }
        JsonValue::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                check_float_leaves(v, &format!("{path}[{i}]"), floats)?;
            }
            Ok(())
        }
        JsonValue::Str(s) => {
            if s.starts_with("0x") || s.starts_with("-0x") {
                let f = simcore::hexfloat::parse(s)
                    .map_err(|e| format!("{path}: {s:?} is not a hex-float: {e:?}"))?;
                if !f.is_finite() {
                    return Err(format!("{path}: non-finite hex-float {s:?}"));
                }
                *floats += 1;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

// --------------------------------------------------------------------------- //

fn snapshot(state: &State) -> String {
    simcore::snapshot::from_engine(state).to_json()
}

/// The open-field `DEFAULT_SCENARIO` season, Euler-daily, 1 season.
pub fn season() -> String {
    let weather_years = 1;
    let (state, integrator, resolver) =
        season_setup(&DEFAULT_SCENARIO, weather_years).expect("season_setup");
    let steps = steps_for_years(weather_years);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) = run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps,
        None,
        &mut noop,
    )
    .expect("run_season");

    assert_eq!(rationed, 0, "Tier-0: open season rationed must be 0");
    assert!(events.is_empty(), "Tier-0: open season events must be empty");
    snapshot(&final_state)
}

/// The O₂-poor sealed chamber (3 yr) — FvCB + the decomposer gas loop + f_O2.
pub fn sealed_chamber() -> String {
    let wy = SEALED_CHAMBER_YEARS;
    let scenario = sealed_chamber_scenario();
    let (state, integrator, resolver) = season_setup(&scenario, wy).expect("season_setup");
    let steps = steps_for_years(wy);
    let mut noop = |_: &State| {};
    let (final_state, rationed, events) = run_season(
        &integrator,
        state,
        &resolver,
        BIO_DT,
        steps,
        None,
        &mut noop,
    )
    .expect("run_season");

    assert_eq!(
        rationed, 0,
        "Tier-0: sealed rationed must be 0 (f_O2 self-limits)"
    );
    assert!(events.is_empty(), "Tier-0: sealed events must be empty");
    snapshot(&final_state)
}

/// The perennial (annual-reset) sealed chamber. One run serves two goldens by horizon.
pub fn perennial_chamber(years: usize) -> String {
    let scenario = perennial_chamber_scenario();
    let (final_state, rationed, events) =
        run_perennial_final(&scenario, years).expect("run_perennial");
    assert_eq!(rationed, 0, "Tier-0: perennial rationed must be 0");
    assert!(events.is_empty(), "Tier-0: perennial events must be empty");
    snapshot(&final_state)
}

/// The minimal-consumer sealed chamber (the herbivory sub-loop). Two goldens by horizon.
pub fn consumer_chamber(years: usize) -> String {
    let scenario = consumer_chamber_scenario();
    let (final_state, rationed, events) =
        run_perennial_final(&scenario, years).expect("run_perennial");
    assert_eq!(rationed, 0, "Tier-0: consumer rationed must be 0");
    assert!(events.is_empty(), "Tier-0: consumer events must be empty");
    snapshot(&final_state)
}

/// The standalone Crew `MISSION_SCENARIO`, 7 days.
pub fn crew() -> String {
    let p = params::crew();
    let scenario = MISSION_SCENARIO;
    let (state, registry) = build_crew(&p, &scenario).expect("build_crew");
    let resolver = crew_resolver(&scenario).expect("crew_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = MISSION_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run crew");

    assert_eq!(
        rationed, 0,
        "Tier-0: crew rationed must be 0 (well-fed sizing)"
    );
    assert!(
        events.is_empty(),
        "Tier-0: crew events must be empty (no POPULATION stock)"
    );
    snapshot(&final_state)
}

/// The standalone ECLSS `STEADY_STATE_SCENARIO` — transcendental-free.
pub fn eclss() -> String {
    let p = params::eclss();
    let scenario = STEADY_STATE_SCENARIO;
    let (state, registry) = build_eclss(&p, &scenario).expect("build_eclss");
    let resolver = eclss_resolver(&scenario).expect("eclss_resolver");
    let integrator = EulerIntegrator::new(registry);
    let (final_state, rationed, events) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        STEADY_STATE_STEPS,
    )
    .expect("run eclss");

    assert_eq!(rationed, 0, "Tier-0: eclss rationed must be 0");
    assert!(events.is_empty(), "Tier-0: eclss events must be empty");
    snapshot(&final_state)
}

/// The standalone Power `BOUNDED_SOC_SCENARIO`, 7 days — the half-sine solar schedule.
pub fn power() -> String {
    let charge = params::charge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let (state, registry) = build_power(&charge, &scenario, None).expect("build_power");
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = BOUNDED_SOC_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run power");

    assert_eq!(
        rationed, 0,
        "Tier-0: power rationed must be 0 (well-fed sizing)"
    );
    assert!(events.is_empty(), "Tier-0: power events must be empty");
    snapshot(&final_state)
}

/// Power with the opt-in donor-controlled `SelfDischarge` leg, 14 days.
pub fn power_self_discharge() -> String {
    let charge = params::charge();
    let self_discharge = params::self_discharge();
    let scenario = BOUNDED_SOC_SCENARIO;
    let (state, registry) =
        build_power(&charge, &scenario, Some(self_discharge)).expect("build_power");
    let resolver = power_resolver(&charge, &scenario).expect("power_resolver");
    let integrator = EulerIntegrator::new(registry);
    let steps = SELF_DISCHARGE_DAYS * scenario.steps_per_day;
    let (final_state, rationed, events) =
        run(&integrator, state, &resolver, scenario.dt_seconds, steps).expect("run power");

    assert_eq!(rationed, 0, "Tier-0: power self-discharge rationed must be 0");
    assert!(
        events.is_empty(),
        "Tier-0: power self-discharge events must be empty"
    );
    snapshot(&final_state)
}

/// The standalone Thermal `EQUILIBRIUM_SCENARIO` — the Stefan-Boltzmann `powf(4.0)`.
pub fn thermal() -> String {
    let p = params::thermal();
    let scenario = EQUILIBRIUM_SCENARIO;
    let (state, registry) = build_thermal(&p, &scenario).expect("build_thermal");
    let resolver = thermal_resolver(&scenario).expect("thermal_resolver");
    let integrator = EulerIntegrator::new(registry);
    let (final_state, rationed, events) = run(
        &integrator,
        state,
        &resolver,
        scenario.dt_seconds,
        EQUILIBRIUM_STEPS,
    )
    .expect("run thermal");

    assert_eq!(
        rationed, 0,
        "Tier-0: thermal rationed must be 0 (τ >> dt sizing)"
    );
    assert!(events.is_empty(), "Tier-0: thermal events must be empty");
    snapshot(&final_state)
}

fn perennial_short() -> String {
    perennial_chamber(PERENNIAL_CHAMBER_YEARS)
}
fn perennial_long() -> String {
    perennial_chamber(LONG_HORIZON_YEARS)
}
fn consumer_short() -> String {
    consumer_chamber(CONSUMER_CHAMBER_YEARS)
}
fn consumer_long() -> String {
    consumer_chamber(LONG_HORIZON_YEARS)
}

/// The eleven goldens this crate's reference produces.
///
/// ⚠ The `numerics` column mirrors `tests/test_regression_*.py`'s `@windows_golden_only`
/// placement exactly, which is a measurement rather than a fresh judgement: crew and eclss
/// carry no marker there (linear control loops, no `sin`/`powf`), everything else does.
/// `station::goldens::STATION` carries the other eight, and the four pure-arithmetic
/// goldens across both rosters are the same four Python leaves ungated.
pub const DOMAINS: &[Golden] = &[
    Golden {
        name: "season_euler_state.json",
        run: season,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "sealed_chamber_state.json",
        run: sealed_chamber,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "perennial_chamber_state.json",
        run: perennial_short,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "perennial_long_horizon_state.json",
        run: perennial_long,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "consumer_chamber_state.json",
        run: consumer_short,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "consumer_long_horizon_state.json",
        run: consumer_long,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "crew_state.json",
        run: crew,
        numerics: Numerics::PureArithmetic,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "eclss_state.json",
        run: eclss,
        numerics: Numerics::PureArithmetic,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "power_state.json",
        run: power,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "power_self_discharge_state.json",
        run: power_self_discharge,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
    Golden {
        name: "thermal_state.json",
        run: thermal,
        numerics: Numerics::Transcendental,
        cost: Cost::Cheap,
        shape: Shape::StateSnapshot,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// ⚠⚠ **These exist because the structural path is unreachable on the development
    /// box.** [`compare`] routes to [`compare_structural`] only for a `Transcendental`
    /// golden off Windows — so on the generation platform the entire Linux branch is dead
    /// code, and it would first execute on CI, on the day something diverges, with nobody
    /// ever having seen it work. A gate whose only exercise is a future failure is the
    /// green-by-skip shape wearing different clothes.
    ///
    /// So the comparator is tested directly, on hand-built pairs, on every platform.
    const GOLD: &str = r#"{
  "aux": {},
  "n": 168,
  "stocks": [
    {
      "amount": "0x1.87e90ff972484p+3",
      "id": "boundary.crew_humidity",
      "kind": "boundary",
      "unit": "kg"
    }
  ]
}
"#;

    fn structural(other: &str) -> Result<(), String> {
        compare_structural(other, GOLD)
    }

    #[test]
    fn a_last_bit_difference_in_a_hex_float_is_structurally_equal() {
        // The whole point: this is what a different libm produces, and it must pass.
        let perturbed = GOLD.replace("0x1.87e90ff972484p+3", "0x1.87e90ff972485p+3");
        assert_ne!(perturbed, GOLD, "the control must actually perturb something");
        assert_eq!(structural(&perturbed), Ok(()));
    }

    #[test]
    fn a_wildly_different_hex_float_is_still_structurally_equal() {
        // ⚠ Stated as a limitation rather than hidden: the structural comparison says
        // NOTHING about magnitude. That is why it is the off-platform fallback and not
        // the contract — the byte compare on the generation platform is the real gate,
        // and `tiers.json`'s measured bands (which no Rust program reads; plan FINDING 2)
        // are what a magnitude claim would need.
        let perturbed = GOLD.replace("0x1.87e90ff972484p+3", "0x1.5p+300");
        assert_eq!(structural(&perturbed), Ok(()));
    }

    #[test]
    fn a_changed_integer_is_caught() {
        let perturbed = GOLD.replace("168", "169");
        assert!(structural(&perturbed).unwrap_err().contains("$.n"));
    }

    #[test]
    fn a_changed_label_is_caught() {
        let perturbed = GOLD.replace("\"kg\"", "\"mol\"");
        assert!(structural(&perturbed).unwrap_err().contains("unit"));
    }

    #[test]
    fn a_renamed_stock_is_caught() {
        let perturbed = GOLD.replace("boundary.crew_humidity", "boundary.crew_water");
        assert!(structural(&perturbed).unwrap_err().contains("id"));
    }

    #[test]
    fn an_added_or_removed_key_is_caught() {
        let perturbed = GOLD.replace("  \"n\": 168,\n", "");
        assert!(structural(&perturbed).unwrap_err().contains("key count"));
    }

    #[test]
    fn an_added_or_removed_stock_is_caught() {
        let perturbed = GOLD.replace("  \"stocks\": [\n", "  \"stocks\": [\n    {},\n");
        assert!(structural(&perturbed).unwrap_err().contains("elements"));
    }

    #[test]
    fn a_reordered_key_is_caught() {
        // The goldens are `sort_keys=True`, so an order change is a real change.
        let perturbed = GOLD.replace(
            "      \"amount\": \"0x1.87e90ff972484p+3\",\n      \"id\": \"boundary.crew_humidity\",",
            "      \"id\": \"boundary.crew_humidity\",\n      \"amount\": \"0x1.87e90ff972484p+3\",",
        );
        assert!(structural(&perturbed).unwrap_err().contains("where the golden has"));
    }

    /// ⚠ A float leaf that became a label — the case a key-name rule would miss, which is
    /// why the hex-float discriminator is `hexfloat::parse` and not a list of key names.
    #[test]
    fn a_float_that_turned_into_a_label_is_caught() {
        let perturbed = GOLD.replace("\"0x1.87e90ff972484p+3\"", "\"unknown\"");
        assert!(structural(&perturbed)
            .unwrap_err()
            .contains("one side is a hex-float"));
    }

    /// The platform policy itself: a pure-arithmetic golden gets no structural mercy.
    #[test]
    fn pure_arithmetic_is_byte_compared_on_every_platform() {
        let perturbed = GOLD.replace("0x1.87e90ff972484p+3", "0x1.87e90ff972485p+3");
        assert!(matches!(
            compare(&perturbed, GOLD, Numerics::PureArithmetic),
            Verdict::Differs(_)
        ));
        assert!(matches!(
            compare(GOLD, GOLD, Numerics::PureArithmetic),
            Verdict::ByteExact
        ));
    }

    /// And identical bytes are `ByteExact` regardless of classification — the structural
    /// path is a fallback, never a substitute when the stronger answer is available.
    #[test]
    fn identical_bytes_are_byte_exact_for_a_transcendental_golden_too() {
        assert!(matches!(
            compare(GOLD, GOLD, Numerics::Transcendental),
            Verdict::ByteExact
        ));
    }
}
