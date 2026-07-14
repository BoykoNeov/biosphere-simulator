# Phase-9 (Step 5) template-override FFI smoke — exercises `SimSession.build_from_file_with`
# (the typed parallel-array template-override entry) end-to-end through the ACTUAL cdylib, the
# one FFI surface no other smoke / cargo test / UI drives (the array unpack: PackedStringArray +
# PackedFloat64Array → length-check → BTreeMap). Runs HEADLESS:
#
#   godot --headless --path godot --script res://from_file_template_smoke.gd -- <abs/template.yaml>
#
# It also *demonstrates the Phase-9 capability*: loading a habitat TEMPLATE with a `crew_count`
# knob (not just a fixed scenario). It builds the template twice — at the default (crew_count =
# 1.0, via `build_from_file`) and at crew_count = 4.0 (via `build_from_file_with`) — and reports
# the initial `crew.food_store` each way; the Python gate asserts the knob bit exactly 4× through
# the array FFI. Read at n = 0 (before stepping) so the ratio is exact `1.0*base` vs `4.0*base`.

extends SceneTree

const STOCK := "crew.food_store"

func _initialize() -> void:
	var report := {
		"ok": false,
		"fp_clean": false,
		"food_default": 0.0,
		"food_4x": 0.0,
	}
	var user_args := OS.get_cmdline_user_args()
	if user_args.is_empty():
		push_error("template_smoke: expected an absolute template path as a user arg (after --)")
		_emit(report)
		quit(1)
		return
	var path: String = user_args[0]

	# @ default (crew_count = 1.0): the no-override entry.
	var base := SimSession.new()
	if not base.build_from_file(path):
		push_error("template_smoke: build_from_file(%s) failed" % path)
		_emit(report)
		quit(1)
		return
	report["fp_clean"] = base.fp_clean()
	report["food_default"] = base.stock_amount(STOCK)

	# @ crew_count = 4.0: the templated-override array FFI (the surface under test).
	var big := SimSession.new()
	if not big.build_from_file_with(path, PackedStringArray(["crew_count"]), PackedFloat64Array([4.0])):
		push_error("template_smoke: build_from_file_with(%s, crew_count=4.0) failed" % path)
		_emit(report)
		quit(1)
		return
	report["food_4x"] = big.stock_amount(STOCK)
	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
