//! The **partition-table switch** as one command — the sibling of `value_switch.rs` for the
//! one frozen biosphere param the value switch cannot reach.
//!
//! ```text
//! cargo run --release -q -p domains --example partition_switch -- fr=0.5,0.75,1.0,1.25,1.5
//! cargo run --release -q -p domains --example partition_switch -- fl=0.75,1.25 --long
//! ```
//!
//! `organ=f1,f2,...` scales that organ's share by each factor at **every** DVS knot, with the
//! other three shares compensated proportionally. The organs are `fl` (leaf), `fs` (stem),
//! `fr` (root) and `fo` (storage/grain).
//!
//! ⚠ **Why this is not `value_switch`.** [`config::with_override`] refuses a table-shaped
//! field before it rewrites anything, and the `+` form joins **scalar** substitutions — so
//! neither form of the value switch can spell a partition column. The September direction
//! plan priced this measurement as *"value switch, minutes"*; that price was wrong, and the
//! ⚠ claiming a `+` column could perturb the whole table at once was added by the plan's own
//! 2026-09-02 re-read. See `domains::lab::partition`.
//!
//! ⚠ **It writes nothing and takes no decision.** Same contract as the value switch: the
//! perturbed rows live in a `String` for the length of one run. Nothing here discharges
//! `allocation.yaml`'s `TODO(cite)` — "is this table a suspect?" and "where did its numbers
//! come from?" are different questions.

use domains::lab::partition::{self, ORGANS};
use domains::lab::report::{measure, render, Column};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let long = args.iter().any(|a| a == "--long");
    // ⚠ Unknown flags are rejected rather than ignored, for the reason `value_switch` gives:
    // `--lon` would silently produce the SHORT report, whose cost is a wrong reading.
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && *a != "--long") {
        fail(&format!("unknown flag {bad:?} (the only flag is --long)"));
    }
    let specs: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if specs.is_empty() {
        eprintln!(
            "usage: partition_switch organ=f1[,f2,...] [more...] [--long]\n\
             \n\
             Scales one partition organ's share by each factor at every DVS knot,\n\
             compensating the other three proportionally, and tabulates the quantities\n\
             the science gates are read off. Writes nothing.\n\
             \n\
             organs: {ORGANS:?}\n\
             example: partition_switch fr=0.5,0.75,1.0,1.25,1.5"
        );
        std::process::exit(2);
    }

    let mut columns: Vec<Column> = vec![measure(
        "frozen",
        &domains::biosphere::params::biosphere(),
        long,
    )];
    for spec in specs {
        let (organ, values) = match spec.split_once('=') {
            Some(pair) => pair,
            None => fail(&format!("{spec:?} is not `organ=f1[,f2,...]`")),
        };
        let organ = organ.trim();
        let factors: Vec<f64> = values
            .split(',')
            .map(|raw| match raw.trim().parse::<f64>() {
                Ok(v) => v,
                Err(_) => fail(&format!("{:?} is not a number", raw.trim())),
            })
            .collect();
        print!("{}", partition::render_header(organ, &factors));
        for factor in factors {
            match partition::biosphere_with_share(organ, factor) {
                Ok(p) => columns.push(measure(&partition::label_of(organ, factor), &p, long)),
                Err(e) => fail(&e.to_string()),
            }
        }
    }

    print!("{}", render(&columns, long));
}

/// Loud, never a fallback to the baseline — a harness that quietly ran the frozen tree after a
/// bad argument would report "this table does not matter", which is the finding at stake.
fn fail(message: &str) -> ! {
    eprintln!("partition_switch: {message}");
    std::process::exit(2)
}
