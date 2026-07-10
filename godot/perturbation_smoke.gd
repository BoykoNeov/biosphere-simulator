# Phase-8 (P8.5) cross-boundary PERTURBATION smoke — the "FFI didn't corrupt determinism"
# proof, now on a PERTURBED (single-rate `station`) run, and the load-bearing demonstration
# of both the failure cascade AND the P8.4 rationing seam through the actual cdylib. Runs
# HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://perturbation_smoke.gd
#
# It drives a DEEP brownout (the interactive "perturb systems" primitive `build_perturbed`)
# on the `station` scenario through the ACTUAL `godot_bridge` cdylib Godot loads: the blackout
# empties the battery so LoadDraw rations (the failure cascade emerges), and — because the raw
# per-flow legs no longer equal what moves under rationing — each inspected flow carries a
# `scale < 1` (the seam fix). Between unique markers on stdout it emits a JSON report with:
#   * the Rust-side hex-float snapshot (`snapshot_json`, the golden codec),
#   * the min per-flow `scale` seen across the run (< 1.0 ⇒ rationing was observed),
#   * total `rationed`, the FP flags, and the step count.
# The Python comparator (`tests/crossport/test_godot_perturbations.py`) asserts the snapshot is
# byte-identical to the headless `emit_perturbed_brownout` example (FFI-determinism), that
# rationing emerged (`rationed > 0`, `min_scale < 1`), and that FTZ/DAZ are OFF.

extends SceneTree

const SCENARIO := "station"
const KIND := "brownout"
const SPD := 24                     # station::scenario HEAT_CLOSURE_SCENARIO power.steps_per_day
const WINDOW_START := 2 * SPD       # 48  — blackout begins day 2
const WINDOW_END := 8 * SPD         # 192 — blackout ends day 8
const STEPS := 12 * SPD             # 288 — 12-day horizon (matches emit_perturbed_brownout)
const FACTOR := 0.0                 # full blackout

func _initialize() -> void:
	var report := {
		"ok": false,
		"scenario": SCENARIO,
		"kind": KIND,
		"fp_clean": false,
		"mxcsr": 0,
		"step_count": -1,
		"rationed": -1,
		"min_scale": 1.0,
		"snapshot": "",
	}
	var sim := SimSession.new()
	if not sim.build_perturbed(SCENARIO, KIND, WINDOW_START, WINDOW_END, FACTOR):
		_emit(report)
		quit(1)
		return
	# FP-env read on the same thread that will call step().
	report["mxcsr"] = sim.mxcsr()
	report["fp_clean"] = sim.fp_clean()

	# Step the horizon; at each pre-step state inspect the flows and track the min per-flow
	# scale (a display read — it does NOT mutate the trajectory, so the final snapshot stays
	# byte-identical to the pure-stepping headless reference). scale < 1 ⇒ that step rations.
	var min_scale := 1.0
	for i in STEPS:
		var insp: Variant = JSON.parse_string(sim.flow_inspection_json())
		if typeof(insp) == TYPE_DICTIONARY:
			for flow in insp["flows"]:
				var s := float(flow["scale"])
				if s < min_scale:
					min_scale = s
		if not sim.step():
			report["min_scale"] = min_scale
			_emit(report)
			quit(2)
			return
	report["min_scale"] = min_scale
	report["step_count"] = sim.step_count()
	report["rationed"] = sim.total_rationed()
	report["snapshot"] = sim.snapshot_json()
	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
