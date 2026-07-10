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
@onready var label: Label = $Label

func _ready() -> void:
	sim = SimSession.new()
	if not sim.build(SCENARIO):
		label.text = "build(%s) failed — see stderr" % SCENARIO
		sim = null

func _process(_delta: float) -> void:
	if sim == null:
		return
	sim.step()
	label.text = _render(sim.observation_json())

# Parse the Rust-side display projection and lay it out as a text dashboard. Every value
# here was computed in the core; this function only formats.
func _render(json_text: String) -> String:
	var proj: Variant = JSON.parse_string(json_text)
	if typeof(proj) != TYPE_DICTIONARY:
		return "observation_json() parse failed"

	var shared := {}
	for id in proj["shared_stock_ids"]:
		shared[id] = true

	var lines := PackedStringArray()
	lines.append("%s   n = %d" % [SCENARIO, int(proj["n"])])
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

func _fmt(v: Variant) -> String:
	# str() renders small residuals in scientific form (no %e in GDScript).
	return "n/a" if v == null else str(float(v))
