extends Node2D

var simulation: Node

const METRIC_NAMES := ["Count", "Density", "SurfDens"]
const RELATION_SYMBOLS := [">=", "<="]
const MRNA_NAMES := ["Zymase", "Motor", "Membrane"]
const MRNA_COLORS: Array[Color] = [
	Color(0.3, 0.8, 0.3),   # zymase
	Color(1.0, 0.6, 0.2),   # motor
	Color(0.3, 0.85, 0.9),  # membrane
]

const PANEL_W := 740.0
const ROW_H := 48.0
const HEADER_H := 54.0
const FOOTER_H := 48.0
const PAD := 16.0
const FONT_SIZE := 18
const TITLE_SIZE := 28
const SMALL_SIZE := 14

# Column X offsets (relative to panel_x)
const COL_METRIC := 14.0
const COL_METRIC_W := 90.0
const COL_SUBJECT := 112.0
const COL_SUBJECT_W := 100.0
const COL_RELATION := 220.0
const COL_RELATION_W := 40.0
const COL_THRESH := 268.0
const COL_THRESH_W := 140.0
const COL_ARROW := 416.0
const COL_TARGET := 448.0
const COL_TARGET_W := 100.0
const COL_VALUE := 556.0
const COL_VALUE_W := 80.0
const COL_ENABLED := 644.0
const COL_ENABLED_W := 36.0
const COL_DELETE := 688.0
const COL_DELETE_W := 36.0

# Short metric names for threshold display
const METRIC_SHORT := ["Cnt", "Dens", "SD"]

# Popup menu constants
const POPUP_OPTION_H := 36.0
const POPUP_OPTIONS := ["Custom", "Zymase", "Motor", "Membrane"]
const POPUP_W := 150.0

var _panel_x: float = 0.0
var _panel_y: float = 0.0
var _panel_h: float = 0.0

# Inline text editing state
var _editing_rule: int = -1
var _edit_text: String = ""

# Popup menu state
var _popup_rule: int = -1

