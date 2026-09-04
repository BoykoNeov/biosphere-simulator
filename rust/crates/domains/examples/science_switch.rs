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
//! # ⚠ What is reachable from a command line, and what is not
//!
//! Two things: a **knockout** by flow id, and a **temperature form** by name (`form=q10_teh`,
//! `docs/plans/post-roadmap-temperature-kinetics.md`).
//!
//! ⚠ This header said until 2026-09-04 that only the knockout was reachable, *"because there
//! is no second form of any biosphere process in this tree (§2C of the plan, measured)"*. That
//! was true when it was written and the temperature-kinetics item is what ended it — a form is
//! nameable here precisely because it rides the params object rather than a `Box<dyn Flow>`.
//!
//! The lab's replace/add composers still take a *flow*, not a name, and remain test-driven
//! only: `lab::report::compare_changes` takes them, but a command line can only name what
//! already exists.
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

use domains::biosphere::science::KineticsForm;
use domains::lab::mechanism::Composition;
use domains::lab::report::{compare_changes, render, Change};

/// The form names this command accepts — one roster, shared by the usage text and the error,
/// so adding a form cannot leave one of them behind.
const FORM_NAMES: [&str; 2] = ["cardinal", "q10_teh"];

/// `form=<name>` resolved. `cardinal` is accepted (it reproduces the baseline exactly) so a
/// reader can SEE the no-op column rather than being told it would be one.
fn kinetics_form(name: &str) -> Option<KineticsForm> {
    match name {
        "cardinal" => Some(KineticsForm::Cardinal),
        "q10_teh" => Some(KineticsForm::Q10Teh),
        _ => None,
    }
}

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
            "usage: science_switch <flow.id | form=NAME> [more...] [--long]\n\
             \n\
             Runs the frozen biosphere scenarios with each flow knocked out and tabulates the\n\
             quantities the science gates are read off. Writes nothing.\n\
             \n\
             example: science_switch biosphere.root_zone_capture\n\
             example: science_switch form=q10_teh --long\n\
             \n\
             A flow id absent from a scenario's registry is reported n/a for that scenario's\n\
             rows, not as an error: the four canonical builds do not share a flow set."
        );
        std::process::exit(2);
    }

    // ⚠ A `form=` argument that names nothing is REFUSED, never silently read as a flow id:
    // `biosphere.foo` and `form=foo` fail in different places, and a caller who typoed the
    // second would otherwise get a knockout column labelled as a form.
    let variants: Vec<(String, Change)> = ids
        .iter()
        .map(|arg| match arg.strip_prefix("form=") {
            Some(name) => match kinetics_form(name) {
                Some(form) => (format!("form {name}"), Change::Form(form)),
                None => fail(&format!(
                    "unknown temperature form {name:?} (have {FORM_NAMES:?})"
                )),
            },
            None => (
                format!("drop {arg}"),
                Change::Mechanism(Composition::dropping(&[arg.as_str()])),
            ),
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
