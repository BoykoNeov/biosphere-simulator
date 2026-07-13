# Phase-8 (P8.7) interactive SAVE/LOAD dashboard — the player builds a scenario, steps it,
# writes a save file, and later restores it, watching the multi-domain readout resume exactly.
#
# This is the interactive face of `SimSession.save()` / `SimSession.load()`. A save is
# `(recipe, State)` — the record the Rust bridge produces — written to `user://` with Godot's
# `FileAccess`. Loading rebuilds the registry from the recipe and restores the state, so stepping
# continues bit-identically (the `(seed, key, n)` determinism corollary). It uses the synchronous
# `SimSession` on the single-rate palette entries (cabin_gas / station — cheap, no worker thread;
# two-rate scenarios use the time dashboard). Widgets are built in code so the .tscn stays tiny.
#
# On-screen pixels need a display server (interactive run / MCP screenshot); the load-bearing
# proofs are the Rust `session_save_load.rs` resume-parity teeth + the headless `save_smoke.gd`
# cross-boundary disk round-trip. `save_ui_smoke.gd` instantiates THIS scene headless so its
# `_ready` → `_build_ui` and a Build → Save → Load cycle are actually parsed and run.

extends Control

# The single-rate palette entries save/load exercises here (two-rate greenhouse/sealed save fine
# too, but stepping them synchronously would block the UI — that is the time dashboard's job).
const SCENARIOS := ["cabin_gas", "station"]
const SAVE_PATH := "user://station_save.json"

var sim: SimSession
var _scenario := "cabin_gas"
var _status_label: Label
var _dashboard_label: Label

func _ready() -> void:
	_build_ui()

func _on_pick(scenario: String) -> void:
	_scenario = scenario
	_on_build()

func _on_build() -> void:
	sim = SimSession.new()
	if not sim.build(_scenario):
		_status_label.text = "build(%s) rejected — see stderr" % _scenario
		sim = null
		return
	_status_label.text = "built %s — fp_clean=%s" % [_scenario, sim.fp_clean()]
	_refresh()

func _on_step() -> void:
	if sim != null and sim.step():
		_refresh()

func _on_step_many() -> void:
	if sim != null and sim.step_n(100):
		_refresh()

func _on_save() -> void:
	if sim == null:
		_status_label.text = "build a scenario before saving"
		return
	var text: String = sim.save()
	if text == "":
		_status_label.text = "save unavailable for this session — see stderr"
		return
	var f := FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if f == null:
		_status_label.text = "could not open %s for writing" % SAVE_PATH
		return
	f.store_string(text)
	f.close()
	_status_label.text = "saved to %s at n=%d" % [SAVE_PATH, sim.step_count()]

func _on_load() -> void:
	if not FileAccess.file_exists(SAVE_PATH):
		_status_label.text = "no save file at %s" % SAVE_PATH
		return
	var f := FileAccess.open(SAVE_PATH, FileAccess.READ)
	if f == null:
		_status_label.text = "could not open %s for reading" % SAVE_PATH
		return
	var text := f.get_as_text()
	f.close()
	# A fresh session, restored from the save (rebuild-from-recipe + state).
	var restored := SimSession.new()
	if not restored.load(text):
		_status_label.text = "load rejected — see stderr"
		return
	sim = restored
	_status_label.text = "loaded from %s — resumed at n=%d" % [SAVE_PATH, sim.step_count()]
	_refresh()

func _refresh() -> void:
	if sim != null:
		_dashboard_label.text = _render(sim.observation_json())

# ---- rendering (formats the core-computed projection; computes nothing) -----

func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "(no state — build a scenario)"
	var shared := {}
	for id in proj["shared_stock_ids"]:
		shared[id] = true
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
			var mark := " *" if shared.has(stock["id"]) else "  "
			lines.append("%s %s = %s %s" % [mark, stock["id"], str(float(stock["amount"])), stock["unit"]])
	lines.append("(* = shared cross-domain stock)")
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

	root.add_child(_heading("Scenario"))
	var picker := HBoxContainer.new()
	root.add_child(picker)
	for scenario in SCENARIOS:
		picker.add_child(_button(scenario, _on_pick.bind(scenario)))

	var actions := HBoxContainer.new()
	root.add_child(actions)
	actions.add_child(_button("Build", _on_build))
	actions.add_child(_button("Step", _on_step))
	actions.add_child(_button("Step ×100", _on_step_many))

	root.add_child(_heading("Save / Load"))
	var save_load := HBoxContainer.new()
	root.add_child(save_load)
	save_load.add_child(_button("Save", _on_save))
	save_load.add_child(_button("Load", _on_load))

	_status_label = Label.new()
	_status_label.text = "pick a scenario and press Build"
	root.add_child(_status_label)
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
