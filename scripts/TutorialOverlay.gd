extends CanvasLayer

## Lightweight sequential tooltip tutorial triggered on first play.
## Steps advance based on player actions; dismiss on click or action completion.

enum Step { PRESS_TAB, GLUCOSE_EXPLAIN, AMINO_EXPLAIN, OPEN_PANELS, DONE }

var current_step: int = Step.PRESS_TAB
var step_timer: float = 0.0
var visible_flag: bool = true

var tooltip_box: Panel
var tooltip_label: Label

@onready var simulation: Node = get_node("/root/Main/Simulation")

func _ready() -> void:
	layer = 100
	tooltip_box = Panel.new()
	var style := StyleBoxFlat.new()
	style.bg_color = Color(0.05, 0.05, 0.1, 0.9)
	style.border_color = Color(0.4, 0.8, 0.4, 0.8)
	style.set_border_width_all(2)
	style.set_corner_radius_all(6)
	style.set_content_margin_all(12)
	tooltip_box.add_theme_stylebox_override("panel", style)
	add_child(tooltip_box)

	tooltip_label = Label.new()
	tooltip_label.add_theme_font_size_override("font_size", 18)
	tooltip_label.add_theme_color_override("font_color", Color(0.9, 0.95, 0.9))
	tooltip_label.position = Vector2(12, 8)
	tooltip_box.add_child(tooltip_label)

func _unhandled_input(event: InputEvent) -> void:
	if current_step == Step.DONE:
		return
	# Dismiss on click
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if tooltip_box.visible:
			_advance_step()
			get_viewport().set_input_as_handled()

func _process(delta: float) -> void:
	if current_step == Step.DONE:
		tooltip_box.visible = false
		return
	if not simulation:
		return

	# Check advancement conditions
	match current_step:
		Step.PRESS_TAB:
			if simulation.interior_view:
				_advance_step()
		Step.GLUCOSE_EXPLAIN:
			step_timer += delta
			if step_timer > 3.0:
				_advance_step()
		Step.AMINO_EXPLAIN:
			# Advance after first organelle spawn (zymase count > 1)
			if simulation.zymase_count > 1 or simulation.motor_count > 1:
				_advance_step()
			step_timer += delta
			if step_timer > 6.0:
				_advance_step()
		Step.OPEN_PANELS:
			if simulation.regulation_panel_open or simulation.tech_panel_open:
				_advance_step()
			step_timer += delta
			if step_timer > 8.0:
				_advance_step()

	_update_tooltip()

func _advance_step() -> void:
	current_step += 1
	step_timer = 0.0

func _update_tooltip() -> void:
	if current_step >= Step.DONE:
		tooltip_box.visible = false
		return

	var text: String
	var pos: Vector2
	var vp_size := get_viewport().get_visible_rect().size

	match current_step:
		Step.PRESS_TAB:
			text = "Press TAB to look inside your cell"
			pos = Vector2(vp_size.x * 0.5 - 180, vp_size.y * 0.4)
		Step.GLUCOSE_EXPLAIN:
			text = "Yellow dots are glucose. They drift to your\nZymase and become ATP (cyan)."
			pos = Vector2(vp_size.x * 0.5 - 200, vp_size.y * 0.3)
		Step.AMINO_EXPLAIN:
			text = "Purple dots are amino acids. They flow to\nmRNA strands to build organelles."
			pos = Vector2(vp_size.x * 0.5 - 200, vp_size.y * 0.3)
		Step.OPEN_PANELS:
			text = "Press G for gene regulation rules.\nPress T for the tech tree."
			pos = Vector2(vp_size.x * 0.5 - 170, vp_size.y * 0.4)

	tooltip_label.text = text
	var text_size := tooltip_label.get_minimum_size()
	tooltip_box.size = text_size + Vector2(24, 16)
	tooltip_box.position = pos
	tooltip_box.visible = true
