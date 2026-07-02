extends Node2D

@onready var simulation: Node = get_node("../Simulation")

var hovered_index: int = -1
var tooltip_target: int = -1  # -1=none, 0=zymase, 1/2/3=mRNA indices
var interior_entry_count: int = 0
var was_interior: bool = false

func reset_tutorial_state() -> void:
	interior_entry_count = 0
	was_interior = false

func _ready() -> void:
	z_index = 1

func _draw() -> void:
	if not simulation or not simulation.interior_view:
		was_interior = false
		return

	# Track interior entries for A3 label enhancement
	if not was_interior:
		interior_entry_count += 1
		was_interior = true

	var radius: float = simulation.interior_radius

	# Dark backdrop so interior is readable against the world
	draw_circle(Vector2.ZERO, radius + 10.0, Color(0.0, 0.0, 0.0, 0.7))

	# Membrane fill
	draw_circle(Vector2.ZERO, radius, Color(0.08, 0.15, 0.08))
	# Membrane outline
	draw_arc(Vector2.ZERO, radius, 0, TAU, 64, Color(0.3, 0.8, 0.3), 3.0)

	# Zymases (multiple)
	var z_xs: PackedFloat32Array = simulation.zymase_xs
	var z_ys: PackedFloat32Array = simulation.zymase_ys
	var z_bufs: PackedInt32Array = simulation.zymase_buffers
	var z_proc: PackedInt32Array = simulation.zymase_processing_flags
	var z_tmrs: PackedFloat32Array = simulation.zymase_timers
	var zymase_r: float = simulation.zymase_interior_radius

	var font := ThemeDB.fallback_font
	# A3: Larger labels on first 3 interior entries
	var enhanced_labels: bool = interior_entry_count <= 3
	var font_size := 14 if enhanced_labels else 10
	var label_alpha_base: float = 1.0 if enhanced_labels else 0.6

	var zymase_label_width: float = 0.0
	if enhanced_labels:
		zymase_label_width = font.get_string_size("Zymase", HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x

	for zi in range(z_xs.size()):
		var zymase_pos := Vector2(z_xs[zi], z_ys[zi])

		# Drop target highlight: zymase glows green when dragging glucose
		if simulation.drag_active and simulation.dragged_particle_type == 0:
			if zi < z_bufs.size() and z_bufs[zi] < 2:
				draw_circle(zymase_pos, zymase_r + 4.0, Color(0.3, 1.0, 0.3, 0.3))
				draw_arc(zymase_pos, zymase_r + 4.0, 0, TAU, 16, Color(0.4, 1.0, 0.4, 0.8), 2.0)

		draw_circle(zymase_pos, zymase_r, Color(0.2, 0.5, 0.2))
		draw_arc(zymase_pos, zymase_r, 0, TAU, 6, Color(0.4, 0.9, 0.4), 2.0)

		# Zymase label
		var z_label_pos := zymase_pos + Vector2(-15, zymase_r + 14)
		if enhanced_labels:
			draw_rect(Rect2(z_label_pos + Vector2(-3, -font_size + 2), Vector2(zymase_label_width + 6, font_size + 4)), Color(0.0, 0.0, 0.0, 0.5))
		draw_string(font, z_label_pos, "Zymase", HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.4, 0.9, 0.4, label_alpha_base))

		# Zymase processing progress arc
		if zi < z_proc.size() and z_proc[zi] == 1:
			var progress: float = 1.0 - z_tmrs[zi] / 2.0
			draw_arc(zymase_pos, zymase_r + 6.0, -PI / 2, -PI / 2 + progress * TAU, 32, Color(0.4, 1.0, 0.4, 0.8), 2.5)

		# Zymase buffer pips
		var buf_val: int = z_bufs[zi] if zi < z_bufs.size() else 0
		for b in range(2):
			var pip_pos := zymase_pos + Vector2(-5.0 + b * 10.0, zymase_r + 24)
			if b < buf_val:
				draw_circle(pip_pos, 3.0, Color(0.95, 0.75, 0.2))
			else:
				draw_circle(pip_pos, 3.0, Color(0.3, 0.3, 0.3))
				draw_arc(pip_pos, 3.0, 0, TAU, 8, Color(0.5, 0.5, 0.5), 1.0)

	# Membrane motors (multiple)
	var m_xs: PackedFloat32Array = simulation.motor_xs
	var m_ys: PackedFloat32Array = simulation.motor_ys
	var m_charges: PackedFloat32Array = simulation.motor_charges
	var max_atp_per_motor: float = 5.0

	for mi in range(m_xs.size()):
		var motor_pos := Vector2(m_xs[mi], m_ys[mi])
		var motor_charge: float = m_charges[mi]

		# Drop target highlight: motor glows orange when dragging ATP and not full
		if simulation.drag_active and simulation.dragged_particle_type == 2 and motor_charge < max_atp_per_motor:
			draw_circle(motor_pos, 14.0, Color(1.0, 0.7, 0.2, 0.3))
			draw_arc(motor_pos, 14.0, 0, TAU, 16, Color(1.0, 0.8, 0.3, 0.8), 2.0)

		# Motor drag glow (type 100)
		if simulation.drag_active and simulation.dragged_particle_type == 100:
			var drag_pos := Vector2(simulation.dragged_particle_x, simulation.dragged_particle_y)
			var dx: float = drag_pos.x - motor_pos.x
			var dy: float = drag_pos.y - motor_pos.y
			if dx * dx + dy * dy < 1.0:
				draw_arc(motor_pos, 12.0, 0, TAU, 16, Color(1.0, 0.6, 0.2, 0.7), 2.5)

		draw_circle(motor_pos, 8.0, Color(1.0, 0.6, 0.2))
		draw_arc(motor_pos, 10.0, 0, TAU, 16, Color(1.0, 0.8, 0.3), 2.0)

		# Motor charge pips: 5 small circles in semicircle around motor
		var max_charge := int(max_atp_per_motor)
		var current_charge := int(motor_charge)
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
	var mrna_names := ["Zymase", "Motor", "Membrane"]
	var mrna_proc: PackedInt32Array = simulation.mrna_processing_flags
	var mrna_tmrs: PackedFloat32Array = simulation.mrna_timers_display
	var mrna_colors: Array[Color] = [
		Color(0.3, 0.8, 0.3),   # zymase — green
		Color(1.0, 0.6, 0.2),   # motor — orange
		Color(0.3, 0.85, 0.9),  # membrane — cyan
	]
	var mrna_progress: PackedInt32Array = simulation.mrna_progress
	var mrna_required: PackedInt32Array = simulation.mrna_required
	var mrna_sup: PackedInt32Array = simulation.mrna_suppressed
	for i in range(mrna_xs.size()):
		var center := Vector2(mrna_xs[i], mrna_ys[i])
		var col: Color = mrna_colors[mrna_types[i]]
		var is_suppressed: bool = i < mrna_sup.size() and mrna_sup[i] == 1

		# Glow when dragging amino acid and mRNA not full and not suppressed
		if simulation.drag_active and simulation.dragged_particle_type == 1 and not is_suppressed:
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

		# mRNA label (dimmed if suppressed)
		var label_alpha: float = 0.3 if is_suppressed else label_alpha_base
		var mrna_label_pos := center + Vector2(-15, 22)
		if enhanced_labels and not is_suppressed:
			var lw: float = font.get_string_size(mrna_names[i], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
			draw_rect(Rect2(mrna_label_pos + Vector2(-3, -font_size + 2), Vector2(lw + 6, font_size + 4)), Color(0.0, 0.0, 0.0, 0.5))
		draw_string(font, mrna_label_pos, mrna_names[i], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(col.r, col.g, col.b, label_alpha))

		# Suppression indicator: red X near the strand
		if is_suppressed:
			var ix := center + Vector2(18, -4)
			var cross_size := 4.0
			draw_line(ix + Vector2(-cross_size, -cross_size), ix + Vector2(cross_size, cross_size), Color(1.0, 0.25, 0.25, 0.9), 2.0)
			draw_line(ix + Vector2(cross_size, -cross_size), ix + Vector2(-cross_size, cross_size), Color(1.0, 0.25, 0.25, 0.9), 2.0)

		# mRNA processing progress arc
		if i < mrna_proc.size() and mrna_proc[i] == 1:
			var tprog: float = 1.0 - mrna_tmrs[i] / 2.0
			draw_arc(center, 16.0, -PI / 2, -PI / 2 + tprog * TAU, 32, Color(col.r, col.g, col.b, 0.8), 2.5)

	# Nucleus organelles
	if simulation.nucleus_unlocked_flag:
		var nuc_xs: PackedFloat32Array = simulation.nucleus_xs
		var nuc_ys: PackedFloat32Array = simulation.nucleus_ys
		var nuc_targets: PackedInt32Array = simulation.nucleus_target_types
		var nuc_prog: PackedInt32Array = simulation.nucleus_progress
		var nuc_req: PackedInt32Array = simulation.nucleus_required
		var nuc_proc: PackedInt32Array = simulation.nucleus_processing_flags
		var nuc_tmrs: PackedFloat32Array = simulation.nucleus_timers
		var nuc_names := ["Zymase", "Motor", "Membrane"]

		for ni in range(nuc_xs.size()):
			var center := Vector2(nuc_xs[ni], nuc_ys[ni])
			var tt: int = clampi(nuc_targets[ni], 0, 2) if ni < nuc_targets.size() else 0
			var col: Color = mrna_colors[tt]

			# Drop-target glow when dragging nucleotide
			if simulation.drag_active and simulation.dragged_particle_type == 3:
				if ni < nuc_prog.size() and ni < nuc_req.size():
					if nuc_prog[ni] < nuc_req[ni] and nuc_proc[ni] == 0:
						draw_circle(center, 18.0, Color(0.9, 0.3, 0.4, 0.25))
						draw_arc(center, 18.0, 0, TAU, 16, Color(0.9, 0.4, 0.5, 0.7), 2.0)

			# Double circle (outer + inner)
			draw_arc(center, 14.0, 0, TAU, 32, col, 2.5)
			draw_arc(center, 8.0, 0, TAU, 24, col, 2.0)

			# Label
			var label_text: String = "Nuc:" + nuc_names[tt]
			var nuc_label_pos := center + Vector2(-20, 26)
			if enhanced_labels:
				var lw: float = font.get_string_size(label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
				draw_rect(Rect2(nuc_label_pos + Vector2(-3, -font_size + 2), Vector2(lw + 6, font_size + 4)), Color(0.0, 0.0, 0.0, 0.5))
			draw_string(font, nuc_label_pos, label_text, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(col.r, col.g, col.b, label_alpha_base))

			# Progress pips
			if ni < nuc_prog.size() and ni < nuc_req.size():
				var req: int = nuc_req[ni]
				var prog: int = nuc_prog[ni]
				var pip_y := center.y + 14.0
				var total_width: float = (req - 1) * 6.0
				var pip_start_x: float = center.x - total_width / 2.0
				for p in range(req):
					var pip_pos := Vector2(pip_start_x + p * 6.0, pip_y)
					if p < prog:
						draw_circle(pip_pos, 2.5, col)
					else:
						draw_circle(pip_pos, 2.5, Color(0.3, 0.3, 0.3))
						draw_arc(pip_pos, 2.5, 0, TAU, 8, Color(0.5, 0.5, 0.5), 1.0)

			# Processing arc
			if ni < nuc_proc.size() and ni < nuc_tmrs.size() and nuc_proc[ni] == 1:
				var tprog: float = 1.0 - nuc_tmrs[ni] / 2.0
				draw_arc(center, 16.0, -PI / 2, -PI / 2 + tprog * TAU, 32, Color(col.r, col.g, col.b, 0.8), 2.5)

	# Interior particles
	var xs: PackedFloat32Array = simulation.interior_xs
	var ys: PackedFloat32Array = simulation.interior_ys
	var types: PackedInt32Array = simulation.interior_types
	var glucose_color := Color(0.95, 0.75, 0.2)
	var amino_color := Color(0.5, 0.3, 0.85)
	var atp_color := Color(0.3, 0.9, 1.0)
	var nucleotide_color := Color(0.9, 0.3, 0.4)

	for i in range(xs.size()):
		var pos := Vector2(xs[i], ys[i])
		var col: Color
		match types[i]:
			0: col = glucose_color
			1: col = amino_color
			2: col = atp_color
			3: col = nucleotide_color
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
			3: drag_col = nucleotide_color
			100: drag_col = Color(1.0, 0.6, 0.2)  # motor orange
			_: drag_col = Color.WHITE
		draw_arc(drag_pos, 9.0, 0, TAU, 16, drag_col, 2.5)

	# Tooltip
	if tooltip_target >= 0:
		var tooltip_pos: Vector2
		var lines: PackedStringArray
		match tooltip_target:
			0:
				var first_zymase := Vector2(z_xs[0], z_ys[0]) if z_xs.size() > 0 else Vector2.ZERO
				tooltip_pos = first_zymase + Vector2(20, -30)
				lines = PackedStringArray(["Fermentation", "1 Glucose -> 2 ATP", "Time: 2.0s | Buffer: 2"])
			1:
				tooltip_pos = Vector2(mrna_xs[0], mrna_ys[0]) + Vector2(20, -30)
				lines = PackedStringArray(["Zymase Protein", "8 Amino Acids -> Zymase", "Time: 2.0s each"])
			2:
				tooltip_pos = Vector2(mrna_xs[1], mrna_ys[1]) + Vector2(20, -30)
				lines = PackedStringArray(["Motor Protein", "7 Amino Acids -> Motor", "Time: 2.0s each"])
			3:
				tooltip_pos = Vector2(mrna_xs[2], mrna_ys[2]) + Vector2(20, -30)
				lines = PackedStringArray(["Membrane Protein", "5 Amino Acids -> Membrane", "Time: 2.0s each"])
			4:
				var n_xs: PackedFloat32Array = simulation.nucleus_xs
				var n_ys: PackedFloat32Array = simulation.nucleus_ys
				var n_targets: PackedInt32Array = simulation.nucleus_target_types
				var n_req: PackedInt32Array = simulation.nucleus_required
				if n_xs.size() > 0:
					tooltip_pos = Vector2(n_xs[0], n_ys[0]) + Vector2(20, -30)
					var t_name: String = ["Zymase", "Motor", "Membrane"][clampi(n_targets[0], 0, 2)] if n_targets.size() > 0 else "Zymase"
					var req_val: int = n_req[0] if n_req.size() > 0 else 8
					lines = PackedStringArray(["Programmable Nucleus", str(req_val) + " Nucleotides -> " + t_name, "Click to cycle target"])
		if lines.size() > 0:
			var line_h := 14
			var padding := Vector2(6, 4)
			var max_width := 0.0
			for line in lines:
				var w: float = font.get_string_size(line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
				max_width = max(max_width, w)
			var box_size := Vector2(max_width + padding.x * 2, lines.size() * line_h + padding.y * 2)
			draw_rect(Rect2(tooltip_pos, box_size), Color(0.0, 0.0, 0.0, 0.85))
			draw_rect(Rect2(tooltip_pos, box_size), Color(0.5, 0.5, 0.5, 0.5), false, 1.0)
			for li in range(lines.size()):
				draw_string(font, tooltip_pos + Vector2(padding.x, padding.y + (li + 1) * line_h - 2), lines[li], HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color.WHITE)
