//! The cross-boundary proof: **Rust inside Godot versus Rust headless**, driven from Rust.
//!
//! Ported from the nine `tests/crossport/test_godot_*.py` modules (reference flip, Stage 3,
//! §5y decision 3). Seventeen tests, twelve GDScript smokes, one harness.
//!
//! # What only this file can check
//!
//! `station/tests/session_parity.rs` proves `N × session.step() == run_station(N)` **inside
//! one cargo process** — same compiled code, same FP environment — so it is structurally
//! blind to Godot-hosted-versus-headless divergence, which is exactly what Phase 8's exit
//! criterion (*"the exact same simulation runs headless"*) rides on. The concrete break risk
//! is per-thread FP control flags: a game engine that sets **FTZ/DAZ** (flush-to-zero /
//! denormals-are-zero in MXCSR) for SIMD throughput would flush denormal intermediates and
//! diverge. Every smoke below therefore reads `fp_clean()` on the stepping thread as well as
//! comparing bytes — a passing snapshot alone does not prove flush-to-zero is off, because a
//! scenario may simply never produce a denormal.
//!
//! # ⚠ The classification that sent these here was wrong about where they run
//!
//! §5q filed these **D — no natural Rust home** partly on the reading that they are
//! *"`skipif`-ed on CI, so they are local-only today"*. False since Phase 8 Step 8: the
//! `godot-parity` CI job installs headless Godot 4.7 and runs fifteen of the seventeen on
//! every push and every pull request. Only the sealed season-crossing pair is
//! mandatory-local. So this port replaces a **running** gate, not a dormant one, and the CI
//! job swaps `pytest` for `cargo test` rather than gaining or losing coverage.
//!
//! # Skip semantics, deliberately not `#[ignore]`
//!
//! Godot and cargo may both be absent (that is the normal state of a machine that only builds
//! the engine). Rust has no `skip`, so [`available`] returns false and each test prints to
//! stderr and returns. `#[ignore]` was refused for the reason §5u recorded: it is **opt-in**,
//! so it would silence these tests on the developer machine that is the one place the
//! mandatory-local pair can run at all.
//!
//! ⚠ And the slow pair is **not** `#[ignore]` either, for the same reason: pytest's `-m slow`
//! is opt-**out** (it runs locally by default and CI excludes it). The equivalent is a name
//! filter in the CI job, not an attribute that would flip the default on the developer's
//! machine.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use simcore::json::{self, JsonValue};

// --------------------------------------------------------------------------- //
// Horizons — read from the reference where one exists, mirrored where it does not //
// --------------------------------------------------------------------------- //

use station::scenario::{greenhouse_scenario, CABIN_GAS_STEPS, HEAT_CLOSURE_DAYS};

/// `godot/greenhouse_smoke.gd` and `godot/sealed_smoke.gd` step MASTER DAYS, but the
/// `step_count` they report is the session's `n` — the **slow** domain's step count.
fn steps_for(days: u64) -> u64 {
    days * domains::biosphere::STEPS_PER_DAY as u64
}

const SEALED_RESUME_DAYS: u64 = 310;
const SAVE_LOAD_STEPS: u64 = 900;
const PERTURBED_STEPS: u64 = 288;
const COMPOSED_STEPS: u64 = 168;
const CREW_STEPS: u64 = 168;

// --------------------------------------------------------------------------- //
// The harness                                                                  //
// --------------------------------------------------------------------------- //

const SMOKE: (&str, &str) = ("<<<GODOT_SMOKE_BEGIN", "GODOT_SMOKE_END>>>");
const FLOW: (&str, &str) = ("<<<FLOW_SMOKE_BEGIN", "FLOW_SMOKE_END>>>");
const MAIN_UI: (&str, &str) = ("<<<MAIN_UI_SMOKE_BEGIN", "MAIN_UI_SMOKE_END>>>");
const UI: (&str, &str) = ("<<<UI_SMOKE_BEGIN", "UI_SMOKE_END>>>");

/// ⚠ Built by walking UP from the manifest dir, never by appending `"/../.."`.
/// `Path::parent` is **lexical**: on a path that already ends in `..` it strips that
/// component instead of resolving it, so `<crate>/../..` had a parent of `<crate>/..` and the
/// Godot project resolved to `crates/godot_bridge/../godot`. Caught by
/// `the_godot_lookup_agrees_with_the_environment` on the first run, which is the whole reason
/// that test exists.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("crates/godot_bridge is three levels below the repository root")
        .to_path_buf()
}

