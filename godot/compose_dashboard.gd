# Phase-8 (P8.6) interactive fixed-palette "build systems" dashboard — the player toggles a
# bounded set of parts (power plant / radiator / self-discharge leak) and presses Build to
# assemble a station, then steps it and watches the multi-domain readout.
#
# This is the interactive face of `SimSession.build_composed` (the bounded Rust builder). It
# uses the synchronous `SimSession` (composition is single-rate and cheap — no worker thread,
# unlike the two-rate time dashboard). The widgets are built in code (the .tscn is just the
# root Control) so the scene stays tiny.
#
# On-screen pixels need a display server (interactive run / MCP screenshot); the load-bearing
# proofs are the Rust builder_parity tests (the byte-identity anchors) + the headless
# `compose_smoke.gd` cross-boundary smoke. `compose_ui_smoke.gd` instantiates THIS scene
# headless so its `_ready` → `_build_ui` is actually parsed and run.

extends Control

# The fixed, code-defined ENERGY palette (confirmed decision #1: "build systems" = a bounded
# palette, not declarative authoring). Each id maps 1:1 to a `station::builder::Component`.
const PARTS := ["power_plant", "radiator", "self_discharge"]

# A couple of illustrative recipes surfaced as one-click presets (the player can still toggle
# freely). The radiator-less one heats unbounded — a legitimate, conservation-closed *failure*
# outcome the exit criterion wants ("observe failure/stability"), not a bug.
const PRESETS := {
	"Heat-closure station": ["power_plant", "radiator"],
	"No radiator (overheats)": ["power_plant"],
	"Leaky station": ["power_plant", "radiator", "self_discharge"],
}

var sim: SimSession
var _checks := {}                 # part id -> CheckBox
var _status_label: Label
var _dashboard_label: Label

func _ready() -> void:
	_build_ui()

func _selected_parts() -> PackedStringArray:
	var parts := PackedStringArray()
	for id in PARTS:
		if _checks[id].button_pressed:
			parts.append(id)
	return parts

func _on_build() -> void:
	var parts := _selected_parts()
	sim = SimSession.new()
	if not sim.build_composed(parts):
		_status_label.text = "build_composed(%s) rejected — see stderr" % str(parts)
		sim = null
		return
	_status_label.text = "built [%s] — fp_clean=%s" % [", ".join(parts), sim.fp_clean()]
	_refresh()

func _on_preset(recipe: Array) -> void:
	for id in PARTS:
		_checks[id].button_pressed = recipe.has(id)
	_on_build()

func _on_step() -> void:
	if sim != null and sim.step():
		_refresh()

func _on_step_day() -> void:
	if sim != null and sim.step_n(24):
		_refresh()

func _refresh() -> void:
	if sim != null:
		_dashboard_label.text = _render(sim.observation_json())

# ---- rendering (formats the core-computed projection; computes nothing) -----

func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "(no state — build a station)"
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

	root.add_child(_heading("Palette — pick parts, then Build"))
	var palette := HBoxContainer.new()
	root.add_child(palette)
	for id in PARTS:
		var cb := CheckBox.new()
		cb.text = id
		# Default to the heat-closure station (the byte-identity reproduce path).
		cb.button_pressed = id in ["power_plant", "radiator"]
		palette.add_child(cb)
		_checks[id] = cb

	var actions := HBoxContainer.new()
	root.add_child(actions)
	actions.add_child(_button("Build", _on_build))
	actions.add_child(_button("Step", _on_step))
	actions.add_child(_button("Step ×24 (day)", _on_step_day))

	root.add_child(_heading("Presets"))
	var presets := HBoxContainer.new()
	root.add_child(presets)
	for name in PRESETS:
		var recipe: Array = PRESETS[name]
		presets.add_child(_button(name, _on_preset.bind(recipe)))

	_status_label = Label.new()
	_status_label.text = "pick parts and press Build"
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
