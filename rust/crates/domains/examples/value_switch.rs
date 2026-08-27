//! The value-switch harness as **one command** — §9 of
//! `docs/plans/post-roadmap-value-switch-harness.md`.
//!
//! ```text
//! cargo run --release -q -p domains --example value_switch -- extinction_coef=0.60,0.65,0.68
//! cargo run --release -q -p domains --example value_switch -- canopy.yaml:extinction_coef=0.65 --long
//! ```
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
use domains::lab::Substitution;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let long = args.iter().any(|a| a == "--long");
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

    let mut variants: Vec<(String, Vec<Substitution>)> = Vec::new();
    for spec in specs {
        let (target, values) = match spec.split_once('=') {
            Some(pair) => pair,
            None => fail(&format!("{spec:?} is not `field=value[,value...]`")),
        };
        for raw in values.split(',') {
            let value: f64 = match raw.trim().parse() {
                Ok(v) => v,
                Err(_) => fail(&format!("{raw:?} is not a number")),
            };
            let sub = match target.split_once(':') {
                Some((file, field)) => Substitution::new(file, field, value),
                // A bare field: refused rather than guessed when two files declare it.
                None => match Substitution::resolve(target, value) {
                    Ok(s) => s,
                    Err(e) => fail(&e.to_string()),
                },
            };
            variants.push((format!("{}:{}={}", sub.file, sub.field, value), vec![sub]));
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