fn workspace() -> PathBuf {
    repo_root().join("rust")
}

fn godot_project() -> PathBuf {
    repo_root().join("godot")
}

/// `shutil.which("godot")`, hand-rolled: scan `PATH` for the executable.
fn godot_exe() -> Option<PathBuf> {
    let names: &[&str] = if cfg!(windows) {
        &["godot.exe", "godot.cmd", "godot.bat"]
    } else {
        &["godot"]
    };
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        for name in names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// Both tools present. When either is missing the smokes cannot run at all, which is the
/// normal state of a machine that only builds the engine.
fn available() -> bool {
    godot_exe().is_some()
}

/// Announce a non-run in the loudest form a Rust test has. Every early return goes through
/// this, so "it passed" and "it did not run" are never the same line of output.
fn not_run(what: &str) {
    eprintln!(
        "{what}: Godot not found on PATH — the cross-boundary smoke DID NOT RUN. This is \
         expected on a machine without the front-end toolchain; the `godot-parity` CI job \
         installs it."
    );
}

/// Run a child to completion with a wall-clock bound, draining both pipes so a chatty child
/// cannot fill a pipe and deadlock.
///
/// ⚠ Ported deliberately rather than dropped. The Python original passed `timeout=` to every
/// `subprocess.run`, and a hung headless Godot with no bound is a CI job that burns its whole
/// budget before anyone learns which test hung.
fn run_bounded(mut child: Child, limit: Duration, what: &str) -> (bool, String, String) {
    let mut out = child.stdout.take().expect("stdout piped");
    let mut err = child.stderr.take().expect("stderr piped");
    let out_thread = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let err_thread = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });

    let deadline = Instant::now() + limit;
    let status = loop {
        match child.try_wait().expect("try_wait") {
            Some(status) => break Some(status),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            None => std::thread::sleep(Duration::from_millis(25)),
        }
    };

    let stdout = out_thread.join().unwrap_or_default();
    let stderr = err_thread.join().unwrap_or_default();
    assert!(
        status.is_some(),
        "{what} exceeded {limit:?} and was killed.\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    (status.expect("checked above").success(), stdout, stderr)
}

/// Build the cdylib Godot `dlopen`s.
///
/// ⚠ `cargo test -p godot_bridge` does **not** produce it — measured, not assumed: with
/// `crate-type = ["cdylib"]` the test profile builds a test harness, and deleting
/// `target/debug/godot_bridge.dll` and re-running `cargo test -p godot_bridge` leaves it
/// absent. So the build has to happen here, exactly as the Python driver did it. A nested
/// cargo does not deadlock on the build lock — also measured, with a throwaway probe, before
/// this file was written.
fn build_cdylib() {
    let child = Command::new(env!("CARGO"))
        .args(["build", "-q", "-p", "godot_bridge"])
        .current_dir(workspace())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cargo is spawnable from a cargo test");
    let (ok, _, stderr) = run_bounded(
        child,
        Duration::from_secs(900),
        "cargo build -p godot_bridge",
    );
    assert!(ok, "cargo build -p godot_bridge failed:\n{stderr}");
}

