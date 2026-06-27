use crate::rules::{Rule, Threshold};
use crate::tech::Tech;
use crate::types::MRNA_COUNT;

pub struct RuleArrays {
    pub metrics: Vec<i32>,
    pub subjects: Vec<i32>,
    pub relations: Vec<i32>,
    pub thresholds: Vec<f32>,
    pub targets: Vec<i32>,
    pub values: Vec<f32>,
    pub enabled: Vec<i32>,
    pub firing: Vec<i32>,
    pub locked: Vec<i32>,
    pub threshold_modes: Vec<i32>,
    pub threshold_targets: Vec<i32>,
    pub threshold_values: Vec<f32>,
    pub mrna_suppressed: Vec<i32>,
}

pub struct TechArrays {
    pub names: Vec<String>,
    pub descriptions: Vec<String>,
    pub progress: Vec<f32>,
    pub completed: Vec<i32>,
}

pub fn build_rule_arrays(rules: &[Rule], suppressions: &[bool; MRNA_COUNT]) -> RuleArrays {
    let mut arrays = RuleArrays {
        metrics: Vec::with_capacity(rules.len()),
        subjects: Vec::with_capacity(rules.len()),
        relations: Vec::with_capacity(rules.len()),
        thresholds: Vec::with_capacity(rules.len()),
        targets: Vec::with_capacity(rules.len()),
        values: Vec::with_capacity(rules.len()),
        enabled: Vec::with_capacity(rules.len()),
        firing: Vec::with_capacity(rules.len()),
        locked: Vec::with_capacity(rules.len()),
        threshold_modes: Vec::with_capacity(rules.len()),
        threshold_targets: Vec::with_capacity(rules.len()),
        threshold_values: Vec::with_capacity(rules.len()),
        mrna_suppressed: Vec::with_capacity(MRNA_COUNT),
    };

    for rule in rules {
        arrays.metrics.push(rule.metric as i32);
        arrays.subjects.push(rule.subject.strand_index() as i32);
        arrays.relations.push(rule.relation as i32);
        match rule.threshold {
            Threshold::Fixed(v) => {
                arrays.thresholds.push(v);
                arrays.threshold_modes.push(0);
                arrays.threshold_targets.push(-1);
            }
            Threshold::Variable(ref_target) => {
                arrays.thresholds.push(rule.current_threshold_value);
                arrays.threshold_modes.push(1);
                arrays.threshold_targets.push(ref_target.strand_index() as i32);
            }
        }
        arrays.threshold_values.push(rule.current_threshold_value);
        arrays.targets.push(rule.target.strand_index() as i32);
        arrays.values.push(rule.current_value);
        arrays.enabled.push(if rule.enabled { 1 } else { 0 });
        arrays.firing.push(if rule.firing { 1 } else { 0 });
        arrays.locked.push(if rule.locked { 1 } else { 0 });
    }

    for i in 0..MRNA_COUNT {
        arrays.mrna_suppressed.push(if suppressions[i] { 1 } else { 0 });
    }

    arrays
}

