"""Phase-8 (P8.8) cross-boundary **objectives** — the "observe failure or stability" arm
of the exit criterion, carried through the actual `godot_bridge` cdylib.

`godot/objectives_smoke.gd` drives two runs and reads `objectives_json(target)`:

  * STABILITY — a healthy `station` reaching its 7-day horizon → `survived == true`.
  * FAILURE   — a `station` under a deep multi-day blackout (brownout factor 0.0) that
    empties the battery and rations `LoadDraw` → `no_rationing == false` ⇒ `survived ==
    false`, with rationing as the identified cause.

So the SAME objective distinguishes a stable run from a failing one — the failure
cascade emerges from the frozen engine, no game-side domain logic. This asserts the
objectives FFI works across the boundary and that both outcomes are reachable.

**Local-only** like the other Phase-8 smokes: `skipif` when Godot or `cargo` is absent.
"""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"
GODOT_PROJECT_DIR = REPO_ROOT / "godot"

GODOT = shutil.which("godot")
CARGO = shutil.which("cargo")

_SMOKE_BEGIN = "<<<GODOT_SMOKE_BEGIN"
_SMOKE_END = "GODOT_SMOKE_END>>>"

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


def _run_smoke(script: str) -> dict:
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
    assert _SMOKE_BEGIN in out and _SMOKE_END in out, (
        f"smoke markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_SMOKE_BEGIN, 1)[1].split(_SMOKE_END, 1)[0].strip()
    return json.loads(payload)


def test_godot_objectives_stability_and_failure() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke("objectives_smoke.gd")

    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    # STABILITY: reached the horizon healthy → survived.
    stable = report["stable"]
    assert stable["reached_target"] is True
    assert stable["no_rationing"] is True
    assert stable["conserved"] is True
    assert stable["no_extinction"] is True
    assert stable["survived"] is True, f"healthy station should survive: {stable}"

    # FAILURE: the blackout rationed → the objective fails, with rationing the cause.
    failure = report["failure"]
    assert failure["reached_target"] is True, (
        "the failure run still reaches its horizon"
    )
    assert failure["rationed"] > 0, "the deep blackout should ration LoadDraw"
    assert failure["no_rationing"] is False
    assert failure["survived"] is False, (
        f"a rationed station is a failure even at the horizon: {failure}"
    )