/// A fresh checkout needs one import pass before a `--script` run can see the `SimSession`
/// class, so that `.godot/extension_list.cfg` registers the `.gdextension`.
fn ensure_godot_import() {
    let ext_list = godot_project().join(".godot").join("extension_list.cfg");
    if ext_list.exists() {
        return;
    }
    let child = Command::new(godot_exe().expect("checked by available()"))
        .args(["--headless", "--path"])
        .arg(godot_project())
        .arg("--import")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("godot is spawnable");
    // `--import` may exit non-zero on benign editor warnings; the real gate is that the
    // extension list got written.
    let (_, stdout, stderr) = run_bounded(child, Duration::from_secs(300), "godot --import");
    assert!(
        ext_list.exists(),
        "Godot import did not register the extension:\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

/// Run one GDScript smoke through the actual cdylib and return `(report, stderr)`.
/// A scenario path (or two) handed to the GDScript after `--`.
///
/// ⚠ Three of the twelve smokes are **useless without one**. Omitting them did not crash:
/// `from_file`, the template override and the authored marker each printed a well-formed
/// report with `ok: false` and zeroed numbers, which a laxer port (markers present, JSON
/// parses) would have called a pass. They were caught because the assertions read the
/// report's own `ok` flag rather than trusting that a report existed at all.
fn run_smoke_with(
    script: &str,
    markers: (&str, &str),
    limit: Duration,
    args: &[PathBuf],
) -> (JsonValue, String) {
    build_cdylib();
    ensure_godot_import();
    let child = Command::new(godot_exe().expect("checked by available()"))
        .args(["--headless", "--path"])
        .arg(godot_project())
        .arg("--script")
        .arg(format!("res://{script}"))
        .args(if args.is_empty() {
            vec![]
        } else {
            vec!["--".to_string()]
        })
        .args(
            args.iter()
                .map(|p| p.to_str().expect("utf-8 scenario path").to_string()),
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("godot is spawnable");
    let (_, stdout, stderr) = run_bounded(child, limit, &format!("godot --script {script}"));

    let (begin, end) = markers;
    let start = stdout.find(begin).unwrap_or_else(|| {
        panic!(
            "smoke markers missing in {script} — the extension may not have loaded:\n\
             stdout:\n{stdout}\nstderr:\n{stderr}"
        )
    }) + begin.len();
    let stop = stdout[start..]
        .find(end)
        .unwrap_or_else(|| panic!("{script} printed {begin} but never {end}:\nstdout:\n{stdout}"))
        + start;
    let payload = stdout[start..stop].trim();
    let report = json::parse(payload)
        .unwrap_or_else(|e| panic!("{script} report is not JSON ({e:?}):\n{payload}"));
    (report, stderr)
}

/// The nine smokes that need no scenario argument.
fn run_smoke(script: &str, markers: (&str, &str), limit: Duration) -> (JsonValue, String) {
    run_smoke_with(script, markers, limit, &[])
}

/// The authored / frozen scenario files the file-loading smokes are pointed at.
fn scenario(name: &str) -> PathBuf {
    let path = repo_root()
        .join("rust")
        .join("data")
        .join("scenarios")
        .join(name);
    assert!(
        path.is_file(),
        "the smoke needs {} and it is not there",
        path.display()
    );
    path
}

/// The headless reference: an `emit_*` program's stdout.
fn emit(package: &str, example: &str, args: &[&str]) -> String {
    let mut cmd = Command::new(env!("CARGO"));
    cmd.args(["run", "-q", "-p", package, "--example", example]);
    if !args.is_empty() {
        cmd.arg("--").args(args);
    }
    let child = cmd
        .current_dir(workspace())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cargo is spawnable");
    let (ok, stdout, stderr) = run_bounded(child, Duration::from_secs(1800), example);
    assert!(ok, "{example} failed:\n{stderr}");
    stdout
}

// --------------------------------------------------------------------------- //
// Report accessors — a missing field is a failure, never a default             //
// --------------------------------------------------------------------------- //

fn field<'a>(report: &'a JsonValue, key: &str) -> &'a JsonValue {
    match report {
        JsonValue::Object(entries) => entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
            .unwrap_or_else(|| panic!("the smoke report has no field {key:?}: {report:?}")),
        other => panic!("the smoke report is not an object: {other:?}"),
    }
}

fn as_bool(report: &JsonValue, key: &str) -> bool {
    match field(report, key) {
        JsonValue::Bool(b) => *b,
        other => panic!("{key} is not a bool: {other:?}"),
    }
}

/// ⚠ `simcore::json` keeps a number as its **raw lexeme** and offers `as_u64` / `as_i64`
/// only — there is no `as_f64`, because the snapshot's floats are hex-float STRINGS and an
/// f64 accessor would invite reading them with the wrong one. The smoke reports are ordinary
/// JSON, so the lexeme is parsed here, and a lexeme that will not parse is a failure rather
/// than a zero.
fn as_num(report: &JsonValue, key: &str) -> f64 {
    match field(report, key) {
        JsonValue::Number(lexeme) => lexeme
            .parse::<f64>()
            .unwrap_or_else(|e| panic!("{key} = {lexeme:?} is not a number: {e}")),
        other => panic!("{key} is not a number: {other:?}"),
    }
}

fn as_str<'a>(report: &'a JsonValue, key: &str) -> &'a str {
    match field(report, key) {
        JsonValue::Str(s) => s.as_str(),
        other => panic!("{key} is not a string: {other:?}"),
    }
}

fn as_array<'a>(report: &'a JsonValue, key: &str) -> &'a [JsonValue] {
    match field(report, key) {
        JsonValue::Array(items) => items.as_slice(),
        other => panic!("{key} is not an array: {other:?}"),
    }
}

