use crate::types::*;
use rand::Rng;

pub const GLUCOSE_MIN_SEP: f32 = 12.0;
pub const DIFFUSION_SPEED: f32 = 60.0;
pub const BASE_MAX_GLUCOSE: i32 = 5;
pub const BASE_MAX_AMINO: i32 = 5;
pub const BASE_MAX_NUCLEOTIDE: i32 = 5;

pub struct ParticleCounts {
    pub glucose: i32,
    pub amino_acid: i32,
    pub atp: i32,
    pub nucleotide: i32,
}

pub struct AbsorptionEvent {
    pub resource_index: usize,
    pub entry_x: f32,
    pub entry_y: f32,
    pub resource_type: i32,
    pub amount: f32,
}

/// Brownian diffusion for particles, clamped to membrane. Skips dragged particle.
pub fn diffuse(
    particles: &mut [InteriorParticle],
    dragged_index: Option<usize>,
    dt: f32,
    rng: &mut impl Rng,
) {
    let sigma = DIFFUSION_SPEED * dt.sqrt();
    let max_r = INTERIOR_RADIUS * 0.9;
    for (i, p) in particles.iter_mut().enumerate() {
        if Some(i) == dragged_index {
            continue;
        }
        p.x += rng.gen_range(-sigma..sigma);
        p.y += rng.gen_range(-sigma..sigma);
        let dist = (p.x * p.x + p.y * p.y).sqrt();
        if dist > max_r {
            let scale = max_r / dist;
            p.x *= scale;
            p.y *= scale;
        }
    }
}

/// O(n²) pairwise separation for all particles.
pub fn separate(particles: &mut [InteriorParticle], rng: &mut impl Rng) {
    let n = particles.len();
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = particles[j].x - particles[i].x;
            let dy = particles[j].y - particles[i].y;
            let d = (dx * dx + dy * dy).sqrt();
            if d < GLUCOSE_MIN_SEP && d > 0.001 {
                let overlap = (GLUCOSE_MIN_SEP - d) * 0.5;
                let nx = dx / d;
                let ny = dy / d;
                particles[i].x -= nx * overlap;
                particles[i].y -= ny * overlap;
                particles[j].x += nx * overlap;
                particles[j].y += ny * overlap;
            } else if d <= 0.001 {
                let a = rng.gen_range(0.0..std::f32::consts::TAU);
                let half = GLUCOSE_MIN_SEP * 0.5;
                particles[i].x -= a.cos() * half;
                particles[i].y -= a.sin() * half;
                particles[j].x += a.cos() * half;
                particles[j].y += a.sin() * half;
            }
        }
    }
}

/// Re-clamp particles to membrane boundary.
pub fn clamp_to_membrane(particles: &mut [InteriorParticle]) {
    let max_r = INTERIOR_RADIUS * 0.9;
    for p in particles.iter_mut() {
        let dist = (p.x * p.x + p.y * p.y).sqrt();
        if dist > max_r {
            let scale = max_r / dist;
            p.x *= scale;
            p.y *= scale;
        }
    }
}

/// Detect resources overlapping the player cell. Respects capacity limits.
pub fn detect_absorptions(
    px: f32,
    py: f32,
    radius: f32,
    resources: &[Resource],
    mut cur_glucose: i32,
    max_glucose: i32,
    mut cur_amino: i32,
    max_amino: i32,
    mut cur_nucleotide: i32,
    max_nucleotide: i32,
    rng: &mut impl Rng,
) -> Vec<AbsorptionEvent> {
    let pickup_dist = radius + RESOURCE_RADIUS;
    let pickup_dist_sq = pickup_dist * pickup_dist;
    let mut events = Vec::new();

    for (i, r) in resources.iter().enumerate() {
        let dx = px - r.x;
        let dy = py - r.y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < pickup_dist_sq {
            // Check capacity
            match r.resource_type {
                0 => {
                    if cur_glucose >= max_glucose {
                        continue;
                    }
                    cur_glucose += 1;
                }
                1 => {
                    if cur_amino >= max_amino {
                        continue;
                    }
                    cur_amino += 1;
                }
                3 => {
                    if cur_nucleotide >= max_nucleotide {
                        continue;
                    }
                    cur_nucleotide += 1;
                }
                _ => continue,
            }

            // Compute entry point on membrane from resource direction
            let dir_x = r.x - px;
            let dir_y = r.y - py;
            let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            let (entry_x, entry_y) = if dir_len > 0.001 {
                let nx = dir_x / dir_len;
                let ny = dir_y / dir_len;
                (nx * INTERIOR_RADIUS * 0.85, ny * INTERIOR_RADIUS * 0.85)
            } else {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                (
                    angle.cos() * INTERIOR_RADIUS * 0.85,
                    angle.sin() * INTERIOR_RADIUS * 0.85,
                )
            };

            events.push(AbsorptionEvent {
                resource_index: i,
                entry_x,
                entry_y,
                resource_type: r.resource_type,
                amount: r.amount,
            });
        }
    }
    events
}

