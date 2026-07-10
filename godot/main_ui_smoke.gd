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

const BASELINE_FRAMES := 4     # let the baseline dashboard render
const PERTURBED_FRAMES := 4    # after the interactive trigger, confirm the header/panel update
                               # (rationing visibility is proven by perturbation_smoke.gd)

var inst: Node
var frames := 0
var perturbed := false
var perturb_ok := false

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

	# Phase 1: baseline dashboard rendered — then fire the P8.5 interactive trigger (the same
	# path a `2` keypress takes), a deep brownout that will ration LoadDraw as the battery
	# empties (so the flow panel eventually shows a scale < 1).
	if not perturbed and frames >= BASELINE_FRAMES:
		perturb_ok = inst._apply_perturbation("brownout", 0.0, "deep brownout (smoke)")
		perturbed = true
		frames = 0
		return false

	if not perturbed or frames < PERTURBED_FRAMES:
		return false

	var text: String = inst.get_node("Label").text
	_emit({
		"ok": text.contains("[flows]") and text.contains("station") and perturb_ok,
		"has_flows_panel": text.contains("[flows]"),
		"has_contributing": text.contains("[contributing flows]"),
		"perturbation_triggered": perturb_ok,
		"header_shows_perturbation": text.contains("[perturbation:"),
		"rationing_visible": text.contains("rationed →"),
		"text_len": text.length(),
	})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<MAIN_UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("MAIN_UI_SMOKE_END>>>")