fn strings(report: &JsonValue, key: &str) -> Vec<String> {
    as_array(report, key)
        .iter()
        .map(|v| match v {
            JsonValue::Str(s) => s.clone(),
            other => panic!("{key} holds a non-string {other:?}"),
        })
        .collect()
}

/// `ok` plus the FP-environment read every stepping smoke carries.
fn assert_ok_and_fp_clean(report: &JsonValue, what: &str) {
    assert!(
        as_bool(report, "ok"),
        "{what} did not complete ok: {report:?}"
    );
    assert!(
        as_bool(report, "fp_clean"),
        "{what}: FTZ/DAZ are NOT both off on the stepping thread. A game engine that flushes \
         denormals for SIMD throughput diverges from the IEEE-default headless run, and the \
         byte comparison alone cannot see it when a scenario never produces a denormal."
    );
}

/// Compare a Godot-produced snapshot against the headless reference, byte for byte.
fn assert_same_snapshot(report: &JsonValue, headless: &str, what: &str) {
    // ⚠ NOT trimmed, and this is the whole assertion. The first draft compared
    // `produced.trim()` against `headless.trim()`, which reads as tidiness and is a
    // narrowing: a control that made the headless reference print a trailing newline left
    // this test GREEN. The Python original compares the two raw strings (only the JSON
    // envelope between the markers is stripped, which `run_smoke_with` does too), and
    // "byte-for-byte" has to mean bytes or it means nothing.
    let produced = as_str(report, "snapshot");
    assert_eq!(
        produced, headless,
        "{what}: the Godot-hosted snapshot is not byte-identical to the headless reference. \
         Both sides are pure Rust through the frozen `simcore::snapshot` codec, so this is \
         the FFI boundary changing the simulation."
    );
}

/// The frozen-reference bind: the same snapshot also matches a committed golden, compared on
/// parsed f64 (Tier 1, bit-exact) rather than on JSON bytes.
fn assert_matches_golden(report: &JsonValue, golden: &str, what: &str) {
    let produced = as_str(report, "snapshot");
    let committed = domains::goldens::committed(golden);
    match domains::goldens::compare(
        produced,
        &committed,
        domains::goldens::Numerics::PureArithmetic,
    ) {
        domains::goldens::Verdict::ByteExact | domains::goldens::Verdict::StructurallyEqual => {}
        domains::goldens::Verdict::Differs(why) => {
            panic!("{what}: the Godot-hosted run does not match the frozen golden {golden}.\n{why}")
        }
    }
}

// --------------------------------------------------------------------------- //
// 1. The stepping smokes — bit-exact across the FFI boundary                   //
// --------------------------------------------------------------------------- //

#[test]
fn cabin_gas_crosses_the_boundary_bit_exact() {
    if !available() {
        return not_run("cabin_gas_crosses_the_boundary_bit_exact");
    }
    let (report, _) = run_smoke("smoke.gd", SMOKE, Duration::from_secs(300));
    assert_ok_and_fp_clean(&report, "cabin_gas smoke");
    assert_eq!(as_str(&report, "scenario"), "cabin_gas");
    assert_eq!(as_num(&report, "step_count") as u64, CABIN_GAS_STEPS);
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    assert_same_snapshot(
        &report,
        &emit("station", "emit_cabin_gas", &[]),
        "cabin_gas",
    );
    assert_matches_golden(&report, "cabin_gas_state.json", "cabin_gas");
}

#[test]
fn greenhouse_two_rate_crosses_the_boundary() {
    if !available() {
        return not_run("greenhouse_two_rate_crosses_the_boundary");
    }
    let (report, _) = run_smoke("greenhouse_smoke.gd", SMOKE, Duration::from_secs(900));
    assert_ok_and_fp_clean(&report, "greenhouse smoke");
    assert_eq!(as_str(&report, "scenario"), "greenhouse");
    assert_eq!(
        as_num(&report, "step_count") as u64,
        steps_for(greenhouse_scenario().days as u64)
    );
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    assert_same_snapshot(
        &report,
        &emit("station", "emit_greenhouse", &[]),
        "greenhouse",
    );
}

