"""Phase-8 (P8.1) cross-boundary parity smoke — the FIRST time the Godot FFI boundary
exists, so the first check the intra-process `session_parity.rs` (Step 0) structurally
cannot make (advisor #2).

The Step-0 parity teeth prove `N×session.step() == run_station(N)` **inside one cargo
process** — same compiled code, same FP environment — so they are blind to
Godot-hosted-vs-headless divergence, which is exactly what the Phase-8 exit criterion
("the exact same simulation runs headless") rides on. The concrete break risk is
per-thread FP control flags: a game engine that sets **FTZ/DAZ** (flush-to-zero /
denormals-are-zero in MXCSR) for SIMD throughput would flush denormal intermediates to
zero and diverge from the IEEE-default headless run.

This test drives the Tier-1 `cabin_gas` scenario (transcendental-free ⇒ bit-exact on any
platform/libm) through the **actual `godot_bridge` cdylib Godot loads**, via
`godot/smoke.gd`, and asserts:

  * **bit-exact snapshot** — the Godot-produced `sim_io` hex-float snapshot equals the
    headless `emit_cabin_gas` output byte-for-byte (pure-Rust both sides, no formatting
    confound), AND matches the frozen `cabin_gas_state.json` golden at Tier 1 (parsed
    f64, bit-exact) — the frozen-reference bind;
  * **FP env clean** — `fp_clean()` read on the stepping thread reports FTZ and DAZ both
    OFF (the direct check the bit-exact smoke alone cannot make: `cabin_gas` may never
    produce a denormal, so a passing snapshot does not by itself prove flush-to-zero is
    off);
  * **Tier-0 discretes** — `rationed == 0`, `step_count == 900`.

**Local-only**, exactly like the Rust-vs-Python `test_crossport.py` gates: `skipif` when
`godot` or `cargo` is absent. The CI runners install neither Godot nor the gdext
toolchain, so this never runs on CI — it is the developer / release-time boundary proof
(the Step-8 gating version promotes it). No MCP needed (headless).
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
    the `SimSession` class."""
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
    # `--import` may exit non-zero on benign editor warnings; the real gate is that the
    # extension list got written.
    assert ext_list.exists(), (
        "Godot import did not register the extension:\n"
        f"stdout:\n{proc.stdout}\nstderr:\n{proc.stderr}"
    )


def _run_smoke() -> dict:
    """Run `smoke.gd` headless through the actual cdylib; return the parsed report."""
    assert GODOT is not None  # narrowed by the module-level skipif
    proc = subprocess.run(
        [
            GODOT,
            "--headless",
            "--path",
            str(GODOT_PROJECT_DIR),
            "--script",
            "res://smoke.gd",
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


def test_godot_cabin_gas_cross_boundary_parity() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke()

    # Tier-0 discretes: the run completed well-fed.
    assert report["ok"] is True, f"smoke did not complete ok: {report}"
    assert report["scenario"] == "cabin_gas"
    assert report["step_count"] == CABIN_GAS_STEPS
    assert report["rationed"] == 0

    # FP-environment parity: FTZ/DAZ OFF on the stepping thread. This is the check the
    # bit-exact snapshot alone cannot make (cabin_gas may never produce a denormal).
    assert report["fp_clean"] is True, (
        f"FTZ/DAZ set on the Godot stepping thread (mxcsr={report['mxcsr']:#x}); "
        "denormal intermediates would flush to zero and diverge from headless"
    )

    godot_snapshot = report["snapshot"]

    # (1) Bit-exact vs the headless Rust reference — pure Rust `to_json()` both sides,
    # no formatting confound: any FFI/FP corruption of the trajectory changes the bytes.
    headless_snapshot = _emit_cabin_gas_headless()
    assert godot_snapshot == headless_snapshot, (
        "Godot cdylib snapshot differs from headless emit_cabin_gas byte-for-byte — "
        "the FFI boundary corrupted determinism"
    )

    # (2) Tier-1 bit-exact vs the frozen golden — the frozen-reference bind (f64).
    golden = compare.load_json(GOLDEN_DIR / "cabin_gas_state.json")
    result = compare.compare(golden, json.loads(godot_snapshot), tier=1)
    assert result.ok, result.report()
