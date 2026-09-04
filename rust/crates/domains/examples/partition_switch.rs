//! The **partition-table switch** as one command — the sibling of `value_switch.rs` for the
//! one frozen biosphere param the value switch cannot reach.
//!
//! ```text
//! cargo run --release -q -p domains --example partition_switch -- fr=0.5,0.75,1.0,1.25,1.5
//! cargo run --release -q -p domains --example partition_switch -- fl=0.75,1.25 --long
//! cargo run --release -q -p domains --example partition_switch -- fl@1=1.5,2,2.5,3 --long
//! ```
//!
//! `organ=f1,f2,...` scales that organ's share by each factor at **every** DVS knot, with the
//! other three shares compensated proportionally. The organs are `fl` (leaf), `fs` (stem),
//! `fr` (root) and `fo` (storage/grain).
//!
//! `organ@dvs=f1,f2,...` scales it at **one** knot instead, named by that knot's own `dvs`
//! value (the frozen knots are `0`, `1`, `2`). ⚠ The two forms are different experiments and
//! their columns are not rungs of one ladder — the every-knot ladder's ceiling is set by the
//! knot with the *least* headroom (`fl` is 0.55 at `dvs 0`, so ×1.818 is its top), which says
//! nothing about the knot the observable actually responds to. `open_season`'s peak LAI falls
//! at **DVS 1.306**, and `fl` at the `dvs 1` knot is 0.30, so that rung reaches ×3.333. See
//! `domains::lab::partition`'s "The second axis".
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

use domains::lab::partition::{self, Knot, ORGANS};
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
            "usage: partition_switch organ[@dvs]=f1[,f2,...] [more...] [--long]\n\
             \n\
             Scales one partition organ's share by each factor — at every DVS knot, or at\n\
             the one knot `@dvs` names — compensating the other three proportionally, and\n\
             tabulates the quantities the science gates are read off. Writes nothing.\n\
             \n\
             organs: {ORGANS:?}   knots: 0, 1, 2\n\
             example: partition_switch fr=0.5,0.75,1.0,1.25,1.5\n\
             example: partition_switch fl@1=1.5,2,2.5,3 --long"
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
        // `fl` or `fl@1` — the knot is part of the TARGET, not a flag, so a run that names
        // one cannot lose it on the way to the caption.
        let (organ, knot) = match organ.trim().split_once('@') {
            None => (organ.trim(), Knot::Every),
            Some((o, dvs)) => match dvs.trim().parse::<f64>() {
                Ok(v) => (o.trim(), Knot::At(v)),
                Err(_) => fail(&format!("{:?} is not a DVS knot value", dvs.trim())),
            },
        };
        let factors: Vec<f64> = values
            .split(',')
            .map(|raw| match raw.trim().parse::<f64>() {
                Ok(v) => v,
                Err(_) => fail(&format!("{:?} is not a number", raw.trim())),
            })
            .collect();
        print!("{}", partition::render_header(organ, knot, &factors));
        for factor in factors {
            match partition::biosphere_with_share(organ, knot, factor) {
                Ok(p) => columns.push(measure(&partition::label_of(organ, knot, factor), &p, long)),
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
