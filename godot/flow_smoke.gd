# Phase-8 (P8.4) flow-inspection smoke — a headless check that the flow-level display
# projection crosses the actual `godot_bridge` cdylib boundary intact (the Rust unit tests
# exercise `station::inspection` in-process; this proves `flow_inspection_json()` returns
# well-formed JSON THROUGH gdext that GDScript can parse). Not a parity gate (the projection
# is zero-parity, plain decimal floats) — a wiring check, run:
#
#   godot --headless --path godot --script res://flow_smoke.gd
#
# It builds the single-rate `station` (Power → Thermal) entry, steps a day, and reports the
# inspected flows + their legs + the "which flows touch thermal.node" join between markers,
# and confirms a two-rate entry (`greenhouse`) returns "" (inspection is single-rate only).
# The Python comparator (`tests/crossport/test_godot_flow_inspection.py`) captures stdout and
# asserts the known station flows are present with balanced-looking legs.

extends SceneTree

const SCENARIO := "station"
const STEPS := 24  # one balanced day (dt = 3600 s)
const NODE := "thermal.node"

func _initialize() -> void:
	var report := {
		"ok": false,
		"scenario": SCENARIO,
		"n": -1,
		"flow_ids": [],
		"node_contributors": [],  # [[flow_id, amount], ...]
		"two_rate_empty": false,
	}
	var sim := SimSession.new()
	if not sim.build(SCENARIO):
		_emit(report)
		quit(1)
		return
	sim.step_n(STEPS)

	var insp: Variant = JSON.parse_string(sim.flow_inspection_json())
	if typeof(insp) != TYPE_DICTIONARY:
		_emit(report)
		quit(2)
		return
	report["n"] = int(insp["n"])
	for flow in insp["flows"]:
		report["flow_ids"].append(flow["id"])
		for leg in flow["legs"]:
			if leg["stock"] == NODE:
				report["node_contributors"].append([flow["id"], float(leg["amount"])])

	# A two-rate entry defers inspection → empty string (not an error).
	var two := SimSession.new()
	if two.build("greenhouse"):
		report["two_rate_empty"] = two.flow_inspection_json() == ""

	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<FLOW_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("FLOW_SMOKE_END>>>")
