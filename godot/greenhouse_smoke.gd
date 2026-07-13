# Phase-8 (P8.8) cross-boundary TWO-RATE parity smoke (greenhouse) — the first two-rate
# scenario driven through the ACTUAL cdylib. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://greenhouse_smoke.gd
#
# Step-1's smoke proved the FFI boundary preserves determinism for the SINGLE-RATE `cabin_gas`.
# This closes the two-rate gap: each `step()` here is one MASTER DAY = one slow biosphere step +
# 1440 fast cabin sub-steps, so it exercises the two-rate driver (`advance_one_master_day`)
# across the boundary. `greenhouse` is `reset = None`, 7 master days — cheap enough to run in the
# fast crossport gate (the sealed re-sow arm is the slow companion, `sealed_smoke.gd`).
#
# The Python comparator (`tests/crossport/test_godot_two_rate_parity.py`) asserts the snapshot is
# byte-identical to the headless `emit_greenhouse` output, plus FTZ/DAZ OFF and `rationed == 0`.

extends SceneTree

const SCENARIO := "greenhouse"
const DAYS := 7  # station::scenario greenhouse_scenario().days

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
	# FP-env read on the same thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()
	if not sim.step_n(DAYS):
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
