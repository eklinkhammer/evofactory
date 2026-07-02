extends SceneTree

## Headless integration test runner for Simulation API.
## Run: godot --headless --script res://scripts/test_runner.gd --path .

var sim: Node
var pass_count := 0
var fail_count := 0
var current_group := ""

func _initialize():
	sim = ClassDB.instantiate(&"Simulation")
	root.add_child(sim)

	test_initial_state()
	test_tech_panel_contract()
	test_rule_panel_contract()
	test_rule_mutations()
	test_max_rules_limit()
	test_restart_sync()
	test_bounds_safety()

	print("")
	if fail_count == 0:
		print("ALL %d ASSERTIONS PASSED" % pass_count)
	else:
		print("FAILED: %d / %d assertions failed" % [fail_count, pass_count + fail_count])

	sim.queue_free()
	quit(1 if fail_count > 0 else 0)

# ── Helpers ──

func tick():
	sim.tick(0.016)

func assert_eq(a, b, msg := ""):
	if a == b:
		pass_count += 1
	else:
		fail_count += 1
		print("  FAIL [%s] %s: got %s, expected %s" % [current_group, msg, str(a), str(b)])

func assert_true(v: bool, msg := ""):
	assert_eq(v, true, msg)

func assert_gt(a, b, msg := ""):
	if a > b:
		pass_count += 1
	else:
		fail_count += 1
		print("  FAIL [%s] %s: %s not > %s" % [current_group, msg, str(a), str(b)])

func group(name: String):
	current_group = name
	print("  %s..." % name)

# ── Test groups ──

func test_initial_state():
	group("Initial state")
	tick()
	assert_eq(sim.tech_count, 6, "tech_count == 6")
	assert_eq(sim.rule_count, 1, "rule_count == 1")
	assert_gt(sim.tech_names.size(), 0, "tech_names populated")
	assert_gt(sim.rule_metrics.size(), 0, "rule_metrics populated")

func test_tech_panel_contract():
	group("Tech panel contract")
	tick()
	assert_eq(sim.tech_names.size(), sim.tech_count, "names.size == tech_count")
	assert_eq(sim.tech_descriptions.size(), sim.tech_count, "descriptions.size == tech_count")
	assert_eq(sim.tech_progress.size(), sim.tech_count, "progress.size == tech_count")
	assert_eq(sim.tech_completed.size(), sim.tech_count, "completed.size == tech_count")
	assert_eq(sim.tech_names[0], "Motor Efficiency", "first tech name")

	# Toggle
	assert_eq(sim.tech_panel_open, false, "panel starts closed")
	sim.toggle_tech_panel()
	assert_eq(sim.tech_panel_open, true, "panel opens after toggle")
	sim.toggle_tech_panel()
	assert_eq(sim.tech_panel_open, false, "panel closes after second toggle")

	# Select
	sim.select_tech(2)
	assert_eq(sim.tech_selected, 2, "select_tech(2)")
	sim.select_tech(0)
	assert_eq(sim.tech_selected, 0, "select_tech(0)")

func test_rule_panel_contract():
	group("Rule panel contract")
	tick()
	var n = sim.rule_count
	assert_eq(sim.rule_metrics.size(), n, "metrics.size == rule_count")
	assert_eq(sim.rule_subjects.size(), n, "subjects.size == rule_count")
	assert_eq(sim.rule_relations.size(), n, "relations.size == rule_count")
	assert_eq(sim.rule_thresholds.size(), n, "thresholds.size == rule_count")
	assert_eq(sim.rule_targets.size(), n, "targets.size == rule_count")
	assert_eq(sim.rule_values.size(), n, "values.size == rule_count")
	assert_eq(sim.rule_enabled.size(), n, "enabled.size == rule_count")
	assert_eq(sim.rule_firing.size(), n, "firing.size == rule_count")
	assert_eq(sim.rule_locked.size(), n, "locked.size == rule_count")
	assert_eq(sim.rule_threshold_modes.size(), n, "threshold_modes.size == rule_count")
	assert_eq(sim.rule_threshold_targets.size(), n, "threshold_targets.size == rule_count")
	assert_eq(sim.rule_threshold_values.size(), n, "threshold_values.size == rule_count")

	# Default rule is locked
	assert_eq(sim.rule_locked[0], 1, "default rule is locked")

	# mrna_suppressed array has correct size
	assert_eq(sim.mrna_suppressed.size(), 3, "mrna_suppressed.size == 3")

	# Regulation panel toggle
	assert_eq(sim.regulation_panel_open, false, "regulation panel starts closed")
	sim.toggle_regulation_panel()
	assert_eq(sim.regulation_panel_open, true, "regulation panel opens after toggle")
	sim.toggle_regulation_panel()
	assert_eq(sim.regulation_panel_open, false, "regulation panel closes after second toggle")

