use crate::types::MRNA_COUNT;

pub const BASE_MAX_MOTORS: usize = 3;
pub const MOTOR_SCALE: f32 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
pub enum MrnaTarget {
    Zymase,
    Motor,
    Membrane,
}

impl MrnaTarget {
    pub fn strand_index(self) -> usize {
        match self {
            MrnaTarget::Zymase => 0,
            MrnaTarget::Motor => 1,
            MrnaTarget::Membrane => 2,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            MrnaTarget::Zymase => "Zymase",
            MrnaTarget::Motor => "Motor",
            MrnaTarget::Membrane => "Membrane",
        }
    }
}

#[derive(Clone, Debug)]
pub enum RuleCondition {
    CountAtScaledCap { base: usize, scale: f32 },
}

#[derive(Clone, Debug)]
pub enum RuleAction {
    SuppressMrna,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub target: MrnaTarget,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub enabled: bool,
    pub firing: bool,
    pub current_limit: usize,
}

impl Rule {
    pub fn description(&self) -> String {
        let cond = match &self.condition {
            RuleCondition::CountAtScaledCap { base, scale } => {
                format!(
                    "{} >= {} + {} * sqrt(cell size)",
                    self.target.display_name(),
                    base,
                    scale
                )
            }
        };
        let action = match &self.action {
            RuleAction::SuppressMrna => "suppress mRNA",
        };
        format!("{}: {} -> {}", self.target.display_name(), cond, action)
    }
}

pub fn default_rules() -> Vec<Rule> {
    vec![Rule {
        target: MrnaTarget::Motor,
        condition: RuleCondition::CountAtScaledCap {
            base: BASE_MAX_MOTORS,
            scale: MOTOR_SCALE,
        },
        action: RuleAction::SuppressMrna,
        enabled: true,
        firing: false,
        current_limit: BASE_MAX_MOTORS,
    }]
}

/// Evaluate all rules and return per-strand suppression flags.
/// Also updates each rule's `firing` and `current_limit` fields.
pub fn evaluate_suppressions(
    rules: &mut Vec<Rule>,
    motor_count: usize,
    zymase_count: usize,
    expansion_count: i32,
) -> [bool; MRNA_COUNT] {
    let mut suppressions = [false; MRNA_COUNT];

    for rule in rules.iter_mut() {
        if !rule.enabled {
            rule.firing = false;
            continue;
        }

        let strand = rule.target.strand_index();
        let current_count = match rule.target {
            MrnaTarget::Motor => motor_count,
            MrnaTarget::Zymase => zymase_count,
            MrnaTarget::Membrane => 0, // no membrane count limit yet
        };

        match &rule.condition {
            RuleCondition::CountAtScaledCap { base, scale } => {
                let limit = base + (scale * (expansion_count as f32).sqrt()) as usize;
                rule.current_limit = limit;
                rule.firing = current_count >= limit;
            }
        }

        if rule.firing {
            match &rule.action {
                RuleAction::SuppressMrna => {
                    suppressions[strand] = true;
                }
            }
        }
    }

    suppressions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_contains_motor_cap() {
        let rules = default_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].target, MrnaTarget::Motor);
        assert!(rules[0].enabled);
        assert!(!rules[0].firing);
    }

    #[test]
    fn motor_cap_fires_at_limit() {
        let mut rules = default_rules();
        let sup = evaluate_suppressions(&mut rules, BASE_MAX_MOTORS, 1, 0);
        assert!(rules[0].firing);
        assert!(sup[1]); // motor strand suppressed
        assert!(!sup[0]);
        assert!(!sup[2]);
    }

    #[test]
    fn motor_cap_idle_below_limit() {
        let mut rules = default_rules();
        let sup = evaluate_suppressions(&mut rules, 1, 1, 0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn motor_cap_scales_with_expansion() {
        let mut rules = default_rules();
        // expansion_count=4, sqrt(4)=2, limit = 3 + 0.5*2 = 4
        let sup = evaluate_suppressions(&mut rules, 3, 1, 4);
        assert!(!rules[0].firing);
        assert_eq!(rules[0].current_limit, 4);
        assert!(!sup[1]);

        let sup = evaluate_suppressions(&mut rules, 4, 1, 4);
        assert!(rules[0].firing);
        assert!(sup[1]);
    }

    #[test]
    fn disabled_rule_does_not_fire() {
        let mut rules = default_rules();
        rules[0].enabled = false;
        let sup = evaluate_suppressions(&mut rules, BASE_MAX_MOTORS + 10, 1, 0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn strand_indices_correct() {
        assert_eq!(MrnaTarget::Zymase.strand_index(), 0);
        assert_eq!(MrnaTarget::Motor.strand_index(), 1);
        assert_eq!(MrnaTarget::Membrane.strand_index(), 2);
    }
}
