//! The value-switch harness as **one command** — §9 of
//! `docs/plans/post-roadmap-value-switch-harness.md`.
//!
//! ```text
//! cargo run --release -q -p domains --example value_switch -- extinction_coef=0.60,0.65,0.68
//! cargo run --release -q -p domains --example value_switch -- canopy.yaml:extinction_coef=0.65 --long
//! cargo run --release -q -p domains --example value_switch -- o2=2.0+gamma_star=0.4071
//! ```
//!
//! `,` sweeps one target into one column per value; `+` joins several targets into ONE
//! column, which is the only way to measure a FORM that moves two numbers together. The two
//! cannot be mixed in one spec — see [`domains::lab::parse_variants`].
//!
//! A bare field name resolves only when exactly one frozen file declares it; `carbon_fraction`
//! is declared by two and must be addressed as `file.yaml:field`.
//!
//! ⚠ **It writes nothing.** The substitution rewrites the `include_str!`-ed text in memory for
//! the length of one run, so no param file, golden, manifest digest or gate bound can move —
//! which is what makes an experiment cheap enough to be worth running and impossible to
//! mistake for a commitment. `--long` adds the 15-year rows, without which no
//! `liveness_floors` quantity is in the table at all (the report says so).
//!
//! ⚠ **It takes no decision.** The `extinction_coef` question this was built for is open and
//! the user's; this regenerates the evidence it was already priced on.

use domains::lab::report::{compare, render};
use domains::lab::{parse_variants, Substitution};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let long = args.iter().any(|a| a == "--long");
    // ⚠ An unknown flag is rejected, not ignored. `--lon` would otherwise silently produce the
    // SHORT report — which is precisely the one that cannot show opposed movement, so the
    // typo's cost is a wrong reading rather than a missing section.
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && *a != "--long") {
        fail(&format!("unknown flag {bad:?} (the only flag is --long)"));
    }
    let specs: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if specs.is_empty() {
        eprintln!(
            "usage: value_switch [file.yaml:]field=v1[,v2,...] [more...] [--long]\n\
             \n\
             Runs the frozen biosphere scenarios at each value and tabulates the quantities\n\
             the science gates are read off. Writes nothing.\n\
             \n\
             example: value_switch extinction_coef=0.60,0.65,0.68"
        );
        std::process::exit(2);
    }

    // ⚠ The grammar lives in `domains::lab::parse_variants`, not here. It was inline in this
    // binary until 2026-09-02, which meant the one thing it could get wrong — collapsing a
    // coupled `+` column into two independent ones — was reachable by no test at all, because
    // an `examples/` binary's `main` is not a test subject. Lifted so it is gated.
    let mut variants: Vec<(String, Vec<Substitution>)> = Vec::new();
    for spec in specs {
        match parse_variants(spec) {
            Ok(vs) => variants.extend(vs),
            Err(e) => fail(&e.to_string()),
        }
    }

    match compare(&variants, long) {
        Ok(columns) => print!("{}", render(&columns, long)),
        Err(e) => fail(&e.to_string()),
    }
}

/// ⚠ Loud, never a fallback to the baseline. A harness that quietly ran the frozen tree after
/// a bad argument would report "this parameter does not matter" — the failure §7 names.
fn fail(message: &str) -> ! {
    eprintln!("value_switch: {message}");
    std::process::exit(2)
}
