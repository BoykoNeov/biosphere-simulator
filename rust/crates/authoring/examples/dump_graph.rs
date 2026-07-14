//! Interpret an authored scenario **file** through the Rust `authoring` port and emit
//! its canonical structural [`graph_dump`] (Phase 9, Step 4b).
//!
//! Usage: `cargo run --example dump_graph -- <scenario.yaml> [param=value ...]`
//!
//! The Tier-0 file-level parse-parity surface: the crossport test diffs this against
//! the Python `render_graph_dump` of the *same* file, catching graph facts a
//! final-state snapshot is blind to (flow priorities, present-but-inert flows, the
//! bit-exact boundary-eval of an initial amount / forcing constant).
//!
//! [`graph_dump`]: authoring::graph_dump

use std::collections::BTreeMap;
use std::path::PathBuf;

use authoring::{load_scenario, render_graph_dump};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = PathBuf::from(
        args.next()
            .expect("usage: dump_graph <scenario.yaml> [param=value ...]"),
    );
    let overrides = parse_overrides(args);

    let built = load_scenario(&path, &overrides).expect("load_scenario");
    print!("{}", render_graph_dump(&built));
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
