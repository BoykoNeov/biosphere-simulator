# Phase-9 (Step 5) cross-boundary parity smoke — "Godot loads a scenario FILE at runtime,"
# the "author, not program" payoff, driven through the ACTUAL `godot_bridge` cdylib Godot loads.
# Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://from_file_smoke.gd -- <abs/scenario.yaml>
#
# The scenario path is passed as a user arg (after `--`) rather than hardcoded as a `res://`
# URI: (1) `build_from_file` takes an OS filesystem path (the frozen `authoring` loader reads
# it with `std::fs`), and (2) passing the exact same absolute path the Python test hands to the
# headless `emit_authored` reference guarantees both load the *identical committed file* — so a
# byte-identity failure can only mean the FFI boundary corrupted determinism, never a
# two-different-files confound (the Step-1 `smoke.gd` FTZ/DAZ discipline, one authoring level up).
#
# It emits, between unique markers on stdout, a JSON report carrying the Rust-side hex-float
# snapshot (`snapshot_json`, the golden codec — no GDScript float printing in the parity path),
# the MXCSR word + `fp_clean` read ON THE STEPPING THREAD, and the Tier-0 discretes. The Python
# comparator (`tests/crossport/test_godot_from_file.py`) asserts the snapshot is byte-identical
# to headless `emit_authored` AND Tier-1 bit-exact vs the frozen `crew_state.json`, and FTZ/DAZ
# are OFF. The anchor is `crew_mission.yaml` — transcendental-free (Tier-1, platform-independent
# bit-exact) and already reproducing `crew_state.json` per Step 4b, so the smoke re-proves that
# known golden *through the file-load FFI boundary*.

extends SceneTree

func _initialize() -> void:
	var report := {
		"ok": false,
		"fp_clean": false,
		"mxcsr": 0,
		"total_steps": -1,
		"step_count": -1,
		"rationed": -1,
		"snapshot": "",
	}
	var user_args := OS.get_cmdline_user_args()
	if user_args.is_empty():
		push_error("from_file_smoke: expected an absolute scenario path as a user arg (after --)")
		_emit(report)
		quit(1)
		return
	var path: String = user_args[0]

	var sim := SimSession.new()
	if not sim.build_from_file(path):
		push_error("from_file_smoke: build_from_file(%s) failed" % path)
		_emit(report)
		quit(1)
		return
	# FP-env read must happen on the same thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()
	# The file declares its own horizon — step exactly what the headless run does.
	var steps: int = sim.total_steps()
	report["total_steps"] = steps
	if not sim.step_n(steps):
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
