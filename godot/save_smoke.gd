# Phase-8 (P8.7) cross-boundary SAVE/LOAD smoke — the disk round-trip carried through the
# ACTUAL cdylib. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://save_smoke.gd
#
# The genuinely-new content over the prior smokes is a REAL save/load cycle THROUGH DISK: build
# the Tier-1 `cabin_gas` scenario, step to a mid-run save point, `save()` the record and write it
# with Godot's `FileAccess`, then DROP the session, make a FRESH `SimSession`, read the file back,
# `load()` it (rebuild-from-recipe + restore state), and resume to the full horizon. The final
# snapshot must equal the headless `emit_cabin_gas` (== the frozen `cabin_gas_state.json`)
# byte-for-byte — i.e. save/load through the real engine + Godot's file API preserved determinism
# exactly (the `(seed, key, n)` corollary), not just intra-process. Plus `rationed == 0`, the
# full step count, and FTZ/DAZ OFF on the stepping thread.

extends SceneTree

const SCENARIO := "cabin_gas"
const SAVE_AT := 300
const TOTAL := 900                 # CABIN_GAS_STEPS — reaching it reproduces emit_cabin_gas
const SAVE_PATH := "user://save_smoke.json"

func _initialize() -> void:
	var report := {
		"ok": false,
		"fp_clean": false,
		"mxcsr": 0,
		"step_count": -1,
		"rationed": -1,
		"snapshot": "",
		"saved_ok": false,
		"loaded_ok": false,
	}
	var sim := SimSession.new()
	if not sim.build(SCENARIO):
		_emit(report); quit(1); return
	# FP-env read on the thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()

	# Step to the save point.
	for i in SAVE_AT:
		if not sim.step():
			_emit(report); quit(2); return

	# SAVE: get the record and write it to disk with FileAccess.
	var save_text: String = sim.save()
	if save_text == "":
		_emit(report); quit(3); return
	var wf := FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if wf == null:
		_emit(report); quit(4); return
	wf.store_string(save_text)
	wf.close()
	report["saved_ok"] = true

	# Drop the original; a FRESH session reads the file and LOADs it (the real round-trip —
	# rebuild-from-recipe + restore state across a brand-new session object).
	sim = null
	var resumed := SimSession.new()
	var rf := FileAccess.open(SAVE_PATH, FileAccess.READ)
	if rf == null:
		_emit(report); quit(5); return
	var loaded_text := rf.get_as_text()
	rf.close()
	if not resumed.load(loaded_text):
		_emit(report); quit(6); return
	report["loaded_ok"] = true

	# Resume to the full horizon.
	for i in (TOTAL - SAVE_AT):
		if not resumed.step():
			_emit(report); quit(7); return
	report["step_count"] = resumed.step_count()
	report["rationed"] = resumed.total_rationed()
	report["snapshot"] = resumed.snapshot_json()
	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
