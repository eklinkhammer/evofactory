extends Node2D

@onready var simulation: Node = get_node("../Simulation")
@onready var camera: Camera2D = get_node("../Camera")

func world_to_screen(wx: float, wy: float) -> Vector2:
	return Vector2(wx - wy, (wx + wy) * 0.5)

func _draw() -> void:
	# Compute visible area from camera
	var vp_size := get_viewport_rect().size
	var cam_pos := camera.position
	var cam_zoom := camera.zoom
	# Half-size in screen coords
	var half_w: float = (vp_size.x / cam_zoom.x) * 0.5
	var half_h: float = (vp_size.y / cam_zoom.y) * 0.5

	# Ground fill — large rect covering visible area
	var ground_color := Color(0.12, 0.18, 0.12)
	var ground_rect := Rect2(
		cam_pos.x - half_w - 100.0,
		cam_pos.y - half_h - 100.0,
		half_w * 2.0 + 200.0,
		half_h * 2.0 + 200.0
	)
	draw_rect(ground_rect, ground_color, true)

	# Grid lines — compute world-space range visible on screen
	# Inverse of dimetric: wx = sy + sx/2, wy = sy - sx/2
	# We need the world-coord bounding box that maps into the visible screen rect
	var screen_left: float = cam_pos.x - half_w
	var screen_right: float = cam_pos.x + half_w
	var screen_top: float = cam_pos.y - half_h
	var screen_bottom: float = cam_pos.y + half_h

	# Map screen corners back to world coords to find world range
	# screen_to_world: wx = sy + sx/2, wy = sy - sx/2
	var corners_wx: Array[float] = []
	var corners_wy: Array[float] = []
	for sx in [screen_left, screen_right]:
		for sy in [screen_top, screen_bottom]:
			corners_wx.append(sy + sx * 0.5)
			corners_wy.append(sy - sx * 0.5)

	var world_min_x: float = corners_wx.min()
	var world_max_x: float = corners_wx.max()
	var world_min_y: float = corners_wy.min()
	var world_max_y: float = corners_wy.max()

	var grid_color := Color(0.18, 0.24, 0.18)
	var step := 100.0

	# Snap to grid
	var start_x: float = floor(world_min_x / step) * step
	var start_y: float = floor(world_min_y / step) * step

	var val := start_x
	while val <= world_max_x:
		var p0 := world_to_screen(val, world_min_y)
		var p1 := world_to_screen(val, world_max_y)
		draw_line(p0, p1, grid_color, 1.0)
		val += step

	val = start_y
	while val <= world_max_y:
		var p0 := world_to_screen(world_min_x, val)
		var p1 := world_to_screen(world_max_x, val)
		draw_line(p0, p1, grid_color, 1.0)
		val += step

	# Resources
	if simulation:
		var rxs: PackedFloat32Array = simulation.resource_xs
		var rys: PackedFloat32Array = simulation.resource_ys
		var rtypes: PackedInt32Array = simulation.resource_types
		var glucose_color := Color(0.95, 0.75, 0.2)
		var amino_color := Color(0.5, 0.3, 0.85)
		var nucleotide_color := Color(0.9, 0.3, 0.4)
		var r_radius: float = simulation.resource_radius
		for i in range(rxs.size()):
			var pos := world_to_screen(rxs[i], rys[i])
			match rtypes[i]:
				0: draw_circle(pos, r_radius, glucose_color)
				1: draw_circle(pos, r_radius, amino_color)
				3: draw_circle(pos, r_radius, nucleotide_color)
				_: draw_circle(pos, r_radius, glucose_color)

	# Player cell (exterior LOD only — interior renderer handles zoomed-in view)
	if simulation and not simulation.interior_view:
		var player_pos := world_to_screen(simulation.player_x, simulation.player_y)
		var radius: float = simulation.player_radius
		var energy: float = clampf(simulation.player_energy_ratio, 0.0, 1.0)
		var cell_color := Color(0.8 * (1.0 - energy), 0.8 * energy, 0.2)
		var outline_color := Color(0.6 * (1.0 - energy), 0.6 * energy, 0.15)
		if energy < 0.2:
			cell_color.a = 0.4
			outline_color.a = 0.4
		draw_circle(player_pos, radius, cell_color)
		draw_arc(player_pos, radius, 0, TAU, 32, outline_color, 2.0)

		# Motor indicators on membrane (multiple)
		var m_xs: PackedFloat32Array = simulation.motor_xs
		var m_ys: PackedFloat32Array = simulation.motor_ys
		for mi in range(m_xs.size()):
			var motor_angle: float = atan2(m_ys[mi], m_xs[mi])
			var motor_screen_pos := player_pos + Vector2(cos(motor_angle), sin(motor_angle)) * radius
			draw_circle(motor_screen_pos, 4.0, Color(1.0, 0.6, 0.2))

		# Sensor range visualization (when autonomous with sensors)
		if simulation.autonomous and simulation.auto_sensor_count > 0:
			var sensor_range: float = simulation.auto_sensor_range
			draw_arc(player_pos, sensor_range + radius, 0, TAU, 64, Color(0.3, 0.7, 1.0, 0.2), 1.5)
