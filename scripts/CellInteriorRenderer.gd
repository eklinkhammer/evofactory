extends Node2D

@onready var simulation: Node = get_node("../Simulation")

func _draw() -> void:
	if not simulation or not simulation.interior_view:
		return

	var radius: float = simulation.interior_radius

	# Membrane fill
	draw_circle(Vector2.ZERO, radius, Color(0.08, 0.15, 0.08))
	# Membrane outline
	draw_arc(Vector2.ZERO, radius, 0, TAU, 64, Color(0.3, 0.8, 0.3), 3.0)

	# Interior particles
	var xs: PackedFloat32Array = simulation.interior_xs
	var ys: PackedFloat32Array = simulation.interior_ys
	var types: PackedInt32Array = simulation.interior_types
	var glucose_color := Color(0.95, 0.75, 0.2)
	var amino_color := Color(0.5, 0.3, 0.85)

	for i in range(xs.size()):
		var pos := Vector2(xs[i], ys[i])
		var col: Color = glucose_color if types[i] == 0 else amino_color
		draw_circle(pos, 5.0, col)
