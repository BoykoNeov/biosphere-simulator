# Phase-8 (P8.8) cross-boundary SEALED parity smoke — the "fast-forward decades survives the
# FFI boundary" arm. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://sealed_smoke.gd
#
# It drives the full five-domain sealed station (two-rate, re-sown) through the ACTUAL cdylib for
# SEALED_RESUME_DAYS = 310 master days — a few days past one 305-day season, so the re-sow
# (`slow_reset`) adopt branch fires ACROSS the boundary (the genuinely-new coverage over Step-1's
# single-rate cabin_gas). Each `step()` is 1440 fast sub-steps, so this is ~450k sub-steps of
# real compute — SLOW-marked on the Python side. The full multi-year horizon is proven
# intra-process (`tests/session_parity.rs`) + by the frozen golden; this proves the FFI boundary
# didn't corrupt the two-rate, season-crossing trajectory.
#
# The Python comparator (`tests/crossport/test_godot_two_rate_parity.py::…sealed…`) asserts the
# snapshot is byte-identical to the headless `emit_sealed_resume` output, plus FTZ/DAZ OFF and
# `rationed == 0`.

extends SceneTree

const SCENARIO := "sealed"
const DAYS := 310  # station::scenario::SEALED_RESUME_DAYS (SEALED_STATION_SEASON_DAYS + 5)

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
