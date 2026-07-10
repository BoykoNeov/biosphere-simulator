"""Phase-8 (P8.5) cross-boundary PERTURBATION smoke — the "FFI didn't corrupt
determinism" proof extended to a *perturbed* run, plus the demonstration of the failure
cascade and the P8.4 rationing seam through the actual `godot_bridge` cdylib.

Where `test_godot_parity.py` (Step 1) drives an unperturbed Tier-1 `cabin_gas` run, this
drives a **deep brownout** on the single-rate `station` scenario via the interactive
`build_perturbed` primitive (`godot/perturbation_smoke.gd`). The blackout empties the
battery so `LoadDraw` rations — the failure cascade — and because a rationed flow's raw
legs no longer equal what moved, each inspected flow carries a `scale < 1` (the seam
fix). It asserts:

  * **bit-exact snapshot** — the Godot-produced `sim_io` hex-float snapshot equals the
    headless `emit_perturbed_brownout` output byte-for-byte (pure Rust both sides, no
    formatting confound): the FFI boundary preserved determinism through a *perturbed*
    trajectory. Tier-2 (Power's `sin` + the `T⁴` radiator) but exact locally (same
    libm) — no golden pins a perturbed run (the "diagnostics, no golden" precedent);
  * **the failure cascade emerged** — `rationed > 0` and `min_scale < 1.0` (rationing
    was observed through the flow inspection — the seam is live and truthful);
  * **FP env clean** — `fp_clean()` on the stepping thread reports FTZ/DAZ both OFF;
  * **Tier-0 discretes** — `step_count == 288`.

**Local-only**, like `test_godot_parity.py` / `test_crossport.py`: `skipif` when Godot
or `cargo` is absent (CI installs neither the gdext toolchain nor Godot). The
developer / release-time boundary proof.
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

# godot/perturbation_smoke.gd: STEPS = 12 * 24.
PERTURBED_STEPS = 288

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


def _run_smoke() -> dict:
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://perturbation_smoke.gd",
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


def _emit_perturbed_brownout_headless() -> str:
    """The headless Rust reference: `emit_perturbed_brownout` stdout is `to_json()`."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", "--example", "emit_perturbed_brownout"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"emit_perturbed_brownout failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_perturbed_brownout_cross_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke()

    # Tier-0 discretes + the interactive primitive succeeded.
    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["scenario"] == "station"
    assert report["kind"] == "brownout"
    assert report["step_count"] == PERTURBED_STEPS

    # The failure cascade emerged AND the rationing seam is live/truthful through the
    # cdylib: the blackout empties the battery so LoadDraw rations (rationed > 0), and
    # the flow inspection surfaced a scale < 1 (raw legs != what moved — the P8.4 seam).
    assert report["rationed"] > 0, (
        "deep brownout should drive rationing (the failure cascade)"
    )
    min_scale = report["min_scale"]
    assert min_scale < 1.0, (
        f"flow inspection should surface a rationed flow (scale < 1), got {min_scale}"
    )

    # FP-environment parity: FTZ/DAZ OFF on the stepping thread.
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    # Bit-exact vs the headless Rust reference — pure Rust `to_json()` both sides: any
    # FFI/FP corruption of the *perturbed* trajectory changes the bytes.
    headless_snapshot = _emit_perturbed_brownout_headless()
    assert report["snapshot"] == headless_snapshot, (
        "Godot cdylib snapshot differs from headless emit_perturbed_brownout "
        "byte-for-byte — the FFI boundary corrupted determinism under perturbation"
    )