/// Push existing glucose away from entry point and add the new particle.
pub fn apply_absorption(
    particles: &mut Vec<InteriorParticle>,
    event: &AbsorptionEvent,
    rng: &mut impl Rng,
) {
    if event.resource_type == 0 {
        // Push existing glucose away from entry point
        for p in particles.iter_mut() {
            if p.resource_type != 0 {
                continue;
            }
            let dx = p.x - event.entry_x;
            let dy = p.y - event.entry_y;
            let d = (dx * dx + dy * dy).sqrt();
            if d < GLUCOSE_MIN_SEP {
                if d < 0.001 {
                    let a = rng.gen_range(0.0..std::f32::consts::TAU);
                    p.x = event.entry_x + a.cos() * GLUCOSE_MIN_SEP;
                    p.y = event.entry_y + a.sin() * GLUCOSE_MIN_SEP;
                } else {
                    let nx = dx / d;
                    let ny = dy / d;
                    p.x = event.entry_x + nx * GLUCOSE_MIN_SEP;
                    p.y = event.entry_y + ny * GLUCOSE_MIN_SEP;
                }
            }
        }
    }
    particles.push(InteriorParticle {
        x: event.entry_x,
        y: event.entry_y,
        resource_type: event.resource_type,
    });
}

/// Count particles by type.
pub fn count_particles(particles: &[InteriorParticle]) -> ParticleCounts {
    let mut glucose = 0;
    let mut amino_acid = 0;
    let mut atp = 0;
    let mut nucleotide = 0;
    for p in particles {
        match p.resource_type {
            0 => glucose += 1,
            1 => amino_acid += 1,
            2 => atp += 1,
            3 => nucleotide += 1,
            _ => {}
        }
    }
    ParticleCounts {
        glucose,
        amino_acid,
        atp,
        nucleotide,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diffuse_particles_move() {
        let mut particles = vec![InteriorParticle {
            x: 50.0,
            y: 50.0,
            resource_type: 0,
        }];
        let mut rng = rand::thread_rng();
        let orig_x = particles[0].x;
        let orig_y = particles[0].y;
        diffuse(&mut particles, None, 1.0, &mut rng);
        // Extremely unlikely to stay at exact same position
        assert!(particles[0].x != orig_x || particles[0].y != orig_y);
    }

    #[test]
    fn diffuse_skips_dragged() {
        let mut particles = vec![InteriorParticle {
            x: 50.0,
            y: 50.0,
            resource_type: 0,
        }];
        let mut rng = rand::thread_rng();
        diffuse(&mut particles, Some(0), 1.0, &mut rng);
        assert_eq!(particles[0].x, 50.0);
        assert_eq!(particles[0].y, 50.0);
    }

    #[test]
    fn diffuse_clamps_to_membrane() {
        let mut particles = vec![InteriorParticle {
            x: INTERIOR_RADIUS * 0.89,
            y: 0.0,
            resource_type: 0,
        }];
        let mut rng = rand::thread_rng();
        // Run many iterations; particle should never exceed 0.9 * INTERIOR_RADIUS
        for _ in 0..100 {
            diffuse(&mut particles, None, 0.016, &mut rng);
        }
        let dist = (particles[0].x * particles[0].x + particles[0].y * particles[0].y).sqrt();
        assert!(dist <= INTERIOR_RADIUS * 0.9 + 0.01);
    }

    #[test]
    fn separate_overlapping_pushed_apart() {
        let mut particles = vec![
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 0,
            },
            InteriorParticle {
                x: 2.0,
                y: 0.0,
                resource_type: 0,
            },
        ];
        let mut rng = rand::thread_rng();
        separate(&mut particles, &mut rng);
        let dx = particles[1].x - particles[0].x;
        let dy = particles[1].y - particles[0].y;
        let d = (dx * dx + dy * dy).sqrt();
        assert!((d - GLUCOSE_MIN_SEP).abs() < 0.01);
    }

    #[test]
    fn separate_coincident_random_separation() {
        let mut particles = vec![
            InteriorParticle {
                x: 10.0,
                y: 10.0,
                resource_type: 0,
            },
            InteriorParticle {
                x: 10.0,
                y: 10.0,
                resource_type: 0,
            },
        ];
        let mut rng = rand::thread_rng();
        separate(&mut particles, &mut rng);
        let dx = particles[1].x - particles[0].x;
        let dy = particles[1].y - particles[0].y;
        let d = (dx * dx + dy * dy).sqrt();
        assert!((d - GLUCOSE_MIN_SEP).abs() < 0.01);
    }

    #[test]
    fn clamp_to_membrane_outside_scaled_back() {
        let mut particles = vec![InteriorParticle {
            x: INTERIOR_RADIUS,
            y: 0.0,
            resource_type: 0,
        }];
        clamp_to_membrane(&mut particles);
        let dist = (particles[0].x * particles[0].x + particles[0].y * particles[0].y).sqrt();
        assert!((dist - INTERIOR_RADIUS * 0.9).abs() < 0.01);
    }

    #[test]
    fn clamp_to_membrane_inside_unchanged() {
        let mut particles = vec![InteriorParticle {
            x: 50.0,
            y: 0.0,
            resource_type: 0,
        }];
        clamp_to_membrane(&mut particles);
        assert_eq!(particles[0].x, 50.0);
    }

    #[test]
    fn detect_absorptions_in_range() {
        let resources = vec![Resource {
            x: 5.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let mut rng = rand::thread_rng();
        let events =
            detect_absorptions(0.0, 0.0, 10.0, &resources, 0, 5, 0, 5, 0, 5, &mut rng);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].resource_index, 0);
    }

    #[test]
    fn detect_absorptions_out_of_range() {
        let resources = vec![Resource {
            x: 500.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let mut rng = rand::thread_rng();
        let events =
            detect_absorptions(0.0, 0.0, 10.0, &resources, 0, 5, 0, 5, 0, 5, &mut rng);
        assert!(events.is_empty());
    }

    #[test]
    fn detect_absorptions_at_capacity_skipped() {
        let resources = vec![Resource {
            x: 5.0,
            y: 0.0,
            resource_type: 0,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let mut rng = rand::thread_rng();
        let events =
            detect_absorptions(0.0, 0.0, 10.0, &resources, 5, 5, 0, 5, 0, 5, &mut rng);
        assert!(events.is_empty());
    }

    #[test]
    fn count_particles_mixed() {
        let particles = vec![
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 0,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 0,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 1,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 2,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 2,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 2,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 3,
            },
            InteriorParticle {
                x: 0.0,
                y: 0.0,
                resource_type: 3,
            },
        ];
        let counts = count_particles(&particles);
        assert_eq!(counts.glucose, 2);
        assert_eq!(counts.amino_acid, 1);
        assert_eq!(counts.atp, 3);
        assert_eq!(counts.nucleotide, 2);
    }

    #[test]
    fn detect_absorptions_nucleotide_in_range() {
        let resources = vec![Resource {
            x: 5.0,
            y: 0.0,
            resource_type: 3,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let mut rng = rand::thread_rng();
        let events =
            detect_absorptions(0.0, 0.0, 10.0, &resources, 0, 5, 0, 5, 0, 5, &mut rng);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].resource_index, 0);
    }

    #[test]
    fn detect_absorptions_nucleotide_at_capacity() {
        let resources = vec![Resource {
            x: 5.0,
            y: 0.0,
            resource_type: 3,
            amount: 1.0,
            chunk_x: 0,
            chunk_y: 0,
        }];
        let mut rng = rand::thread_rng();
        let events =
            detect_absorptions(0.0, 0.0, 10.0, &resources, 0, 5, 0, 5, 5, 5, &mut rng);
        assert!(events.is_empty());
    }
}
