extends Node2D

@onready var simulation: Node = $Simulation
@onready var world_renderer: Node2D = $WorldRenderer
@onready var interior_renderer: Node2D = $CellInteriorRenderer
@onready var camera: Camera2D = $Camera

var dragging := false

func _ready() -> void:
	simulation.spawn_resources(30)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo:
		if event.keycode == KEY_TAB:
			# Cancel active drag when leaving interior
			if simulation.interior_view and dragging:
				simulation.cancel_drag()
				dragging = false
			simulation.toggle_interior_view()
			if simulation.interior_view:
				camera.enter_interior()
			else:
				camera.exit_interior()

	# Drag-and-drop mouse input (interior view only)
	if not simulation.interior_view:
		return

	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		var local_pos := interior_renderer.to_local(get_global_mouse_position())
		if event.pressed:
			if simulation.try_pick_particle(local_pos.x, local_pos.y):
				dragging = true
		else:
			if dragging:
				simulation.drop_particle(local_pos.x, local_pos.y)
				dragging = false

	if event is InputEventMouseMotion and dragging:
		var local_pos := interior_renderer.to_local(get_global_mouse_position())
		simulation.drag_particle(local_pos.x, local_pos.y)

func _process(delta: float) -> void:
	# Toggle visibility
	world_renderer.visible = not simulation.interior_view
	interior_renderer.visible = simulation.interior_view

	# Restart when dead
	if not simulation.player_alive:
		if Input.is_key_pressed(KEY_R):
			simulation.restart()
			camera.exit_interior()
			dragging = false
		simulation.tick(delta)
		if simulation.interior_view:
			interior_renderer.queue_redraw()
		else:
			world_renderer.queue_redraw()
		return

	# Don't move player when shift is held (shift+arrows = camera pan)
	if not Input.is_key_pressed(KEY_SHIFT):
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

	# Hover tracking for interior view
	if simulation.interior_view:
		var local_pos := interior_renderer.to_local(get_global_mouse_position())
		interior_renderer.hovered_index = simulation.get_nearest_particle_index(local_pos.x, local_pos.y)

	simulation.tick(delta)

	if simulation.interior_view:
		interior_renderer.queue_redraw()
	else:
		world_renderer.queue_redraw()
