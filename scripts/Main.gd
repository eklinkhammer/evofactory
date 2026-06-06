extends Node2D

@onready var simulation: Node = $Simulation
@onready var world_renderer: Node2D = $WorldRenderer

func _ready() -> void:
	simulation.spawn_resources(30)

func _process(delta: float) -> void:
	# Don't move player when shift is held (shift+arrows = camera pan)
	if Input.is_key_pressed(KEY_SHIFT):
		return

	var dx := 0.0
	var dy := 0.0

	if Input.is_action_pressed("ui_right"):
		dx += 1.0
	if Input.is_action_pressed("ui_left"):
		dx -= 1.0
	if Input.is_action_pressed("ui_down"):
		dy += 1.0
	if Input.is_action_pressed("ui_up"):
		dy -= 1.0

	# Normalize diagonal movement
	if dx != 0.0 and dy != 0.0:
		var inv_sqrt2 := 0.7071
		dx *= inv_sqrt2
		dy *= inv_sqrt2

	# Transform screen-space input to world-space for dimetric projection
	var wx := dx * 0.5 + dy
	var wy := -dx * 0.5 + dy

	simulation.move_player(wx, wy)
	simulation.tick(delta)

	world_renderer.queue_redraw()
