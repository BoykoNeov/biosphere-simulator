"""Phase-8 (P8.4) flow-inspection cross-boundary smoke — a headless wiring check that
the flow-level display projection (`station::inspection`) crosses the actual
`godot_bridge` cdylib boundary intact.

This is **not** a parity gate — the flow inspection is a zero-parity display read (plain
decimal floats; the bit-exact hex-float path stays on `snapshot_json`, gated by
`test_godot_parity.py`). It only proves `flow_inspection_json()` returns well-formed
JSON THROUGH gdext that GDScript parses, carrying the real station flows and their legs,
and that a two-rate entry correctly defers (returns ""). The *truthfulness* of the legs
— that they reconstruct the applied step delta — is proven in Rust (station::inspection
+ station::session teeth); this checks the boundary, not the math.

**Local-only**, exactly like the other Godot crossport smokes: `skipif` when `godot` or
`cargo` is absent. CI installs neither, so this never runs there — it is the developer /
release-time boundary proof. No MCP needed (headless).
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

_SMOKE_BEGIN = "<<<FLOW_SMOKE_BEGIN"
_SMOKE_END = "FLOW_SMOKE_END>>>"

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
        f"{script} markers missing — extension may not have loaded:\n"
        f"stdout:\n{out}\nstderr:\n{proc.stderr}"
    )
    payload = out.split(begin, 1)[1].split(end, 1)[0].strip()
    return json.loads(payload), proc.stderr


def _run_smoke() -> dict:
    report, _ = _run_script("flow_smoke.gd", _SMOKE_BEGIN, _SMOKE_END)
    return report


def test_godot_flow_inspection_crosses_the_boundary() -> None:
    _build_cdylib()
    _ensure_godot_import()
    report = _run_smoke()

    assert report["ok"] is True, f"flow smoke did not complete ok: {report}"
    assert report["scenario"] == "station"
    assert report["n"] == 24

    # The Power → Thermal station registry, id-sorted: charge + load draw + radiator
    # (HeatInput dropped — Power's dissipation is the input, the Step-1 seam).
    flow_ids = report["flow_ids"]
    for expected in (
        "power.solar_charge",
        "power.load_draw",
        "thermal.radiator_reject",
    ):
        assert expected in flow_ids, f"missing flow {expected!r} in {flow_ids}"

    # The "select thermal.node → contributing flows" join survived the boundary: the
    # radiator rejects heat OFF the node (negative leg) and Power's dissipation legs
    # feed it (a positive contributor) — the Step-1 cross-domain seam, made inspectable.
    contributors = {fid: amount for fid, amount in report["node_contributors"]}
    assert "thermal.radiator_reject" in contributors, contributors
    assert contributors["thermal.radiator_reject"] < 0.0, (
        "radiator should withdraw from node"
    )
    assert any(a > 0.0 for a in contributors.values()), (
        "dissipation should feed the node"
    )

    # A two-rate entry defers inspection (single-rate only) → empty string, not error.
    assert report["two_rate_empty"] is True


def test_main_dashboard_renders_the_flow_panel() -> None:
    """Instantiate `main.tscn` headless so `main.gd`'s `_process` → `_render_flows` runs
    through the engine — the Step-3 `ui_smoke.gd` precedent: the flow panel is UI code
    otherwise loaded by nothing (a parse error / shadow would ship undetected)."""
    _build_cdylib()
    _ensure_godot_import()
    report, stderr = _run_script(
        "main_ui_smoke.gd", "<<<MAIN_UI_SMOKE_BEGIN", "MAIN_UI_SMOKE_END>>>"
    )
    # Any GDScript fault in main.gd prints one of these to stderr (the MCP addon's own
    # noise — port-in-use / gdaimcp capture — is not a GDScript fault and is ignored).
    for marker in ("SCRIPT ERROR", "Parse Error", "SHADOWED_VARIABLE"):
        assert marker not in stderr, f"GDScript fault in main.gd:\n{stderr}"
    assert report["ok"] is True, (
        f"main dashboard did not render the flow panel: {report}"
    )
    assert report["has_flows_panel"] is True
    assert report["has_contributing"] is True
    # P8.5: the interactive "perturb systems" trigger runs headless (the keypress path)
    # and updates the dashboard header — the load-bearing rationing/cascade is proven by
    # test_godot_perturbations.py; here we only confirm the UI trigger path is wired.
    assert report["perturbation_triggered"] is True
    assert report["header_shows_perturbation"] is True
