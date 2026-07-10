# Phase-8 (P8.3) time-controls cross-boundary smoke — proves the OFF-RENDER-THREAD stepping
# path is bit-exact through the actual cdylib AND that the worker (stepping) thread has a
# clean FP env. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://time_smoke.gd
#
# Unlike `smoke.gd` (Step 1, which steps the synchronous `SimSession` on the render thread),
# this drives a `TimeController`: `build()` spawns the Rust worker thread, `fast_forward_to()`
# advances it off-thread, and this MainLoop polls `is_fast_forwarding()` / `step_count()` each
# iteration (never blocking) until the target is reached. It then emits, between unique
# markers, a JSON report carrying:
#   * the Rust-side hex-float snapshot (`snapshot_json`, the golden codec),
#   * the MXCSR word + `fp_clean` read ON THE WORKER (stepping) THREAD,
#   * the Tier-0 discretes (step count, rationed, error message).
# The Python comparator (`tests/crossport/test_godot_time_controls.py`) asserts the worker's
# fast-forward result is Tier-1 bit-exact vs the frozen `cabin_gas_state.json` and headless
# `emit_cabin_gas`, and that FTZ/DAZ are OFF on the worker thread.

extends SceneTree

const SCENARIO := "cabin_gas"
const TARGET := 900  # station::scenario::CABIN_GAS_STEPS
const FRAME_BUDGET := 200000  # generous upper bound on poll iterations before declaring a hang

var tc: TimeController
var report := {
	"ok": false,
	"scenario": SCENARIO,
	"fp_clean": false,
	"mxcsr": 0,
	"step_count": -1,
	"rationed": -1,
	"error_msg": "",
	"snapshot": "",
}
var frames := 0

func _initialize() -> void:
	tc = TimeController.new()
	if not tc.build(SCENARIO):
		report["error_msg"] = "build failed"
		_emit()
		quit(1)
		return
	# FP env is read by the worker on its own thread and published during build; capture it.
	report["fp_clean"] = tc.fp_clean()
	report["mxcsr"] = tc.worker_mxcsr()
	# Kick off the off-thread fast-forward; _process polls for completion.
	tc.fast_forward_to(TARGET)

# MainLoop._process returns true to quit. Poll the worker without blocking the loop.
func _process(_delta: float) -> bool:
	if tc == null:
		return true
	frames += 1
	if frames > FRAME_BUDGET:
		report["error_msg"] = "timeout waiting for fast-forward"
		report["step_count"] = tc.step_count()
		_emit()
		return true
	# Still advancing? keep spinning.
	if tc.is_fast_forwarding() or tc.step_count() < TARGET:
		return false
	# Reached the target and paused — capture the result.
	report["step_count"] = tc.step_count()
	report["rationed"] = tc.total_rationed()
	report["error_msg"] = tc.error_message()
	report["fp_clean"] = tc.fp_clean()
	report["mxcsr"] = tc.worker_mxcsr()
	report["snapshot"] = tc.snapshot_json()
	report["ok"] = report["error_msg"] == ""
	_emit()
	return true

func _emit() -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
