extends Node2D

var simulation: Node

func _draw() -> void:
	if not simulation or not simulation.tech_panel_open:
		return

	var font := ThemeDB.fallback_font
	var font_size := 20
	var title_size := 28
	var desc_size := 16
	var label_size := 16

	var viewport_size := get_viewport_rect().size
	var panel_w := 500.0
	var row_h := 100.0
	var panel_h: float = 60.0 + float(simulation.tech_count) * row_h + 20.0
	var panel_x := (viewport_size.x - panel_w) / 2.0
	var panel_y := (viewport_size.y - panel_h) / 2.0

	# Background
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.0, 0.0, 0.0, 0.9))
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.4, 0.4, 0.4, 0.6), false, 2.0)

	# Title
	draw_string(font, Vector2(panel_x + 20, panel_y + 38), "Technology", HORIZONTAL_ALIGNMENT_LEFT, -1, title_size, Color.WHITE)

	var names: PackedStringArray = simulation.tech_names
	var descriptions: PackedStringArray = simulation.tech_descriptions
	var progress: PackedFloat32Array = simulation.tech_progress
	var completed: PackedInt32Array = simulation.tech_completed

	var y_offset := panel_y + 64.0

	for i in range(simulation.tech_count):
		var tech_name: String = names[i] if i < names.size() else ""
		var tech_desc: String = descriptions[i] if i < descriptions.size() else ""
		var tech_prog: float = progress[i] if i < progress.size() else 0.0
		var is_complete: bool = i < completed.size() and completed[i] == 1

		# Tech name
		draw_string(font, Vector2(panel_x + 20, y_offset + 24), tech_name, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size, Color(0.9, 0.8, 0.3))

		# Description
		draw_string(font, Vector2(panel_x + 20, y_offset + 46), tech_desc, HORIZONTAL_ALIGNMENT_LEFT, -1, desc_size, Color(0.7, 0.7, 0.7))

		# Progress bar
		var bar_x := panel_x + 20.0
		var bar_y := y_offset + 58.0
		var bar_w := 340.0
		var bar_h := 18.0

		# Background
		draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.2, 0.2, 0.2))
		# Outline
		draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.5, 0.5, 0.5), false, 1.0)
		# Fill
		var fill_color := Color(0.2, 0.8, 0.2) if not is_complete else Color(0.3, 1.0, 0.3)
		draw_rect(Rect2(bar_x, bar_y, bar_w * tech_prog, bar_h), fill_color)

		# Percentage / complete label
		var label_x := bar_x + bar_w + 12.0
		if is_complete:
			draw_string(font, Vector2(label_x, bar_y + 15), "COMPLETE", HORIZONTAL_ALIGNMENT_LEFT, -1, label_size, Color(0.3, 1.0, 0.3))
		else:
			var pct := "%d%%" % int(tech_prog * 100.0)
			draw_string(font, Vector2(label_x, bar_y + 15), pct, HORIZONTAL_ALIGNMENT_LEFT, -1, label_size, Color(0.8, 0.8, 0.8))

		y_offset += row_h
