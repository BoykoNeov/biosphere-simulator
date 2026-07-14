# Phase-9 (Step 5) UI-load smoke — instantiates the interactive `from_file_dashboard.tscn`
# HEADLESS so its GDScript is actually parsed and its `_ready` → `_build_ui` runs through the
# engine (the P8.3/P8.6 `ui_smoke.gd` precedent). The cargo tests + `from_file_smoke.gd`
# exercise the `build_from_file` API but never load the dashboard scene; this closes that gap —
# a parse error, a wrong signal arity, or a bad `.bind()`/`globalize_path` in
# `from_file_dashboard.gd` would otherwise ship undetected.
#
#   godot --headless --path godot --script res://from_file_ui_smoke.gd
#
# Headless provides a dummy DisplayServer, so Control/LineEdit/Button all instantiate and
# `.connect()` runs. The script builds the dashboard, presses Load (the default LineEdit path —
# `res://../tests/.../crew_mission.yaml`, globalized to an OS path) through the real
# `build_from_file`, steps once, then emits a marked report. The Python gate also asserts stderr
# carries no GDScript error, and that the load actually took (a live sim with n > 0), proving the
# `res://`-parent-dir default resolves end-to-end through the FFI.

extends SceneTree

const FRAMES_TO_RUN := 4

var inst: Node
var frames := 0

func _initialize() -> void:
	var scene: PackedScene = load("res://from_file_dashboard.tscn")
	if scene == null:
		_emit({"ok": false, "error": "from_file_dashboard.tscn failed to load"})
		quit(1)
		return
	inst = scene.instantiate()
	# add_child fires _ready → _build_ui (the code path never otherwise exercised). Load is
	# driven from _process (below) so _ready has run first — the LineEdit exists by then.
	get_root().add_child(inst)

func _process(_delta: float) -> bool:
	if inst == null:
		_emit({"ok": false, "error": "instance is null"})
		return true
	frames += 1
	if frames == 1:
		# _ready → _build_ui has now run; load the default authored scenario through the real
		# `build_from_file`, then step it once.
		inst._on_load()
		inst._on_step()
		return false
	if frames < FRAMES_TO_RUN:
		return false
	# _build_ui adds a VBoxContainer (with rows) — a non-zero count proves it ran; a live sim
	# past step 0 after Load proves the file loaded + stepped through the UI.
	var loaded: bool = inst.sim != null and inst.sim.step_count() > 0
	_emit({
		"ok": inst.get_child_count() > 0 and loaded,
		"child_count": inst.get_child_count(),
		"step_count": inst.sim.step_count() if inst.sim != null else -1,
	})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("UI_SMOKE_END>>>")
