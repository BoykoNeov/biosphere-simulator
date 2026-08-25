//! The headless CLI's bit-identity gate — the reference checking its own command-line entry
//! point, ported from `tests/crossport/test_headless_cli.py` (reference flip, Stage 3, §5y
//! decision 2).
//!
//! # What is being claimed
//!
//! Phase 8's exit criterion is *"the exact same simulation runs headless"*. The `sim` binary
//! is the concrete artifact: it builds a fixed-palette session through the **same**
//! [`station::palette::build_scenario`] the Godot cdylib uses, advances it, and prints the
//! bit-exact `sim_io` hex-float snapshot. So it is the same simulation *by construction*, not
//! by an agreeing re-implementation — and this file is what stops that construction quietly
//! breaking.
//!
//! # ⚠ Why there is no `cargo` on the reference side, unlike the Python original
//!
//! The Python test shelled out twice — once for `sim`, once for `cargo run --example
//! emit_cabin_gas` — because from outside the workspace an `examples/` program is a binary and
//! a subprocess was the only way to reach it. From inside, it is not: every `emit_*` example is
//! a one-line wrapper, `print!("{}", station::goldens::cabin_gas())`, so the reference side is
//! a **function call**. That is the same bytes with no subprocess, and it stays byte-exact on
//! every platform.
//!
//! ⚠ It would have been easy to route this through the committed goldens instead — no
//! subprocess at all, one line shorter. That is a **weaker** test and was refused: off the
//! generation platform `domains::goldens::compare` falls back to a *structural* comparison for
//! transcendental goldens, so `greenhouse` would stop being byte-compared on Linux CI while
//! still looking like it was. The claim here is byte-identity, so byte-identity is what runs.
//!
//! # And the one place a nested `cargo` IS worth it
//!
//! Calling the library function assumes the examples really are thin wrappers. Nothing in
//! `rust/` referenced them before this file (grepped), so that assumption was gated by nothing
//! until the next golden regeneration. [`the_emit_examples_are_the_thin_wrappers_this_file_assumes`]
//! gates it directly — and deliberately **not** by scanning the example's source text, which is
//! the trap §5x recorded on the `gdext` rule. It runs the program and compares the bytes.

use std::process::Command;

use station::goldens;
use station::scenario::{greenhouse_scenario, CABIN_GAS_STEPS, HEAT_CLOSURE_DAYS};

/// Run `sim <scenario> <steps>` and return its stdout, asserting a clean exit.
fn sim(scenario: &str, steps: u64) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_sim"))
        .args([scenario, &steps.to_string()])
        .output()
        .expect("the `sim` binary is built by cargo for this test and must be spawnable");
    assert!(
        out.status.success(),
        "`sim {scenario} {steps}` exited {:?}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("the snapshot codec emits UTF-8")
}

/// One palette entry: the CLI's argument pair, the reference run behind it, and the
/// `emit_*` program that is supposed to be a thin wrapper over that same run.
struct Case {
    scenario: &'static str,
    steps: u64,
    reference: fn() -> String,
    example: &'static str,
}

/// The three palette entries the Python original covered: two single-rate and one two-rate.
/// The step counts are read from the frozen horizons rather than retyped, so a horizon change
/// cannot leave this file comparing a run against a different run's length.
fn cases() -> Vec<Case> {
    vec![
        Case {
            scenario: "cabin_gas",
            steps: CABIN_GAS_STEPS,
            reference: goldens::cabin_gas,
            example: "emit_cabin_gas",
        },
        Case {
            scenario: "station",
            steps: HEAT_CLOSURE_DAYS * 24,
            reference: goldens::station,
            example: "emit_station",
        },
        Case {
            scenario: "greenhouse",
            steps: greenhouse_scenario().days as u64,
            reference: goldens::greenhouse,
            example: "emit_greenhouse",
        },
    ]
}

#[test]
fn the_cli_is_byte_for_byte_the_same_simulation_as_the_reference_run() {
    for case in cases() {
        let (scenario, steps) = (case.scenario, case.steps);
        let cli = sim(scenario, steps);
        let expected = (case.reference)();
        assert_eq!(
            cli, expected,
            "`sim {scenario} {steps}` differs from the reference run byte-for-byte — the \
             headless CLI is no longer the same simulation. Both sides go through \
             `build_scenario` and the frozen `simcore::snapshot` codec, so a difference here \
             means the binary has grown a path of its own (a different builder, a different \
             horizon, or output that is no longer just the snapshot)."
        );
    }
}

#[test]
fn the_emit_examples_are_the_thin_wrappers_this_file_assumes() {
    // ⚠ The assumption every comparison above rests on, and the ONLY thing here that needs a
    // subprocess. Before this test nothing in `rust/` referenced the `emit_*` programs at all
    // — a stray `println!` in one of them would have gone unnoticed until someone regenerated
    // a golden, at which point the diff would look like the science had moved.
    //
    // Gated by RUNNING the program, never by reading its source: "the file contains one line"
    // is a text scan, and §5x is the record of a text scan being unable to express the rule it
    // was written for.
    let workspace = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    for case in cases() {
        let (scenario, example) = (case.scenario, case.example);
        let out = Command::new(env!("CARGO"))
            .args(["run", "-q", "-p", "station", "--example", example])
            .current_dir(workspace)
            .output()
            .unwrap_or_else(|e| panic!("cannot run `cargo run --example {example}`: {e}"));
        assert!(
            out.status.success(),
            "`cargo run --example {example}` exited {:?}\nstderr:\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        );
        let emitted = String::from_utf8(out.stdout).expect("the snapshot codec emits UTF-8");
        assert_eq!(
            emitted,
            (case.reference)(),
            "`{example}` no longer prints exactly `station::goldens::{scenario}()`. Every \
             comparison in this file, and the golden regeneration script, treat the example as \
             a thin wrapper over that function; if it has grown a newline or a header, the \
             goldens it writes are no longer the reference's own bytes."
        );
    }
}

#[test]
fn the_cli_rejects_bad_arguments_without_panicking() {
    // A server harness can trust the exit code. The failure mode this rules out is not "it
    // errors" but "it errors in a way a shell cannot see": a panic, or a zero exit with empty
    // output that a caller would parse as an empty snapshot.
    for args in [
        vec!["no_such_scenario", "10"],    // unknown palette entry
        vec!["cabin_gas"],                 // arity
        vec!["cabin_gas", "not-a-number"], // unparseable step count
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_sim"))
            .args(&args)
            .output()
            .expect("spawnable");
        assert!(
            !out.status.success(),
            "`sim {}` exited 0 — a bad invocation must be visible to the shell",
            args.join(" ")
        );
        assert!(
            out.stdout.is_empty(),
            "`sim {}` failed but still wrote {} bytes to stdout; a caller would parse that as \
             a snapshot",
            args.join(" "),
            out.stdout.len()
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            !stderr.contains("panicked at"),
            "`sim {}` panicked rather than exiting non-zero:\n{stderr}",
            args.join(" ")
        );
        assert!(
            !stderr.is_empty(),
            "`sim {}` failed silently — a non-zero exit with no diagnostic tells an operator \
             nothing",
            args.join(" ")
        );
    }
}
