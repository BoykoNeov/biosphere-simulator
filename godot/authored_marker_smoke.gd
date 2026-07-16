# "Authored ≠ validated" marker smoke — the Phase-9 follow-up surfacing, driven through the
# ACTUAL `godot_bridge` cdylib Godot loads. Runs HEADLESS, no editor, no MCP:
#
#   godot --headless --path godot --script res://authored_marker_smoke.gd -- <authored.yaml> <plain.yaml>
#
# `SimSession.has_authored_kinetics()` tells a consumer that a run used an authored `kinetics`
# rate law, so the platform vouches for its conservation + determinism and NOTHING about its
# science (`docs/authoring-reference.md`, decision B). The cargo tests already cover the loader
# returning the marker; what only Godot can exercise is the **live `SimSession` object**, whose
# build paths all replace `inner` in place.
#
# The load-bearing assertion is therefore `after_palette_rebuild`: a session built from an
# authored file and THEN rebuilt into a frozen palette scenario must report `false`. A stale
# `true` there would be a UI crying wolf on reference science; a stale marker the other way
# (a missed set) would silently present an uncalibrated run as reference — the failure that
# actually matters. `plain_file_marker` pins the other edge: file-loaded but kinetics-free is
# `false` (the marker is about authored rate laws, not about being authored).

extends SceneTree

func _initialize() -> void:
	var report := {
		"ok": false,
		"authored_marker": null,
		"after_palette_rebuild": null,
		"plain_file_marker": null,
	}
	var user_args := OS.get_cmdline_user_args()
	if user_args.size() < 2:
		push_error("authored_marker_smoke: expected <authored.yaml> <plain.yaml> as user args")
		_emit(report)
		quit(1)
		return
	var authored_path: String = user_args[0]
	var plain_path: String = user_args[1]

	# One session, reused across build paths — exactly how the field could go stale.
	var sim := SimSession.new()
	if not sim.build_from_file(authored_path):
		push_error("authored_marker_smoke: build_from_file(%s) failed" % authored_path)
		_emit(report)
		quit(1)
		return
	report["authored_marker"] = sim.has_authored_kinetics()

	# Rebuild the SAME object into a frozen palette scenario: the marker must clear.
	if not sim.build("station"):
		push_error("authored_marker_smoke: build(\"station\") failed")
		_emit(report)
		quit(1)
		return
	report["after_palette_rebuild"] = sim.has_authored_kinetics()

	# A file-loaded scenario with no kinetics flow is NOT marked.
	var plain := SimSession.new()
	if not plain.build_from_file(plain_path):
		push_error("authored_marker_smoke: build_from_file(%s) failed" % plain_path)
		_emit(report)
		quit(1)
		return
	report["plain_file_marker"] = plain.has_authored_kinetics()

	report["ok"] = true
	_emit(report)
	quit(0)

func _emit(report: Dictionary) -> void:
	print("<<<GODOT_SMOKE_BEGIN")
	print(JSON.stringify(report))
	print("GODOT_SMOKE_END>>>")
