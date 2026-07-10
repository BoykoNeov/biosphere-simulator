# Phase-8 (P8.2) multi-domain dashboard — the display projection made visible. The Godot
# game loop OWNS the loop (work item #1): one sim `step()` per rendered frame; the core
# computes EVERY number (grouping, totals, temperature, SOC, residual) in `station::display`
# and hands it over as one JSON blob via `observation_json()` — the UI only parses and lays
# it out. Full on-screen pixels need a display server (interactive run / MCP screenshot);
# the load-bearing correctness proof is the Rust `station::display` tests + the bridge unit
# tests, not this. Defaults to the `station` (Power → Thermal) palette entry — the one with a
# real node temperature and battery SOC to show.

extends Control

const SCENARIO := "station"

var sim: SimSession
# The declared cross-domain stock ids (from the display projection) — reused by the flow
# panel's "select a stock → contributing flows" join.
var _shared_ids: Array = []
# The active perturbation label (P8.5) — "" when running the baseline scenario. A key press
# rebuilds the session with a windowed perturbation (build-time config; a live "trigger now"
# is deferred — the window opens from n=0 on rebuild).
var _active_perturbation := ""
@onready var label: Label = $Label

func _ready() -> void:
	sim = SimSession.new()
	if not sim.build(SCENARIO):
		label.text = "build(%s) failed — see stderr" % SCENARIO
		sim = null

# P8.5 interactive "perturb systems": a key press rebuilds the `station` scenario with a
# windowed perturbation, so the player watches the cross-domain cascade unfold from n=0.
#   1 → graceful brownout   2 → deep brownout (rationing)   3 → radiator failure   0 → baseline
# The composers compute the cascade (station::perturbations); the UI only picks and displays.
func _input(event: InputEvent) -> void:
	if sim == null or not (event is InputEventKey and event.pressed):
		return
	match event.keycode:
		KEY_1:
			_apply_perturbation("brownout", 0.5, "brownout (graceful)")
		KEY_2:
			_apply_perturbation("brownout", 0.0, "brownout (deep → rationing)")
		KEY_3:
			_apply_perturbation("radiator_failure", 0.0, "radiator failure (overheat)")
		KEY_0:
			_active_perturbation = ""
			sim.build(SCENARIO)  # back to the baseline

# Rebuild `sim` with a windowed perturbation over an illustrative [day 2, day 8) window
# (24 steps/day). Sets the active-perturbation label and returns whether the rebuild
# succeeded. `magnitude` is the one knob (brownout factor / radiator health).
func _apply_perturbation(kind: String, magnitude: float, label_text: String) -> bool:
	if sim.build_perturbed(SCENARIO, kind, 2 * 24, 8 * 24, magnitude):
		_active_perturbation = label_text
		return true
	_active_perturbation = "%s — build_perturbed failed (see stderr)" % label_text
	return false

func _process(_delta: float) -> void:
	if sim == null:
		return
	sim.step()
	label.text = _render(sim.observation_json()) + "\n\n" + _render_flows(sim.flow_inspection_json())

# Parse the Rust-side display projection and lay it out as a text dashboard. Every value
# here was computed in the core; this function only formats.
func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "observation_json() parse failed"

	_shared_ids = proj["shared_stock_ids"]
	var shared := {}
	for id in _shared_ids:
		shared[id] = true

	var lines := PackedStringArray()
	var header := "%s   n = %d" % [SCENARIO, int(proj["n"])]
	if _active_perturbation != "":
		header += "   [perturbation: %s]" % _active_perturbation
	lines.append(header)
	lines.append("(keys: 1 brownout · 2 deep brownout · 3 radiator failure · 0 baseline)")
	lines.append("rationed = %d   events = %d   residual = %s" % [
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
			# GDScript's `%` formatter supports %d/%f/%s/%x but NOT %g/%e — use str()
			# for the wide-dynamic-range amounts (tiny mols to ~1e9 J).
			lines.append("%s %s = %s %s" % [mark, stock["id"], str(float(stock["amount"])), stock["unit"]])
	lines.append("(* = shared cross-domain stock)")
	return "\n".join(lines)

# Parse the Rust-side flow inspection (P8.4) and lay it out as a "where matter/energy
# moves" panel: each flow with its legs, plus the "select a stock → contributing flows"
# join for the highlighted shared stocks. Every leg amount was computed in the core; this
# only formats. Empty JSON ⇒ a two-rate scenario (inspection is single-rate only) — say so.
func _render_flows(json_text: String) -> String:
	if json_text == "":
		return "[flows] inspection unavailable (two-rate scenario — deferred)"
	var insp: Variant = JSON.parse_string(json_text)
	if typeof(insp) != TYPE_DICTIONARY:
		return "flow_inspection_json() parse failed"

	var flows: Array = insp["flows"]
	var lines := PackedStringArray()
	lines.append("[flows]  (where matter/energy moves this step; legs are REQUESTED)")
	for flow in flows:
		# P8.5: `scale` < 1 ⇒ the Euler backstop rationed this flow (a deep brownout empties
		# the battery so LoadDraw cannot be met). Legs are the request; delivered = leg·scale.
		var scale := float(flow["scale"])
		var scale_note := "   ⚠ rationed → %.3f delivered" % scale if scale < 1.0 else ""
		lines.append("  %s%s" % [flow["id"], scale_note])
		for leg in flow["legs"]:
			# +deposit / -withdraw, wide dynamic range → str() (no %g/%e in GDScript).
			var amount := float(leg["amount"])
			var sign := "+" if amount >= 0.0 else ""
			var delivered := "  (delivered %s)" % str(amount * scale) if scale < 1.0 else ""
			lines.append("      %s%s  %s%s" % [sign, str(amount), leg["stock"], delivered])

	# The "select a stock" view for the shared cross-domain stocks: which flows touch each.
	lines.append("")
	lines.append("[contributing flows] (select a stock → its flows)")
	for stock_id in _shared_ids:
		var contributors := PackedStringArray()
		for flow in flows:
			for leg in flow["legs"]:
				if leg["stock"] == stock_id:
					var amount := float(leg["amount"])
					var sign := "+" if amount >= 0.0 else ""
					contributors.append("%s(%s%s)" % [flow["id"], sign, str(amount)])
		lines.append("  %s ← %s" % [stock_id, ", ".join(contributors) if contributors.size() > 0 else "(none)"])
	return "\n".join(lines)

func _fmt(v: Variant) -> String:
	# str() renders small residuals in scientific form (no %e in GDScript).
	return "n/a" if v == null else str(float(v))
