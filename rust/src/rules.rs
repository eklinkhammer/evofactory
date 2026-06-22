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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Threshold {
    Fixed(f32),
    Variable(MrnaTarget),
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub metric: Metric,
    pub subject: MrnaTarget,
    pub relation: Relation,
    pub threshold: Threshold,
    pub target: MrnaTarget,
    pub enabled: bool,
    pub firing: bool,
    pub current_value: f32,
    pub current_threshold_value: f32,
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

pub fn default_threshold_for_metric(metric: Metric) -> Threshold {
    Threshold::Fixed(match metric {
        Metric::Count => 3.0,
        Metric::Density => 0.005,
        Metric::SurfaceDensity => 0.027,
    })
}

pub fn default_rules() -> Vec<Rule> {
    vec![Rule {
        metric: Metric::SurfaceDensity,
        subject: MrnaTarget::Motor,
        relation: Relation::GreaterEqual,
        threshold: Threshold::Fixed(0.027),
        target: MrnaTarget::Motor,
        enabled: true,
        firing: false,
        current_value: 0.0,
        current_threshold_value: 0.0,
        locked: true,
    }]
}

pub fn compute_metric_value(
    metric: Metric,
    target: MrnaTarget,
    motor_count: usize,
    zymase_count: usize,
    expansion_count: i32,
    radius: f32,
) -> f32 {
    let pi = std::f32::consts::PI;

    let raw_count = match target {
        MrnaTarget::Motor => motor_count as f32,
        MrnaTarget::Zymase => zymase_count as f32,
        MrnaTarget::Membrane => expansion_count as f32,
    };

    match metric {
        Metric::Count => raw_count,
        Metric::Density => {
            let area = pi * radius * radius;
            if area > 0.0 { raw_count / area } else { 0.0 }
        }
        Metric::SurfaceDensity => {
            let circumference = 2.0 * pi * radius;
            if circumference > 0.0 { raw_count / circumference } else { 0.0 }
        }
    }
}

/// Evaluate all rules and return per-strand suppression flags.
/// Also updates each rule's `firing`, `current_value`, and `current_threshold_value` fields.
pub fn evaluate_suppressions(
    rules: &mut Vec<Rule>,
    motor_count: usize,
    zymase_count: usize,
    expansion_count: i32,
    player_radius: f32,
) -> [bool; MRNA_COUNT] {
    let mut suppressions = [false; MRNA_COUNT];

    for rule in rules.iter_mut() {
        if !rule.enabled {
            rule.firing = false;
            continue;
        }

        let value = compute_metric_value(
            rule.metric,
            rule.subject,
            motor_count,
            zymase_count,
            expansion_count,
            player_radius,
        );

        rule.current_value = value;

        let threshold_value = match rule.threshold {
            Threshold::Fixed(v) => v,
            Threshold::Variable(ref_target) => compute_metric_value(
                rule.metric,
                ref_target,
                motor_count,
                zymase_count,
                expansion_count,
                player_radius,
            ),
        };

        rule.current_threshold_value = threshold_value;

        rule.firing = match rule.relation {
            Relation::GreaterEqual => value >= threshold_value,
            Relation::LessEqual => value <= threshold_value,
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
        // 4 motors at radius 23 (after 1 expansion): 4/(2*pi*23) ≈ 0.0277 >= 0.027
        let sup = evaluate_suppressions(&mut rules, 4, 1, 1, 23.0);
        assert!(rules[0].firing);
        assert!(sup[1]); // motor strand suppressed
        assert!(!sup[0]);
        assert!(!sup[2]);
    }

    #[test]
    fn surface_density_idle_below_threshold() {
        let mut rules = default_rules();
        // 3 motors at radius 23: 3/(2*pi*23) ≈ 0.0208 < 0.027
        let sup = evaluate_suppressions(&mut rules, 3, 1, 1, 23.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn surface_density_scales_with_radius() {
        let mut rules = default_rules();
        // 4 motors at radius 40: 4/(2*pi*40) ≈ 0.0159 < 0.027
        let sup = evaluate_suppressions(&mut rules, 4, 1, 0, 40.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);

        // 7 motors at radius 40: 7/(2*pi*40) ≈ 0.0279 >= 0.027
        let sup = evaluate_suppressions(&mut rules, 7, 1, 0, 40.0);
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
            threshold: Threshold::Fixed(3.0),
            target: MrnaTarget::Motor,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
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
            threshold: Threshold::Fixed(2.0),
            target: MrnaTarget::Zymase,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
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
            threshold: Threshold::Fixed(5.0),
            target: MrnaTarget::Membrane,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
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

    #[test]
    fn variable_threshold_count_comparison() {
        // Suppress Zymase when Count(Zymase) >= Count(Motor)
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Zymase,
            relation: Relation::GreaterEqual,
            threshold: Threshold::Variable(MrnaTarget::Motor),
            target: MrnaTarget::Zymase,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
            locked: false,
        }];

        // 3 zymases >= 2 motors → fires
        let sup = evaluate_suppressions(&mut rules, 2, 3, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[0]);

        // 1 zymase < 2 motors → does not fire
        let sup = evaluate_suppressions(&mut rules, 2, 1, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[0]);
    }

    #[test]
    fn variable_threshold_density_comparison() {
        // Suppress Motor when Density(Motor) >= Density(Zymase)
        let mut rules = vec![Rule {
            metric: Metric::Density,
            subject: MrnaTarget::Motor,
            relation: Relation::GreaterEqual,
            threshold: Threshold::Variable(MrnaTarget::Zymase),
            target: MrnaTarget::Motor,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
            locked: false,
        }];

        // 5 motors, 3 zymases at r=15 → density(motor) > density(zymase) → fires
        let sup = evaluate_suppressions(&mut rules, 5, 3, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[1]);

        // 2 motors, 5 zymases → density(motor) < density(zymase) → does not fire
        let sup = evaluate_suppressions(&mut rules, 2, 5, 0, 15.0);
        assert!(!rules[0].firing);
        assert!(!sup[1]);
    }

    #[test]
    fn variable_threshold_equal_values() {
        // Count(Motor) >= Count(Zymase) with equal counts → fires (>=)
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Motor,
            relation: Relation::GreaterEqual,
            threshold: Threshold::Variable(MrnaTarget::Zymase),
            target: MrnaTarget::Motor,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
            locked: false,
        }];

        // 3 motors == 3 zymases → fires
        let sup = evaluate_suppressions(&mut rules, 3, 3, 0, 15.0);
        assert!(rules[0].firing);
        assert!(sup[1]);
    }

    #[test]
    fn variable_threshold_resolved_value_stored() {
        let mut rules = vec![Rule {
            metric: Metric::Count,
            subject: MrnaTarget::Zymase,
            relation: Relation::GreaterEqual,
            threshold: Threshold::Variable(MrnaTarget::Motor),
            target: MrnaTarget::Zymase,
            enabled: true,
            firing: false,
            current_value: 0.0,
            current_threshold_value: 0.0,
            locked: false,
        }];

        evaluate_suppressions(&mut rules, 4, 2, 0, 15.0);
        assert_eq!(rules[0].current_value, 2.0); // zymase count
        assert_eq!(rules[0].current_threshold_value, 4.0); // motor count (the threshold reference)
    }
}
