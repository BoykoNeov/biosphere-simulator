# Phase-8 (P8.7) UI-load smoke — instantiates the interactive `save_dashboard.tscn` HEADLESS so
# its GDScript is actually parsed and its `_ready` → `_build_ui` runs, then drives a full
# Build → Step → Save → Load cycle through the real bridge (the P8.3/P8.6 `ui_smoke` precedent).
# The cargo tests + `save_smoke.gd` exercise the save/load API but never load the dashboard scene;
# this closes that gap — a parse error, wrong signal arity, or a bad `.bind()` in
# `save_dashboard.gd` would otherwise ship undetected.
#
#   godot --headless --path godot --script res://save_ui_smoke.gd
#
# Headless provides a dummy DisplayServer, so Control/Button/Label all instantiate and
# `.connect()` runs. The Python gate also asserts stderr carries no GDScript error and that the
# Save actually wrote a file that Load restored (step count preserved across the load).

extends SceneTree

const FRAMES_TO_RUN := 6

var inst: Node
var frames := 0
var n_before_save := -1

func _initialize() -> void:
	var scene: PackedScene = load("res://save_dashboard.tscn")
	if scene == null:
		_emit({"ok": false, "error": "save_dashboard.tscn failed to load"})
		quit(1)
		return
	inst = scene.instantiate()
	# add_child fires _ready → _build_ui (the code path never otherwise exercised).
	get_root().add_child(inst)

func _process(_delta: float) -> bool:
	if inst == null:
		_emit({"ok": false, "error": "instance is null"})
		return true
	frames += 1
	if frames == 1:
		# _ready → _build_ui has run; build the default scenario and step it.
		inst._on_build()
		inst._on_step_many()
		n_before_save = inst.sim.step_count()
		return false
	if frames == 2:
		# Save the stepped session to disk.
		inst._on_save()
		return false
	if frames == 3:
		# Load it back into a fresh session (the round-trip through FileAccess).
		inst._on_load()
		return false
	if frames < FRAMES_TO_RUN:
		return false
	# A non-zero child count proves _build_ui ran; a live sim resumed at the saved n proves the
	# Build → Save → Load cycle wired through the UI end to end.
	var loaded_ok: bool = inst.sim != null and inst.sim.step_count() == n_before_save
	_emit({
		"ok": inst.get_child_count() > 0 and loaded_ok and n_before_save > 0,
		"child_count": inst.get_child_count(),
		"n_before_save": n_before_save,
		"n_after_load": inst.sim.step_count() if inst.sim != null else -1,
	})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("UI_SMOKE_END>>>")
