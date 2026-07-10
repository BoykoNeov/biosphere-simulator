# Phase-8 (P8.1) minimal vertical slice — the "one live value renders in a Label"
# acceptance clause. The Godot game loop OWNS the loop (work item #1): one sim `step()`
# per rendered frame; the core computes every number, the UI only displays. Full pixel
# rendering needs a display server (interactive run / MCP screenshot); the load-bearing
# determinism proof is the headless `smoke.gd`, not this.

extends Control

var sim: SimSession
@onready var label: Label = $Label

func _ready() -> void:
	sim = SimSession.new()
	if not sim.build("cabin_gas"):
		label.text = "build(cabin_gas) failed — see stderr"
		sim = null

func _process(_delta: float) -> void:
	if sim == null:
		return
	sim.step()
	label.text = "cabin_gas   n = %d\neclss.cabin_o2 = %.6f mol\nfp_clean = %s" % [
		sim.step_count(),
		sim.stock_amount("eclss.cabin_o2"),
		str(sim.fp_clean()),
	]
