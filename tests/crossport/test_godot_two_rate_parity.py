"""Phase-8 (P8.8) cross-boundary **two-rate** parity — closing the gap Step-1's smoke
left (it drove only the single-rate `cabin_gas`).

A two-rate `step()` is one MASTER DAY = one slow biosphere step + 1440 fast cabin
sub-steps (`advance_one_master_day`), so it exercises a different driver across the FFI
boundary. Two arms:

  * **greenhouse** (fast) — 7 master days, `reset = None`. Cheap; runs in the fast
    crossport gate. Proves the two-rate driver survives the boundary bit-exact.
  * **sealed** (slow) — SEALED_RESUME_DAYS = 310 master days, a few past one 305-day
    season so the re-sow (`slow_reset`) adopt branch fires ACROSS the boundary (the
    genuinely-new coverage). ~450k sub-steps ⇒ `@pytest.mark.slow`. The full multi-year
    parity is proven intra-process (`session_parity.rs`) + the frozen golden; this
    proves the FFI boundary didn't corrupt the season-crossing two-rate trajectory.

Both arms compare the Godot-cdylib snapshot **byte-for-byte** to the headless `emit_*`
reference (pure Rust both sides, same libm ⇒ bit-exact on any platform) and assert
FTZ/DAZ OFF and `rationed == 0`.

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

# godot/greenhouse_smoke.gd DAYS / godot/sealed_smoke.gd DAYS (SEALED_RESUME_DAYS).
GREENHOUSE_DAYS = 7
SEALED_RESUME_DAYS = 310

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


def _run_smoke(script: str, timeout: int) -> dict:
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
        timeout=timeout,
    )
    out = proc.stdout
    assert _SMOKE_BEGIN in out and _SMOKE_END in out, (
        f"smoke markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_SMOKE_BEGIN, 1)[1].split(_SMOKE_END, 1)[0].strip()
    return json.loads(payload)


def _emit_headless(example: str, timeout: int) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", "--example", example],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    assert proc.returncode == 0, f"{example} failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_greenhouse_two_rate_cross_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke("greenhouse_smoke.gd", timeout=300)

    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["scenario"] == "greenhouse"
    assert report["step_count"] == GREENHOUSE_DAYS
    assert report["rationed"] == 0
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    headless = _emit_headless("emit_greenhouse", timeout=300)
    assert report["snapshot"] == headless, (
        "Godot cdylib greenhouse snapshot differs from headless emit_greenhouse — the "
        "FFI boundary corrupted the two-rate trajectory"
    )


@pytest.mark.slow
def test_godot_sealed_season_crossing_cross_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    # ~450k sub-steps through the debug cdylib + headless emit build/run: big timeout.
    report = _run_smoke("sealed_smoke.gd", timeout=900)

    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["scenario"] == "sealed"
    assert report["step_count"] == SEALED_RESUME_DAYS
    assert report["rationed"] == 0
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    headless = _emit_headless("emit_sealed_resume", timeout=900)
    assert report["snapshot"] == headless, (
        "Godot cdylib sealed (season-crossing) snapshot differs from headless "
        "emit_sealed_resume — the FFI boundary corrupted the re-sown two-rate run"
    )
