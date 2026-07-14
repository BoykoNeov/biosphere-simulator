"""Phase-9 (Step 5) cross-boundary parity smoke — "Godot loads a scenario FILE at
runtime," the "author, not program" payoff, proven through the ACTUAL `godot_bridge`
cdylib Godot loads.

This is the Step-1 `test_godot_parity.py` pattern lifted one authoring level up: instead
of a fixed-palette scenario id, the session is built from a declarative `.yaml` file via
`SimSession.build_from_file` (the frozen `authoring` boundary parses + interprets it).
The anchor is `crew_mission.yaml` —
transcendental-free (Tier-1, bit-exact on any platform/libm) and already reproducing the
frozen `crew_state.json` per Phase-9 Step 4b, so the smoke re-proves that known golden
*through the file-load FFI boundary*.

The scenario's **absolute path is passed to both sides** — the Godot smoke (via a `--`
user arg, `OS.get_cmdline_user_args()`) and the headless `emit_authored` reference — so
both load the identical committed file; a byte-identity failure can then only mean the
FFI boundary corrupted determinism, never a two-different-files confound (the advisor's
path-resolution guard).

Asserts:
  * **bit-exact snapshot** — the Godot-produced `sim_io` hex-float snapshot equals the
    headless `emit_authored <file>` output byte-for-byte (pure-Rust both sides), AND
    matches the frozen `crew_state.json` golden at Tier 1 (parsed f64, bit-exact);
  * **FP env clean** — `fp_clean()` on the stepping thread reports FTZ/DAZ both OFF;
  * **Tier-0 discretes** — `rationed == 0`, `total_steps == step_count == 168` (the file
    declares its own horizon).

**Local-only**, like the other `test_godot_*` gates: `skipif` when `godot` or `cargo` is
absent. CI runners install neither, so this never runs on CI — the developer /
release-time boundary proof.
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))

import compare  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
GOLDEN_DIR = REPO_ROOT / "tests" / "regression" / "golden"
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"
GODOT_PROJECT_DIR = REPO_ROOT / "godot"
SCENARIO = REPO_ROOT / "tests" / "authoring" / "scenarios" / "crew_mission.yaml"
_SCEN_DIR = REPO_ROOT / "tests" / "authoring" / "scenarios"
TEMPLATE = _SCEN_DIR / "crew_habitat_template.yaml"

# The crew_mission horizon (MISSION_DAYS 7 * steps_per_day 24), declared in the file.
CREW_STEPS = 168

GODOT = shutil.which("godot")
CARGO = shutil.which("cargo")

_SMOKE_BEGIN = "<<<GODOT_SMOKE_BEGIN"
_SMOKE_END = "GODOT_SMOKE_END>>>"
_UI_BEGIN = "<<<UI_SMOKE_BEGIN"
_UI_END = "UI_SMOKE_END>>>"

pytestmark = pytest.mark.skipif(
    GODOT is None or CARGO is None,
    reason="Godot and/or cargo absent (cross-boundary smoke is local-only)",
)


def _build_cdylib() -> None:
    """Build the `godot_bridge` cdylib Godot dlopen's (idempotent; fast once built)."""
    proc = subprocess.run(
        ["cargo", "build", "-q", "-p", "godot_bridge"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo build -p godot_bridge failed:\n{proc.stderr}"


def _ensure_godot_import() -> None:
    """Ensure `.godot/extension_list.cfg` exists so the `.gdextension` is registered."""
    assert GODOT is not None
    ext_list = GODOT_PROJECT_DIR / ".godot" / "extension_list.cfg"
    if ext_list.exists():
        return
    proc = subprocess.run(
        [GODOT, "--headless", "--path", str(GODOT_PROJECT_DIR), "--import"],
        capture_output=True,
        text=True,
        timeout=300,
    )
    assert ext_list.exists(), (
        "Godot import did not register the extension:\n"
        f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
    )


def _run_smoke() -> dict:
    """Run `from_file_smoke.gd` headless through the actual cdylib, passing the absolute
    scenario path as a `--` user arg; return the parsed report."""
    assert GODOT is not None
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://from_file_smoke.gd",
            "--",
            str(SCENARIO),
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert _SMOKE_BEGIN in out and _SMOKE_END in out, (
        "smoke markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_SMOKE_BEGIN, 1)[1].split(_SMOKE_END, 1)[0].strip()
    return json.loads(payload)


def _emit_authored_headless() -> str:
    """Headless Rust reference: `emit_authored <file>` stdout is exactly to_json()."""
    proc = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "authoring",
            "--example",
            "emit_authored",
            "--",
            str(SCENARIO),
        ],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"emit_authored failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_file_loaded_scenario_cross_boundary_parity() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke()

    # Tier-0 discretes: the file-declared horizon ran well-fed.
    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["total_steps"] == CREW_STEPS, "the file declares its 168-step horizon"
    assert report["step_count"] == CREW_STEPS
    assert report["rationed"] == 0

    # FP-environment parity: FTZ/DAZ OFF on the stepping thread.
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x}); "
        "denormal intermediates would flush to zero and diverge from headless"
    )

    godot_snapshot = report["snapshot"]

    # (1) Bit-exact vs the headless Rust reference loading the SAME file — pure Rust
    # `to_json()` both sides: any FFI/FP corruption of the trajectory changes the bytes.
    headless_snapshot = _emit_authored_headless()
    assert godot_snapshot == headless_snapshot, (
        "Godot cdylib file-loaded snapshot differs from headless emit_authored "
        "byte-for-byte — the file-load FFI boundary corrupted determinism"
    )

    # (2) Tier-1 bit-exact vs the frozen golden — the authored crew re-proves
    # crew_state.json through the file-load boundary (f64 parsed, bit-exact).
    golden = compare.load_json(GOLDEN_DIR / "crew_state.json")
    result = compare.compare(golden, json.loads(godot_snapshot), tier=1)
    assert result.ok, result.report()


