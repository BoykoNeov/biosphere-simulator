//! Interpret + run an authored scenario **file** through the Rust `authoring` port and
//! emit its final `State` as `sim_io`-shaped JSON (Phase 9, Step 4b).
//!
//! Usage: `cargo run --example emit_authored -- <scenario.yaml> [param=value ...]`
//!
//! This is the file-level parse-parity + byte-identity driver: the crossport test runs
//! it on each anchor and diffs the JSON against the frozen golden (crew /
//! template@1.0 → `crew_state.json` byte-for-byte, transcendental-free ⇒ Tier-1) or
//! against the Python interpreter's own run of the same file (every anchor,
//! Rust-parse ≡ Python-parse ≡ same trajectory). Tier-0 invariants (`rationed == 0`,
//! `events == ()`, conservation every step) are asserted here in Rust — a completed
//! run is itself the conservation proof (the every-step gate is inside `step_report`).

use std::collections::BTreeMap;
use std::path::PathBuf;

use authoring::{load_scenario, run_scenario};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(
        args.next()
            .expect("usage: emit_authored <scenario.yaml> [param=value ...]"),
    );
    let overrides = parse_overrides(args);

    let built = load_scenario(&path, &overrides).expect("load_scenario");
    let result = run_scenario(built).expect("run_scenario");

    assert_eq!(
        result.total_rationed, 0,
        "Tier-0: authored scenario rationed must be 0 (well-fed / structural)"
    );
    assert!(
        result.events.is_empty(),
        "Tier-0: authored scenario events must be empty"
    );

    print!(
        "{}",
        simcore::snapshot::from_engine(&result.final_state).to_json()
    );
}

/// Parse `param=value` CLI overrides into the template-parameter map.
fn parse_overrides(args: impl Iterator<Item = String>) -> BTreeMap<String, f64> {
    let mut overrides = BTreeMap::new();
    for arg in args {
        let (name, value) = arg
            .split_once('=')
            .unwrap_or_else(|| panic!("override {arg:?} is not name=value"));
        let value: f64 = value
            .parse()
            .unwrap_or_else(|_| panic!("override {arg:?} value is not a number"));
        overrides.insert(name.to_string(), value);
    }
    overrides
}
