use crate::types::MRNA_COUNT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Metric {
    Count,
    Density,
    SurfaceDensity,
}

impl Metric {
    pub fn next(self) -> Self {
        match self {
            Metric::Count => Metric::Density,
            Metric::Density => Metric::SurfaceDensity,
            Metric::SurfaceDensity => Metric::Count,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Metric::Count => "Count",
            Metric::Density => "Density",
            Metric::SurfaceDensity => "SurfDens",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Relation {
    GreaterEqual,
    LessEqual,
}

impl Relation {
    pub fn next(self) -> Self {
        match self {
            Relation::GreaterEqual => Relation::LessEqual,
            Relation::LessEqual => Relation::GreaterEqual,
        }
    }

    pub fn display_symbol(self) -> &'static str {
        match self {
            Relation::GreaterEqual => ">=",
            Relation::LessEqual => "<=",
        }
    }
}

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

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => MrnaTarget::Zymase,
            1 => MrnaTarget::Motor,
            _ => MrnaTarget::Membrane,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            MrnaTarget::Zymase => "Zymase",
            MrnaTarget::Motor => "Motor",
            MrnaTarget::Membrane => "Membrane",
        }
    }

    pub fn next(self) -> Self {
        match self {
            MrnaTarget::Zymase => MrnaTarget::Motor,
            MrnaTarget::Motor => MrnaTarget::Membrane,
            MrnaTarget::Membrane => MrnaTarget::Zymase,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub metric: Metric,
    pub subject: MrnaTarget,
    pub relation: Relation,
    pub threshold: f32,
    pub target: MrnaTarget,
    pub enabled: bool,
    pub firing: bool,
    pub current_value: f32,
    pub locked: bool,
}

pub const MAX_RULES: usize = 5;

pub fn threshold_step(metric: Metric) -> f32 {
    match metric {
        Metric::Count => 1.0,
        Metric::Density => 0.0001,
        Metric::SurfaceDensity => 0.001,
    }
}

pub fn default_threshold_for_metric(metric: Metric) -> f32 {
    match metric {
        Metric::Count => 3.0,
        Metric::Density => 0.005,
        Metric::SurfaceDensity => 0.031,
    }
}

pub fn default_rules() -> Vec<Rule> {
    vec![Rule {
        metric: Metric::SurfaceDensity,
        subject: MrnaTarget::Motor,
        relation: Relation::GreaterEqual,
        threshold: 0.031,
        target: MrnaTarget::Motor,
        enabled: true,
        firing: false,
        current_value: 0.0,
        locked: true,
    }]
}

/// Evaluate all rules and return per-strand suppression flags.
/// Also updates each rule's `firing` and `current_value` fields.
pub fn evaluate_suppressions(
    rules: &mut Vec<Rule>,
    motor_count: usize,
    zymase_count: usize,
    expansion_count: i32,
    player_radius: f32,
) -> [bool; MRNA_COUNT] {
    let mut suppressions = [false; MRNA_COUNT];

    let pi = std::f32::consts::PI;

    for rule in rules.iter_mut() {
        if !rule.enabled {
            rule.firing = false;
            continue;
        }

        let raw_count = match rule.subject {
            MrnaTarget::Motor => motor_count as f32,
            MrnaTarget::Zymase => zymase_count as f32,
            MrnaTarget::Membrane => expansion_count as f32,
        };

        let value = match rule.metric {
            Metric::Count => raw_count,
            Metric::Density => {
                let area = pi * player_radius * player_radius;
                if area > 0.0 { raw_count / area } else { 0.0 }
            }
            Metric::SurfaceDensity => {
                let circumference = 2.0 * pi * player_radius;
                if circumference > 0.0 { raw_count / circumference } else { 0.0 }
            }
        };

        rule.current_value = value;

        rule.firing = match rule.relation {
            Relation::GreaterEqual => value >= rule.threshold,
            Relation::LessEqual => value <= rule.threshold,
        };

        if rule.firing {
            let strand = rule.target.strand_index();
            suppressions[strand] = true;
        }
    }

    suppressions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_rules_contains_motor_surface_density() {
        let rules = default_rules();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].metric, Metric::SurfaceDensity);
        assert_eq!(rules[0].subject, MrnaTarget::Motor);
        assert_eq!(rules[0].relation, Relation::GreaterEqual);
        assert_eq!(rules[0].target, MrnaTarget::Motor);
        assert!(rules[0].enabled);
        assert!(!rules[0].firing);
    }

    #[test]
    fn surface_density_fires_at_threshold() {
        let mut rules = default_rules();
        // 3 motors at radius 15: 3/(2*pi*15) ≈ 0.0318 >= 0.031
        let sup = evaluate_suppressions(&mut rules, 3, 1, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[1]); // motor strand suppressed
        assert!(!sup[0]);
        assert!(!sup[2]);
    }

    #[test]
    fn surface_density_idle_below_threshold() {
        let mut rules = default_rules();
        // 1 motor at radius 15: 1/(2*pi*15) ≈ 0.0106 < 0.031
        let sup = evaluate_suppressions(&mut rules, 1, 1, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn surface_density_scales_with_radius() {
        let mut rules = default_rules();
        // 3 motors at radius 30: 3/(2*pi*30) ≈ 0.0159 < 0.031
        let sup = evaluate_suppressions(&mut rules, 3, 1, 0, 30.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);

        // 6 motors at radius 30: 6/(2*pi*30) ≈ 0.0318 >= 0.031
        let sup = evaluate_suppressions(&mut rules, 6, 1, 0, 30.0);
        assert!(rules[0].firing);
        assert!(sup[1]);
    }

    #[test]
    fn disabled_rule_does_not_fire() {
        let mut rules = default_rules();
        rules[0].enabled = false;
        let sup = evaluate_suppressions(&mut rules, 10, 1, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn count_metric_works() {
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Motor,
            relation: Relation::GreaterEqual,
            threshold: 3.0,
            target: MrnaTarget::Motor,
            enabled: true,
            firing: false,
            current_value: 0.0,
            locked: false,
        }];
        let sup = evaluate_suppressions(&mut rules, 3, 1, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[1]);

        let sup = evaluate_suppressions(&mut rules, 2, 1, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn less_equal_relation_works() {
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Zymase,
            relation: Relation::LessEqual,
            threshold: 2.0,
            target: MrnaTarget::Zymase,
            enabled: true,
            firing: false,
            current_value: 0.0,
            locked: false,
        }];
        let sup = evaluate_suppressions(&mut rules, 1, 2, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[0]); // zymase suppressed

        let sup = evaluate_suppressions(&mut rules, 1, 3, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[0]);
    }

    #[test]
    fn membrane_subject_uses_expansion_count() {
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Membrane,
            relation: Relation::GreaterEqual,
            threshold: 5.0,
            target: MrnaTarget::Membrane,
            enabled: true,
            firing: false,
            current_value: 0.0,
            locked: false,
        }];
        let sup = evaluate_suppressions(&mut rules, 1, 1, 5, 15.0);
        assert!(rules[0].firing);
        assert!(sup[2]); // membrane suppressed

        let sup = evaluate_suppressions(&mut rules, 1, 1, 4, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[2]);
    }

    #[test]
    fn strand_indices_correct() {
        assert_eq!(MrnaTarget::Zymase.strand_index(), 0);
        assert_eq!(MrnaTarget::Motor.strand_index(), 1);
        assert_eq!(MrnaTarget::Membrane.strand_index(), 2);
    }

    #[test]
    fn metric_cycling() {
        assert_eq!(Metric::Count.next(), Metric::Density);
        assert_eq!(Metric::Density.next(), Metric::SurfaceDensity);
        assert_eq!(Metric::SurfaceDensity.next(), Metric::Count);
    }

    #[test]
    fn relation_cycling() {
        assert_eq!(Relation::GreaterEqual.next(), Relation::LessEqual);
        assert_eq!(Relation::LessEqual.next(), Relation::GreaterEqual);
    }

    #[test]
    fn target_cycling() {
        assert_eq!(MrnaTarget::Zymase.next(), MrnaTarget::Motor);
        assert_eq!(MrnaTarget::Motor.next(), MrnaTarget::Membrane);
        assert_eq!(MrnaTarget::Membrane.next(), MrnaTarget::Zymase);
    }

    #[test]
    fn current_value_stored() {
        let mut rules = default_rules();
        evaluate_suppressions(&mut rules, 3, 1, 0, 15.0);
        // 3 / (2 * pi * 15) ≈ 0.0318
        assert!((rules[0].current_value - 3.0 / (2.0 * std::f32::consts::PI * 15.0)).abs() < 0.001);
    }
}
