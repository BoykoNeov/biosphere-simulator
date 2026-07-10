# Phase-8 (P8.1) cross-boundary parity smoke — the load-bearing "the FFI didn't corrupt
# determinism" proof (advisor #2). Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://smoke.gd
#
# It drives the Tier-1 `cabin_gas` scenario (transcendental-free ⇒ bit-exact on any
# platform/libm) through the ACTUAL `godot_bridge` cdylib Godot loads — the boundary the
# intra-process `tests/session_parity.rs` structurally cannot see (FP-environment
# divergence such as FTZ/DAZ flags a game engine may set per-thread). It emits, between
# unique markers on stdout, a JSON report carrying:
#   * the Rust-side hex-float snapshot (`snapshot_json`, the golden codec — no GDScript
#     float printing in the parity path),
#   * the MXCSR word + a `fp_clean` flag read ON THE STEPPING THREAD,
#   * the Tier-0 discretes (step count, rationed).
# The Python comparator (`tests/crossport/test_godot_parity.py`) captures stdout, parses
# the report, and asserts the snapshot is Tier-1 bit-exact vs the frozen
# `cabin_gas_state.json` and that FTZ/DAZ are OFF.

extends SceneTree

const SCENARIO := "cabin_gas"
const STEPS := 900  # station::scenario::CABIN_GAS_STEPS

func _initialize() -> void:
	var report := {
		"ok": false,
		"scenario": SCENARIO,
		"fp_clean": false,
		"mxcsr": 0,
		"step_count": -1,
		"rationed": -1,
		"snapshot": "",
	}
	var sim := SimSession.new()
	if not sim.build(SCENARIO):
		_emit(report)
		quit(1)
		return
	# FP-env read must happen on the same thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()
	if not sim.step_n(STEPS):
		_emit(report)
		quit(2)
		return
	report["step_count"] = sim.step_count()
	report["rationed"] = sim.total_rationed()
	report["snapshot"] = sim.snapshot_json()
	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
