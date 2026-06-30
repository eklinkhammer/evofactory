use crate::types::Resource;

pub const BASE_SENSOR_RANGE: f32 = 80.0;
pub const SENSOR_RANGE_PER_UNIT: f32 = 40.0;
pub const RANDOM_WALK_INTERVAL: f32 = 2.0;
pub const RANDOM_WALK_STRENGTH: f32 = 0.3;
pub const CHEMOTAXIS_STRENGTH: f32 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SeekTarget {
    Nearest,
    Glucose,
    AminoAcid,
}

impl SeekTarget {
    pub fn next(self) -> SeekTarget {
        match self {
            SeekTarget::Nearest => SeekTarget::Glucose,
            SeekTarget::Glucose => SeekTarget::AminoAcid,
            SeekTarget::AminoAcid => SeekTarget::Nearest,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            SeekTarget::Nearest => "Nearest",
            SeekTarget::Glucose => "Glucose",
            SeekTarget::AminoAcid => "Amino Acid",
        }
    }

    pub fn as_i32(self) -> i32 {
        match self {
            SeekTarget::Nearest => 0,
            SeekTarget::Glucose => 1,
            SeekTarget::AminoAcid => 2,
        }
    }

    pub fn from_i32(v: i32) -> SeekTarget {
        match v {
            1 => SeekTarget::Glucose,
            2 => SeekTarget::AminoAcid,
            _ => SeekTarget::Nearest,
        }
    }
}

/// Compute chemotaxis direction based on nearby resources.
/// Returns a normalized (dx, dy) direction vector toward weighted resource center,
/// or (0, 0) if no matching resources are in range.
pub fn compute_chemotaxis_direction(
    player_x: f32,
    player_y: f32,
    player_radius: f32,
    sensor_count: i32,
    seek_target: SeekTarget,
    resources: &[Resource],
) -> (f32, f32) {
    let range = BASE_SENSOR_RANGE + sensor_count as f32 * SENSOR_RANGE_PER_UNIT;
    let effective_range = range + player_radius;
    let range_sq = effective_range * effective_range;

    let mut sum_dx: f32 = 0.0;
    let mut sum_dy: f32 = 0.0;

    for r in resources {
        // Filter by seek target
        match seek_target {
            SeekTarget::Glucose => {
                if r.resource_type != 0 {
                    continue;
                }
            }
            SeekTarget::AminoAcid => {
                if r.resource_type != 1 {
                    continue;
                }
            }
            SeekTarget::Nearest => {}
        }

        let dx = r.x - player_x;
        let dy = r.y - player_y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq > range_sq || dist_sq < 0.001 {
            continue;
        }

        // Inverse-square weighting: closer resources pull much harder
        let weight = 1.0 / dist_sq;
        sum_dx += dx * weight;
        sum_dy += dy * weight;
    }

    let mag = (sum_dx * sum_dx + sum_dy * sum_dy).sqrt();
    if mag > 0.001 {
        (sum_dx / mag, sum_dy / mag)
    } else {
        (0.0, 0.0)
    }
}

pub fn sensor_range(sensor_count: i32) -> f32 {
    BASE_SENSOR_RANGE + sensor_count as f32 * SENSOR_RANGE_PER_UNIT
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seek_target_cycles() {
        assert_eq!(SeekTarget::Nearest.next(), SeekTarget::Glucose);
        assert_eq!(SeekTarget::Glucose.next(), SeekTarget::AminoAcid);
        assert_eq!(SeekTarget::AminoAcid.next(), SeekTarget::Nearest);
    }

    #[test]
    fn seek_target_round_trips() {
        for t in [SeekTarget::Nearest, SeekTarget::Glucose, SeekTarget::AminoAcid] {
            assert_eq!(SeekTarget::from_i32(t.as_i32()), t);
        }
    }

    #[test]
    fn chemotaxis_no_resources_returns_zero() {
        let (dx, dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Nearest, &[]);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn chemotaxis_single_resource_points_toward_it() {
        let resources = vec![Resource {
            x: 50.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let (dx, dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Nearest, &resources);
        assert!(dx > 0.9, "Expected dx > 0.9, got {}", dx);
        assert!(dy.abs() < 0.01, "Expected dy near 0, got {}", dy);
    }

    #[test]
    fn chemotaxis_filters_by_seek_target() {
        let resources = vec![
            Resource { x: 50.0, y: 0.0, resource_type: 0, amount: 1.0, chunk_x: 0, chunk_y: 0 },
            Resource { x: -50.0, y: 0.0, resource_type: 1, amount: 1.0, chunk_x: 0, chunk_y: 0 },
        ];
        // Seek only amino acid → should go left
        let (dx, _dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::AminoAcid, &resources);
        assert!(dx < -0.9, "Expected dx < -0.9 for amino acid seek, got {}", dx);

        // Seek only glucose → should go right
        let (dx, _dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Glucose, &resources);
        assert!(dx > 0.9, "Expected dx > 0.9 for glucose seek, got {}", dx);
    }

    #[test]
    fn chemotaxis_out_of_range_returns_zero() {
        let resources = vec![Resource {
            x: 500.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let (dx, dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Nearest, &resources);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn sensor_range_scales_with_count() {
        assert_eq!(sensor_range(0), BASE_SENSOR_RANGE);
        assert_eq!(sensor_range(1), BASE_SENSOR_RANGE + SENSOR_RANGE_PER_UNIT);
        assert_eq!(sensor_range(3), BASE_SENSOR_RANGE + 3.0 * SENSOR_RANGE_PER_UNIT);
    }

    #[test]
    fn chemotaxis_resource_at_player_position_ignored() {
        // Resource exactly at player position (dist_sq < 0.001) should not cause issues
        let resources = vec![Resource {
            x: 0.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let (dx, dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Nearest, &resources);
        assert_eq!(dx, 0.0);
        assert_eq!(dy, 0.0);
    }

    #[test]
    fn chemotaxis_closer_resource_wins() {
        // Two resources in opposite directions, closer one should dominate
        let resources = vec![
            Resource { x: 30.0, y: 0.0, resource_type: 0, amount: 1.0, chunk_x: 0, chunk_y: 0 },
            Resource { x: -90.0, y: 0.0, resource_type: 0, amount: 1.0, chunk_x: 0, chunk_y: 0 },
        ];
        let (dx, _dy) = compute_chemotaxis_direction(0.0, 0.0, 10.0, 1, SeekTarget::Nearest, &resources);
        // Closer resource at +30 should pull harder than distant one at -90
        assert!(dx > 0.0, "Expected positive dx (toward closer resource), got {}", dx);
    }

    #[test]
    fn seek_target_from_i32_out_of_range_defaults_to_nearest() {
        assert_eq!(SeekTarget::from_i32(-1), SeekTarget::Nearest);
        assert_eq!(SeekTarget::from_i32(3), SeekTarget::Nearest);
        assert_eq!(SeekTarget::from_i32(99), SeekTarget::Nearest);
    }
}
