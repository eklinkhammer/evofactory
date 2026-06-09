use crate::types::*;
use rand::Rng;

pub const ZYMASE_CRAFT_TIME: f32 = 2.0;
pub const ZYMASE_BUFFER_SIZE: i32 = 2;
pub const ZYMASE_COLLISION_DIST: f32 = 20.0;
pub const MRNA_CRAFT_TIME: f32 = 2.0;
pub const MRNA_REQUIRED: [i32; MRNA_COUNT] = [8, 7, 5];
pub const MRNA_COLLISION_DIST: f32 = 20.0;
pub const MOTOR_COLLISION_DIST: f32 = 25.0;
pub const FERMENTATION_YIELD: f32 = 2.0;
pub const GROWTH_SCALE: f32 = 8.0;
pub const MRNA_ANGLES: [f32; MRNA_COUNT] = [
    150.0 * std::f32::consts::PI / 180.0, // zymase — upper-left
    270.0 * std::f32::consts::PI / 180.0, // motor  — bottom
    30.0 * std::f32::consts::PI / 180.0,  // membrane — upper-right
];

pub enum CraftOutput {
    SpawnParticle { x: f32, y: f32, resource_type: i32 },
    SpawnZymase { x: f32, y: f32 },
    SpawnMotor { angle: f32 },
    GrowCell,
}

// --- Shared target-finding ---

/// Find nearest zymase within collision range that has buffer space.
pub fn find_zymase_target(x: f32, y: f32, zymases: &[Zymase]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_dist = ZYMASE_COLLISION_DIST;
    for (i, e) in zymases.iter().enumerate() {
        let dx = x - e.x;
        let dy = y - e.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < best_dist && e.buffer < ZYMASE_BUFFER_SIZE {
            best_dist = dist;
            best = Some(i);
        }
    }
    best
}

/// Find nearest mRNA strand that accepts amino acids.
/// Strands flagged in `suppressions` are skipped entirely.
pub fn find_mrna_target(
    x: f32,
    y: f32,
    mrna_xs: &[f32],
    mrna_ys: &[f32],
    progress: &[i32; MRNA_COUNT],
    processing: &[bool; MRNA_COUNT],
    suppressions: &[bool; MRNA_COUNT],
) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_dist = MRNA_COLLISION_DIST;
    for m in 0..MRNA_COUNT {
        if suppressions[m] {
            continue;
        }
        let dx = x - mrna_xs[m];
        let dy = y - mrna_ys[m];
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < best_dist && progress[m] < MRNA_REQUIRED[m] && !processing[m] {
            best_dist = dist;
            best = Some(m);
        }
    }
    best
}

/// Find nearest motor that can accept ATP.
pub fn find_motor_target(x: f32, y: f32, motors: &[Motor]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_dist = MOTOR_COLLISION_DIST;
    for (i, motor) in motors.iter().enumerate() {
        let dx = x - motor.x;
        let dy = y - motor.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < best_dist && motor.charge < MAX_ATP {
            best_dist = dist;
            best = Some(i);
        }
    }
    best
}

// --- Feeding ---

/// Feed glucose to a zymase. Returns true if consumed.
pub fn feed_zymase(zymase: &mut Zymase) -> bool {
    if zymase.buffer >= ZYMASE_BUFFER_SIZE {
        return false;
    }
    zymase.buffer += 1;
    if !zymase.processing {
        zymase.buffer -= 1;
        zymase.processing = true;
        zymase.timer = ZYMASE_CRAFT_TIME;
    }
    true
}

/// Feed amino acid to an mRNA strand. Starts craft timer if threshold reached.
pub fn feed_mrna(
    strand: usize,
    progress: &mut [i32; MRNA_COUNT],
    processing: &mut [bool; MRNA_COUNT],
    timers: &mut [f32; MRNA_COUNT],
) -> bool {
    if progress[strand] >= MRNA_REQUIRED[strand] || processing[strand] {
        return false;
    }
    progress[strand] += 1;
    if progress[strand] >= MRNA_REQUIRED[strand] {
        processing[strand] = true;
        timers[strand] = MRNA_CRAFT_TIME;
    }
    true
}

