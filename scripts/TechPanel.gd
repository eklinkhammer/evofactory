extends Node2D

var simulation: Node
var _wrap_cache_selected := -1
var _wrap_cache_desc := ""
var _wrap_cache_width := 0.0
var _wrap_cache_lines: PackedStringArray = PackedStringArray()

func _input(event: InputEvent) -> void:
	if not simulation or not simulation.tech_panel_open:
		return
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		var viewport_size := get_viewport_rect().size
		var panel_w := viewport_size.x * 0.9
		var panel_h := viewport_size.y * 0.9
		var panel_x := (viewport_size.x - panel_w) / 2.0
		var panel_y := (viewport_size.y - panel_h) / 2.0
		var left_col_w := 250.0
		var header_h := 50.0
		var section_header_h := 30.0
		var row_h := 60.0
		var mx: float = event.position.x
		var my: float = event.position.y

		# Check if click is in left column
		var left_x := panel_x + 10.0
		var left_right := panel_x + left_col_w

		if mx < left_x or mx > left_right:
			return

		# Find current research index (first incomplete tech, or last if all complete)
		var current_idx := _get_current_research_idx()

		# Current research section starts after title
		var cy := panel_y + header_h + section_header_h
		# Current research row
		if my >= cy and my < cy + row_h:
			simulation.select_tech(current_idx)
			queue_redraw()
			get_viewport().set_input_as_handled()
			return

		# Future research section
		var fy := cy + row_h + section_header_h
		for i in range(simulation.tech_count):
			if i == current_idx:
				continue
			if my >= fy and my < fy + row_h:
				simulation.select_tech(i)
				queue_redraw()
				get_viewport().set_input_as_handled()
				return
			fy += row_h

func _get_current_research_idx() -> int:
	if not simulation:
		return 0
	var completed: PackedInt32Array = simulation.tech_completed
	for i in range(simulation.tech_count):
		if i < completed.size() and completed[i] == 0:
			return i
	return max(0, simulation.tech_count - 1)

func _draw() -> void:
	if not simulation or not simulation.tech_panel_open:
		return

	var font := ThemeDB.fallback_font
	var title_size := 26
	var section_size := 18
	var name_size := 18
	var desc_size := 15
	var detail_name_size := 24
	var detail_desc_size := 16
	var detail_status_size := 16
	var label_size := 14

	var viewport_size := get_viewport_rect().size
	var panel_w := viewport_size.x * 0.9
	var panel_h := viewport_size.y * 0.9
	var panel_x := (viewport_size.x - panel_w) / 2.0
	var panel_y := (viewport_size.y - panel_h) / 2.0
	var left_col_w := 250.0
	var header_h := 50.0
	var section_header_h := 30.0
	var row_h := 60.0

	# Full-screen dark background
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.05, 0.05, 0.08, 0.95))
	draw_rect(Rect2(panel_x, panel_y, panel_w, panel_h), Color(0.3, 0.35, 0.3, 0.8), false, 2.0)

	# Title bar
	draw_rect(Rect2(panel_x, panel_y, panel_w, header_h), Color(0.1, 0.12, 0.1, 1.0))
	draw_string(font, Vector2(panel_x + 20, panel_y + 34), "TECHNOLOGY RESEARCH", HORIZONTAL_ALIGNMENT_LEFT, -1, title_size, Color(0.9, 0.85, 0.4))

	# Divider below title
	draw_line(Vector2(panel_x, panel_y + header_h), Vector2(panel_x + panel_w, panel_y + header_h), Color(0.3, 0.35, 0.3, 0.8), 1.0)

	# Vertical divider between columns
	var divider_x := panel_x + left_col_w
	draw_line(Vector2(divider_x, panel_y + header_h), Vector2(divider_x, panel_y + panel_h), Color(0.3, 0.35, 0.3, 0.8), 1.0)

	var names: PackedStringArray = simulation.tech_names
	var descriptions: PackedStringArray = simulation.tech_descriptions
	var progress: PackedFloat32Array = simulation.tech_progress
	var completed: PackedInt32Array = simulation.tech_completed
	var selected: int = simulation.tech_selected

	var current_idx := _get_current_research_idx()

	# ---- LEFT COLUMN ----
	var ly := panel_y + header_h

	# "CURRENT RESEARCH" section header
	draw_string(font, Vector2(panel_x + 14, ly + 20), "CURRENT RESEARCH", HORIZONTAL_ALIGNMENT_LEFT, -1, section_size, Color(0.6, 0.65, 0.6))
	ly += section_header_h

	# Current research row
	_draw_tech_row(panel_x + 10.0, ly, left_col_w - 20.0, row_h, current_idx, names, progress, completed, selected == current_idx)
	ly += row_h

	# "FUTURE RESEARCH" section header
	draw_string(font, Vector2(panel_x + 14, ly + 20), "FUTURE RESEARCH", HORIZONTAL_ALIGNMENT_LEFT, -1, section_size, Color(0.6, 0.65, 0.6))
	ly += section_header_h

	# Future research rows (all techs except current)
	for i in range(simulation.tech_count):
		if i == current_idx:
			continue
		_draw_tech_row(panel_x + 10.0, ly, left_col_w - 20.0, row_h, i, names, progress, completed, selected == i)
		ly += row_h

	# ---- RIGHT COLUMN (detail pane) ----
	var right_x := divider_x + 30.0
	var right_y := panel_y + header_h + 30.0
	var right_w := panel_w - left_col_w - 60.0

	if selected >= 0 and selected < simulation.tech_count:
		var sel_name: String = names[selected] if selected < names.size() else ""
		var sel_desc: String = descriptions[selected] if selected < descriptions.size() else ""
		var sel_prog: float = progress[selected] if selected < progress.size() else 0.0
		var sel_complete: bool = selected < completed.size() and completed[selected] == 1

		# Tech name
		draw_string(font, Vector2(right_x, right_y + 24), sel_name, HORIZONTAL_ALIGNMENT_LEFT, -1, detail_name_size, Color(0.9, 0.85, 0.4))

		# Description (word-wrap with cache)
		var desc_y := right_y + 60.0
		var max_line_w := right_w
		var wrapped := _get_wrapped_lines(sel_desc, font, detail_desc_size, max_line_w, selected)
		for wline in wrapped:
			draw_string(font, Vector2(right_x, desc_y), wline, HORIZONTAL_ALIGNMENT_LEFT, -1, detail_desc_size, Color(0.75, 0.75, 0.75))
			desc_y += 22.0

		# Status line
		desc_y += 20.0
		var status_text := ""
		var status_color := Color.WHITE
		if sel_complete:
			status_text = "Status: Complete"
			status_color = Color(0.3, 1.0, 0.3)
		elif sel_prog > 0.0:
			status_text = "Status: Researching"
			status_color = Color(0.9, 0.85, 0.4)
		else:
			status_text = "Status: Locked"
			status_color = Color(0.5, 0.5, 0.5)
		draw_string(font, Vector2(right_x, desc_y), status_text, HORIZONTAL_ALIGNMENT_LEFT, -1, detail_status_size, status_color)

		# Progress bar (if not locked / has progress or researching)
		if sel_prog > 0.0 or sel_complete:
			desc_y += 30.0
			var bar_w: float = min(right_w, 400.0)
			var bar_h := 22.0
			# Background
			draw_rect(Rect2(right_x, desc_y, bar_w, bar_h), Color(0.15, 0.15, 0.15))
			draw_rect(Rect2(right_x, desc_y, bar_w, bar_h), Color(0.4, 0.4, 0.4), false, 1.0)
			# Fill
			var fill_color := Color(0.3, 1.0, 0.3) if sel_complete else Color(0.2, 0.7, 0.2)
			draw_rect(Rect2(right_x, desc_y, bar_w * sel_prog, bar_h), fill_color)
			# Percentage
			var pct_text := "100%" if sel_complete else ("%d%%" % int(sel_prog * 100.0))
			draw_string(font, Vector2(right_x + bar_w + 12.0, desc_y + 17), pct_text, HORIZONTAL_ALIGNMENT_LEFT, -1, label_size, Color(0.8, 0.8, 0.8))


