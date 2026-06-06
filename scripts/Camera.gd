extends Camera2D

var zoom_speed := 0.1
var min_zoom := 0.2
var max_zoom := 3.0
var pan_speed := 500.0
var is_panning := false

func _ready() -> void:
	zoom = Vector2(0.8, 0.8)

func _unhandled_input(event: InputEvent) -> void:
	# Zoom with scroll wheel
	if event is InputEventMouseButton:
		var mouse_event := event as InputEventMouseButton
		if mouse_event.pressed:
			if mouse_event.button_index == MOUSE_BUTTON_WHEEL_UP:
				zoom *= 1.0 + zoom_speed
			elif mouse_event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
				zoom *= 1.0 - zoom_speed
			zoom = zoom.clamp(Vector2(min_zoom, min_zoom), Vector2(max_zoom, max_zoom))

		# Pan with middle mouse button
		if mouse_event.button_index == MOUSE_BUTTON_MIDDLE:
			is_panning = mouse_event.pressed

	# Middle mouse drag to pan
	if event is InputEventMouseMotion and is_panning:
		var motion := event as InputEventMouseMotion
		position -= motion.relative / zoom

func _process(delta: float) -> void:
	# Pan with shift + arrow keys
	if Input.is_key_pressed(KEY_SHIFT):
		var pan_dir := Vector2.ZERO
		if Input.is_action_pressed("ui_right"):
			pan_dir.x += 1.0
		if Input.is_action_pressed("ui_left"):
			pan_dir.x -= 1.0
		if Input.is_action_pressed("ui_down"):
			pan_dir.y += 1.0
		if Input.is_action_pressed("ui_up"):
			pan_dir.y -= 1.0
		position += pan_dir * pan_speed * delta / zoom.x
