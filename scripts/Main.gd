extends Node2D

@onready var simulation: Node = $Simulation
@onready var world_renderer: Node2D = $WorldRenderer
@onready var interior_renderer: Node2D = $CellInteriorRenderer
@onready var camera: Camera2D = $Camera

func _ready() -> void:
	simulation.spawn_resources(30)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_TAB:
			simulation.toggle_interior_view()
			if simulation.interior_view:
				camera.enter_interior()
			else:
				camera.exit_interior()

func _process(delta: float) -> void:
	# Toggle visibility
	world_renderer.visible = not simulation.interior_view
	interior_renderer.visible = simulation.interior_view

	# Restart when dead
	if not simulation.player_alive:
		if Input.is_key_pressed(KEY_R):
			simulation.restart()
			camera.exit_interior()
		simulation.tick(delta)
		if simulation.interior_view:
			interior_renderer.queue_redraw()
		else:
			world_renderer.queue_redraw()
		return

	# Skip overworld movement in interior view
	if simulation.interior_view:
		simulation.tick(delta)
		interior_renderer.queue_redraw()
		return

	# Don't move player when shift is held (shift+arrows = camera pan)
	if Input.is_key_pressed(KEY_SHIFT):
		simulation.tick(delta)
		world_renderer.queue_redraw()
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
