# Phase-8 (P8.3) interactive time-controls dashboard — play / pause / single-step /
# fast-forward with a speed control and a horizon scrubber, all driven off the render thread.
#
# The `TimeController` owns a Rust worker thread that does the stepping; this script only
# SENDS commands (play/pause/step/fast_forward_to/set_speed) and READS the latest published
# snapshot (`observation_json`) each frame — it never blocks on a step, so the UI stays
# responsive even while fast-forwarding a two-rate scenario (1440 sub-steps per master day).
#
# The widgets are built in code (the .tscn is just the root Control), so the scene stays
# small and there is one obvious place the wiring lives. On-screen pixels need a display
# server (interactive run / MCP screenshot); the load-bearing correctness proofs are the
# Rust `time_control` tests + the headless `time_smoke.gd` cross-boundary smoke.

extends Control

# Fixed palette (confirmed decision #1). "station" has real temperature + battery SOC;
# "greenhouse" and "sealed" are the two-rate ones (each step = a whole master day) that make
# the off-render-thread fast-forward tangible — "sealed" is the multi-year "fast-forward
# decades" scenario (re-sown each season) with the richest readouts.
const SCENARIOS := ["station", "greenhouse", "sealed", "cabin_gas"]

var tc: TimeController
var target_n := 0

var _scenario_picker: OptionButton
var _speed_slider: HSlider
var _speed_label: Label
var _horizon_slider: HSlider
var _horizon_label: Label
var _status_label: Label
var _dashboard_label: Label

func _ready() -> void:
	_build_ui()
	_rebuild(SCENARIOS[0])

func _rebuild(scenario: String) -> void:
	tc = TimeController.new()
	if not tc.build(scenario):
		_status_label.text = "build(%s) failed — see stderr" % scenario
		tc = null
		return
	target_n = int(_horizon_slider.value)
	_status_label.text = "built %s — fp_clean=%s (worker thread)" % [scenario, tc.fp_clean()]

func _process(_delta: float) -> void:
	if tc == null:
		return
	_dashboard_label.text = _render(tc.observation_json())
	var mode := "playing" if tc.is_playing() else ("fast-forwarding → %d" % tc.target_n() if tc.is_fast_forwarding() else "paused")
	var err := tc.error_message()
	_status_label.text = "%s   n=%d   mode=%s   fp_clean=%s%s" % [
		_scenario_picker.get_item_text(_scenario_picker.selected),
		tc.step_count(), mode, tc.fp_clean(),
		"" if err == "" else ("   ERROR: " + err),
	]

# ---- command handlers (wired to the buttons/sliders) -----------------------

func _on_scenario_selected(index: int) -> void:
	_rebuild(SCENARIOS[index])

func _on_play() -> void:
	if tc != null:
		tc.play()

func _on_pause() -> void:
	if tc != null:
		tc.pause()

func _on_step() -> void:
	if tc != null:
		tc.single_step()

func _on_fast_forward() -> void:
	if tc != null:
		tc.fast_forward_to(int(_horizon_slider.value))

func _on_speed_changed(value: float) -> void:
	_speed_label.text = "speed %d" % int(value)
	if tc != null:
		tc.set_speed(int(value))

func _on_horizon_changed(value: float) -> void:
	_horizon_label.text = "→ n=%d" % int(value)

# ---- rendering (formats the core-computed projection; computes nothing) -----

func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "(waiting for first snapshot…)"
	var shared := {}
	for id in proj["shared_stock_ids"]:
		shared[id] = true
	var lines := PackedStringArray()
	lines.append("rationed=%d  events=%d  residual=%s" % [
		int(proj["rationed"]), int(proj["events"]), _fmt(proj["max_residual"]),
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

func _fmt(v: Variant) -> String:
	return "n/a" if v == null else str(float(v))

# ---- UI construction (code, not .tscn — keeps the scene tiny) ---------------

func _build_ui() -> void:
	var root := VBoxContainer.new()
	root.set_anchors_preset(Control.PRESET_FULL_RECT)
	root.offset_left = 16
	root.offset_top = 16
	root.offset_right = -16
	root.offset_bottom = -16
	add_child(root)

	var controls := HBoxContainer.new()
	root.add_child(controls)
	_scenario_picker = OptionButton.new()
	for scenario_name in SCENARIOS:
		_scenario_picker.add_item(scenario_name)
	_scenario_picker.item_selected.connect(_on_scenario_selected)
	controls.add_child(_scenario_picker)
	controls.add_child(_button("Play", _on_play))
	controls.add_child(_button("Pause", _on_pause))
	controls.add_child(_button("Step", _on_step))
	controls.add_child(_button("Fast-forward →", _on_fast_forward))

	var speed_row := HBoxContainer.new()
	root.add_child(speed_row)
	_speed_label = Label.new()
	_speed_label.text = "speed 1"
	_speed_label.custom_minimum_size = Vector2(90, 0)
	speed_row.add_child(_speed_label)
	_speed_slider = HSlider.new()
	_speed_slider.min_value = 1
	_speed_slider.max_value = 500
	_speed_slider.value = 1
	_speed_slider.custom_minimum_size = Vector2(240, 0)
	_speed_slider.value_changed.connect(_on_speed_changed)
	speed_row.add_child(_speed_slider)

	var horizon_row := HBoxContainer.new()
	root.add_child(horizon_row)
	_horizon_label = Label.new()
	_horizon_label.text = "→ n=200"
	_horizon_label.custom_minimum_size = Vector2(90, 0)
	horizon_row.add_child(_horizon_label)
	_horizon_slider = HSlider.new()
	_horizon_slider.min_value = 0
	_horizon_slider.max_value = 2000
	_horizon_slider.value = 200
	_horizon_slider.custom_minimum_size = Vector2(240, 0)
	_horizon_slider.value_changed.connect(_on_horizon_changed)
	horizon_row.add_child(_horizon_slider)

	_status_label = Label.new()
	root.add_child(_status_label)
	_dashboard_label = Label.new()
	root.add_child(_dashboard_label)

func _button(text: String, handler: Callable) -> Button:
	var b := Button.new()
	b.text = text
	b.pressed.connect(handler)
	return b
