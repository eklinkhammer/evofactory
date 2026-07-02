extends CanvasLayer

var label: Label
var reg_panel: Node2D
var tech_panel: Node2D
var autonomy_panel: Node2D

# A1: Hotkey hint labels
var hint_labels: Array[Label] = []
var hint_used := { "tab": false, "g": false, "t": false, "a": false }
var hint_alpha := { "tab": 1.0, "g": 1.0, "t": 1.0, "a": 1.0 }

# A4: Resource legend label
var legend_label: RichTextLabel

@onready var simulation: Node = get_node("/root/Main/Simulation")

func _ready() -> void:
	label = Label.new()
	label.position = Vector2(16, 16)
	label.add_theme_font_size_override("font_size", 28)
	add_child(label)
	var panel_script := load("res://scripts/RegulationPanel.gd")
	reg_panel = Node2D.new()
	reg_panel.set_script(panel_script)
	reg_panel.simulation = simulation
	add_child(reg_panel)

	var tech_script := load("res://scripts/TechPanel.gd")
	tech_panel = Node2D.new()
	tech_panel.set_script(tech_script)
	tech_panel.simulation = simulation
	add_child(tech_panel)

	var autonomy_script := load("res://scripts/AutonomyPanel.gd")
	autonomy_panel = Node2D.new()
	autonomy_panel.set_script(autonomy_script)
	autonomy_panel.simulation = simulation
	add_child(autonomy_panel)

	# A1: Create hotkey hint labels at bottom of screen
	var hint_texts := ["[TAB] Interior", "[G] Genes", "[T] Tech", "[A] Auto"]
	for i in range(hint_texts.size()):
		var h := Label.new()
		h.text = hint_texts[i]
		h.add_theme_font_size_override("font_size", 16)
		h.add_theme_color_override("font_color", Color(0.8, 0.8, 0.8, 1.0))
		add_child(h)
		hint_labels.append(h)

	# A4: Resource legend in bottom-right
	legend_label = RichTextLabel.new()
	legend_label.bbcode_enabled = true
	legend_label.fit_content = true
	legend_label.scroll_active = false
	legend_label.size = Vector2(260, 30)
	legend_label.bbcode_text = "[color=#f2bf33]●[/color] Glucose   [color=#804dd9]●[/color] Amino Acid   [color=#e64d66]●[/color] Nucleotide"
	add_child(legend_label)

func _input(event: InputEvent) -> void:
	# A1: Track hotkey usage for fade
	if event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_TAB: hint_used["tab"] = true
			KEY_G: hint_used["g"] = true
			KEY_T: hint_used["t"] = true
			KEY_A: hint_used["a"] = true

func _process(delta: float) -> void:
	# Update hint/legend positions each frame so they adapt to window resizes
	var vp_size := get_viewport().get_visible_rect().size
	var total_width := hint_labels.size() * 140.0
	var start_x := (vp_size.x - total_width) * 0.5
	for i in range(hint_labels.size()):
		hint_labels[i].position = Vector2(start_x + i * 140.0, vp_size.y - 36)
	legend_label.position = Vector2(vp_size.x - 260, vp_size.y - 60)

	if simulation:
		reg_panel.simulation = simulation
		if simulation.regulation_panel_open:
			reg_panel.visible = true
			reg_panel.queue_redraw()
		else:
			reg_panel.visible = false
		tech_panel.simulation = simulation
		if simulation.tech_panel_open:
			tech_panel.visible = true
			tech_panel.queue_redraw()
		else:
			tech_panel.visible = false
		autonomy_panel.simulation = simulation
		if simulation.autonomy_panel_open:
			autonomy_panel.visible = true
			autonomy_panel.queue_redraw()
		else:
			autonomy_panel.visible = false
		if not simulation.player_alive:
			# A5: Death explanation
			var death_reason := "Your cell ran out of energy. Motors, ATP, and glucose were all depleted."
			label.text = "GAME OVER\n%s\nPress R to restart" % death_reason
			label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
		else:
			if simulation.interior_view:
				label.text = "Motor: %d/%d | ATP: %d | Glucose: %d | Amino: %d | Nucl: %d\nParticles diffuse to organelles. Drag to place organelles." % [
					int(simulation.motor_charge_display),
					int(simulation.player_max_atp),
					simulation.atp_particle_count,
					int(simulation.player_glucose),
					simulation.amino_acid_particle_count,
					simulation.nucleotide_particle_count,
				]
			else:
				label.text = "Motor: %d/%d | Free ATP: %d | Glucose: %d" % [
					int(simulation.motor_charge_display),
					int(simulation.player_max_atp),
					simulation.atp_particle_count,
					int(simulation.player_glucose),
				]
			if simulation.autonomous:
				label.text += " | AUTO"
			if simulation.player_energy_ratio < 0.3:
				label.add_theme_color_override("font_color", Color(1.0, 0.3, 0.3))
			else:
				label.add_theme_color_override("font_color", Color(1.0, 1.0, 1.0))

		# A4: Resource legend — only show in exterior view when alive
		legend_label.visible = simulation.player_alive and not simulation.interior_view

	# A1: Fade out used hotkey hints
	var hint_keys := ["tab", "g", "t", "a"]
	for i in range(hint_labels.size()):
		var key: String = hint_keys[i]
		if hint_used[key]:
			hint_alpha[key] = maxf(hint_alpha[key] - delta * 0.5, 0.0)
		hint_labels[i].add_theme_color_override("font_color", Color(0.8, 0.8, 0.8, hint_alpha[key]))
		hint_labels[i].visible = hint_alpha[key] > 0.01
