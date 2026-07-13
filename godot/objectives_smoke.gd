# Phase-8 (P8.8) cross-boundary OBJECTIVES smoke — the "observe failure or stability" arm of
# the exit criterion, carried through the ACTUAL cdylib. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://objectives_smoke.gd
#
# The objectives layer (`station::objectives`) turns the session diagnostics into a player's
# win/fail condition. This smoke drives TWO runs through the real bridge and reads
# `objectives_json(target)` off each:
#   * STABILITY — a healthy `station` reaching its 7-day horizon → `survived == true`.
#   * FAILURE   — a `station` under a deep multi-day brownout (blackout, factor 0.0) that empties
#     the battery and rations `LoadDraw` → `no_rationing == false` ⇒ `survived == false`.
# So the SAME objective distinguishes stability from failure — no game-side domain logic; the
# perturbation cascade emerges from the frozen engine.
#
# The Python comparator (`tests/crossport/test_godot_objectives.py`) parses the two reports and
# asserts the stability run survived and the failure run did not (with rationing as the cause).

extends SceneTree

const SPD := 24                          # HEAT_CLOSURE_SCENARIO power.steps_per_day
const HORIZON := 7 * SPD                  # 168 steps — the healthy horizon
const FAIL_HORIZON := 12 * SPD            # 288 steps — long enough to feel the blackout

func _initialize() -> void:
	var report := {"ok": false, "fp_clean": false, "mxcsr": 0, "stable": {}, "failure": {}}

	# --- STABILITY: a healthy station reaches its horizon and survives. ---
	var stable := SimSession.new()
	if not stable.build("station"):
		_emit(report)
		quit(1)
		return
	report["mxcsr"] = stable.mxcsr()
	report["fp_clean"] = stable.fp_clean()
	if not stable.step_n(HORIZON):
		_emit(report)
		quit(2)
		return
	report["stable"] = JSON.parse_string(stable.objectives_json(HORIZON))

	# --- FAILURE: a deep brownout empties the battery and rations. ---
	var failing := SimSession.new()
	# Blackout on days [2, 10): factor 0.0.
	if not failing.build_perturbed("station", "brownout", 2 * SPD, 10 * SPD, 0.0):
		_emit(report)
		quit(3)
		return
	if not failing.step_n(FAIL_HORIZON):
		_emit(report)
		quit(4)
		return
	report["failure"] = JSON.parse_string(failing.objectives_json(FAIL_HORIZON))

	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