func _draw() -> void:
	if not simulation or not simulation.regulation_panel_open:
		return

	var font := ThemeDB.fallback_font

	var viewport_size := get_viewport_rect().size
	var rule_count: int = simulation.rule_count
	_panel_h = HEADER_H + float(rule_count) * ROW_H + FOOTER_H + 16.0
	_panel_x = (viewport_size.x - PANEL_W) / 2.0
	_panel_y = (viewport_size.y - _panel_h) / 2.0

	# Background
	draw_rect(Rect2(_panel_x, _panel_y, PANEL_W, _panel_h), Color(0.0, 0.0, 0.0, 0.9))
	draw_rect(Rect2(_panel_x, _panel_y, PANEL_W, _panel_h), Color(0.4, 0.4, 0.4, 0.6), false, 2.0)

	# Title
	draw_string(font, Vector2(_panel_x + PAD, _panel_y + 38), "Gene Regulation", HORIZONTAL_ALIGNMENT_LEFT, -1, TITLE_SIZE, Color.WHITE)

	var metrics: PackedInt32Array = simulation.rule_metrics
	var subjects: PackedInt32Array = simulation.rule_subjects
	var relations: PackedInt32Array = simulation.rule_relations
	var thresholds: PackedFloat32Array = simulation.rule_thresholds
	var targets: PackedInt32Array = simulation.rule_targets
	var values: PackedFloat32Array = simulation.rule_values
	var enabled: PackedInt32Array = simulation.rule_enabled
	var firing: PackedInt32Array = simulation.rule_firing
	var locked: PackedInt32Array = simulation.rule_locked
	var threshold_modes: PackedInt32Array = simulation.rule_threshold_modes
	var threshold_targets: PackedInt32Array = simulation.rule_threshold_targets

	for ri in range(rule_count):
		var y_base := _panel_y + HEADER_H + float(ri) * ROW_H
		var y_text := y_base + 32.0

		var metric_idx: int = metrics[ri] if ri < metrics.size() else 0
		var subject_idx: int = subjects[ri] if ri < subjects.size() else 0
		var relation_idx: int = relations[ri] if ri < relations.size() else 0
		var threshold: float = thresholds[ri] if ri < thresholds.size() else 0.0
		var target_idx: int = targets[ri] if ri < targets.size() else 0
		var value: float = values[ri] if ri < values.size() else 0.0
		var is_enabled: bool = ri < enabled.size() and enabled[ri] == 1
		var is_firing: bool = ri < firing.size() and firing[ri] == 1
		var is_locked: bool = ri < locked.size() and locked[ri] == 1
		var thresh_mode: int = threshold_modes[ri] if ri < threshold_modes.size() else 0
		var thresh_target: int = threshold_targets[ri] if ri < threshold_targets.size() else -1

		var dim := Color(0.35, 0.35, 0.35) if is_locked else (Color(0.4, 0.4, 0.4) if not is_enabled else Color(0.85, 0.85, 0.85))
		var fire_col := Color(1.0, 0.3, 0.3) if is_firing else dim

		# Metric
		_draw_button(font, _panel_x + COL_METRIC, y_base, COL_METRIC_W, METRIC_NAMES[metric_idx], dim)

		# Subject — show as "Metric(Name)" via combined metric+subject columns
		var subj_col: Color = MRNA_COLORS[subject_idx] if (is_enabled and not is_locked) else dim
		_draw_button(font, _panel_x + COL_SUBJECT, y_base, COL_SUBJECT_W, MRNA_NAMES[subject_idx], subj_col)

		# Relation
		_draw_button(font, _panel_x + COL_RELATION, y_base, COL_RELATION_W, RELATION_SYMBOLS[relation_idx], dim)

		# Threshold (text entry, variable display, or fixed display)
		if _editing_rule == ri:
			var rect := Rect2(_panel_x + COL_THRESH, y_base + 4, COL_THRESH_W, ROW_H - 8)
			draw_rect(rect, Color(0.15, 0.15, 0.25, 0.9))
			draw_rect(rect, Color(0.4, 0.6, 1.0, 0.8), false, 2.0)
			var display_text := _edit_text + "|"
			draw_string(font, Vector2(_panel_x + COL_THRESH + 6, y_text), display_text, HORIZONTAL_ALIGNMENT_LEFT, int(COL_THRESH_W - 8), FONT_SIZE, Color(1.0, 1.0, 1.0))
		elif thresh_mode == 1 and thresh_target >= 0 and thresh_target < MRNA_NAMES.size():
			# Variable threshold — show as "Metric(Target)" with target color
			var var_col: Color = MRNA_COLORS[thresh_target] if (is_enabled and not is_locked) else dim
			var var_label := METRIC_SHORT[metric_idx] + "(" + MRNA_NAMES[thresh_target] + ")"
			_draw_button(font, _panel_x + COL_THRESH, y_base, COL_THRESH_W, var_label, var_col)
		else:
			var thresh_str: String
			if metric_idx == 0:
				thresh_str = "%d" % int(threshold)
			elif metric_idx == 1:
				thresh_str = "%.4f" % threshold
			else:
				thresh_str = "%.3f" % threshold
			_draw_button(font, _panel_x + COL_THRESH, y_base, COL_THRESH_W, thresh_str, dim)

		# Arrow
		draw_string(font, Vector2(_panel_x + COL_ARROW, y_text), "->", HORIZONTAL_ALIGNMENT_LEFT, -1, FONT_SIZE, fire_col)

		# Target
		var tgt_col: Color = MRNA_COLORS[target_idx] if (is_enabled and not is_locked) else dim
		_draw_button(font, _panel_x + COL_TARGET, y_base, COL_TARGET_W, MRNA_NAMES[target_idx], tgt_col)

		# Current value display
		var val_str: String
		if metric_idx == 0:
			val_str = "%.0f" % value
		elif metric_idx == 1:
			val_str = "%.4f" % value
		else:
			val_str = "%.3f" % value
		draw_string(font, Vector2(_panel_x + COL_VALUE, y_text), val_str, HORIZONTAL_ALIGNMENT_LEFT, -1, SMALL_SIZE, Color(0.6, 0.6, 0.6, 0.9))

		if not is_locked:
			# Enabled toggle
			var en_str := "E" if is_enabled else "D"
			var en_col := Color(0.3, 0.9, 0.3) if is_enabled else Color(0.6, 0.3, 0.3)
			_draw_button(font, _panel_x + COL_ENABLED, y_base, COL_ENABLED_W, en_str, en_col)

			# Delete button
			_draw_button(font, _panel_x + COL_DELETE, y_base, COL_DELETE_W, "x", Color(0.7, 0.3, 0.3))
		else:
			draw_string(font, Vector2(_panel_x + COL_ENABLED, y_text), "locked", HORIZONTAL_ALIGNMENT_LEFT, -1, SMALL_SIZE, Color(0.5, 0.4, 0.2))

	# Add rule button
	var add_y := _panel_y + HEADER_H + float(rule_count) * ROW_H + 8.0
	if rule_count < 5:
		_draw_button(font, _panel_x + PAD, add_y, 130.0, "+ Add Rule", Color(0.5, 0.8, 0.5))

	# Draw popup menu overlay
	if _popup_rule >= 0 and _popup_rule < rule_count:
		_draw_popup(font, _popup_rule)

