"""Phase-8 (P8.6) cross-boundary COMPOSITION smoke: the byte-identity anchor
carried through the actual `godot_bridge` cdylib.

Where `test_godot_parity.py` (Step 1) drives a pre-built `cabin_gas` scenario
and `test_godot_perturbations.py` (Step 5) drives a perturbed `station`, this
drives a station **composed from the fixed palette** via
`build_composed(["power_plant", "radiator"])`, the interactive "build systems"
primitive (`godot/compose_smoke.gd`). It asserts:

  * **bit-exact vs the frozen reference**: the Godot-produced `sim_io`
    hex-float snapshot equals the headless `emit_station` output byte-for-byte.
    Because `{power_plant, radiator}` reproduces `build_station` bit-for-bit
    (proven Rust-side in `station/tests/builder_parity.rs`), the palette builder
    is a **pure refactor across the FFI boundary**: the boundary preserved
    determinism AND the composition equals the frozen station. Tier-2 (Power's
    `sin` + the `T^4` radiator) but exact locally (same libm); no golden pins a
    composed run (a composition is a runtime object, not a frozen scenario);
  * **Tier-0 discretes**: `rationed == 0`, `step_count == 168` (7 diurnal days);
  * **FP env clean**: `fp_clean()` on the stepping thread reports FTZ/DAZ OFF.

**Local-only**, like the other Phase-8 smokes: `skipif` when Godot or `cargo` is
absent (CI installs neither the gdext toolchain nor Godot). The developer /
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


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"
GODOT_PROJECT_DIR = REPO_ROOT / "godot"

GODOT = shutil.which("godot")
CARGO = shutil.which("cargo")

# godot/compose_smoke.gd: STEPS = 7 * 24.
COMPOSED_STEPS = 168

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


def _run_smoke() -> dict:
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://compose_smoke.gd",
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


def _emit_station_headless() -> str:
    """The headless Rust reference: `emit_station` stdout is build_station JSON."""
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", "--example", "emit_station"],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"emit_station failed:\n{proc.stderr}"
    return proc.stdout


def test_godot_composed_station_cross_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke()

    # Tier-0 discretes + the composition primitive succeeded.
    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["parts"] == ["power_plant", "radiator"]
    assert report["step_count"] == COMPOSED_STEPS
    assert report["rationed"] == 0, "the composed heat-closure station is well-fed"

    # FP-environment parity: FTZ/DAZ OFF on the stepping thread.
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x})"
    )

    # Bit-exact vs the frozen reference: the palette composition {power_plant, radiator}
    # reproduces build_station byte-for-byte through the FFI boundary (a pure refactor).
    headless_snapshot = _emit_station_headless()
    assert report["snapshot"] == headless_snapshot, (
        "Godot cdylib composed snapshot differs from headless emit_station: "
        "either the palette builder is not a pure refactor or the FFI diverged"
    )


def _run_ui_smoke() -> tuple[dict, str]:
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://compose_ui_smoke.gd",
        ],
        capture_output=True,
        text=True,
        timeout=300,
    )
    out = proc.stdout
    assert _UI_BEGIN in out and _UI_END in out, (
        "UI smoke markers missing — the palette scene may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(_UI_BEGIN, 1)[1].split(_UI_END, 1)[0].strip()
    return json.loads(payload), proc.stderr


def test_godot_compose_palette_ui_loads() -> None:
    """The palette dashboard scene instantiates headless: its `_ready` runs
    `_build_ui`, Build wires the default composition through the real builder, and
    stepping advances the live sim. Closes the gap that cargo tests +
    `compose_smoke.gd` never load the scene."""
    _build_cdylib()
    _ensure_godot_import()
    report, stderr = _run_ui_smoke()
    assert report["ok"] is True, f"palette UI smoke failed: {report}\nstderr:\n{stderr}"
    assert report["child_count"] > 0, "dashboard built no widgets (_build_ui skipped)"
    # A GDScript error prints "SCRIPT ERROR" to stderr even if the scene loads.
    assert "SCRIPT ERROR" not in stderr, f"GDScript error in UI smoke:\n{stderr}"
