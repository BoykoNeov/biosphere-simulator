"""Phase-8 (P8.3) time-controls cross-boundary smoke — the OFF-RENDER-THREAD analogue
of the Step-1 `test_godot_parity.py`.

Step 1 stepped the synchronous `SimSession` on the render thread and proved the FFI
boundary does not corrupt determinism. Step 3 moves stepping to a **worker thread**
owned by a `TimeController` (so fast-forwarding a two-rate scenario — 1440 sub-steps per
master day — does not freeze the UI). That relocation reopens the two Step-1 risks *on
the new thread*:

  * **FP environment** — FTZ/DAZ (flush-to-zero / denormals-are-zero in MXCSR) are
    per-thread, so the worker thread must carry the same IEEE-default env headless does.
    `time_smoke.gd` reads `fp_clean()` / `worker_mxcsr()`, which the worker publishes
    from *its own* thread.
  * **Determinism across the boundary** — the worker calls the same `CoreSession::step`,
    so a fast-forward to `N` on the worker thread must be bit-identical to a synchronous
    `step_n(N)`. (The intra-process version is proved in Rust by
    `time_control::tests::worker_fast_forward_is_bit_exact_vs_synchronous_*`; here it is
    proved through the **actual cdylib Godot loads**.)

This drives the Tier-1 `cabin_gas` scenario (transcendental-free ⇒ bit-exact on any
platform/libm) via `TimeController.fast_forward_to(900)` and asserts:

  * **bit-exact snapshot** — the worker's fast-forward result equals the headless
    `emit_cabin_gas` output byte-for-byte, AND matches the frozen `cabin_gas_state.json`
    golden at Tier 1 (parsed f64, bit-exact);
  * **FP env clean** — `fp_clean()` reports FTZ and DAZ both OFF on the worker thread;
  * **Tier-0 discretes** — `rationed == 0`, `step_count == 900`, no stepping fault.

**Local-only** (`skipif` when `godot` or `cargo` is absent), exactly like the other
crossport gates — the CI runners install neither Godot nor the gdext toolchain.
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

GODOT = shutil.which("godot")
CARGO = shutil.which("cargo")

# station::scenario::CABIN_GAS_STEPS (the frozen cabin_gas horizon).
CABIN_GAS_STEPS = 900

_SMOKE_BEGIN = "<<<GODOT_SMOKE_BEGIN"
_SMOKE_END = "GODOT_SMOKE_END>>>"

pytestmark = pytest.mark.skipif(
    GODOT is None or CARGO is None,
    reason="Godot and/or cargo absent (Phase-8 cross-boundary smoke is local-only)",
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
    """Ensure the project's `.godot/extension_list.cfg` exists so the `.gdextension` is
    registered — a fresh checkout needs one import pass before a `--script` run can see
    the `TimeController` class."""
    assert GODOT is not None  # narrowed by the module-level skipif
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


def _run_time_smoke() -> dict:
    """Run `time_smoke.gd` headless through the cdylib; return the parsed report."""
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://time_smoke.gd",
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


_UI_BEGIN = "<<<UI_SMOKE_BEGIN"
_UI_END = "UI_SMOKE_END>>>"


def _run_ui_smoke() -> tuple[dict, str]:
    """Instantiate `time_dashboard.tscn` headless; return (report, stderr)."""
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://ui_smoke.gd",
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert _UI_BEGIN in out and _UI_END in out, (
        "ui smoke markers missing — scene may have failed to load:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_UI_BEGIN, 1)[1].split(_UI_END, 1)[0].strip()
    return json.loads(payload), proc.stderr


def _emit_cabin_gas_headless() -> str:
    """The headless Rust reference: `emit_cabin_gas` stdout is exactly `to_json()`."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", "--example", "emit_cabin_gas"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"emit_cabin_gas failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_time_controls_off_thread_parity() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_time_smoke()

    # Tier-0 discretes: the off-thread fast-forward completed well-fed with no fault.
    assert report["ok"] is True, f"time smoke did not complete ok: {report}"
    assert report["scenario"] == "cabin_gas"
    assert report["step_count"] == CABIN_GAS_STEPS
    assert report["rationed"] == 0
    assert report["error_msg"] == "", f"worker stepping faulted: {report['error_msg']}"

    # FP-environment parity ON THE WORKER (stepping) THREAD — the P8.3 relocation of the
    # Step-1 check: FTZ/DAZ must be OFF where the sim now actually runs.
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the worker stepping thread (mxcsr={report['mxcsr']:#x}); "
        "denormal intermediates would flush to zero and diverge from headless"
    )

    godot_snapshot = report["snapshot"]

    # (1) Bit-exact vs the headless Rust reference — pure Rust `to_json()` both sides:
    # any FFI/FP/threading corruption of the trajectory changes the bytes.
    headless_snapshot = _emit_cabin_gas_headless()
    assert godot_snapshot == headless_snapshot, (
        "worker-thread fast-forward snapshot differs from headless emit_cabin_gas "
        "byte-for-byte — the off-thread stepping corrupted determinism"
    )

    # (2) Tier-1 bit-exact vs the frozen golden — the frozen-reference bind (f64).
    golden = compare.load_json(GOLDEN_DIR / "cabin_gas_state.json")
    result = compare.compare(golden, json.loads(godot_snapshot), tier=1)
    assert result.ok, result.report()


def test_time_dashboard_scene_instantiates_headless() -> None:
    """The interactive `time_dashboard.tscn` (speed control + horizon scrubber) actually
    loads and builds its widgets through the engine. The other gates drive the
    `TimeController` API but never load this scene, so a GDScript parse error / wrong
    signal arity / base-class shadow would otherwise ship undetected (advisor)."""
    _build_cdylib()
    _ensure_godot_import()
    report, stderr = _run_ui_smoke()
    assert report["ok"] is True, f"dashboard failed to build its UI: {report}"
    assert report["child_count"] > 0, "dashboard _build_ui added no widgets"
    # Headless runs the script; any GDScript fault prints one of these to stderr.
    for marker in ("SCRIPT ERROR", "Parse Error", "SHADOWED_VARIABLE"):
        assert marker not in stderr, f"GDScript fault in time_dashboard.gd:\n{stderr}"