/// Feed ATP to a motor. Returns true if consumed.
pub fn feed_motor(motor: &mut Motor) -> bool {
    if motor.charge >= MAX_ATP {
        return false;
    }
    motor.charge = (motor.charge + 1.0).min(MAX_ATP);
    true
}

// --- Timed crafting ---

/// Tick all zymases. Returns spawn commands for produced ATP particles.
pub fn tick_zymases(zymases: &mut [Zymase], dt: f32, rng: &mut impl Rng) -> Vec<CraftOutput> {
    let mut outputs = Vec::new();
    for ei in 0..zymases.len() {
        if zymases[ei].processing {
            zymases[ei].timer -= dt;
            if zymases[ei].timer <= 0.0 {
                let ex = zymases[ei].x;
                let ey = zymases[ei].y;
                for _ in 0..FERMENTATION_YIELD as i32 {
                    outputs.push(CraftOutput::SpawnParticle {
                        x: ex + rng.gen_range(-10.0..10.0),
                        y: ey + rng.gen_range(-10.0..10.0),
                        resource_type: 2,
                    });
                }
                zymases[ei].processing = false;
                zymases[ei].timer = 0.0;
                if zymases[ei].buffer > 0 {
                    zymases[ei].buffer -= 1;
                    zymases[ei].processing = true;
                    zymases[ei].timer = ZYMASE_CRAFT_TIME;
                }
            }
        }
    }
    outputs
}

/// Tick all mRNA strands. Returns spawn commands for completed crafts.
/// Strands flagged in `suppressions` skip output on completion (safety net
/// for the edge case where suppression activates mid-craft).
pub fn tick_mrna(
    processing: &mut [bool; MRNA_COUNT],
    timers: &mut [f32; MRNA_COUNT],
    progress: &mut [i32; MRNA_COUNT],
    mrna_xs: &[f32],
    mrna_ys: &[f32],
    suppressions: &[bool; MRNA_COUNT],
    dt: f32,
) -> Vec<CraftOutput> {
    let mut outputs = Vec::new();
    for m in 0..MRNA_COUNT {
        if processing[m] {
            timers[m] -= dt;
            if timers[m] <= 0.0 {
                processing[m] = false;
                timers[m] = 0.0;
                progress[m] = 0;
                if suppressions[m] {
                    continue;
                }
                if m == 0 {
                    outputs.push(CraftOutput::SpawnZymase {
                        x: mrna_xs[0],
                        y: mrna_ys[0],
                    });
                } else if m == 1 {
                    outputs.push(CraftOutput::SpawnMotor {
                        angle: MRNA_ANGLES[1],
                    });
                } else if m == 2 {
                    outputs.push(CraftOutput::GrowCell);
                }
            }
        }
    }
    outputs
}

// --- Auto-consumption ---