pub fn build_tech_arrays(techs: &[Tech]) -> TechArrays {
    let mut arrays = TechArrays {
        names: Vec::with_capacity(techs.len()),
        descriptions: Vec::with_capacity(techs.len()),
        progress: Vec::with_capacity(techs.len()),
        completed: Vec::with_capacity(techs.len()),
    };

    for t in techs {
        arrays.names.push(t.name.clone());
        arrays.descriptions.push(t.description.clone());
        arrays.progress.push(t.progress);
        arrays.completed.push(if t.completed { 1 } else { 0 });
    }

    arrays
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{self, Metric, MrnaTarget, Relation, Threshold};
    use crate::tech;

    // ── RuleArrays length consistency ──

    #[test]
    fn rule_arrays_all_same_length() {
        let rules = rules::default_rules();
        let sup = [false; MRNA_COUNT];
        let a = build_rule_arrays(&rules, &sup);
        let n = rules.len();
        assert_eq!(a.metrics.len(), n);
        assert_eq!(a.subjects.len(), n);
        assert_eq!(a.relations.len(), n);
        assert_eq!(a.thresholds.len(), n);
        assert_eq!(a.targets.len(), n);
        assert_eq!(a.values.len(), n);
        assert_eq!(a.enabled.len(), n);
        assert_eq!(a.firing.len(), n);
        assert_eq!(a.locked.len(), n);
        assert_eq!(a.threshold_modes.len(), n);
        assert_eq!(a.threshold_targets.len(), n);
        assert_eq!(a.threshold_values.len(), n);
    }

    #[test]
    fn rule_arrays_empty_rules() {
        let rules: Vec<Rule> = vec![];
        let sup = [false; MRNA_COUNT];
        let a = build_rule_arrays(&rules, &sup);
        assert_eq!(a.metrics.len(), 0);
        assert_eq!(a.subjects.len(), 0);
        assert_eq!(a.mrna_suppressed.len(), MRNA_COUNT);
    }

    // ── Enum→int mapping ──

    #[test]
    fn metric_enum_to_int_mapping() {
        let mut rules = rules::default_rules();
        rules[0].metric = Metric::Count;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.metrics[0], Metric::Count as i32);

        rules[0].metric = Metric::Density;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.metrics[0], Metric::Density as i32);

        rules[0].metric = Metric::SurfaceDensity;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.metrics[0], Metric::SurfaceDensity as i32);
    }

    #[test]
    fn relation_enum_to_int_mapping() {
        let mut rules = rules::default_rules();
        rules[0].relation = Relation::GreaterEqual;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.relations[0], Relation::GreaterEqual as i32);

        rules[0].relation = Relation::LessEqual;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.relations[0], Relation::LessEqual as i32);
    }

    #[test]
    fn subject_and_target_use_strand_index() {
        let mut rules = rules::default_rules();
        rules[0].subject = MrnaTarget::Zymase;
        rules[0].target = MrnaTarget::Membrane;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.subjects[0], 0); // Zymase
        assert_eq!(a.targets[0], 2); // Membrane
    }

    // ── Fixed vs Variable threshold encoding ──

    #[test]
    fn fixed_threshold_encoding() {
        let rules = rules::default_rules(); // default has Fixed(0.027)
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.threshold_modes[0], 0);
        assert_eq!(a.threshold_targets[0], -1);
        assert!((a.thresholds[0] - 0.027).abs() < 0.001);
    }

    #[test]
    fn variable_threshold_encoding() {
        let rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Zymase,
            relation: Relation::GreaterEqual,
            threshold: Threshold::Variable(MrnaTarget::Motor),
            target: MrnaTarget::Zymase,
            enabled: true,
            firing: false,
            current_value: 3.0,
            current_threshold_value: 5.0,
            locked: false,
        }];
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.threshold_modes[0], 1);
        assert_eq!(a.threshold_targets[0], MrnaTarget::Motor.strand_index() as i32);
        assert_eq!(a.thresholds[0], 5.0); // uses current_threshold_value for variable
    }

    // ── Enabled / firing / locked flags ──

    #[test]
    fn enabled_flag_maps_correctly() {
        let mut rules = rules::default_rules();
        rules[0].enabled = true;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.enabled[0], 1);

        rules[0].enabled = false;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.enabled[0], 0);
    }

    #[test]
    fn firing_flag_maps_correctly() {
        let mut rules = rules::default_rules();
        rules[0].firing = true;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.firing[0], 1);

        rules[0].firing = false;
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.firing[0], 0);
    }

    #[test]
    fn locked_flag_maps_correctly() {
        let rules = rules::default_rules(); // default rule is locked
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.locked[0], 1);
    }

    // ── Suppressions array ──

    #[test]
    fn suppressions_array_matches_input() {
        let rules = rules::default_rules();
        let sup = [true, false, true];
        let a = build_rule_arrays(&rules, &sup);
        assert_eq!(a.mrna_suppressed, vec![1, 0, 1]);
    }

    #[test]
    fn suppressions_array_all_false() {
        let rules = rules::default_rules();
        let sup = [false; MRNA_COUNT];
        let a = build_rule_arrays(&rules, &sup);
        assert_eq!(a.mrna_suppressed, vec![0, 0, 0]);
    }

    // ── Multiple rules ──

    #[test]
    fn multiple_rules_all_arrays_grow() {
        let rules = vec![
            rules::default_rules().remove(0),
            Rule {
                metric: Metric::Count,
                subject: MrnaTarget::Zymase,
                relation: Relation::LessEqual,
                threshold: Threshold::Fixed(2.0),
                target: MrnaTarget::Zymase,
                enabled: false,
                firing: false,
                current_value: 0.0,
                current_threshold_value: 0.0,
                locked: false,
            },
        ];
        let a = build_rule_arrays(&rules, &[false; MRNA_COUNT]);
        assert_eq!(a.metrics.len(), 2);
        assert_eq!(a.enabled[0], 1);
        assert_eq!(a.enabled[1], 0);
        assert_eq!(a.locked[0], 1);
        assert_eq!(a.locked[1], 0);
    }

    // ── Pipeline test: default_rules → evaluate → build ──

    #[test]
    fn pipeline_default_rules_through_build() {
        let mut rules = rules::default_rules();
        let sup = rules::evaluate_suppressions(&mut rules, 4, 1, 1, 23.0);
        let a = build_rule_arrays(&rules, &sup);

        assert_eq!(a.metrics.len(), 1);
        assert_eq!(a.firing[0], 1); // rule should fire at this config
        assert!(a.values[0] > 0.0);
        assert!(a.threshold_values[0] > 0.0);
        assert_eq!(a.mrna_suppressed[1], 1); // motor strand suppressed
    }

    #[test]
    fn pipeline_non_firing_rule() {
        let mut rules = rules::default_rules();
        let sup = rules::evaluate_suppressions(&mut rules, 1, 1, 0, 30.0);
        let a = build_rule_arrays(&rules, &sup);

        assert_eq!(a.firing[0], 0);
        assert_eq!(a.mrna_suppressed, vec![0, 0, 0]);
    }

    // ── TechArrays ──

    #[test]
    fn tech_arrays_all_same_length() {
        let techs = tech::default_techs();
        let a = build_tech_arrays(&techs);
        let n = techs.len();
        assert_eq!(a.names.len(), n);
        assert_eq!(a.descriptions.len(), n);
        assert_eq!(a.progress.len(), n);
        assert_eq!(a.completed.len(), n);
    }

    #[test]
    fn tech_arrays_empty() {
        let techs: Vec<Tech> = vec![];
        let a = build_tech_arrays(&techs);
        assert_eq!(a.names.len(), 0);
        assert_eq!(a.completed.len(), 0);
    }

    #[test]
    fn tech_arrays_names_preserved() {
        let techs = tech::default_techs();
        let a = build_tech_arrays(&techs);
        assert_eq!(a.names[0], "Motor Efficiency");
        assert_eq!(a.names[1], "Membrane Reinforcement");
        assert_eq!(a.names[2], "ATP Synthesis II");
        assert_eq!(a.names[3], "Flagellar Coordination");
    }

    #[test]
    fn tech_arrays_completed_flag() {
        let mut techs = tech::default_techs();
        techs[0].completed = true;
        techs[0].progress = 1.0;
        let a = build_tech_arrays(&techs);
        assert_eq!(a.completed[0], 1);
        assert_eq!(a.completed[1], 0);
        assert_eq!(a.completed[2], 0);
        assert_eq!(a.completed[3], 0);
        assert_eq!(a.progress[0], 1.0);
    }

    #[test]
    fn tech_arrays_progress_values() {
        let mut techs = tech::default_techs();
        techs[1].progress = 0.5;
        let a = build_tech_arrays(&techs);
        assert_eq!(a.progress[0], 0.0);
        assert_eq!(a.progress[1], 0.5);
    }

    // ── Pipeline test: default_techs with ticked rules ──

    #[test]
    fn pipeline_techs_after_completion() {
        let mut techs = tech::default_techs();
        let mut rules = rules::default_rules();

        // Fire the rule so tech completes
        rules::evaluate_suppressions(&mut rules, 4, 1, 1, 23.0);
        tech::tick_techs(&mut techs, &mut rules);

        let a = build_tech_arrays(&techs);
        assert_eq!(a.completed[0], 1);
        assert_eq!(a.progress[0], 1.0);
        // Other techs remain incomplete
        assert_eq!(a.completed[1], 0);
        assert_eq!(a.completed[2], 0);
        assert_eq!(a.completed[3], 0);
    }
}
