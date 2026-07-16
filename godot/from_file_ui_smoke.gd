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
#
# It then Loads a SECOND, kinetics-bearing scenario to drive the **"authored ≠ validated"**
# banner (`docs/authoring-reference.md`, decision B) through the real widget: the marker's whole
# point is what a player SEES, so asserting the getter's bool elsewhere does not cover the
# `_uncalibrated_label.visible = sim.has_authored_kinetics()` binding. Both edges are captured —
# hidden for the kinetics-free default, shown for the authored file — and the reload back to the
# default proves the banner clears rather than latching on.

extends SceneTree

const FRAMES_TO_RUN := 6

# A kinetics-bearing scenario (relative to the Godot project dir, like the dashboard's own
# default) — the banner's positive case.
const AUTHORED_SCENARIO := "res://../tests/authoring/scenarios/self_discharge_dsl.yaml"

var inst: Node
var frames := 0
# `step_count` is captured at the first Load+Step, because the later banner Loads replace the
# session (a fresh one sits at step 0) — the original "the default loaded AND stepped" gate must
# keep meaning what it did, not be relaxed to accommodate the new reloads.
var stepped_count := -1
# Seeded to the FAILING polarity so a frame that never runs cannot pass by default.
var banner_default_visible := true
var banner_authored_visible := false
var banner_after_reload_visible := true

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
		# `build_from_file`, then step it once. The default (`crew_mission`) is kinetics-free,
		# so the banner must stay hidden — being file-loaded is not an uncalibrated claim.
		inst._on_load()
		inst._on_step()
		stepped_count = inst.sim.step_count() if inst.sim != null else -1
		banner_default_visible = inst._uncalibrated_label.visible
		return false
	if frames == 2:
		# Load a kinetics-bearing scenario through the same widget: the banner must appear.
		inst._path_edit.text = AUTHORED_SCENARIO
		inst._on_load()
		banner_authored_visible = inst._uncalibrated_label.visible
		return false
	if frames == 3:
		# Back to the kinetics-free default: the banner must clear, not latch.
		inst._path_edit.text = inst.DEFAULT_SCENARIO
		inst._on_load()
		banner_after_reload_visible = inst._uncalibrated_label.visible
		return false
	if frames < FRAMES_TO_RUN:
		return false
	# _build_ui adds a VBoxContainer (with rows) — a non-zero count proves it ran; a live sim
	# past step 0 after Load proves the file loaded + stepped through the UI.
	var loaded: bool = inst.sim != null and stepped_count > 0
	_emit({
		"ok": inst.get_child_count() > 0 and loaded,
		"child_count": inst.get_child_count(),
		"step_count": stepped_count,
		"banner_default_visible": banner_default_visible,
		"banner_authored_visible": banner_authored_visible,
		"banner_after_reload_visible": banner_after_reload_visible,
	})
	return true

func _emit(report: Dictionary) -> void:
	print("<<<UI_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("UI_SMOKE_END>>>")