func test_rule_mutations():
	group("Rule mutations")
	# Reset
	sim.restart()
	tick()
	var base_count = sim.rule_count

	# add_rule → arrays grow
	sim.add_rule()
	tick()
	assert_eq(sim.rule_count, base_count + 1, "add_rule increases count")
	assert_eq(sim.rule_metrics.size(), sim.rule_count, "metrics grows with add_rule")

	# cycle_rule_metric → value changes
	var old_metric = sim.rule_metrics[1]
	sim.cycle_rule_metric(1)
	tick()
	assert_true(sim.rule_metrics[1] != old_metric, "cycle_rule_metric changes value")

	# cycle_rule_subject → value changes
	var old_subject = sim.rule_subjects[1]
	sim.cycle_rule_subject(1)
	tick()
	assert_true(sim.rule_subjects[1] != old_subject, "cycle_rule_subject changes value")

	# cycle_rule_relation → value changes
	var old_relation = sim.rule_relations[1]
	sim.cycle_rule_relation(1)
	tick()
	assert_true(sim.rule_relations[1] != old_relation, "cycle_rule_relation changes value")

	# cycle_rule_target → value changes
	var old_target = sim.rule_targets[1]
	sim.cycle_rule_target(1)
	tick()
	assert_true(sim.rule_targets[1] != old_target, "cycle_rule_target changes value")

	# set_rule_threshold → value correct
	sim.set_rule_threshold(1, 42.0)
	tick()
	assert_eq(sim.rule_thresholds[1], 42.0, "set_rule_threshold sets value")

	# set_rule_threshold_variable → mode and target change
	sim.set_rule_threshold_variable(1, 1) # target index 1 = Motor
	tick()
	assert_eq(sim.rule_threshold_modes[1], 1, "variable threshold mode is 1")
	assert_eq(sim.rule_threshold_targets[1], 1, "variable threshold target is Motor")

	# set_rule_threshold_fixed → mode reverts
	sim.set_rule_threshold_fixed(1)
	tick()
	assert_eq(sim.rule_threshold_modes[1], 0, "fixed threshold mode is 0")

	# locked rule guard: cycling metric on locked rule[0] is a no-op
	var locked_metric = sim.rule_metrics[0]
	sim.cycle_rule_metric(0)
	tick()
	assert_eq(sim.rule_metrics[0], locked_metric, "cycle_rule_metric is no-op on locked rule")

	# toggle_rule_enabled → flag flips
	var old_enabled = sim.rule_enabled[1]
	sim.toggle_rule_enabled(1)
	tick()
	assert_true(sim.rule_enabled[1] != old_enabled, "toggle_rule_enabled flips flag")

	# remove_rule → arrays shrink
	var pre_remove = sim.rule_count
	sim.remove_rule(1)
	tick()
	assert_eq(sim.rule_count, pre_remove - 1, "remove_rule decreases count")
	assert_eq(sim.rule_metrics.size(), sim.rule_count, "metrics shrinks with remove_rule")

func test_max_rules_limit():
	group("Max rules limit")
	sim.restart()
	tick()
	# Add rules up to and beyond the limit (MAX_RULES = 5)
	for i in range(10):
		sim.add_rule()
	tick()
	assert_true(sim.rule_count <= 5, "rule_count capped at 5")

func test_restart_sync():
	group("Restart sync")
	# Mutate state
	sim.add_rule()
	sim.add_rule()
	sim.toggle_tech_panel()
	tick()

	# Restart and tick
	sim.restart()
	tick()
	assert_eq(sim.rule_count, 1, "restart resets rule_count to 1")
	assert_eq(sim.tech_count, 6, "restart keeps tech_count at 6")
	assert_eq(sim.tech_panel_open, false, "restart closes tech panel")
	assert_eq(sim.regulation_panel_open, false, "restart closes regulation panel")
	assert_eq(sim.rule_locked[0], 1, "restart re-locks default rule")

func test_bounds_safety():
	group("Bounds safety")
	# Out-of-bounds indices should not crash
	sim.select_tech(-1)
	sim.select_tech(999)
	sim.cycle_rule_metric(-1)
	sim.cycle_rule_metric(999)
	sim.cycle_rule_subject(999)
	sim.cycle_rule_relation(999)
	sim.cycle_rule_target(999)
	sim.set_rule_threshold(999, 1.0)
	sim.toggle_rule_enabled(999)
	sim.remove_rule(-1)
	sim.remove_rule(999)
	tick()
	assert_true(true, "no crash from out-of-bounds calls")
