# Phase-8 (P8.6) cross-boundary COMPOSITION smoke — the byte-identity anchor carried through
# the ACTUAL cdylib. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://compose_smoke.gd
#
# It builds the ENERGY station from the FIXED PALETTE (`build_composed(["power_plant",
# "radiator"])`, the interactive "build systems" primitive) through the real `godot_bridge`
# cdylib Godot loads, then steps the 7-day heat-closure horizon and emits a JSON report with:
#   * the Rust-side hex-float snapshot (`snapshot_json`, the golden codec),
#   * total `rationed`, the FP flags, and the step count.
# The Python comparator (`tests/crossport/test_godot_compose.py`) asserts the composed snapshot
# is byte-identical to the headless `emit_station` output — i.e. the palette composition
# {power_plant, radiator} reproduces the frozen `build_station` bit-for-bit ACROSS the FFI
# boundary (the builder is a pure refactor, and the boundary didn't corrupt determinism) — plus
# `rationed == 0` and FTZ/DAZ OFF on the stepping thread.

extends SceneTree

const PARTS := ["power_plant", "radiator"]
const SPD := 24                 # HEAT_CLOSURE_SCENARIO power.steps_per_day
const HEAT_CLOSURE_DAYS := 7
const STEPS := HEAT_CLOSURE_DAYS * SPD   # 168 — matches emit_station

func _initialize() -> void:
	var report := {
		"ok": false,
		"parts": PARTS,
		"fp_clean": false,
		"mxcsr": 0,
		"step_count": -1,
		"rationed": -1,
		"snapshot": "",
	}
	var sim := SimSession.new()
	if not sim.build_composed(PackedStringArray(PARTS)):
		_emit(report)
		quit(1)
		return
	# FP-env read on the same thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()

	for i in STEPS:
		if not sim.step():
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