def _run_template_smoke() -> dict:
    """Run `from_file_template_smoke.gd` (the `build_from_file_with` array-FFI driver)
    headless through the actual cdylib, passing the template's absolute path."""
    assert GODOT is not None
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://from_file_template_smoke.gd",
            "--",
            str(TEMPLATE),
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert _SMOKE_BEGIN in out and _SMOKE_END in out, (
        "template smoke markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_SMOKE_BEGIN, 1)[1].split(_SMOKE_END, 1)[0].strip()
    return json.loads(payload)


def test_godot_build_from_file_with_template_override() -> None:
    """The templated-override array FFI (`build_from_file_with`) end-to-end through the
    cdylib — the one FFI surface no other smoke / cargo test / UI drives. Also shows the
    Phase-9 "load a template, not just a fixed scenario" capability: `crew_count` bites
    exactly 4× (initial `crew.food_store`, read at n=0) through the array FFI."""
    _build_cdylib()
    _ensure_godot_import()
    report = _run_template_smoke()
    assert report["ok"] is True, f"template smoke did not complete ok: {report}"
    assert report["fp_clean"] is True
    base = report["food_default"]
    big = report["food_4x"]
    assert base > 0.0, f"default food store should be positive: {base}"
    # 1.0*base vs 4.0*base at n=0 — exact 4× (build-time eval, no accumulation).
    assert big == pytest.approx(4.0 * base, rel=1e-12), (
        f"crew_count=4.0 should 4× the food store via the array FFI: {base=} {big=}"
    )


def _run_ui_smoke() -> tuple[dict, str]:
    assert GODOT is not None
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://from_file_ui_smoke.gd",
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert _UI_BEGIN in out and _UI_END in out, (
        "UI smoke markers missing — the dashboard scene may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_UI_BEGIN, 1)[1].split(_UI_END, 1)[0].strip()
    return json.loads(payload), proc.stderr


def test_godot_from_file_ui_loads() -> None:
    """The `from_file_dashboard` scene instantiates headless: its `_ready` runs
    `_build_ui`, Load resolves the default `res://../tests/.../crew_mission.yaml`
    (globalized to an OS path) through the real `build_from_file`, and stepping advances
    the live sim. Closes the gap that the cargo tests + `from_file_smoke.gd` never load
    the dashboard scene, and proves the `res://`-parent-dir default resolves e2e."""
    _build_cdylib()
    _ensure_godot_import()
    report, stderr = _run_ui_smoke()
    assert report["ok"] is True, f"from-file UI smoke failed: {report}\nstderr:{stderr}"
    assert report["child_count"] > 0, "dashboard built no widgets (_build_ui skipped)"
    assert report["step_count"] > 0, "the default scenario did not load + step"
    assert "SCRIPT ERROR" not in stderr, f"GDScript error in UI smoke:\n{stderr}"
