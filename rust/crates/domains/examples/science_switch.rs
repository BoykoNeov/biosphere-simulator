//! The **mechanism**-switch harness as one command — slice 4 of
//! `docs/plans/post-roadmap-science-switch.md`.
//!
//! ```text
//! cargo run --release -q -p domains --example science_switch -- biosphere.root_zone_capture
//! cargo run --release -q -p domains --example science_switch -- biosphere.decomposition --long
//! ```
//!
//! Each argument is a flow id to **knock out**, and each becomes one column of the same table
//! `value_switch` prints — the renderer is shared, deliberately: a mechanism's effect and a
//! coefficient's effect are read off the same quantities against the same recorded bounds, and
//! two renderers would be two copies of the rule for reading them.
//!
//! # ⚠ Why only the knockout is reachable from a command line
//!
//! The lab's other two composers take a *flow*, not a name — an alternative form of a process
//! is code, and there is no second form of any biosphere process in this tree (§2C of the
//! plan, measured). `lab::report::compare_changes` takes replacements and additions and the
//! tests drive them; a command line can only name what already exists.
//!
//! # ⚠ A column can be blank, and that is a result rather than a fault
//!
//! The frozen scenarios do not share a flow set: ten of the twenty-three biosphere flows are
//! in all four canonical builds, and the soil-carbon and nitrogen processes are in the
//! chambers only. So knocking out `biosphere.decomposition` cannot be asked of the open field,
//! and those rows print `n/a` with the reason rather than vanishing.
//!
//! ⚠ **It writes nothing and it takes no decision.** A knockout regenerates evidence about a
//! mechanism's contribution; it says nothing about whether the mechanism belongs there.

use domains::lab::mechanism::Composition;
use domains::lab::report::{compare_changes, render, Change};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let long = args.iter().any(|a| a == "--long");
    // ⚠ An unknown flag is rejected, not ignored — `--lon` would otherwise silently give the
    // SHORT report, which is the one that cannot show opposed movement.
    if let Some(bad) = args.iter().find(|a| a.starts_with("--") && *a != "--long") {
        fail(&format!("unknown flag {bad:?} (the only flag is --long)"));
    }
    let ids: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    if ids.is_empty() {
        eprintln!(
            "usage: science_switch <flow.id> [more flow ids...] [--long]\n\
             \n\
             Runs the frozen biosphere scenarios with each flow knocked out and tabulates the\n\
             quantities the science gates are read off. Writes nothing.\n\
             \n\
             example: science_switch biosphere.root_zone_capture\n\
             \n\
             A flow id absent from a scenario's registry is reported n/a for that scenario's\n\
             rows, not as an error: the four canonical builds do not share a flow set."
        );
        std::process::exit(2);
    }

    let variants: Vec<(String, Change)> = ids
        .iter()
        .map(|id| {
            (
                format!("drop {id}"),
                Change::Mechanism(Composition::dropping(&[id.as_str()])),
            )
        })
        .collect();

    match compare_changes(&variants, long) {
        Ok(columns) => print!("{}", render(&columns, long)),
        Err(e) => fail(&e.to_string()),
    }
}

/// ⚠ Loud, never a fallback to the baseline. A harness that quietly ran the frozen tree after
/// a bad argument would report "this mechanism does not matter" — the failure the composers'
/// own mis-target guards exist to prevent, re-introduced at the command line.
fn fail(message: &str) -> ! {
    eprintln!("science_switch: {message}");
    std::process::exit(2)
}
