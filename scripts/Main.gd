extends Node2D

@onready var simulation: Node = $Simulation
@onready var world_renderer: Node2D = $WorldRenderer
@onready var interior_renderer: Node2D = $CellInteriorRenderer
@onready var camera: Camera2D = $Camera

var dragging := false
var press_pos := Vector2.ZERO

func _ready() -> void:
	simulation.spawn_resources(60)

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
				interior_renderer.scale = Vector2.ONE
				camera.exit_interior()

	# Drag-and-drop mouse input (interior view only)
	if not simulation.interior_view:
		return

	if event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT:
		var local_pos := interior_renderer.to_local(get_global_mouse_position())
		if event.pressed:
			press_pos = event.position
			if simulation.try_pick_particle(local_pos.x, local_pos.y):
				dragging = true
		else:
			var is_click: bool = event.position.distance_to(press_pos) < 5.0
			if dragging and not is_click:
				simulation.drop_particle(local_pos.x, local_pos.y)
				dragging = false
			else:
				if dragging:
					simulation.cancel_drag()
					dragging = false
				_check_tooltip_click(local_pos)

	if event is InputEventMouseMotion and dragging:
		var local_pos := interior_renderer.to_local(get_global_mouse_position())
		simulation.drag_particle(local_pos.x, local_pos.y)

func _process(delta: float) -> void:
	# World always visible; interior only when in interior view
	world_renderer.visible = true
	interior_renderer.visible = simulation.interior_view

	# Position and scale interior overlay to match cell in world space
	if simulation.interior_view:
		var player_screen: Vector2 = world_renderer.world_to_screen(simulation.player_x, simulation.player_y)
		interior_renderer.position = player_screen
		camera.position = player_screen
		# Scale interior to fit inside the cell's world-space circle
		var s: float = simulation.player_radius / simulation.interior_radius
		interior_renderer.scale = Vector2(s, s)

	# Restart when dead
	if not simulation.player_alive:
		if Input.is_key_pressed(KEY_R):
			simulation.restart()
			camera.exit_interior()
			dragging = false
		simulation.tick(delta)
		world_renderer.queue_redraw()
		if simulation.interior_view:
			interior_renderer.queue_redraw()
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

	# Always redraw world; redraw interior when active
	world_renderer.queue_redraw()
	if simulation.interior_view:
		interior_renderer.queue_redraw()

func _check_tooltip_click(local_pos: Vector2) -> void:
	var hit := -1
	var z_xs: PackedFloat32Array = simulation.zymase_xs
	var z_ys: PackedFloat32Array = simulation.zymase_ys
	for zi in range(z_xs.size()):
		if local_pos.distance_to(Vector2(z_xs[zi], z_ys[zi])) < 20.0:
			hit = 0
			break
	if hit < 0:
		var mrna_xs: PackedFloat32Array = simulation.mrna_xs
		var mrna_ys: PackedFloat32Array = simulation.mrna_ys
		for i in range(mrna_xs.size()):
			if local_pos.distance_to(Vector2(mrna_xs[i], mrna_ys[i])) < 20.0:
				hit = i + 1
				break
	if hit >= 0 and interior_renderer.tooltip_target == hit:
		interior_renderer.tooltip_target = -1
	elif hit >= 0:
		interior_renderer.tooltip_target = hit
	else:
		interior_renderer.tooltip_target = -1