func _draw_popup(font: Font, rule_idx: int) -> void:
	var popup_x := _panel_x + COL_THRESH
	var row_y := _panel_y + HEADER_H + float(rule_idx) * ROW_H
	var popup_y := row_y + ROW_H
	var popup_h := POPUP_OPTION_H * POPUP_OPTIONS.size()

	# Get current metric for this rule to show qualified names
	var metrics: PackedInt32Array = simulation.rule_metrics
	var metric_idx: int = metrics[rule_idx] if rule_idx < metrics.size() else 0
	var metric_prefix: String = METRIC_SHORT[metric_idx]

	# Background
	draw_rect(Rect2(popup_x, popup_y, POPUP_W, popup_h), Color(0.1, 0.1, 0.1, 0.95))
	draw_rect(Rect2(popup_x, popup_y, POPUP_W, popup_h), Color(0.5, 0.5, 0.5, 0.7), false, 2.0)

	# Options
	var option_colors: Array[Color] = [
		Color(0.85, 0.85, 0.85),  # Custom
		MRNA_COLORS[0],            # Zymase
		MRNA_COLORS[1],            # Motor
		MRNA_COLORS[2],            # Membrane
	]
	for oi in range(POPUP_OPTIONS.size()):
		var oy := popup_y + float(oi) * POPUP_OPTION_H
		var oy_text := oy + 26.0
		var label: String
		if oi == 0:
			label = "Custom"
		else:
			label = metric_prefix + "(" + POPUP_OPTIONS[oi] + ")"
		draw_string(font, Vector2(popup_x + 8, oy_text), label, HORIZONTAL_ALIGNMENT_LEFT, int(POPUP_W - 16), FONT_SIZE, option_colors[oi])

func _draw_button(font: Font, x: float, y: float, w: float, text: String, color: Color) -> void:
	var rect := Rect2(x, y + 4, w, ROW_H - 8)
	draw_rect(rect, Color(0.2, 0.2, 0.2, 0.6))
	draw_rect(rect, Color(0.5, 0.5, 0.5, 0.4), false, 1.0)
	draw_string(font, Vector2(x + 6, y + 32), text, HORIZONTAL_ALIGNMENT_LEFT, int(w - 8), FONT_SIZE, color)

func _commit_edit() -> void:
	if _editing_rule >= 0 and simulation:
		var val := _edit_text.to_float()
		simulation.set_rule_threshold(_editing_rule, val)
	_editing_rule = -1
	_edit_text = ""

func _cancel_edit() -> void:
	_editing_rule = -1
	_edit_text = ""

func _start_edit(rule_idx: int) -> void:
	var thresholds: PackedFloat32Array = simulation.rule_thresholds
	var metrics: PackedInt32Array = simulation.rule_metrics
	if rule_idx < thresholds.size():
		var metric_idx: int = metrics[rule_idx] if rule_idx < metrics.size() else 0
		var threshold: float = thresholds[rule_idx]
		if metric_idx == 0:
			_edit_text = "%d" % int(threshold)
		elif metric_idx == 1:
			_edit_text = "%.4f" % threshold
		else:
			_edit_text = "%.3f" % threshold
	else:
		_edit_text = ""
	_editing_rule = rule_idx

func _open_popup(rule_idx: int) -> void:
	_popup_rule = rule_idx

func _close_popup() -> void:
	_popup_rule = -1

