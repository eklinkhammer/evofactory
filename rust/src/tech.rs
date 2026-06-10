use crate::rules::Rule;

#[derive(Clone, Debug)]
pub enum TechTrigger {
    RuleFiring { rule_index: usize },
}

#[derive(Clone, Debug)]
pub struct Tech {
    pub name: String,
    pub description: String,
    pub progress: f32,
    pub completed: bool,
    pub trigger: TechTrigger,
}

pub fn default_techs() -> Vec<Tech> {
    vec![Tech {
        name: "Motor Efficiency".into(),
        description: "Reached motor production cap".into(),
        progress: 0.0,
        completed: false,
        trigger: TechTrigger::RuleFiring { rule_index: 0 },
    }]
}

pub fn tick_techs(
    techs: &mut [Tech],
    rules: &[Rule],
    motor_count: usize,
    zymase_count: usize,
) {
    for tech in techs.iter_mut() {
        if tech.completed {
            continue;
        }
        match tech.trigger {
            TechTrigger::RuleFiring { rule_index } => {
                if rule_index >= rules.len() {
                    continue;
                }
                let rule = &rules[rule_index];
                if !rule.firing {
                    continue;
                }
                let current_count = match rule.target {
                    crate::rules::MrnaTarget::Motor => motor_count,
                    crate::rules::MrnaTarget::Zymase => zymase_count,
                    crate::rules::MrnaTarget::Membrane => 0,
                };
                let limit = rule.current_limit.max(1);
                tech.progress = (current_count as f32 / limit as f32).clamp(0.0, 1.0);
                if tech.progress >= 1.0 {
                    tech.completed = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules;

    #[test]
    fn tech_progresses_with_motor_count() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();
        // Fire the rule: motors at cap
        rules::evaluate_suppressions(&mut rule_set, 2, 1, 0);
        // Rule not firing yet (2 < 3)
        tick_techs(&mut techs, &rule_set, 2, 1);
        assert_eq!(techs[0].progress, 0.0);
        assert!(!techs[0].completed);

        // Now at cap: motors = 3, limit = 3
        rules::evaluate_suppressions(&mut rule_set, 3, 1, 0);
        tick_techs(&mut techs, &rule_set, 3, 1);
        assert!((techs[0].progress - 1.0).abs() < f32::EPSILON);
        assert!(techs[0].completed);
    }

    #[test]
    fn tech_stays_completed() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();

        // Complete it
        rules::evaluate_suppressions(&mut rule_set, 3, 1, 0);
        tick_techs(&mut techs, &rule_set, 3, 1);
        assert!(techs[0].completed);

        // Drop below cap — stays completed
        rules::evaluate_suppressions(&mut rule_set, 1, 1, 0);
        tick_techs(&mut techs, &rule_set, 1, 1);
        assert!(techs[0].completed);
        assert!((techs[0].progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tech_no_progress_when_rule_not_firing() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();
        rules::evaluate_suppressions(&mut rule_set, 1, 1, 0);
        tick_techs(&mut techs, &rule_set, 1, 1);
        assert_eq!(techs[0].progress, 0.0);
        assert!(!techs[0].completed);
    }
}
