# Phase-9 (Step 5) interactive "load a scenario file" dashboard — the interactive face of
# `SimSession.build_from_file` (the "author, not program" payoff). The player types (or a preset
# fills) an authored `.yaml` path, presses Load, then steps it and watches the multi-domain
# readout. Registry construction stays in Rust (the frozen `authoring` boundary interprets the
# file); this GDScript computes nothing — it formats the core-computed observation projection.
#
# Uses the synchronous `SimSession` (authored scenarios are single-rate — the `authoring::run`
# harness's scope — so no worker thread, unlike the two-rate time dashboard). Widgets are built
# in code (the .tscn is just the root Control) so the scene stays tiny.
#
# On-screen pixels need a display server (interactive run / MCP screenshot); the load-bearing
# proofs are the Rust faithfulness cargo test + the headless `from_file_smoke.gd` cross-boundary
# smoke. `from_file_ui_smoke.gd` instantiates THIS scene headless so its `_ready` → `_build_ui`
# is actually parsed and run.

extends Control

var sim: SimSession
var _path_edit: LineEdit
var _status_label: Label
var _uncalibrated_label: Label
var _dashboard_label: Label

# The "authored ≠ validated" banner (`docs/authoring-reference.md`, decision B). An authored
# `kinetics` rate law gets conservation + determinism from the platform and NO calibration claim,
# so a run that used one must never read as reference science. Shown only when the core says so —
# `has_authored_kinetics()` is a session constant, read once at Load like `fp_clean()`.
const UNCALIBRATED_TEXT := "⚠ UNCALIBRATED — this run uses authored kinetics. Conservation and determinism are guaranteed; the science is not. Not reference."
const UNCALIBRATED_COLOR := Color(1.0, 0.65, 0.0)

# A default scenario path relative to the Godot project dir (../tests/...). Globalized to an OS
# path at Load time — `build_from_file` reads a filesystem path, not a `res://` URI.
const DEFAULT_SCENARIO := "res://../tests/authoring/scenarios/crew_mission.yaml"

func _ready() -> void:
	_build_ui()

func _os_path() -> String:
	var text := _path_edit.text.strip_edges()
	# A `res://` URI is globalized to a real OS path; anything else is taken as-is (an already
	# absolute path a modder pasted in).
	if text.begins_with("res://") or text.begins_with("user://"):
		return ProjectSettings.globalize_path(text)
	return text

func _on_load() -> void:
	var path := _os_path()
	sim = SimSession.new()
	if not sim.build_from_file(path):
		_status_label.text = "build_from_file(%s) rejected — see stderr" % path
		_uncalibrated_label.visible = false
		sim = null
		return
	_status_label.text = "loaded %s — %d steps, fp_clean=%s" % [
		path, sim.total_steps(), sim.fp_clean(),
	]
	_uncalibrated_label.visible = sim.has_authored_kinetics()
	_refresh()

func _on_step() -> void:
	if sim != null and sim.step():
		_refresh()

func _on_run_to_end() -> void:
	# Fast-forward the file-declared horizon (authored runs are single-rate ⇒ cheap).
	if sim != null and sim.total_steps() > 0 and sim.step_n(sim.total_steps() - sim.step_count()):
		_refresh()

func _refresh() -> void:
	if sim != null:
		_dashboard_label.text = _render(sim.observation_json())

# ---- rendering (formats the core-computed projection; computes nothing) -----

func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "(no state — load a scenario)"
	var lines := PackedStringArray()
	lines.append("n=%d  rationed=%d  events=%d" % [
		int(proj["n"]), int(proj["rationed"]), int(proj["events"]),
	])
	if proj["temperature_k"] != null:
		lines.append("node T = %.3f K" % float(proj["temperature_k"]))
	if proj["soc_percent_of_initial"] != null:
		lines.append("battery SOC = %.1f %% of initial" % float(proj["soc_percent_of_initial"]))
	lines.append("")
	var domains: Dictionary = proj["domains"]
	for domain in domains:
		lines.append("[%s]" % domain)
		for stock in domains[domain]:
			lines.append("   %s = %s %s" % [stock["id"], str(float(stock["amount"])), stock["unit"]])
	return "\n".join(lines)

# ---- UI construction (code, not .tscn — keeps the scene tiny) ---------------

func _build_ui() -> void:
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.offset_left = 16
	root.offset_top = 16
	root.offset_right = -16
	root.offset_bottom = -16
	add_child(root)

	root.add_child(_heading("Scenario file (an authored .yaml) — type a path, then Load"))
	_path_edit = LineEdit.new()
	_path_edit.text = DEFAULT_SCENARIO
	_path_edit.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	root.add_child(_path_edit)

	var actions := HBoxContainer.new()
	root.add_child(actions)
	actions.add_child(_button("Load", _on_load))
	actions.add_child(_button("Step", _on_step))
	actions.add_child(_button("Run to end", _on_run_to_end))

	_status_label = Label.new()
	_status_label.text = "type a scenario path and press Load"
	root.add_child(_status_label)

	_uncalibrated_label = Label.new()
	_uncalibrated_label.text = UNCALIBRATED_TEXT
	_uncalibrated_label.add_theme_color_override("font_color", UNCALIBRATED_COLOR)
	_uncalibrated_label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_uncalibrated_label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	# Hidden until a load says otherwise — a banner shown by default would be noise, and a
	# banner that is *always* on stops being read.
	_uncalibrated_label.visible = false
	root.add_child(_uncalibrated_label)

	_dashboard_label = Label.new()
	root.add_child(_dashboard_label)

func _heading(text: String) -> Label:
	var l := Label.new()
	l.text = text
	return l

func _button(text: String, handler: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.pressed.connect(handler)
	return b