/// ⚠ The mandatory-local one — ~450k sub-steps, several minutes. The CI job excludes it **by
/// name**, not with `#[ignore]`: pytest's `-m slow` is opt-out, so an attribute here would
/// have silently stopped it running on the only machines that ever run it.
#[test]
fn sealed_season_crossing_crosses_the_boundary() {
    if !available() {
        return not_run("sealed_season_crossing_crosses_the_boundary");
    }
    let (report, _) = run_smoke("sealed_smoke.gd", SMOKE, Duration::from_secs(3600));
    assert_ok_and_fp_clean(&report, "sealed smoke");
    assert_eq!(as_str(&report, "scenario"), "sealed");
    assert_eq!(
        as_num(&report, "step_count") as u64,
        steps_for(SEALED_RESUME_DAYS)
    );
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    assert_same_snapshot(
        &report,
        &emit("station", "emit_sealed_resume", &[]),
        "sealed resume",
    );
}

#[test]
fn the_perturbed_brownout_crosses_the_boundary() {
    if !available() {
        return not_run("the_perturbed_brownout_crosses_the_boundary");
    }
    let (report, _) = run_smoke("perturbation_smoke.gd", SMOKE, Duration::from_secs(600));
    assert_ok_and_fp_clean(&report, "perturbation smoke");
    assert_eq!(as_str(&report, "scenario"), "station");
    assert_eq!(as_str(&report, "kind"), "brownout");
    assert_eq!(as_num(&report, "step_count") as u64, PERTURBED_STEPS);
    // The point of this scenario: the cascade must actually emerge across the boundary.
    assert!(
        as_num(&report, "rationed") > 0.0,
        "the deep brownout should ration: {report:?}"
    );
    assert!(
        as_num(&report, "min_scale") < 1.0,
        "a rationed flow should surface a scale below 1, got {}",
        as_num(&report, "min_scale")
    );
    assert_same_snapshot(
        &report,
        &emit("station", "emit_perturbed_brownout", &[]),
        "perturbed brownout",
    );
}

#[test]
fn the_composed_station_crosses_the_boundary() {
    if !available() {
        return not_run("the_composed_station_crosses_the_boundary");
    }
    let (report, _) = run_smoke("compose_smoke.gd", SMOKE, Duration::from_secs(600));
    assert_ok_and_fp_clean(&report, "compose smoke");
    assert_eq!(strings(&report, "parts"), vec!["power_plant", "radiator"]);
    assert_eq!(as_num(&report, "step_count") as u64, COMPOSED_STEPS);
    assert_eq!(
        as_num(&report, "rationed") as u64,
        0,
        "the composed heat-closure station is well-fed"
    );
    assert_eq!(
        COMPOSED_STEPS,
        HEAT_CLOSURE_DAYS * 24,
        "the composed horizon is the heat-closure one"
    );
    assert_same_snapshot(
        &report,
        &emit("station", "emit_station", &[]),
        "composed station",
    );
}

#[test]
fn save_and_load_cross_the_boundary() {
    if !available() {
        return not_run("save_and_load_cross_the_boundary");
    }
    let (report, _) = run_smoke("save_smoke.gd", SMOKE, Duration::from_secs(600));
    assert_ok_and_fp_clean(&report, "save/load smoke");
    assert!(
        as_bool(&report, "saved_ok"),
        "save() + FileAccess write did not complete"
    );
    assert!(
        as_bool(&report, "loaded_ok"),
        "load() from the file did not complete"
    );
    assert_eq!(as_num(&report, "step_count") as u64, SAVE_LOAD_STEPS);
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    assert_same_snapshot(
        &report,
        &emit("station", "emit_cabin_gas", &[]),
        "save/load",
    );
}

#[test]
fn the_time_controls_step_off_thread_without_diverging() {
    if !available() {
        return not_run("the_time_controls_step_off_thread_without_diverging");
    }
    let (report, _) = run_smoke("time_smoke.gd", SMOKE, Duration::from_secs(600));
    assert_ok_and_fp_clean(&report, "time-controls smoke");
    assert_eq!(as_str(&report, "scenario"), "cabin_gas");
    assert_eq!(as_num(&report, "step_count") as u64, CABIN_GAS_STEPS);
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    assert_eq!(
        as_str(&report, "error_msg"),
        "",
        "worker stepping faulted: {:?}",
        as_str(&report, "error_msg")
    );
    assert_same_snapshot(
        &report,
        &emit("station", "emit_cabin_gas", &[]),
        "time controls",
    );
    assert_matches_golden(&report, "cabin_gas_state.json", "time controls");
}

