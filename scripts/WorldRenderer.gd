extends Node2D

@onready var simulation: Node = get_node("../Simulation")

func world_to_screen(wx: float, wy: float) -> Vector2:
	return Vector2(wx - wy, (wx + wy) * 0.5)

func _draw() -> void:
	# World boundary diamond (square world projected to dimetric)
	var bound: float = simulation.world_bound
	var corners: Array[Vector2] = [
		world_to_screen(bound, bound),    # top
		world_to_screen(bound, -bound),   # right
		world_to_screen(-bound, -bound),  # bottom
		world_to_screen(-bound, bound),   # left
	]

	# Ground fill
	var ground_color := Color(0.12, 0.18, 0.12)
	var packed_corners := PackedVector2Array(corners)
	draw_colored_polygon(packed_corners, ground_color)

	# Grid lines
	var grid_color := Color(0.18, 0.24, 0.18)
	var step := 100.0
	var val := -bound
	while val <= bound:
		draw_line(
			world_to_screen(val, -bound),
			world_to_screen(val, bound),
			grid_color, 1.0
		)
		draw_line(
			world_to_screen(-bound, val),
			world_to_screen(bound, val),
			grid_color, 1.0
		)
		val += step

	# Boundary outline
	var border_color := Color(0.4, 0.55, 0.4)
	for i in range(4):
		draw_line(corners[i], corners[(i + 1) % 4], border_color, 2.0)

	# Resources
	if simulation:
		var rxs: PackedFloat32Array = simulation.resource_xs
		var rys: PackedFloat32Array = simulation.resource_ys
		var rtypes: PackedInt32Array = simulation.resource_types
		var glucose_color := Color(0.95, 0.75, 0.2)
		var amino_color := Color(0.5, 0.3, 0.85)
		var r_radius: float = simulation.resource_radius
		for i in range(rxs.size()):
			var pos := world_to_screen(rxs[i], rys[i])
			if rtypes[i] == 0:
				draw_circle(pos, r_radius, glucose_color)
			else:
				draw_circle(pos, r_radius, amino_color)

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

		# Motor indicator on membrane
		var motor_angle: float = atan2(simulation.motor_interior_y, simulation.motor_interior_x)
		var motor_screen_pos := player_pos + Vector2(cos(motor_angle), sin(motor_angle)) * radius
		draw_circle(motor_screen_pos, 4.0, Color(1.0, 0.6, 0.2))
