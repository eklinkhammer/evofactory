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
		label.text = "Glucose: %d | Amino Acids: %d" % [
			int(simulation.player_glucose),
			int(simulation.player_amino_acids),
		]