#[test]
fn a_file_loaded_scenario_crosses_the_boundary() {
    if !available() {
        return not_run("a_file_loaded_scenario_crosses_the_boundary");
    }
    let (report, _) = run_smoke_with(
        "from_file_smoke.gd",
        SMOKE,
        Duration::from_secs(600),
        &[scenario("crew_mission.yaml")],
    );
    assert_ok_and_fp_clean(&report, "from-file smoke");
    assert_eq!(
        as_num(&report, "total_steps") as u64,
        CREW_STEPS,
        "the scenario file declares its own 168-step horizon"
    );
    assert_eq!(as_num(&report, "step_count") as u64, CREW_STEPS);
    assert_eq!(as_num(&report, "rationed") as u64, 0);
    let file = scenario("crew_mission.yaml");
    let headless = emit(
        "authoring",
        "emit_authored",
        &[file.to_str().expect("utf-8 path")],
    );
    assert_same_snapshot(&report, &headless, "from-file");
    assert_matches_golden(&report, "crew_state.json", "from-file");
}

// --------------------------------------------------------------------------- //
// 2. The inspection smokes — the boundary carries structure, not just numbers  //
// --------------------------------------------------------------------------- //

#[test]
fn flow_inspection_crosses_the_boundary() {
    if !available() {
        return not_run("flow_inspection_crosses_the_boundary");
    }
    let (report, _) = run_smoke("flow_smoke.gd", FLOW, Duration::from_secs(600));
    assert!(
        as_bool(&report, "ok"),
        "flow smoke did not complete ok: {report:?}"
    );
    assert_eq!(as_str(&report, "scenario"), "station");
    assert_eq!(as_num(&report, "n") as u64, 24);

    // The Power → Thermal station registry, id-sorted (HeatInput dropped — Power's
    // dissipation IS the input, the Step-1 seam).
    let flow_ids = strings(&report, "flow_ids");
    for expected in [
        "power.solar_charge",
        "power.load_draw",
        "thermal.radiator_reject",
    ] {
        assert!(
            flow_ids.iter().any(|f| f == expected),
            "missing flow {expected:?} in {flow_ids:?}"
        );
    }

    // The "select thermal.node → contributing flows" join survived the boundary: the
    // radiator rejects heat OFF the node (a negative leg) and Power's dissipation feeds it
    // (a positive contributor) — the Step-1 cross-domain seam, made inspectable.
    let contributors: Vec<(String, f64)> = as_array(&report, "node_contributors")
        .iter()
        .map(|pair| match pair {
            JsonValue::Array(kv) if kv.len() == 2 => match (&kv[0], &kv[1]) {
                (JsonValue::Str(id), JsonValue::Number(lexeme)) => (
                    id.clone(),
                    lexeme
                        .parse::<f64>()
                        .unwrap_or_else(|e| panic!("contributor {id} amount {lexeme:?}: {e}")),
                ),
                other => panic!("node_contributors entry is not (id, amount): {other:?}"),
            },
            other => panic!("node_contributors entry is not a pair: {other:?}"),
        })
        .collect();
    let radiator = contributors
        .iter()
        .find(|(id, _)| id == "thermal.radiator_reject")
        .unwrap_or_else(|| panic!("thermal.radiator_reject absent from {contributors:?}"));
    assert!(
        radiator.1 < 0.0,
        "the radiator should WITHDRAW from the node, got {}",
        radiator.1
    );
    assert!(
        contributors.iter().any(|(_, amount)| *amount > 0.0),
        "dissipation should FEED the node: {contributors:?}"
    );

    // A two-rate entry defers inspection (single-rate only) → an empty string, not an error.
    assert!(as_bool(&report, "two_rate_empty"));
}

#[test]
fn the_objectives_read_stability_and_failure_across_the_boundary() {
    if !available() {
        return not_run("the_objectives_read_stability_and_failure_across_the_boundary");
    }
    let (report, _) = run_smoke("objectives_smoke.gd", SMOKE, Duration::from_secs(600));
    assert_ok_and_fp_clean(&report, "objectives smoke");

    let stable = field(&report, "stable");
    for flag in [
        "reached_target",
        "no_rationing",
        "conserved",
        "no_extinction",
        "survived",
    ] {
        assert!(
            as_bool(stable, flag),
            "a healthy station should satisfy {flag}: {stable:?}"
        );
    }

    // The other half, and the one that makes the first half mean something: a station that
    // is SUPPOSED to fail must be reported as failing.
    let failure = field(&report, "failure");
    assert!(
        as_bool(failure, "reached_target"),
        "the failing run still reaches its horizon: {failure:?}"
    );
    assert!(
        as_num(failure, "rationed") > 0.0,
        "the deep blackout should ration LoadDraw: {failure:?}"
    );
    assert!(!as_bool(failure, "no_rationing"));
    assert!(
        !as_bool(failure, "survived"),
        "the deep blackout must not be reported as survived: {failure:?}"
    );
}

