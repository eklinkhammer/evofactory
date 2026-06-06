extends CanvasLayer

var label: Label

@onready var simulation: Node = get_node("/root/Main/Simulation")

func _ready() -> void:
	label = Label.new()
	label.position = Vector2(16, 16)
	label.add_theme_font_size_override("font_size", 18)
	add_child(label)

func _process(_delta: float) -> void:
	if simulation:
		if not simulation.player_alive:
			label.text = "GAME OVER - Press R to restart"
			label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
		else:
			label.text = "ATP: %d/%d | Glucose: %d | Amino Acids: %d" % [
				int(simulation.player_atp),
				int(simulation.player_max_atp),
				int(simulation.player_glucose),
				int(simulation.player_amino_acids),
			]
			if simulation.player_energy_ratio < 0.3:
				label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
			else:
				label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))