func _draw_tech_row(x: float, y: float, w: float, h: float, idx: int, names: PackedStringArray, progress: PackedFloat32Array, completed: PackedInt32Array, is_selected: bool) -> void:
	var font := ThemeDB.fallback_font
	var name_size := 16
	var label_size := 12

	# Row background
	var bg_color := Color(0.15, 0.18, 0.15, 0.9) if is_selected else Color(0.08, 0.08, 0.1, 0.5)
	draw_rect(Rect2(x, y, w, h - 2), bg_color)
	if is_selected:
		draw_rect(Rect2(x, y, w, h - 2), Color(0.4, 0.5, 0.3, 0.8), false, 2.0)

	var tech_name: String = names[idx] if idx < names.size() else ""
	var tech_prog: float = progress[idx] if idx < progress.size() else 0.0
	var is_complete: bool = idx < completed.size() and completed[idx] == 1

	# Tech name
	var name_color := Color(0.9, 0.85, 0.4) if (tech_prog > 0.0 or is_complete) else Color(0.5, 0.5, 0.5)
	draw_string(font, Vector2(x + 8, y + 20), tech_name, HORIZONTAL_ALIGNMENT_LEFT, -1, name_size, name_color)

	# Mini progress bar
	var bar_x := x + 8.0
	var bar_y := y + 28.0
	var bar_w := w - 16.0
	var bar_h := 14.0

	draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.15, 0.15, 0.15))
	if is_complete:
		draw_rect(Rect2(bar_x, bar_y, bar_w, bar_h), Color(0.3, 1.0, 0.3))
	elif tech_prog > 0.0:
		draw_rect(Rect2(bar_x, bar_y, bar_w * tech_prog, bar_h), Color(0.2, 0.7, 0.2))

	# Label below bar
	var lbl := ""
	if is_complete:
		lbl = "COMPLETE"
	elif tech_prog > 0.0:
		lbl = "%d%%" % int(tech_prog * 100.0)
	else:
		lbl = "LOCKED"
	var lbl_color := Color(0.3, 1.0, 0.3) if is_complete else (Color(0.8, 0.8, 0.8) if tech_prog > 0.0 else Color(0.4, 0.4, 0.4))
	draw_string(font, Vector2(bar_x, bar_y + bar_h + 12), lbl, HORIZONTAL_ALIGNMENT_LEFT, -1, label_size, lbl_color)

func _get_wrapped_lines(text: String, font: Font, font_size: int, max_w: float, selected_idx: int) -> PackedStringArray:
	if selected_idx == _wrap_cache_selected and text == _wrap_cache_desc and max_w == _wrap_cache_width:
		return _wrap_cache_lines
	_wrap_cache_selected = selected_idx
	_wrap_cache_desc = text
	_wrap_cache_width = max_w
	_wrap_cache_lines = PackedStringArray()
	var words := text.split(" ")
	var line := ""
	for word in words:
		var test_line := line + (" " if line.length() > 0 else "") + word
		var test_w := font.get_string_size(test_line, HORIZONTAL_ALIGNMENT_LEFT, -1, font_size).x
		if test_w > max_w and line.length() > 0:
			_wrap_cache_lines.append(line)
			line = word
		else:
			line = test_line
	if line.length() > 0:
		_wrap_cache_lines.append(line)
	return _wrap_cache_lines
