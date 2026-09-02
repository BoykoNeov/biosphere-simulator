//! Regenerate the regression goldens **from the reference** — the blessed path, ported to
//! Rust by Stage-3 slice **S6, build item 2** (`docs/plans/post-roadmap-reference-flip.md`).
//!
//! # What this replaces, and why it had to move
//!
//! `tests/crossport/regen_goldens_from_rust.py` was *"the committed, reviewable entry point
//! for the act the plan names — the goldens are generated from Rust"*, and slice 5 made it
//! **the only one**. It was a Python driver that shelled `cargo run -q -p <crate> --example
//! <emitter>` nineteen times and compared stdout against the committed bytes. That shape was
//! forced: an `examples/` program is a binary target, so Python was the only caller that
//! could reach the runs at all. Slice S2 moved the runs into [`domains::goldens`] and
//! [`crate::goldens`], where the example and this tool both call the same functions — so the
//! driver no longer needs a process boundary, and with it goes the `-p <crate> --example`
//! discipline the Python tool had to carry as prose (the `emit_crew` name collision, where a
//! built binary path could reach `simcore`'s codec fixture and rewrite the golden *from
//! itself*). A function pointer cannot be ambiguous.
//!
//! # ⚠⚠ The write path, and the check S6 recorded losing
//!
//! Until S6 every candidate was validated before it could reach the disk —
//! `sim_io.snapshot.loads(produced)`, i.e. *"a golden that does not round-trip must never be
//! written"*. `src/sim_io` was deleted, so `--write` started refusing rather than writing
//! unvalidated bytes over a freeze contract's values. This restores it, through
//! [`domains::goldens::validate`] and the declared [`domains::goldens::Shape`].
//!
//! ⚠ And it restores it **fixed**. The Python check was per-candidate and inline, so a
//! failure aborted mid-loop with earlier goldens already rewritten. Measured: that failure
//! was not hypothetical — `sealed_energy_drift_summary.json` has had no `version` key since
//! C5 moved it into the emitted group, and the validator raised on a missing one, so a real
//! `--write` would have died part-way through. Here **every** selected golden is run and
//! validated before **any** byte is written.
//!
//! # ⚠ Reporting is the default; `--write` is explicit
//!
//! Unchanged from the Python tool, and for its reason: rewriting a golden is a deliberate
//! act whose diff is reviewed, never a side effect of running something. ⚠ Twenty of the
//! twenty-one goldens are named by a **freeze manifest**, so a `--write` that moves one
//! desynchronises that contract's `golden_sha256` — re-run the manifest writer as part of
//! the unfreeze ceremony (`cargo run -p <crate> --example dump_*_inventory --
//! --write-manifest`) and review both diffs.
//!
//! ⚠ **This is a tool and not a gate.** It always exits 0; the gate that a run still
//! produces the committed bytes is `{domains,station}/tests/golden_regression.rs`, which
//! runs in every `cargo test`.
//!
//! # ⚠ Profile
//!
//! Byte-neutral — measured by the Python tool across debug and `--release` for all
//! nineteen. It is a speed choice only, and it is a large one: `sealed_station_state.json`
//! is ~1.3 M sub-steps and was measured at 378 s on the stock dev profile against 93 s in
//! release. Run this tool with `--release`, or narrow it with `--only`.

use std::path::{Path, PathBuf};

use domains::goldens::{compare, golden_dir, validate, Golden, Verdict};

use crate::goldens::all;

/// What the command line asked for. As in `config::manifest_cli`, the parse is a library
/// function because an `examples/` program is a binary target and no test can call it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// Write the changed goldens. Off by default — see the module header.
    pub write: bool,
    /// Restrict to goldens whose name contains this substring.
    ///
    /// ⚠ **A narrowed run is a narrowed report, and the summary says so.** It exists
    /// because a full run is minutes: regenerating the one golden a ceremony moved should
    /// not cost a 1.3 M-sub-step station run. The hazard it introduces — believing "0
    /// changed" covers the directory when it covered one file — is answered by the
    /// summary line naming the filter, not by leaving the option out.
    pub only: Option<String>,
}

