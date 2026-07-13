"""Phase-8 (P8.7) cross-boundary SAVE/LOAD smoke: a disk round-trip carried through
the actual `godot_bridge` cdylib.

Where `test_godot_parity.py` (Step 1) drives a pre-built `cabin_gas` and
`test_godot_compose.py` (Step 6) drives a composed `station`, this drives a real
**save/load cycle through disk** (`godot/save_smoke.gd`): build `cabin_gas`, step to
a mid-run save point, `save()` the record and write it with Godot `FileAccess`, then
in a FRESH `SimSession` read the file and `load()` it (rebuild-from-recipe + restore
state) and resume to the full horizon. It asserts:

  * **bit-exact vs the frozen reference**: the resumed session's `sim_io` hex-float
    snapshot at n=900 equals the headless `emit_cabin_gas` output byte-for-byte.
    `cabin_gas` is Tier-1 (transcendental-free) so this is bit-exact on any platform;
    it proves save/load through the real engine + Godot's file API preserved
    determinism exactly (the `(seed, key, n)` corollary), not just intra-process;
  * **Tier-0 discretes**: `saved_ok`, `loaded_ok`, `rationed == 0`, `step_count == 900`;
  * **FP env clean**: `fp_clean()` on the stepping thread reports FTZ/DAZ OFF.

**Local-only**, like the other Phase-8 smokes: `skipif` when Godot or `cargo` is
absent (CI installs neither the gdext toolchain nor Godot).
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).parent))


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"
GODOT_PROJECT_DIR = REPO_ROOT / "godot"

GODOT = shutil.which("godot")
CARGO = shutil.which("cargo")

# godot/save_smoke.gd: TOTAL = 900 (CABIN_GAS_STEPS).
SAVE_LOAD_STEPS = 900

_SMOKE_BEGIN = "<<<GODOT_SMOKE_BEGIN"
_SMOKE_END = "GODOT_SMOKE_END>>>"
_UI_BEGIN = "<<<UI_SMOKE_BEGIN"
_UI_END = "UI_SMOKE_END>>>"

pytestmark = pytest.mark.skipif(
    GODOT is None or CARGO is None,
    reason="Godot and/or cargo absent (Phase-8 cross-boundary smoke is local-only)",
)


def _build_cdylib() -> None:
    proc = subprocess.run(
        ["cargo", "build", "-q", "-p", "godot_bridge"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo build -p godot_bridge failed:\n{proc.stderr}"


def _ensure_godot_import() -> None:
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


def _run_script(script: str, begin: str, end: str) -> tuple[dict, str]:
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            f"res://{script}",
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert begin in out and end in out, (
        f"smoke markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(begin, 1)[1].split(end, 1)[0].strip()
    return json.loads(payload), proc.stderr


def _emit_cabin_gas_headless() -> str:
    """The headless Rust reference: `emit_cabin_gas` stdout is build_cabin JSON."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", "--example", "emit_cabin_gas"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"emit_cabin_gas failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_save_load_cross_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report, _stderr = _run_script("save_smoke.gd", _SMOKE_BEGIN, _SMOKE_END)

    # Tier-0 discretes + the disk round-trip actually happened.
    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["saved_ok"] is True, "save() + FileAccess write did not complete"
    assert report["loaded_ok"] is True, "load() from the file did not complete"
    assert report["step_count"] == SAVE_LOAD_STEPS
    assert report["rationed"] == 0, (
        "cabin_gas is well-fed across the save/load boundary"
    )

    # FP-environment parity: FTZ/DAZ OFF on the stepping thread.
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    # Bit-exact vs the frozen reference: save at 300, write the file, load into a fresh
    # session, resume to 900 -> reproduces emit_cabin_gas across FFI + file.
    headless_snapshot = _emit_cabin_gas_headless()
    assert report["snapshot"] == headless_snapshot, (
        "Godot cdylib save/load snapshot differs from headless emit_cabin_gas: "
        "either save/load lost determinism or the FFI/file round-trip diverged"
    )


def test_godot_save_load_ui_loads() -> None:
    """The save/load dashboard instantiates headless: `_ready` runs `_build_ui`, then a
    Build/Step/Save/Load cycle runs through the real bridge and resumes at the saved
    step count. Closes the gap that cargo tests + the smoke never load the scene."""
    _build_cdylib()
    _ensure_godot_import()
    report, stderr = _run_script("save_ui_smoke.gd", _UI_BEGIN, _UI_END)
    assert report["ok"] is True, (
        f"save/load UI smoke failed: {report}\nstderr:\n{stderr}"
    )
    assert report["child_count"] > 0, "dashboard built no widgets (_build_ui skipped)"
    assert report["n_after_load"] == report["n_before_save"], (
        "load did not restore the saved step count through the UI"
    )
    # A GDScript error prints "SCRIPT ERROR" to stderr even if the scene loads.
    assert "SCRIPT ERROR" not in stderr, f"GDScript error in UI smoke:\n{stderr}"
