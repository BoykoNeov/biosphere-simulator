"""Phase-8 (P8.8) — the **headless CLI** bit-identity gate (confirmed decision #2:
"runs headless on a server" is satisfied by architecture, not netcode).

The `station` crate ships a plain `sim <scenario> <steps>` binary (`src/bin/sim.rs`)
that builds a fixed-palette session through the **same**
`station::palette::build_scenario` the Godot cdylib uses, advances it, and prints the
bit-exact `sim_io` hex-float snapshot. Because it shares the builder with the front-end
and prints through the frozen `simcore::snapshot` codec, its output is the **same
simulation** as the corresponding `emit_*` example — by construction, not by an agreeing
re-implementation.

This test proves that byte-for-byte for three palette entries (two single-rate, one
two-rate), demonstrating the headless entry point exists and is the exact same sim the
front-end drives:

  * `sim cabin_gas 900`   == `emit_cabin_gas`   (single-rate, Tier-1)
  * `sim station 168`     == `emit_station`     (single-rate, Power → Thermal)
  * `sim greenhouse 7`    == `emit_greenhouse`  (two-rate master days)

Both sides are pure Rust on the same libm, so this is bit-exact on **any** platform — it
needs **no Godot** and runs in the CI `crossport` job (cargo present). It is the CLI
analogue of the cross-boundary Godot smokes, minus the FFI boundary.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_WORKSPACE_DIR = REPO_ROOT / "rust"

CARGO = shutil.which("cargo")

pytestmark = pytest.mark.skipif(CARGO is None, reason="cargo absent")

# (scenario id, CLI step count, matching emit example). The step counts mirror the
# frozen horizons: CABIN_GAS_STEPS=900, HEAT_CLOSURE_DAYS*24=168, greenhouse days=7.
_CASES = [
    ("cabin_gas", "900", "emit_cabin_gas"),
    ("station", "168", "emit_station"),
    ("greenhouse", "7", "emit_greenhouse"),
]


def _run(args: list[str]) -> str:
    proc = subprocess.run(
        ["cargo", "run", "-q", "-p", "station", *args],
        cwd=RUST_WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    assert proc.returncode == 0, f"cargo run {args} failed:\n{proc.stderr}"
    return proc.stdout


@pytest.mark.parametrize("scenario,steps,example", _CASES)
def test_headless_cli_matches_emit_example(
    scenario: str, steps: str, example: str
) -> None:
    cli = _run(["--bin", "sim", "--", scenario, steps])
    emit = _run(["--example", example])
    assert cli == emit, (
        f"`sim {scenario} {steps}` differs from `{example}` byte-for-byte — the "
        "headless CLI is not the same simulation as the emit reference"
    )


def test_headless_cli_rejects_bad_args() -> None:
    """A wrong scenario id / bad step count / arity error exits non-zero (a server
    harness can trust the exit code), not a panic or silent empty output."""
    for args in (
        ["--bin", "sim", "--", "no_such_scenario", "10"],
        ["--bin", "sim", "--", "cabin_gas"],
    ):
        proc = subprocess.run(
            ["cargo", "run", "-q", "-p", "station", *args],
            cwd=RUST_WORKSPACE_DIR,
            capture_output=True,
            text=True,
        )
        assert proc.returncode != 0, f"expected failure exit for {args}, got 0"
