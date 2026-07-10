# Phase-8 (P8.2) dashboard smoke — a headless check that the display projection crosses the
# actual `godot_bridge` cdylib boundary intact (the Rust unit tests exercise `station::display`
# in-process; this proves `observation_json()` returns well-formed JSON THROUGH gdext that
# GDScript can parse and lay out). Not a parity gate (the projection is zero-parity, plain
# decimal floats) — a wiring check, run:
#
#   godot --headless --path godot --script res://dashboard_smoke.gd
#
# It builds the `station` (Power → Thermal) entry, steps a day, and prints the parsed
# dashboard between markers so a human / CI eye can confirm the grouped domains + derived
# readouts (node temperature, battery SOC, residual, shared-stock highlight) render.

extends SceneTree

const SCENARIO := "station"
const STEPS := 24  # one balanced day (dt = 3600 s)

func _initialize() -> void:
	print("<<<DASHBOARD_SMOKE_BEGIN")
	var sim := SimSession.new()
	if not sim.build(SCENARIO):
		print("build(%s) failed" % SCENARIO)
		print("DASHBOARD_SMOKE_END>>>")
		quit(1)
		return
	sim.step_n(STEPS)
	var proj: Variant = JSON.parse_string(sim.observation_json())
	if typeof(proj) != TYPE_DICTIONARY:
		print("observation_json() did not parse to a Dictionary")
		print("DASHBOARD_SMOKE_END>>>")
		quit(2)
		return
	# Echo the raw JSON (proves the GString survived the boundary) + a couple of parsed keys.
	print("json = %s" % sim.observation_json())
	print("n = %d, temperature_k = %s, soc = %s, domains = %s" % [
		int(proj["n"]),
		str(proj["temperature_k"]),
		str(proj["soc_percent_of_initial"]),
		str(proj["domains"].keys()),
	])
	print("DASHBOARD_SMOKE_END>>>")
	quit(0)
