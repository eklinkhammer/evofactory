use crate::rules::{Relation, Rule};

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

pub fn tick_techs(techs: &mut [Tech], rules: &mut [Rule]) {
    for tech in techs.iter_mut() {
        if tech.completed {
            continue;
        }
        match tech.trigger {
            TechTrigger::RuleFiring { rule_index } => {
                if rule_index >= rules.len() {
                    continue;
                }
                if rules[rule_index].firing {
                    tech.progress = 1.0;
                    tech.completed = true;
                    // Unlock the rule now that the tech is researched
                    rules[rule_index].locked = false;
                } else if rules[rule_index].threshold > 0.0 {
                    // Show gradual progress toward the threshold
                    let rule = &rules[rule_index];
                    tech.progress = match rule.relation {
                        Relation::GreaterEqual => {
                            (rule.current_value / rule.threshold).clamp(0.0, 0.99)
                        }
                        Relation::LessEqual => {
                            if rule.current_value > 0.0 {
                                (rule.threshold / rule.current_value).clamp(0.0, 0.99)
                            } else {
                                0.99 // value is 0, which is <= any positive threshold
                            }
                        }
                    };
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
    fn tech_shows_gradual_progress() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();

        // 1 motor at r=15: surfdens = 1/(2*pi*15) ≈ 0.0106, threshold = 0.031
        rules::evaluate_suppressions(&mut rule_set, 1, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert!(techs[0].progress > 0.0);
        assert!(techs[0].progress < 1.0);
        assert!(!techs[0].completed);

        // 2 motors: surfdens ≈ 0.0212, progress should be higher
        let prev_progress = techs[0].progress;
        rules::evaluate_suppressions(&mut rule_set, 2, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert!(techs[0].progress > prev_progress);
        assert!(!techs[0].completed);
    }

    #[test]
    fn tech_completes_when_rule_fires() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();
        assert!(rule_set[0].locked);

        // Now at threshold: 3 motors at r=15
        rules::evaluate_suppressions(&mut rule_set, 3, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert!((techs[0].progress - 1.0).abs() < f32::EPSILON);
        assert!(techs[0].completed);
        // Rule should be unlocked
        assert!(!rule_set[0].locked);
    }

    #[test]
    fn tech_stays_completed() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();

        // Complete it
        rules::evaluate_suppressions(&mut rule_set, 3, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert!(techs[0].completed);

        // Drop below — stays completed
        rules::evaluate_suppressions(&mut rule_set, 1, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert!(techs[0].completed);
        assert!((techs[0].progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn tech_no_progress_when_rule_not_firing() {
        let mut techs = default_techs();
        let mut rule_set = rules::default_rules();
        // With 0 motors, surface density is 0 → progress should be 0
        rules::evaluate_suppressions(&mut rule_set, 0, 1, 0, 15.0);
        tick_techs(&mut techs, &mut rule_set);
        assert_eq!(techs[0].progress, 0.0);
        assert!(!techs[0].completed);
    }
}
