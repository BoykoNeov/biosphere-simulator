# Phase-8 (P8.4) main-dashboard UI-load smoke — instantiates `main.tscn` HEADLESS so its
# `main.gd` is parsed and its `_ready` → `_process` → `_render` + `_render_flows` actually
# run through the engine. The cargo tests + `flow_smoke.gd` exercise the bridge API but never
# load the dashboard scene; this closes that gap (the Step-3 `ui_smoke.gd` precedent — a parse
# error or a base-class shadow in `main.gd`'s new flow panel would otherwise ship undetected).
#
#   godot --headless --path godot --script res://main_ui_smoke.gd
#
# Headless provides a dummy DisplayServer, so the Label instantiates and `_process` ticks;
# only actual pixels need a real display server. After a few frames the report carries the
# Label text so the Python gate can confirm the flow panel rendered ("[flows]" present) and
# stderr is free of GDScript errors.

extends SceneTree

const FRAMES_TO_RUN := 4

var inst: Node
var frames := 0

func _initialize() -> void:
	var scene: PackedScene = load("res://main.tscn")
	if scene == null:
		_emit({"ok": false, "error": "main.tscn failed to load"})
		quit(1)
		return
	inst = scene.instantiate()
	get_root().add_child(inst)  # fires _ready

func _process(_delta: float) -> bool:
	if inst == null:
		_emit({"ok": false, "error": "instance is null"})
		return true
	frames += 1
	if frames < FRAMES_TO_RUN:
		return false
	var text: String = inst.get_node("Label").text
	_emit({
		"ok": text.contains("[flows]") and text.contains("station"),
		"has_flows_panel": text.contains("[flows]"),
		"has_contributing": text.contains("[contributing flows]"),
		"text_len": text.length(),
	})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<MAIN_UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("MAIN_UI_SMOKE_END>>>")
