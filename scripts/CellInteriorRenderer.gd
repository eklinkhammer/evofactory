extends Node2D

@onready var simulation: Node = get_node("../Simulation")

var hovered_index: int = -1

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

	# Drop target highlight: enzyme glows green when dragging glucose
	if simulation.drag_active and simulation.dragged_particle_type == 0:
		draw_circle(enzyme_pos, enzyme_r + 4.0, Color(0.3, 1.0, 0.3, 0.3))
		draw_arc(enzyme_pos, enzyme_r + 4.0, 0, TAU, 16, Color(0.4, 1.0, 0.4, 0.8), 2.0)

	draw_circle(enzyme_pos, enzyme_r, Color(0.2, 0.5, 0.2))
	draw_arc(enzyme_pos, enzyme_r, 0, TAU, 6, Color(0.4, 0.9, 0.4), 2.0)

	# Membrane motor
	var motor_pos := Vector2(simulation.motor_interior_x, simulation.motor_interior_y)

	# Drop target highlight: motor glows orange when dragging ATP
	if simulation.drag_active and simulation.dragged_particle_type == 2:
		draw_circle(motor_pos, 14.0, Color(1.0, 0.7, 0.2, 0.3))
		draw_arc(motor_pos, 14.0, 0, TAU, 16, Color(1.0, 0.8, 0.3, 0.8), 2.0)

	draw_circle(motor_pos, 8.0, Color(1.0, 0.6, 0.2))
	draw_arc(motor_pos, 10.0, 0, TAU, 16, Color(1.0, 0.8, 0.3), 2.0)

	# Motor charge pips: 5 small circles in semicircle around motor
	var max_charge := int(simulation.player_max_atp)
	var current_charge := int(simulation.motor_charge_display)
	for i in range(max_charge):
		var pip_angle: float = PI + (float(i) / float(max_charge - 1)) * PI
		var pip_pos := motor_pos + Vector2(cos(pip_angle), sin(pip_angle)) * 18.0
		if i < current_charge:
			draw_circle(pip_pos, 3.0, Color(1.0, 0.8, 0.3))
		else:
			draw_circle(pip_pos, 3.0, Color(0.3, 0.3, 0.3))
			draw_arc(pip_pos, 3.0, 0, TAU, 8, Color(0.5, 0.5, 0.5), 1.0)
	# Completion ring when motor fully charged
	if current_charge >= max_charge:
		draw_arc(motor_pos, 14.0, 0, TAU, 24, Color(1.0, 0.8, 0.3), 2.5)

	# mRNA strands
	var mrna_xs: PackedFloat32Array = simulation.mrna_xs
	var mrna_ys: PackedFloat32Array = simulation.mrna_ys
	var mrna_types: PackedInt32Array = simulation.mrna_types
	var mrna_colors: Array[Color] = [
		Color(0.3, 0.8, 0.3),   # enzyme — green
		Color(1.0, 0.6, 0.2),   # motor — orange
		Color(0.3, 0.85, 0.9),  # membrane — cyan
	]
	var mrna_progress: PackedInt32Array = simulation.mrna_progress
	var mrna_required: PackedInt32Array = simulation.mrna_required
	for i in range(mrna_xs.size()):
		var center := Vector2(mrna_xs[i], mrna_ys[i])
		var col: Color = mrna_colors[mrna_types[i]]

		# Glow when dragging amino acid and mRNA not full
		if simulation.drag_active and simulation.dragged_particle_type == 1:
			if i < mrna_progress.size() and i < mrna_required.size():
				if mrna_progress[i] < mrna_required[i]:
					draw_circle(center, 18.0, Color(0.5, 0.3, 0.85, 0.25))
					draw_arc(center, 18.0, 0, TAU, 16, Color(0.6, 0.4, 0.9, 0.7), 2.0)

		# Wavy strand: 4 segments zig-zagging horizontally
		var seg_len := 3.0
		var amp := 2.5
		var s_start := center + Vector2(-6.0, 0.0)
		for s in range(4):
			var p0 := s_start + Vector2(s * seg_len, amp if s % 2 == 0 else -amp)
			var p1 := s_start + Vector2((s + 1) * seg_len, -amp if s % 2 == 0 else amp)
			draw_line(p0, p1, col, 2.0)
		# Colored dot at the right end
		draw_circle(s_start + Vector2(4 * seg_len, amp if 4 % 2 == 0 else -amp), 3.0, col)

		# Progress pips below strand
		if i < mrna_progress.size() and i < mrna_required.size():
			var req: int = mrna_required[i]
			var prog: int = mrna_progress[i]
			var pip_y := center.y + 10.0
			var total_width: float = (req - 1) * 6.0
			var pip_start_x: float = center.x - total_width / 2.0
			for p in range(req):
				var pip_pos := Vector2(pip_start_x + p * 6.0, pip_y)
				if p < prog:
					draw_circle(pip_pos, 2.5, col)
				else:
					draw_circle(pip_pos, 2.5, Color(0.3, 0.3, 0.3))
					draw_arc(pip_pos, 2.5, 0, TAU, 8, Color(0.5, 0.5, 0.5), 1.0)
			# Completion ring when full
			if prog >= req:
				draw_arc(center, 14.0, 0, TAU, 24, col, 2.5)

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

		# Hover ring — white arc around nearest draggable particle
		if i == hovered_index and not simulation.drag_active:
			draw_arc(pos, 8.0, 0, TAU, 16, Color(1.0, 1.0, 1.0, 0.8), 2.0)

	# Drag glow — arc around dragged particle in its color
	if simulation.drag_active:
		var drag_pos := Vector2(simulation.dragged_particle_x, simulation.dragged_particle_y)
		var drag_col: Color
		match simulation.dragged_particle_type:
			0: drag_col = glucose_color
			1: drag_col = amino_color
			2: drag_col = atp_color
			_: drag_col = Color.WHITE
		draw_arc(drag_pos, 9.0, 0, TAU, 16, drag_col, 2.5)