/// Parse the tool's arguments, or return the usage text to print on stderr.
///
/// ⚠ An argument this does not understand is an error, never a fall-through: the two wrong
/// outcomes are silently reporting on nothing (`--only` typo'd) and silently *writing*.
pub fn parse_args(args: &[String]) -> Result<Request, String> {
    const USAGE: &str = "usage: regen_goldens [--only <substring>] [--write]\n  \
         (no argument reports the diff against the committed goldens and writes nothing)\n  \
         --write rewrites the changed ones; 20 of the 21 are named by a freeze manifest, so \
         re-run that contract's manifest writer and review both diffs";
    let mut request = Request {
        write: false,
        only: None,
    };
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "--write" => request.write = true,
            "--only" => {
                let value = rest
                    .next()
                    .ok_or_else(|| format!("{USAGE}\n--only needs a substring"))?;
                request.only = Some(value.clone());
            }
            other => return Err(format!("{USAGE}\nunknown argument: {other}")),
        }
    }
    Ok(request)
}

/// The goldens a request selects, in roster order.
///
/// ⚠ Separated from [`regenerate`] so the filter can be tested without running anything —
/// the runs are minutes and the selection is the part that can be silently wrong.
pub fn select(only: Option<&str>) -> Vec<&'static Golden> {
    all()
        .into_iter()
        .filter(|g| only.is_none_or(|needle| g.name.contains(needle)))
        .collect()
}

/// What happened to one golden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    pub name: &'static str,
    /// The freshly produced bytes differ from the committed ones **and** the gate would
    /// call that a change: a real difference, or a byte difference on the generation
    /// platform, where the gate accepts nothing less than byte-exact.
    pub changed: bool,
    /// The bytes differ only in the last bits of hex-float leaves, on a
    /// [`domains::goldens::Numerics::Transcendental`] golden **off** its generation
    /// platform — the state `golden_regression.rs` accepts as green, and the state this
    /// tool used to report as `CHANGED`. Never true on Windows, and never true together
    /// with `changed`.
    pub ulp_only: bool,
    /// The file was rewritten (only ever true under [`Request::write`]).
    pub written: bool,
}

/// Run the selected goldens, report, and — under `--write` — rewrite the changed ones.
///
/// ⚠⚠ **Two phases, and the split is the fix for a real defect.** Everything is run and
/// validated first; nothing is written until all of it has passed. The Python original
/// validated inline, so a bad candidate aborted the loop with earlier goldens already on
/// disk — and that abort was reachable, not hypothetical (module header).
///
/// Returns `Err` with the validation failure rather than writing anything.
pub fn regenerate(request: &Request) -> Result<Vec<Outcome>, String> {
    let selected = select(request.only.as_deref());
    if selected.is_empty() {
        return Err(format!(
            "--only {:?} matched no golden; the roster is {:?}",
            request.only.as_deref().unwrap_or(""),
            all().iter().map(|g| g.name).collect::<Vec<_>>()
        ));
    }
    regenerate_in(&golden_dir(), &selected, request.write)
}

