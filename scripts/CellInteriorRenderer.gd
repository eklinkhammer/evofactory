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

	# Enzyme at center
	var enzyme_pos := Vector2(simulation.enzyme_interior_x, simulation.enzyme_interior_y)
	var enzyme_r: float = simulation.enzyme_interior_radius
	draw_circle(enzyme_pos, enzyme_r, Color(0.2, 0.5, 0.2))
	draw_arc(enzyme_pos, enzyme_r, 0, TAU, 6, Color(0.4, 0.9, 0.4), 2.0)

	# Membrane motor
	var motor_pos := Vector2(simulation.motor_interior_x, simulation.motor_interior_y)
	draw_circle(motor_pos, 8.0, Color(1.0, 0.6, 0.2))
	draw_arc(motor_pos, 10.0, 0, TAU, 16, Color(1.0, 0.8, 0.3), 2.0)

	# mRNA strands
	var mrna_xs: PackedFloat32Array = simulation.mrna_xs
	var mrna_ys: PackedFloat32Array = simulation.mrna_ys
	var mrna_types: PackedInt32Array = simulation.mrna_types
	var mrna_colors: Array[Color] = [
		Color(0.3, 0.8, 0.3),   # enzyme — green
		Color(1.0, 0.6, 0.2),   # motor — orange
		Color(0.3, 0.85, 0.9),  # membrane — cyan
	]
	for i in range(mrna_xs.size()):
		var center := Vector2(mrna_xs[i], mrna_ys[i])
		var col: Color = mrna_colors[mrna_types[i]]
		# Wavy strand: 4 segments zig-zagging horizontally
		var seg_len := 3.0
		var amp := 2.5
		var start := center + Vector2(-6.0, 0.0)
		for s in range(4):
			var p0 := start + Vector2(s * seg_len, amp if s % 2 == 0 else -amp)
			var p1 := start + Vector2((s + 1) * seg_len, -amp if s % 2 == 0 else amp)
			draw_line(p0, p1, col, 2.0)
		# Colored dot at the right end
		draw_circle(start + Vector2(4 * seg_len, amp if 4 % 2 == 0 else -amp), 3.0, col)

	# Interior particles
	var xs: PackedFloat32Array = simulation.interior_xs
	var ys: PackedFloat32Array = simulation.interior_ys
	var types: PackedInt32Array = simulation.interior_types
	var glucose_color := Color(0.95, 0.75, 0.2)
	var amino_color := Color(0.5, 0.3, 0.85)
	var atp_color := Color(0.3, 0.9, 1.0)

	for i in range(xs.size()):
		var pos := Vector2(xs[i], ys[i])
		var col: Color
		match types[i]:
			0: col = glucose_color
			1: col = amino_color
			2: col = atp_color
			_: col = glucose_color
		draw_circle(pos, 5.0, col)