#[test]
fn a_template_override_reaches_the_engine_through_the_file_boundary() {
    if !available() {
        return not_run("a_template_override_reaches_the_engine_through_the_file_boundary");
    }
    let (report, _) = run_smoke_with(
        "from_file_template_smoke.gd",
        SMOKE,
        Duration::from_secs(600),
        &[scenario("crew_habitat_template.yaml")],
    );
    assert_ok_and_fp_clean(&report, "template smoke");
    let base = as_num(&report, "food_default");
    let big = as_num(&report, "food_4x");
    assert!(
        base > 0.0,
        "the default food store should be positive: {base}"
    );
    let ratio = big / base;
    assert!(
        (ratio - 4.0).abs() <= 4.0 * 1e-12,
        "a 4x template override should scale the store 4x, got {big} / {base} = {ratio}"
    );
}

#[test]
fn the_authored_kinetics_marker_crosses_the_boundary() {
    if !available() {
        return not_run("the_authored_kinetics_marker_crosses_the_boundary");
    }
    let (report, _) = run_smoke_with(
        "authored_marker_smoke.gd",
        SMOKE,
        Duration::from_secs(600),
        &[
            scenario("self_discharge_dsl.yaml"),
            scenario("crew_mission.yaml"),
        ],
    );
    assert!(
        as_bool(&report, "ok"),
        "marker smoke did not complete ok: {report:?}"
    );
    assert!(
        as_bool(&report, "authored_marker"),
        "an authored-kinetics scenario must raise the marker: {report:?}"
    );
    assert!(
        !as_bool(&report, "after_palette_rebuild"),
        "rebuilding a frozen palette entry must LOWER the marker again: {report:?}"
    );
    assert!(
        !as_bool(&report, "plain_file_marker"),
        "a plain scenario file carries no authored kinetics: {report:?}"
    );
}

// --------------------------------------------------------------------------- //
// 3. The UI smokes — the scenes build headless, and say so                     //
// --------------------------------------------------------------------------- //

/// GDScript reports a fault on stderr and keeps going, so a scene can "succeed" while having
/// thrown. Every UI smoke checks the stream as well as the report.
fn assert_no_gdscript_fault(stderr: &str, what: &str) {
    for marker in ["SCRIPT ERROR", "Parse Error", "--- Debugging"] {
        assert!(
            !stderr.contains(marker),
            "GDScript fault in {what} (found {marker:?}):\n{stderr}"
        );
    }
}

fn assert_ui_built(report: &JsonValue, what: &str) {
    assert!(as_bool(report, "ok"), "{what} failed: {report:?}");
    assert!(
        as_num(report, "child_count") > 0.0,
        "{what} built no widgets — `_build_ui` was skipped: {report:?}"
    );
}

#[test]
fn the_main_dashboard_renders_the_flow_panel() {
    if !available() {
        return not_run("the_main_dashboard_renders_the_flow_panel");
    }
    let (report, stderr) = run_smoke("main_ui_smoke.gd", MAIN_UI, Duration::from_secs(600));
    assert_no_gdscript_fault(&stderr, "main_ui_smoke.gd");
    assert!(
        as_bool(&report, "ok"),
        "main dashboard smoke failed: {report:?}"
    );
    assert!(as_bool(&report, "has_flows_panel"));
    assert!(as_bool(&report, "has_contributing"));
    assert!(as_bool(&report, "perturbation_triggered"));
    assert!(as_bool(&report, "header_shows_perturbation"));
}

#[test]
fn the_save_load_dashboard_builds_and_round_trips() {
    if !available() {
        return not_run("the_save_load_dashboard_builds_and_round_trips");
    }
    let (report, stderr) = run_smoke("save_ui_smoke.gd", UI, Duration::from_secs(600));
    assert_no_gdscript_fault(&stderr, "save_ui_smoke.gd");
    assert_ui_built(&report, "the save/load dashboard");
    assert_eq!(
        as_num(&report, "n_after_load"),
        as_num(&report, "n_before_save"),
        "a save/load round trip through the UI must land on the same step count"
    );
}