/// The two-phase act itself, against an explicit directory.
///
/// ⚠⚠ **The directory is a parameter so the abort can be TESTED**, and that is the whole
/// reason this function is separate from [`regenerate`]. The claim above — *nothing is
/// written unless everything validated* — is exactly the claim the Python original got
/// wrong, so asserting it needs a run where validation fails **after** a changed golden has
/// already been compared. Against `rust/data/golden/` that test would rewrite a freeze
/// contract's values to find out whether it rewrites them. Against a temp directory it is
/// an ordinary test. See `station/tests/regen.rs`.
pub fn regenerate_in(
    dir: &Path,
    selected: &[&'static Golden],
    write: bool,
) -> Result<Vec<Outcome>, String> {
    // Phase 1 — run and validate everything.
    let mut produced: Vec<(&'static Golden, String)> = Vec::with_capacity(selected.len());
    for golden in selected {
        let text = (golden.run)();
        validate(golden.name, &text, golden.shape).map_err(|why| {
            format!(
                "REFUSED — nothing was written. {why}
⚠ A golden that is not a well-formed                  artifact of its declared shape must never reach the disk: these files are a                  freeze contract's values. Fix the emitter, not this check."
            )
        })?;
        produced.push((golden, text));
    }

    // Phase 2 — compare, and write only now.
    let mut outcomes = Vec::with_capacity(produced.len());
    for (golden, text) in produced {
        let path = dir.join(golden.name);
        let current = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read committed golden {}: {e}", path.display()))?;
        // ⚠ The SAME verdict the gate reaches, not a bare byte compare. Measured on
        // 2026-09-02 (Linux, the CI platform): a byte compare on the untouched tree
        // reported ELEVEN of nineteen goldens `CHANGED` while `cargo test` was green —
        // every transcendental golden is UCRT-minted, so off Windows it is last-ULP
        // different by design, and the gate compares those STRUCTURALLY. A report that
        // calls the reference "moved" on a tree nothing moved cannot be used as the
        // control an unfreeze ceremony needs ("predict the diff before running it"),
        // and a `--write` of those eleven would have re-minted them on the wrong platform
        // and turned the byte-exact Windows gate red.
        let (changed, ulp_only) = match compare(&text, &current, golden.numerics) {
            Verdict::ByteExact => (false, false),
            Verdict::StructurallyEqual => (false, true),
            Verdict::Differs(_) => (true, false),
        };
        let mut written = false;
        if changed {
            // ⚠ Since slice 5 the golden IS this code's output, so any diff here is the
            // reference itself moving — never "the ports have drifted". Review it as a
            // science change, and re-run the freeze-manifest ceremony if it is frozen.
            println!(
                "  CHANGED    {}  [the reference has moved — review the diff]",
                golden.name
            );
            if write {
                std::fs::write(&path, text.as_bytes())
                    .map_err(|e| format!("cannot write {}: {e}", path.display()))?;
                written = true;
            }
        } else if ulp_only {
            // Not written even under `--write`: the bytes on disk are the generation
            // platform's and the gate accepts them here; ours would fail there.
            println!(
                "  ulp-only   {}  [last-bit libm difference off the generation platform — \
                 structurally equal, the gate is green; NOT rewritten]",
                golden.name
            );
        } else {
            println!("  identical  {}", golden.name);
        }
        outcomes.push(Outcome {
            name: golden.name,
            changed,
            ulp_only,
            written,
        });
    }
    Ok(outcomes)
}

/// The closing summary — separated from [`regenerate`] so its wording is testable and so a
/// narrowed run always says it was narrowed.
pub fn summary(request: &Request, outcomes: &[Outcome]) -> String {
    let changed = outcomes.iter().filter(|o| o.changed).count();
    let verb = if request.write {
        "rewritten"
    } else {
        "would change"
    };
    let scope = match request.only.as_deref() {
        Some(needle) => format!(" (⚠ NARROWED to names containing {needle:?})"),
        None => String::new(),
    };
    let ulp_only = outcomes.iter().filter(|o| o.ulp_only).count();
    let mut out = format!(
        "\n{} of {} goldens run{}; {changed} {verb}.",
        outcomes.len(),
        all().len(),
        scope
    );
    if ulp_only > 0 {
        out.push_str(&format!(
            "\n⚠ {ulp_only} differ only in the last bits of their floats: transcendental \
             goldens compared off their generation platform. The gate accepts them and \
             this tool never rewrites them — regenerate those on the generation platform."
        ));
    }
    if !request.write && changed > 0 {
        out.push_str(
            "\nRe-run with --write to rewrite them, then review the diff — and re-run the \
             manifest writer for whichever freeze contract names them.",
        );
    }
    out
}

/// Where the goldens live, re-exported so the example can name it in its output without
/// reaching for `domains`.
pub fn target_dir() -> PathBuf {
    golden_dir()
}