func _input(event: InputEvent) -> void:
	if not simulation or not simulation.regulation_panel_open:
		return

	# Handle keyboard input when editing threshold
	if _editing_rule >= 0 and event is InputEventKey:
		var key := event as InputEventKey
		if not key.pressed:
			return
		if key.keycode == KEY_ENTER or key.keycode == KEY_KP_ENTER:
			_commit_edit()
			get_viewport().set_input_as_handled()
			return
		elif key.keycode == KEY_ESCAPE:
			_cancel_edit()
			get_viewport().set_input_as_handled()
			return
		elif key.keycode == KEY_BACKSPACE:
			if _edit_text.length() > 0:
				_edit_text = _edit_text.substr(0, _edit_text.length() - 1)
			get_viewport().set_input_as_handled()
			return
		else:
			var c := char(key.unicode)
			if c >= "0" and c <= "9" or c == ".":
				_edit_text += c
			get_viewport().set_input_as_handled()
			return

	if not (event is InputEventMouseButton):
		return
	var mb := event as InputEventMouseButton
	if mb.button_index != MOUSE_BUTTON_LEFT or not mb.pressed:
		return

	var mx := mb.position.x
	var my := mb.position.y

	# If editing and click is outside threshold field, commit
	if _editing_rule >= 0:
		_commit_edit()

	# Handle popup clicks first (popup is drawn on top)
	if _popup_rule >= 0:
		var rule_count: int = simulation.rule_count
		if _popup_rule < rule_count:
			var popup_x := _panel_x + COL_THRESH
			var row_y := _panel_y + HEADER_H + float(_popup_rule) * ROW_H
			var popup_y := row_y + ROW_H
			var popup_h := POPUP_OPTION_H * POPUP_OPTIONS.size()

			if mx >= popup_x and mx <= popup_x + POPUP_W and my >= popup_y and my <= popup_y + popup_h:
				var option_idx := int((my - popup_y) / POPUP_OPTION_H)
				if option_idx >= 0 and option_idx < POPUP_OPTIONS.size():
					var rule_idx := _popup_rule
					_close_popup()
					if option_idx == 0:
						# Custom — open inline text editor
						_start_edit(rule_idx)
					else:
						# Variable threshold: 1=Zymase(0), 2=Motor(1), 3=Membrane(2)
						simulation.set_rule_threshold_variable(rule_idx, option_idx - 1)
					get_viewport().set_input_as_handled()
					return

		# Click outside popup — close it
		_close_popup()
		get_viewport().set_input_as_handled()
		return

	# Check if click is within panel
	if mx < _panel_x or mx > _panel_x + PANEL_W:
		return
	if my < _panel_y or my > _panel_y + _panel_h:
		return

	var rule_count: int = simulation.rule_count
	var locked: PackedInt32Array = simulation.rule_locked

	# Check each rule row
	for ri in range(rule_count):
		var y_base := _panel_y + HEADER_H + float(ri) * ROW_H
		if my < y_base or my > y_base + ROW_H:
			continue

		var is_locked: bool = ri < locked.size() and locked[ri] == 1
		if is_locked:
			get_viewport().set_input_as_handled()
			return

		var local_x := mx - _panel_x

		if _hit(local_x, COL_METRIC, COL_METRIC_W):
			simulation.cycle_rule_metric(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_SUBJECT, COL_SUBJECT_W):
			simulation.cycle_rule_subject(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_RELATION, COL_RELATION_W):
			simulation.cycle_rule_relation(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_THRESH, COL_THRESH_W):
			_open_popup(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_TARGET, COL_TARGET_W):
			simulation.cycle_rule_target(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_ENABLED, COL_ENABLED_W):
			simulation.toggle_rule_enabled(ri)
			get_viewport().set_input_as_handled()
			return
		elif _hit(local_x, COL_DELETE, COL_DELETE_W):
			simulation.remove_rule(ri)
			get_viewport().set_input_as_handled()
			return

	# Check add button
	var add_y := _panel_y + HEADER_H + float(rule_count) * ROW_H + 8.0
	if my >= add_y and my <= add_y + ROW_H and mx >= _panel_x + PAD and mx <= _panel_x + PAD + 130.0:
		if rule_count < 5:
			simulation.add_rule()
			get_viewport().set_input_as_handled()
			return

func _hit(local_x: float, col: float, col_w: float) -> bool:
	return local_x >= col and local_x <= col + col_w