#[test]
fn the_time_dashboard_instantiates_headless() {
    if !available() {
        return not_run("the_time_dashboard_instantiates_headless");
    }
    let (report, stderr) = run_smoke("ui_smoke.gd", UI, Duration::from_secs(600));
    assert_no_gdscript_fault(&stderr, "ui_smoke.gd");
    assert_ui_built(&report, "the time dashboard");
}

#[test]
fn the_compose_palette_dashboard_builds() {
    if !available() {
        return not_run("the_compose_palette_dashboard_builds");
    }
    let (report, stderr) = run_smoke("compose_ui_smoke.gd", UI, Duration::from_secs(600));
    assert_no_gdscript_fault(&stderr, "compose_ui_smoke.gd");
    assert_ui_built(&report, "the compose palette dashboard");
}

#[test]
fn the_from_file_dashboard_builds_and_banners_only_authored_scenarios() {
    if !available() {
        return not_run("the_from_file_dashboard_builds_and_banners_only_authored_scenarios");
    }
    let (report, stderr) = run_smoke("from_file_ui_smoke.gd", UI, Duration::from_secs(600));
    assert_no_gdscript_fault(&stderr, "from_file_ui_smoke.gd");
    assert_ui_built(&report, "the from-file dashboard");
    assert!(
        as_num(&report, "step_count") > 0.0,
        "the default scenario did not load and step: {report:?}"
    );

    // "Authored ≠ validated" made visible: the banner is raised for an authored scenario and
    // for nothing else, including after a reload back to a frozen one.
    assert!(
        !as_bool(&report, "banner_default_visible"),
        "a frozen scenario must NOT show the authored banner: {report:?}"
    );
    assert!(
        as_bool(&report, "banner_authored_visible"),
        "an authored scenario MUST show the banner: {report:?}"
    );
    assert!(
        !as_bool(&report, "banner_after_reload_visible"),
        "reloading a frozen scenario must lower the banner again: {report:?}"
    );
}

// --------------------------------------------------------------------------- //
// 4. The harness checks itself                                                 //
// --------------------------------------------------------------------------- //

/// ⚠ Without this, every test above would pass on a machine where Godot is missing AND on a
/// machine where the marker protocol changed — both by returning early or by reading a report
/// that happens to parse. The accessors are the part that must fail loudly.
#[test]
fn the_report_accessors_refuse_a_missing_or_mistyped_field() {
    let report = json::parse(r#"{"ok": true, "n": 24, "scenario": "station"}"#).expect("parses");
    assert!(as_bool(&report, "ok"));
    assert_eq!(as_num(&report, "n"), 24.0);
    assert_eq!(as_str(&report, "scenario"), "station");

    // Absent, and present-but-wrong-type. Both must panic; neither may quietly yield `false`,
    // which would make every boolean assertion in this file vacuously satisfiable by a report
    // that simply stopped carrying the field.
    for (case, probe) in [
        (
            "absent field",
            Box::new(|| as_bool(&report, "absent")) as Box<dyn Fn() -> bool>,
        ),
        ("mistyped field", Box::new(|| as_bool(&report, "n"))),
        (
            "mistyped number",
            Box::new(|| as_num(&report, "scenario") > 0.0),
        ),
    ] {
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(&probe)).is_err(),
            "{case}: must panic, never default"
        );
    }
}

/// The Godot lookup is the switch every test above hangs on. If it returned `None` on a
/// machine that HAS Godot, the whole file would silently stop running — the failure mode
/// this stage keeps finding, in the one place that would hide it completely.
#[test]
fn the_godot_lookup_agrees_with_the_environment() {
    match godot_exe() {
        Some(path) => assert!(
            path.is_file(),
            "godot_exe() returned {path:?}, which is not a file"
        ),
        None => eprintln!(
            "the_godot_lookup_agrees_with_the_environment: no godot on PATH — every smoke in \
             this file DID NOT RUN."
        ),
    }
    // The project the smokes drive must exist either way; a moved `godot/` directory would
    // otherwise surface as sixteen confusing marker failures.
    assert!(
        godot_project().join("project.godot").is_file(),
        "godot/project.godot is missing — the smokes have no project to run"
    );
    assert!(
        Path::new(&workspace()).join("Cargo.toml").is_file(),
        "the workspace path this file computes is wrong"
    );
}
