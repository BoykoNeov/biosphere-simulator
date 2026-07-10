# Phase-8 (P8.3) UI-load smoke — instantiates the interactive `time_dashboard.tscn` HEADLESS
# so its GDScript is actually parsed and its `_ready` → `_build_ui` runs through the engine.
# The cargo tests + `time_smoke.gd` exercise the `TimeController` API but never load the
# dashboard scene; this closes that gap (advisor): a parse error, wrong signal arity, or base-
# class shadow in `time_dashboard.gd` would otherwise ship undetected.
#
#   godot --headless --path godot --script res://ui_smoke.gd
#
# Headless provides a dummy DisplayServer, so Control/HBoxContainer/HSlider/OptionButton/Button
# all instantiate and `.connect()` runs — only actual pixels need a real display server. The
# script builds the dashboard, lets a few frames tick (so the dashboard's own `_process` +
# `_render` run), then emits a marked report. The Python gate also asserts stderr carries no
# GDScript error.

extends SceneTree

const FRAMES_TO_RUN := 4

var inst: Node
var frames := 0

func _initialize() -> void:
	var scene: PackedScene = load("res://time_dashboard.tscn")
	if scene == null:
		_emit({"ok": false, "error": "time_dashboard.tscn failed to load"})
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
	if frames < FRAMES_TO_RUN:
		return false
	# _build_ui adds a VBoxContainer (with rows) as a child — a non-zero count proves it ran.
	_emit({"ok": inst.get_child_count() > 0, "child_count": inst.get_child_count()})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("UI_SMOKE_END>>>")
