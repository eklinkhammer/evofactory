extends Node2D

var simulation: Node

const PANEL_W := 400.0
const PANEL_H := 280.0
const ROW_H := 48.0
const HEADER_H := 54.0
const PAD := 16.0
const FONT_SIZE := 18
const TITLE_SIZE := 28
const SMALL_SIZE := 14
const BTN_W := 360.0

const SEEK_NAMES := ["Nearest", "Glucose", "Amino Acid"]

var _panel_x: float = 0.0
var _panel_y: float = 0.0

func _draw() -> void:
	if not simulation or not simulation.autonomy_panel_open:
		return

	var font := ThemeDB.fallback_font
	var viewport_size := get_viewport_rect().size
	_panel_x = (viewport_size.x - PANEL_W) / 2.0
	_panel_y = (viewport_size.y - PANEL_H) / 2.0

	# Background
	draw_rect(Rect2(_panel_x, _panel_y, PANEL_W, PANEL_H), Color(0.0, 0.0, 0.0, 0.9))
	draw_rect(Rect2(_panel_x, _panel_y, PANEL_W, PANEL_H), Color(0.4, 0.4, 0.4, 0.6), false, 2.0)

	# Title
	draw_string(font, Vector2(_panel_x + PAD, _panel_y + 38), "Autonomous Movement", HORIZONTAL_ALIGNMENT_LEFT, -1, TITLE_SIZE, Color.WHITE)

	# Row 1: Mode toggle
	var row1_y := _panel_y + HEADER_H
	var is_auto: bool = simulation.autonomous
	var mode_label: String = "Mode: AUTONOMOUS" if is_auto else "Mode: MANUAL"
	var mode_color: Color = Color(0.3, 0.9, 0.3) if is_auto else Color(0.5, 0.5, 0.5)
	_draw_button(font, _panel_x + PAD, row1_y, BTN_W, mode_label, mode_color)

	# Row 2: Seek target
	var row2_y := row1_y + ROW_H
	var seek_idx: int = simulation.auto_seek_target
	var seek_name: String = SEEK_NAMES[seek_idx] if (seek_idx >= 0 and seek_idx < SEEK_NAMES.size()) else "Nearest"
	var seek_color := Color(0.85, 0.85, 0.85) if is_auto else Color(0.4, 0.4, 0.4)
	_draw_button(font, _panel_x + PAD, row2_y, BTN_W, "Seek: " + seek_name, seek_color)

	# Row 3: Sensor info
	var row3_y := row2_y + ROW_H
	var sensor_count: int = simulation.auto_sensor_count
	var sensor_label: String
	if sensor_count > 0:
		var sensor_range: float = simulation.auto_sensor_range
		sensor_label = "Sensors: %d  (range: %.0f)" % [sensor_count, sensor_range]
	else:
		sensor_label = "Sensors: None (random walk)"
	draw_string(font, Vector2(_panel_x + PAD + 6, row3_y + 32), sensor_label, HORIZONTAL_ALIGNMENT_LEFT, int(BTN_W), FONT_SIZE, Color(0.6, 0.6, 0.6))

	# Row 4: Movement status
	var row4_y := row3_y + ROW_H
	var movement_label: String
	if not is_auto:
		movement_label = "Status: Manual control"
	else:
		var mdx: float = simulation.auto_movement_dx
		var mdy: float = simulation.auto_movement_dy
		var dir_str := _direction_label(mdx, mdy)
		if sensor_count > 0:
			movement_label = "Moving: " + dir_str + " (chemotaxis)"
		else:
			movement_label = "Moving: " + dir_str + " (random)"
	draw_string(font, Vector2(_panel_x + PAD + 6, row4_y + 32), movement_label, HORIZONTAL_ALIGNMENT_LEFT, int(BTN_W), FONT_SIZE, Color(0.6, 0.6, 0.6))

func _draw_button(font: Font, x: float, y: float, w: float, text: String, color: Color) -> void:
	var rect := Rect2(x, y + 4, w, ROW_H - 8)
	draw_rect(rect, Color(0.2, 0.2, 0.2, 0.6))
	draw_rect(rect, Color(0.5, 0.5, 0.5, 0.4), false, 1.0)
	draw_string(font, Vector2(x + 6, y + 32), text, HORIZONTAL_ALIGNMENT_LEFT, int(w - 8), FONT_SIZE, color)

func _direction_label(dx: float, dy: float) -> String:
	if abs(dx) < 0.01 and abs(dy) < 0.01:
		return "Idle"
	var angle := atan2(dy, dx)
	# 8 compass directions
	if angle < -2.748 or angle > 2.748:
		return "W"
	elif angle < -1.963:
		return "NW"
	elif angle < -1.178:
		return "N"
	elif angle < -0.393:
		return "NE"
	elif angle < 0.393:
		return "E"
	elif angle < 1.178:
		return "SE"
	elif angle < 1.963:
		return "S"
	else:
		return "SW"

func _input(event: InputEvent) -> void:
	if not simulation or not simulation.autonomy_panel_open:
		return

	if not (event is InputEventMouseButton):
		return
	var mb := event as InputEventMouseButton
	if mb.button_index != MOUSE_BUTTON_LEFT or not mb.pressed:
		return

	var mx := mb.position.x
	var my := mb.position.y

	# Check if click is within panel
	if mx < _panel_x or mx > _panel_x + PANEL_W:
		return
	if my < _panel_y or my > _panel_y + PANEL_H:
		return

	# Row 1: Mode toggle
	var row1_y := _panel_y + HEADER_H
	if my >= row1_y and my <= row1_y + ROW_H:
		simulation.toggle_autonomous_mode()
		get_viewport().set_input_as_handled()
		return

	# Row 2: Seek target cycle
	var row2_y := row1_y + ROW_H
	if my >= row2_y and my <= row2_y + ROW_H:
		simulation.cycle_seek_target()
		get_viewport().set_input_as_handled()
		return

	# Consume other clicks within panel
	get_viewport().set_input_as_handled()