/// Detect and apply auto-consumption for all particles near organelles.
/// Returns indices of consumed particles (sorted, deduplicated, ready for reverse swap_remove).
pub fn auto_consume(
    particles: &[InteriorParticle],
    zymases: &mut [Zymase],
    mrna_xs: &[f32],
    mrna_ys: &[f32],
    mrna_progress: &mut [i32; MRNA_COUNT],
    mrna_processing: &mut [bool; MRNA_COUNT],
    mrna_timers: &mut [f32; MRNA_COUNT],
    motors: &mut [Motor],
    dragged_index: Option<usize>,
    player_glucose: &mut f32,
    suppressions: &[bool; MRNA_COUNT],
) -> Vec<usize> {
    // Phase 1: detect candidates
    let mut consumed: Vec<usize> = Vec::new();
    for (i, p) in particles.iter().enumerate() {
        if Some(i) == dragged_index {
            continue;
        }
        match p.resource_type {
            0 => {
                // Glucose → nearest zymase with buffer space
                for e in zymases.iter() {
                    let dx = p.x - e.x;
                    let dy = p.y - e.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < ZYMASE_COLLISION_DIST && e.buffer < ZYMASE_BUFFER_SIZE {
                        consumed.push(i);
                        break;
                    }
                }
            }
            1 => {
                // Amino acid → incomplete, non-processing, non-suppressed mRNA
                for m in 0..MRNA_COUNT {
                    if suppressions[m] {
                        continue;
                    }
                    let dx = p.x - mrna_xs[m];
                    let dy = p.y - mrna_ys[m];
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < MRNA_COLLISION_DIST
                        && mrna_progress[m] < MRNA_REQUIRED[m]
                        && !mrna_processing[m]
                    {
                        consumed.push(i);
                        break;
                    }
                }
            }
            2 => {
                // ATP → motor with charge < MAX
                for motor in motors.iter() {
                    let dx = p.x - motor.x;
                    let dy = p.y - motor.y;
                    let dist = (dx * dx + dy * dy).sqrt();
                    if dist < MOTOR_COLLISION_DIST && motor.charge < MAX_ATP {
                        consumed.push(i);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    // Phase 2: re-validate and apply state changes
    let mut actually_consumed: Vec<usize> = Vec::new();
    for &i in &consumed {
        let p = &particles[i];
        match p.resource_type {
            0 => {
                if let Some(ei) = find_zymase_target(p.x, p.y, zymases) {
                    *player_glucose -= 1.0;
                    feed_zymase(&mut zymases[ei]);
                    actually_consumed.push(i);
                }
            }
            1 => {
                if let Some(m) =
                    find_mrna_target(p.x, p.y, mrna_xs, mrna_ys, mrna_progress, mrna_processing, suppressions)
                {
                    feed_mrna(m, mrna_progress, mrna_processing, mrna_timers);
                    actually_consumed.push(i);
                }
            }
            2 => {
                if let Some(mi) = find_motor_target(p.x, p.y, motors) {
                    feed_motor(&mut motors[mi]);
                    actually_consumed.push(i);
                }
            }
            _ => {}
        }
    }

    actually_consumed.sort_unstable();
    actually_consumed.dedup();
    actually_consumed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_zymase(x: f32, y: f32, buffer: i32, processing: bool) -> Zymase {
        Zymase {
            x,
            y,
            buffer,
            processing,
            timer: 0.0,
        }
    }

    fn make_motor(x: f32, y: f32, charge: f32) -> Motor {
        Motor { x, y, charge }
    }

    #[test]
    fn find_zymase_target_nearest_with_buffer_space() {
        let zymases = vec![
            make_zymase(100.0, 0.0, 0, false), // far
            make_zymase(5.0, 0.0, 0, false),   // near
        ];
        assert_eq!(find_zymase_target(0.0, 0.0, &zymases), Some(1));
    }

    #[test]
    fn find_zymase_target_full_buffer_returns_none() {
        let zymases = vec![make_zymase(5.0, 0.0, ZYMASE_BUFFER_SIZE, true)];
        assert_eq!(find_zymase_target(0.0, 0.0, &zymases), None);
    }

    #[test]
    fn find_zymase_target_out_of_range_returns_none() {
        let zymases = vec![make_zymase(100.0, 100.0, 0, false)];
        assert_eq!(find_zymase_target(0.0, 0.0, &zymases), None);
    }

    #[test]
    fn find_motor_target_at_max_atp_returns_none() {
        let motors = vec![make_motor(5.0, 0.0, MAX_ATP)];
        assert_eq!(find_motor_target(0.0, 0.0, &motors), None);
    }

    #[test]
    fn find_motor_target_with_space() {
        let motors = vec![make_motor(5.0, 0.0, 0.0)];
        assert_eq!(find_motor_target(0.0, 0.0, &motors), Some(0));
    }

    #[test]
    fn feed_zymase_starts_processing_if_not_already() {
        let mut z = make_zymase(0.0, 0.0, 0, false);
        assert!(feed_zymase(&mut z));
        assert!(z.processing);
        assert_eq!(z.buffer, 0); // consumed immediately into processing
        assert_eq!(z.timer, ZYMASE_CRAFT_TIME);
    }

    #[test]
    fn feed_zymase_buffers_if_already_processing() {
        let mut z = make_zymase(0.0, 0.0, 0, true);
        z.timer = 1.0;
        assert!(feed_zymase(&mut z));
        assert_eq!(z.buffer, 1); // buffered
        assert!(z.processing);
    }

    #[test]
    fn feed_mrna_increments_progress() {
        let mut progress = [0; MRNA_COUNT];
        let mut processing = [false; MRNA_COUNT];
        let mut timers = [0.0; MRNA_COUNT];
        assert!(feed_mrna(0, &mut progress, &mut processing, &mut timers));
        assert_eq!(progress[0], 1);
        assert!(!processing[0]); // not at threshold yet
    }

    #[test]
    fn feed_mrna_starts_timer_at_threshold() {
        let mut progress = [MRNA_REQUIRED[0] - 1, 0, 0];
        let mut processing = [false; MRNA_COUNT];
        let mut timers = [0.0; MRNA_COUNT];
        assert!(feed_mrna(0, &mut progress, &mut processing, &mut timers));
        assert_eq!(progress[0], MRNA_REQUIRED[0]);
        assert!(processing[0]);
        assert_eq!(timers[0], MRNA_CRAFT_TIME);
    }

    #[test]
    fn tick_zymases_completion_yields_atp() {
        let mut rng = rand::thread_rng();
        let mut zymases = vec![Zymase {
            x: 10.0,
            y: 10.0,
            buffer: 0,
            processing: true,
            timer: 0.1,
        }];
        let outputs = tick_zymases(&mut zymases, 0.2, &mut rng);
        assert_eq!(outputs.len(), FERMENTATION_YIELD as usize);
        assert!(!zymases[0].processing);
    }

    #[test]
    fn tick_zymases_auto_starts_buffered() {
        let mut rng = rand::thread_rng();
        let mut zymases = vec![Zymase {
            x: 10.0,
            y: 10.0,
            buffer: 1,
            processing: true,
            timer: 0.1,
        }];
        let _outputs = tick_zymases(&mut zymases, 0.2, &mut rng);
        assert!(zymases[0].processing);
        assert_eq!(zymases[0].buffer, 0);
        assert_eq!(zymases[0].timer, ZYMASE_CRAFT_TIME);
    }

    #[test]
    fn tick_mrna_strand_0_spawns_zymase() {
        let mut processing = [true, false, false];
        let mut timers = [0.1, 0.0, 0.0];
        let mut progress = [MRNA_REQUIRED[0], 0, 0];
        let mrna_xs = [10.0, 20.0, 30.0];
        let mrna_ys = [10.0, 20.0, 30.0];
        let suppressions = [false; MRNA_COUNT];
        let outputs =
            tick_mrna(&mut processing, &mut timers, &mut progress, &mrna_xs, &mrna_ys, &suppressions, 0.2);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], CraftOutput::SpawnZymase { .. }));
    }

    #[test]
    fn tick_mrna_strand_1_spawns_motor() {
        let mut processing = [false, true, false];
        let mut timers = [0.0, 0.1, 0.0];
        let mut progress = [0, MRNA_REQUIRED[1], 0];
        let mrna_xs = [10.0, 20.0, 30.0];
        let mrna_ys = [10.0, 20.0, 30.0];
        let suppressions = [false; MRNA_COUNT];
        let outputs =
            tick_mrna(&mut processing, &mut timers, &mut progress, &mrna_xs, &mrna_ys, &suppressions, 0.2);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], CraftOutput::SpawnMotor { .. }));
    }

    #[test]
    fn tick_mrna_strand_1_suppressed_skips_output() {
        let mut processing = [false, true, false];
        let mut timers = [0.0, 0.1, 0.0];
        let mut progress = [0, MRNA_REQUIRED[1], 0];
        let mrna_xs = [10.0, 20.0, 30.0];
        let mrna_ys = [10.0, 20.0, 30.0];
        let suppressions = [false, true, false];
        let outputs = tick_mrna(
            &mut processing,
            &mut timers,
            &mut progress,
            &mrna_xs,
            &mrna_ys,
            &suppressions,
            0.2,
        );
        assert!(outputs.is_empty());
    }

    #[test]
    fn tick_mrna_strand_2_grows_cell() {
        let mut processing = [false, false, true];
        let mut timers = [0.0, 0.0, 0.1];
        let mut progress = [0, 0, MRNA_REQUIRED[2]];
        let mrna_xs = [10.0, 20.0, 30.0];
        let mrna_ys = [10.0, 20.0, 30.0];
        let suppressions = [false; MRNA_COUNT];
        let outputs =
            tick_mrna(&mut processing, &mut timers, &mut progress, &mrna_xs, &mrna_ys, &suppressions, 0.2);
        assert_eq!(outputs.len(), 1);
        assert!(matches!(outputs[0], CraftOutput::GrowCell));
    }

    #[test]
    fn auto_consume_glucose_near_zymase() {
        let particles = vec![InteriorParticle {
            x: 5.0,
            y: 0.0,
            resource_type: 0,
        }];
        let mut zymases = vec![make_zymase(0.0, 0.0, 0, false)];
        let mrna_xs = [100.0; MRNA_COUNT];
        let mrna_ys = [100.0; MRNA_COUNT];
        let mut mrna_progress = [0; MRNA_COUNT];
        let mut mrna_processing = [false; MRNA_COUNT];
        let mut mrna_timers = [0.0; MRNA_COUNT];
        let mut motors = vec![];
        let mut player_glucose = 1.0;
        let suppressions = [false; MRNA_COUNT];

        let consumed = auto_consume(
            &particles,
            &mut zymases,
            &mrna_xs,
            &mrna_ys,
            &mut mrna_progress,
            &mut mrna_processing,
            &mut mrna_timers,
            &mut motors,
            None,
            &mut player_glucose,
            &suppressions,
        );
        assert_eq!(consumed, vec![0]);
        assert!(zymases[0].processing);
        assert_eq!(player_glucose, 0.0);
    }

    #[test]
    fn auto_consume_particle_near_full_target_not_consumed() {
        let particles = vec![InteriorParticle {
            x: 5.0,
            y: 0.0,
            resource_type: 0,
        }];
        let mut zymases = vec![make_zymase(0.0, 0.0, ZYMASE_BUFFER_SIZE, true)];
        let mrna_xs = [100.0; MRNA_COUNT];
        let mrna_ys = [100.0; MRNA_COUNT];
        let mut mrna_progress = [0; MRNA_COUNT];
        let mut mrna_processing = [false; MRNA_COUNT];
        let mut mrna_timers = [0.0; MRNA_COUNT];
        let mut motors = vec![];
        let mut player_glucose = 1.0;
        let suppressions = [false; MRNA_COUNT];

        let consumed = auto_consume(
            &particles,
            &mut zymases,
            &mrna_xs,
            &mrna_ys,
            &mut mrna_progress,
            &mut mrna_processing,
            &mut mrna_timers,
            &mut motors,
            None,
            &mut player_glucose,
            &suppressions,
        );
        assert!(consumed.is_empty());
        assert_eq!(player_glucose, 1.0);
    }

    #[test]
    fn suppressed_strand_rejects_amino_acids() {
        let particles = vec![InteriorParticle {
            x: 5.0,
            y: 0.0,
            resource_type: 1, // amino acid
        }];
        let mut zymases = vec![];
        let mrna_xs = [5.0, 100.0, 100.0]; // strand 0 is right next to particle
        let mrna_ys = [0.0, 100.0, 100.0];
        let mut mrna_progress = [0; MRNA_COUNT];
        let mut mrna_processing = [false; MRNA_COUNT];
        let mut mrna_timers = [0.0; MRNA_COUNT];
        let mut motors = vec![];
        let mut player_glucose = 0.0;
        let suppressions = [true, false, false]; // strand 0 suppressed

        let consumed = auto_consume(
            &particles,
            &mut zymases,
            &mrna_xs,
            &mrna_ys,
            &mut mrna_progress,
            &mut mrna_processing,
            &mut mrna_timers,
            &mut motors,
            None,
            &mut player_glucose,
            &suppressions,
        );
        assert!(consumed.is_empty());
        assert_eq!(mrna_progress[0], 0); // strand 0 untouched
    }
}
