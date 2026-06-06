extends Node2D

@onready var simulation: Node = get_node("../Simulation")

func world_to_screen(wx: float, wy: float) -> Vector2:
	return Vector2(wx - wy, (wx + wy) * 0.5)

func _draw() -> void:
	# World boundary diamond (square world projected to dimetric)
	var bound := 500.0
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

	# Player cell
	if simulation:
		var player_pos := world_to_screen(simulation.player_x, simulation.player_y)
		var radius: float = simulation.player_radius
		var cell_color := Color(0.3, 0.8, 0.4)
		var outline_color := Color(0.2, 0.6, 0.3)
		draw_circle(player_pos, radius, cell_color)
		draw_arc(player_pos, radius, 0, TAU, 32, outline_color, 2.0)
