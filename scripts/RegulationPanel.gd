extends Node2D

var simulation: Node

func _draw() -> void:
	if not simulation or not simulation.regulation_panel_open:
		return

	var font := ThemeDB.fallback_font
	var font_size := 12
	var title_size := 16

	var viewport_size := get_viewport_rect().size
	var panel_w := 340.0
	var panel_h: float = 40.0 + float(simulation.rule_count) * 60.0 + 30.0
	var panel_x := viewport_size.x - panel_w - 20.0
	var panel_y := 60.0

	# Background
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.0, 0.0, 0.0, 0.8))
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.4, 0.4, 0.4, 0.6), false, 1.0)

	# Title
	draw_string(font, Vector2(panel_x + 10, panel_y + 22), "Gene Regulation", HORIZONTAL_ALIGNMENT_LEFT, -1, title_size, Color.WHITE)

	var mrna_colors: Array[Color] = [
		Color(0.3, 0.8, 0.3),   # zymase
		Color(1.0, 0.6, 0.2),   # motor
		Color(0.3, 0.85, 0.9),  # membrane
	]
	var mrna_names := ["Zymase", "Motor", "Membrane"]

	var y_offset := panel_y + 44.0
	var descriptions: PackedStringArray = simulation.rule_descriptions
	var firing: PackedInt32Array = simulation.rule_firing
	var targets: PackedInt32Array = simulation.rule_targets
	var limits: PackedInt32Array = simulation.rule_limits

	for ri in range(simulation.rule_count):
		var target_idx: int = targets[ri] if ri < targets.size() else 0
		var col: Color = mrna_colors[target_idx]
		var is_firing: bool = ri < firing.size() and firing[ri] == 1
		var limit_val: int = limits[ri] if ri < limits.size() else 0
		var desc: String = descriptions[ri] if ri < descriptions.size() else ""

		# Target name with color
		draw_string(font, Vector2(panel_x + 14, y_offset + 14), mrna_names[target_idx] + " mRNA", HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, col)

		# Condition description
		draw_string(font, Vector2(panel_x + 14, y_offset + 30), desc, HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(0.7, 0.7, 0.7))

		# Status
		if is_firing:
			draw_string(font, Vector2(panel_x + 14, y_offset + 46), "ACTIVE — limit: %d" % limit_val, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(1.0, 0.3, 0.3))
		else:
			draw_string(font, Vector2(panel_x + 14, y_offset + 46), "Idle — limit: %d" % limit_val, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.5, 0.5, 0.5))

		y_offset += 60.0

	# Footer
	draw_string(font, Vector2(panel_x + 10, y_offset + 14), "Active rules block amino acid delivery.", HORIZONTAL_ALIGNMENT_LEFT, -1, 10, Color(0.5, 0.5, 0.5, 0.7))
