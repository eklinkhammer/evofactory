extends Camera2D

var zoom_speed := 0.1
var min_zoom := 0.2
var max_zoom := 3.0
var in_interior := false
var saved_zoom := Vector2(0.8, 0.8)

@onready var simulation: Node = get_node("../Simulation")
@onready var world_renderer: Node2D = get_node("../WorldRenderer")

func _ready() -> void:
	zoom = Vector2(0.8, 0.8)
	position = Vector2.ZERO

func enter_interior() -> void:
	saved_zoom = zoom
	zoom = Vector2(20.0, 20.0)
	in_interior = true

func exit_interior() -> void:
	zoom = saved_zoom
	in_interior = false

func _unhandled_input(event: InputEvent) -> void:
	if in_interior:
		return
	if event is InputEventMouseButton:
		var mouse_event := event as InputEventMouseButton
		if mouse_event.pressed:
			if mouse_event.button_index == MOUSE_BUTTON_WHEEL_UP:
				zoom *= 1.0 + zoom_speed
			elif mouse_event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				zoom *= 1.0 - zoom_speed
			zoom = zoom.clamp(Vector2(min_zoom, min_zoom), Vector2(max_zoom, max_zoom))

func _process(_delta: float) -> void:
	# Camera positioning is handled by Main.gd _process()
	pass
