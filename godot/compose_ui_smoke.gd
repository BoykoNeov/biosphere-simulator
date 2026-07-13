# Phase-8 (P8.6) UI-load smoke — instantiates the interactive `compose_dashboard.tscn`
# HEADLESS so its GDScript is actually parsed and its `_ready` → `_build_ui` runs through the
# engine (the P8.3 `ui_smoke.gd` precedent). The cargo tests + `compose_smoke.gd` exercise the
# `build_composed` API but never load the palette scene; this closes that gap — a parse error,
# wrong signal arity, or a bad `.bind()` in `compose_dashboard.gd` would otherwise ship
# undetected.
#
#   godot --headless --path godot --script res://compose_ui_smoke.gd
#
# Headless provides a dummy DisplayServer, so Control/CheckBox/Button all instantiate and
# `.connect()` runs. The script builds the dashboard, presses Build via the default checked
# parts (power_plant + radiator), lets a few frames tick, then emits a marked report. The Python
# gate also asserts stderr carries no GDScript error.

extends SceneTree

const FRAMES_TO_RUN := 4

var inst: Node
var frames := 0

func _initialize() -> void:
	var scene: PackedScene = load("res://compose_dashboard.tscn")
	if scene == null:
		_emit({"ok": false, "error": "compose_dashboard.tscn failed to load"})
		quit(1)
		return
	inst = scene.instantiate()
	# add_child fires _ready → _build_ui (the code path never otherwise exercised). The build
	# is driven from _process (below) so _ready has run first — checkboxes exist by then.
	get_root().add_child(inst)

func _process(_delta: float) -> bool:
	if inst == null:
		_emit({"ok": false, "error": "instance is null"})
		return true
	frames += 1
	if frames == 1:
		# _ready → _build_ui has now run; drive the default-checked composition through the
		# real builder, then step a day.
		inst._on_build()
		inst._on_step_day()
		return false
	if frames < FRAMES_TO_RUN:
		return false
	# _build_ui adds a VBoxContainer (with rows) — a non-zero count proves it ran; a live sim
	# after Build proves the builder wired through the UI.
	var built: bool = inst.sim != null and inst.sim.step_count() > 0
	_emit({"ok": inst.get_child_count() > 0 and built, "child_count": inst.get_child_count()})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("UI_SMOKE_END>>>")
