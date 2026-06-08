use godot::prelude::*;
use rand::Rng;

const RESOURCE_RADIUS: f32 = 8.0;
const WORLD_BOUND: f32 = 500.0;
const SPAWN_BOUND: f32 = 480.0;
const MIN_RESPAWN_DIST: f32 = 100.0;
const DRIFT_SPEED: f32 = 5.0;

const MOVEMENT_ATP_COST: f32 = 0.5;
const SIZE_METABOLISM_FACTOR: f32 = 0.02;
const FERMENTATION_YIELD: f32 = 2.0;
const STARTING_ATP: f32 = 5.0;
const MAX_ATP: f32 = 5.0;
const PICK_RADIUS: f32 = 18.0;
const MIN_RADIUS: f32 = 15.0;
const VELOCITY_THRESHOLD: f32 = 5.0;
const INTERIOR_RADIUS: f32 = 200.0;
const ENZYME_RADIUS: f32 = 15.0;
const ENZYME_COLLISION_DIST: f32 = 20.0;
const MOTOR_COLLISION_DIST: f32 = 25.0;
const MOTOR_ANGLE: f32 = 0.0;
const MOTOR_MEMBRANE_RADIUS: f32 = INTERIOR_RADIUS;
const MRNA_COUNT: usize = 3;
const MRNA_DIST: f32 = 70.0;
const MRNA_COLLISION_DIST: f32 = 20.0;
const MRNA_REQUIRED: [i32; MRNA_COUNT] = [8, 7, 5];
const GLUCOSE_MIN_SEP: f32 = 12.0;
const MRNA_ANGLES: [f32; MRNA_COUNT] = [
    150.0 * std::f32::consts::PI / 180.0,  // enzyme — upper-left
    270.0 * std::f32::consts::PI / 180.0,  // motor  — bottom
    30.0 * std::f32::consts::PI / 180.0,   // membrane — upper-right
];

struct InteriorParticle {
    x: f32,
    y: f32,
    resource_type: i32,
}

struct Resource {
    x: f32,
    y: f32,
    resource_type: i32, // 0 = glucose, 1 = amino acid
    amount: f32,
}

struct Motor {
    x: f32,
    y: f32,
    charge: f32,
}

#[derive(GodotClass)]
#[class(base=Node)]
pub struct Simulation {
    base: Base<Node>,

    #[var]
    player_x: f32,
    #[var]
    player_y: f32,
    #[var]
    player_radius: f32,

    #[var]
    player_glucose: f32,
    #[var]
    player_amino_acids: f32,

    #[var]
    resource_xs: PackedFloat32Array,
    #[var]
    resource_ys: PackedFloat32Array,
    #[var]
    resource_types: PackedInt32Array,

    #[var]
    resource_radius: f32,
    #[var]
    world_bound: f32,

    #[var]
    player_atp: f32,
    #[var]
    player_max_atp: f32,
    #[var]
    player_alive: bool,
    #[var]
    player_energy_ratio: f32,

    velocity_x: f32,
    velocity_y: f32,

    resources: Vec<Resource>,

    #[var]
    interior_view: bool,
    interior_particles: Vec<InteriorParticle>,
    #[var]
    interior_xs: PackedFloat32Array,
    #[var]
    interior_ys: PackedFloat32Array,
    #[var]
    interior_types: PackedInt32Array,
    #[var]
    interior_radius: f32,

    enzyme_x: f32,
    enzyme_y: f32,

    motors: Vec<Motor>,
    #[var]
    motor_xs: PackedFloat32Array,
    #[var]
    motor_ys: PackedFloat32Array,
    #[var]
    motor_charges: PackedFloat32Array,
    #[var]
    motor_count: i32,
    dragged_motor_index: Option<usize>,
    dragged_mrna_index: Option<usize>,
    dragged_enzyme: bool,

    #[var]
    enzyme_interior_x: f32,
    #[var]
    enzyme_interior_y: f32,
    #[var]
    enzyme_interior_radius: f32,
    #[var]
    motor_interior_x: f32,
    #[var]
    motor_interior_y: f32,
    #[var]
    atp_particle_count: i32,
    #[var]
    glucose_particle_count: i32,
    #[var]
    motor_charge_display: f32,
    #[var]
    mrna_xs: PackedFloat32Array,
    #[var]
    mrna_ys: PackedFloat32Array,
    #[var]
    mrna_types: PackedInt32Array,

    mrna_progress_internal: [i32; MRNA_COUNT],
    #[var]
    mrna_progress: PackedInt32Array,
    #[var]
    mrna_required: PackedInt32Array,
    #[var]
    amino_acid_particle_count: i32,

    dragged_particle_index: Option<usize>,
    #[var]
    dragged_particle_x: f32,
    #[var]
    dragged_particle_y: f32,
    #[var]
    drag_active: bool,
    #[var]
    dragged_particle_type: i32,
}

#[godot_api]
impl INode for Simulation {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            player_x: 0.0,
            player_y: 0.0,
            player_radius: MIN_RADIUS,
            player_glucose: 0.0,
            player_amino_acids: 0.0,
            resource_xs: PackedFloat32Array::new(),
            resource_ys: PackedFloat32Array::new(),
            resource_types: PackedInt32Array::new(),
            resource_radius: RESOURCE_RADIUS,
            world_bound: WORLD_BOUND,
            player_atp: STARTING_ATP,
            player_max_atp: MAX_ATP,
            player_alive: true,
            player_energy_ratio: STARTING_ATP / MAX_ATP,
            velocity_x: 0.0,
            velocity_y: 0.0,
            resources: Vec::new(),
            interior_view: false,
            interior_particles: Vec::new(),
            interior_xs: PackedFloat32Array::new(),
            interior_ys: PackedFloat32Array::new(),
            interior_types: PackedInt32Array::new(),
            interior_radius: INTERIOR_RADIUS,

            enzyme_x: 0.0,
            enzyme_y: 0.0,

            motors: vec![Motor {
                x: MOTOR_ANGLE.cos() * MOTOR_MEMBRANE_RADIUS,
                y: MOTOR_ANGLE.sin() * MOTOR_MEMBRANE_RADIUS,
                charge: STARTING_ATP,
            }],
            motor_xs: PackedFloat32Array::new(),
            motor_ys: PackedFloat32Array::new(),
            motor_charges: PackedFloat32Array::new(),
            motor_count: 1,
            dragged_motor_index: None,
            dragged_mrna_index: None,
            dragged_enzyme: false,

            enzyme_interior_x: 0.0,
            enzyme_interior_y: 0.0,
            enzyme_interior_radius: ENZYME_RADIUS,
            motor_interior_x: MOTOR_ANGLE.cos() * INTERIOR_RADIUS,
            motor_interior_y: MOTOR_ANGLE.sin() * INTERIOR_RADIUS,
            atp_particle_count: 0,
            glucose_particle_count: 0,
            motor_charge_display: STARTING_ATP,
            mrna_xs: PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]),
            mrna_ys: PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]),
            mrna_types: PackedInt32Array::from(&[0i32, 1, 2][..]),

            mrna_progress_internal: [0; MRNA_COUNT],
            mrna_progress: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),
            mrna_required: PackedInt32Array::from(&MRNA_REQUIRED[..]),
            amino_acid_particle_count: 0,

            dragged_particle_index: None,
            dragged_particle_x: 0.0,
            dragged_particle_y: 0.0,
            drag_active: false,
            dragged_particle_type: -1,
        }
    }
}

#[godot_api]
impl Simulation {
    #[func]
    fn toggle_interior_view(&mut self) {
        self.interior_view = !self.interior_view;
    }

    fn sync_interior_arrays(&mut self) {
        self.interior_xs = PackedFloat32Array::new();
        self.interior_ys = PackedFloat32Array::new();
        self.interior_types = PackedInt32Array::new();

        for p in &self.interior_particles {
            self.interior_xs.push(p.x);
            self.interior_ys.push(p.y);
            self.interior_types.push(p.resource_type);
        }
    }

    fn sync_motor_arrays(&mut self) {
        self.motor_xs = PackedFloat32Array::new();
        self.motor_ys = PackedFloat32Array::new();
        self.motor_charges = PackedFloat32Array::new();

        for m in &self.motors {
            self.motor_xs.push(m.x);
            self.motor_ys.push(m.y);
            self.motor_charges.push(m.charge);
        }
        self.motor_count = self.motors.len() as i32;

        // Backward compat: first motor position for exterior view
        if let Some(first) = self.motors.first() {
            self.motor_interior_x = first.x;
            self.motor_interior_y = first.y;
        }

        // Aggregate charge for HUD
        let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
        self.motor_charge_display = total_charge;
        self.player_max_atp = MAX_ATP * self.motors.len() as f32;
    }

    #[func]
    fn move_player(&mut self, dx: f32, dy: f32) {
        let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
        if total_charge <= 0.0 {
            return;
        }
        let powered_motors = self.motors.iter().filter(|m| m.charge > 0.0).count();
        let speed = powered_motors as f32 * 10.0;
        self.velocity_x += dx * speed;
        self.velocity_y += dy * speed;
    }

    #[func]
    fn spawn_resources(&mut self, count: i32) {
        let mut rng = rand::thread_rng();
        for _ in 0..count {
            let x = rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND);
            let y = rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND);
            let resource_type = rng.gen_range(0..2);
            let amount = 1.0;
            self.resources.push(Resource {
                x,
                y,
                resource_type,
                amount,
            });
        }
        self.sync_packed_arrays();
    }

    fn sync_packed_arrays(&mut self) {
        self.resource_xs = PackedFloat32Array::new();
        self.resource_ys = PackedFloat32Array::new();
        self.resource_types = PackedInt32Array::new();

        for r in &self.resources {
            self.resource_xs.push(r.x);
            self.resource_ys.push(r.y);
            self.resource_types.push(r.resource_type);
        }
    }

    fn respawn_resource(&mut self, index: usize) {
        let mut rng = rand::thread_rng();
        let mut x = 0.0;
        let mut y = 0.0;
        for _ in 0..100 {
            x = rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND);
            y = rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND);
            let dx = x - self.player_x;
            let dy = y - self.player_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > MIN_RESPAWN_DIST {
                break;
            }
        }
        self.resources[index].x = x;
        self.resources[index].y = y;
        self.resources[index].resource_type = rng.gen_range(0..2);
        self.resources[index].amount = 1.0;
    }

    #[func]
    fn tick(&mut self, delta: f64) {
        if !self.player_alive {
            return;
        }

        let dt = delta as f32;
        let damping = 0.9_f32.powf(dt * 60.0);
        let bound = WORLD_BOUND;

        // Update player position
        self.player_x += self.velocity_x * dt;
        self.player_y += self.velocity_y * dt;

        self.velocity_x *= damping;
        self.velocity_y *= damping;

        // Bounce off world edges
        if self.player_x > bound {
            self.player_x = bound;
            self.velocity_x = -self.velocity_x * 0.5;
        } else if self.player_x < -bound {
            self.player_x = -bound;
            self.velocity_x = -self.velocity_x * 0.5;
        }

        if self.player_y > bound {
            self.player_y = bound;
            self.velocity_y = -self.velocity_y * 0.5;
        } else if self.player_y < -bound {
            self.player_y = -bound;
            self.velocity_y = -self.velocity_y * 0.5;
        }

        // Drift resources
        let mut rng = rand::thread_rng();
        for r in &mut self.resources {
            r.x += rng.gen_range(-DRIFT_SPEED..DRIFT_SPEED) * dt;
            r.y += rng.gen_range(-DRIFT_SPEED..DRIFT_SPEED) * dt;
            r.x = r.x.clamp(-bound, bound);
            r.y = r.y.clamp(-bound, bound);
        }

        // Check absorption
        let pickup_dist = self.player_radius + RESOURCE_RADIUS;
        let pickup_dist_sq = pickup_dist * pickup_dist;
        let mut respawn_indices = Vec::new();

        for (i, r) in self.resources.iter().enumerate() {
            let dx = self.player_x - r.x;
            let dy = self.player_y - r.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < pickup_dist_sq {
                // Compute entry point on membrane from resource direction
                let dir_x = r.x - self.player_x;
                let dir_y = r.y - self.player_y;
                let dir_len = (dir_x * dir_x + dir_y * dir_y).sqrt();
                let (entry_x, entry_y) = if dir_len > 0.001 {
                    let nx = dir_x / dir_len;
                    let ny = dir_y / dir_len;
                    (nx * INTERIOR_RADIUS * 0.85, ny * INTERIOR_RADIUS * 0.85)
                } else {
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    (angle.cos() * INTERIOR_RADIUS * 0.85, angle.sin() * INTERIOR_RADIUS * 0.85)
                };

                match r.resource_type {
                    0 => {
                        self.player_glucose += r.amount;
                        // Push existing glucose away from entry point
                        for p in self.interior_particles.iter_mut() {
                            if p.resource_type != 0 { continue; }
                            let dx = p.x - entry_x;
                            let dy = p.y - entry_y;
                            let d = (dx * dx + dy * dy).sqrt();
                            if d < GLUCOSE_MIN_SEP {
                                if d < 0.001 {
                                    let a = rng.gen_range(0.0..std::f32::consts::TAU);
                                    p.x = entry_x + a.cos() * GLUCOSE_MIN_SEP;
                                    p.y = entry_y + a.sin() * GLUCOSE_MIN_SEP;
                                } else {
                                    let nx = dx / d;
                                    let ny = dy / d;
                                    p.x = entry_x + nx * GLUCOSE_MIN_SEP;
                                    p.y = entry_y + ny * GLUCOSE_MIN_SEP;
                                }
                            }
                        }
                        self.interior_particles.push(InteriorParticle {
                            x: entry_x,
                            y: entry_y,
                            resource_type: 0,
                        });
                    }
                    1 => {
                        self.player_amino_acids += r.amount;
                        self.interior_particles.push(InteriorParticle {
                            x: entry_x,
                            y: entry_y,
                            resource_type: 1,
                        });
                    }
                    _ => {}
                }
                respawn_indices.push(i);
            }
        }

        for i in respawn_indices {
            self.respawn_resource(i);
        }

        // Movement-based metabolism (distribute across all motors)
        let speed = (self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y).sqrt();
        if speed > VELOCITY_THRESHOLD {
            let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
            if total_charge > 0.0 {
                let cost = MOVEMENT_ATP_COST + (self.player_radius - MIN_RADIUS) * SIZE_METABOLISM_FACTOR;
                let cost_per_frame = cost * dt;
                // Distribute cost evenly across motors that have charge
                let charged_motors: Vec<usize> = self.motors.iter().enumerate()
                    .filter(|(_, m)| m.charge > 0.0)
                    .map(|(i, _)| i)
                    .collect();
                if !charged_motors.is_empty() {
                    let cost_each = cost_per_frame / charged_motors.len() as f32;
                    for &i in &charged_motors {
                        self.motors[i].charge = (self.motors[i].charge - cost_each).max(0.0);
                    }
                }
            } else {
                // No motor charge — dampen velocity (lose propulsion)
                self.velocity_x *= 0.95_f32.powf(dt * 60.0);
                self.velocity_y *= 0.95_f32.powf(dt * 60.0);
            }
        }

        // Sync dragged particle position from GDScript each frame
        if let Some(idx) = self.dragged_particle_index {
            if idx < self.interior_particles.len() {
                self.interior_particles[idx].x = self.dragged_particle_x;
                self.interior_particles[idx].y = self.dragged_particle_y;
            }
        }

        // Sync dragged enzyme position
        if self.dragged_enzyme {
            self.enzyme_interior_x = self.dragged_particle_x;
            self.enzyme_interior_y = self.dragged_particle_y;
            self.enzyme_x = self.dragged_particle_x;
            self.enzyme_y = self.dragged_particle_y;
        }

        // Sync dragged mRNA position
        if let Some(mi) = self.dragged_mrna_index {
            if mi < MRNA_COUNT {
                self.mrna_xs[mi as usize] = self.dragged_particle_x;
                self.mrna_ys[mi as usize] = self.dragged_particle_y;
            }
        }

        // Sync dragged motor position — constrain to membrane edge
        if let Some(mi) = self.dragged_motor_index {
            if mi < self.motors.len() {
                let angle = self.dragged_particle_y.atan2(self.dragged_particle_x);
                self.motors[mi].x = angle.cos() * MOTOR_MEMBRANE_RADIUS;
                self.motors[mi].y = angle.sin() * MOTOR_MEMBRANE_RADIUS;
                self.dragged_particle_x = self.motors[mi].x;
                self.dragged_particle_y = self.motors[mi].y;
            }
        }

        // Pairwise glucose separation
        let n = self.interior_particles.len();
        for i in 0..n {
            if self.interior_particles[i].resource_type != 0 { continue; }
            for j in (i + 1)..n {
                if self.interior_particles[j].resource_type != 0 { continue; }
                let dx = self.interior_particles[j].x - self.interior_particles[i].x;
                let dy = self.interior_particles[j].y - self.interior_particles[i].y;
                let d = (dx * dx + dy * dy).sqrt();
                if d < GLUCOSE_MIN_SEP && d > 0.001 {
                    let overlap = (GLUCOSE_MIN_SEP - d) * 0.5;
                    let nx = dx / d;
                    let ny = dy / d;
                    self.interior_particles[i].x -= nx * overlap;
                    self.interior_particles[i].y -= ny * overlap;
                    self.interior_particles[j].x += nx * overlap;
                    self.interior_particles[j].y += ny * overlap;
                } else if d <= 0.001 {
                    let a = rng.gen_range(0.0..std::f32::consts::TAU);
                    let half = GLUCOSE_MIN_SEP * 0.5;
                    self.interior_particles[i].x -= a.cos() * half;
                    self.interior_particles[i].y -= a.sin() * half;
                    self.interior_particles[j].x += a.cos() * half;
                    self.interior_particles[j].y += a.sin() * half;
                }
            }
        }

        // Count particles for HUD
        let mut atp_count = 0;
        let mut glucose_count = 0;
        let mut amino_count = 0;
        for p in &self.interior_particles {
            match p.resource_type {
                0 => glucose_count += 1,
                1 => amino_count += 1,
                2 => atp_count += 1,
                _ => {}
            }
        }
        self.atp_particle_count = atp_count;
        self.glucose_particle_count = glucose_count;
        self.amino_acid_particle_count = amino_count;

        // Death check: fully depleted when no motor charge AND no ATP/glucose particles
        let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
        if total_charge <= 0.0 && atp_count == 0 && glucose_count == 0 {
            self.player_alive = false;
        }

        // Backward-compatible energy ratio for WorldRenderer cell color
        self.player_atp = total_charge;
        self.player_energy_ratio = total_charge / self.player_max_atp.max(1.0);

        self.sync_packed_arrays();
        self.sync_interior_arrays();
        self.sync_motor_arrays();
        self.sync_mrna_progress();
    }

    fn sync_mrna_progress(&mut self) {
        self.mrna_progress = PackedInt32Array::new();
        for i in 0..MRNA_COUNT {
            self.mrna_progress.push(self.mrna_progress_internal[i]);
        }
    }

    #[func]
    fn try_pick_particle(&mut self, x: f32, y: f32) -> bool {
        let mut best_dist = PICK_RADIUS;

        // Track which kind of thing is closest
        enum PickTarget {
            Particle(usize),
            Motor(usize),
            Enzyme,
            Mrna(usize),
        }
        let mut best: Option<PickTarget> = None;

        // Check particles
        for (i, p) in self.interior_particles.iter().enumerate() {
            let dx = p.x - x;
            let dy = p.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Particle(i));
            }
        }

        // Check motors
        for (i, m) in self.motors.iter().enumerate() {
            let dx = m.x - x;
            let dy = m.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Motor(i));
            }
        }

        // Check enzyme
        {
            let dx = self.enzyme_interior_x - x;
            let dy = self.enzyme_interior_y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Enzyme);
            }
        }

        // Check mRNA strands
        for i in 0..MRNA_COUNT {
            let dx = self.mrna_xs[i as usize] - x;
            let dy = self.mrna_ys[i as usize] - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Mrna(i));
            }
        }

        // Clear all drag states
        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.dragged_enzyme = false;
        self.dragged_mrna_index = None;

        match best {
            Some(PickTarget::Motor(mi)) => {
                self.dragged_motor_index = Some(mi);
                self.dragged_particle_x = self.motors[mi].x;
                self.dragged_particle_y = self.motors[mi].y;
                self.drag_active = true;
                self.dragged_particle_type = 3;
                true
            }
            Some(PickTarget::Particle(idx)) => {
                self.dragged_particle_index = Some(idx);
                self.dragged_particle_x = self.interior_particles[idx].x;
                self.dragged_particle_y = self.interior_particles[idx].y;
                self.drag_active = true;
                self.dragged_particle_type = self.interior_particles[idx].resource_type;
                true
            }
            Some(PickTarget::Enzyme) => {
                self.dragged_enzyme = true;
                self.dragged_particle_x = self.enzyme_interior_x;
                self.dragged_particle_y = self.enzyme_interior_y;
                self.drag_active = true;
                self.dragged_particle_type = 4;
                true
            }
            Some(PickTarget::Mrna(i)) => {
                self.dragged_mrna_index = Some(i);
                self.dragged_particle_x = self.mrna_xs[i as usize];
                self.dragged_particle_y = self.mrna_ys[i as usize];
                self.drag_active = true;
                self.dragged_particle_type = 5;
                true
            }
            None => false,
        }
    }

    #[func]
    fn drag_particle(&mut self, x: f32, y: f32) {
        if self.dragged_particle_index.is_none()
            && self.dragged_motor_index.is_none()
            && !self.dragged_enzyme
            && self.dragged_mrna_index.is_none()
        {
            return;
        }

        if self.dragged_motor_index.is_some() {
            // Motor drag: constrain to membrane edge
            let angle = y.atan2(x);
            self.dragged_particle_x = angle.cos() * MOTOR_MEMBRANE_RADIUS;
            self.dragged_particle_y = angle.sin() * MOTOR_MEMBRANE_RADIUS;
        } else {
            // Clamp to membrane interior (particles, enzyme, mRNA)
            let dist = (x * x + y * y).sqrt();
            let max_r = INTERIOR_RADIUS * 0.9;
            if dist > max_r {
                let scale = max_r / dist;
                self.dragged_particle_x = x * scale;
                self.dragged_particle_y = y * scale;
            } else {
                self.dragged_particle_x = x;
                self.dragged_particle_y = y;
            }
        }
    }

    #[func]
    fn drop_particle(&mut self, x: f32, y: f32) {
        // Motor drop: just finalize position
        if let Some(mi) = self.dragged_motor_index {
            if mi < self.motors.len() {
                let angle = y.atan2(x);
                self.motors[mi].x = angle.cos() * MOTOR_MEMBRANE_RADIUS;
                self.motors[mi].y = angle.sin() * MOTOR_MEMBRANE_RADIUS;
            }
            self.dragged_motor_index = None;
            self.dragged_particle_index = None;
            self.drag_active = false;
            self.dragged_particle_type = -1;
            return;
        }

        // Enzyme drop: finalize position
        if self.dragged_enzyme {
            self.enzyme_interior_x = self.dragged_particle_x;
            self.enzyme_interior_y = self.dragged_particle_y;
            self.enzyme_x = self.dragged_particle_x;
            self.enzyme_y = self.dragged_particle_y;
            self.dragged_enzyme = false;
            self.drag_active = false;
            self.dragged_particle_type = -1;
            return;
        }

        // mRNA drop: finalize position
        if let Some(mi) = self.dragged_mrna_index {
            if mi < MRNA_COUNT {
                self.mrna_xs[mi as usize] = self.dragged_particle_x;
                self.mrna_ys[mi as usize] = self.dragged_particle_y;
            }
            self.dragged_mrna_index = None;
            self.drag_active = false;
            self.dragged_particle_type = -1;
            return;
        }

        let idx = match self.dragged_particle_index {
            Some(i) => i,
            None => return,
        };
        if idx >= self.interior_particles.len() {
            self.dragged_particle_index = None;
            self.drag_active = false;
            self.dragged_particle_type = -1;
            return;
        }

        let p_type = self.interior_particles[idx].resource_type;
        let mut rng = rand::thread_rng();

        // Check drop targets
        if p_type == 0 {
            // Glucose on enzyme?
            let dx = x - self.enzyme_x;
            let dy = y - self.enzyme_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < ENZYME_COLLISION_DIST {
                // Convert glucose → ATP
                self.player_glucose -= 1.0;
                self.interior_particles[idx].resource_type = 2;
                // Spawn extra ATP particles for fermentation yield
                let extra = (FERMENTATION_YIELD as i32) - 1;
                for _ in 0..extra {
                    self.interior_particles.push(InteriorParticle {
                        x: self.enzyme_x + rng.gen_range(-10.0..10.0),
                        y: self.enzyme_y + rng.gen_range(-10.0..10.0),
                        resource_type: 2,
                    });
                }
            }
        } else if p_type == 1 {
            // Amino acid on mRNA?
            for m in 0..MRNA_COUNT {
                let mrna_x = self.mrna_xs[m as usize];
                let mrna_y = self.mrna_ys[m as usize];
                let dx = x - mrna_x;
                let dy = y - mrna_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < MRNA_COLLISION_DIST && self.mrna_progress_internal[m] < MRNA_REQUIRED[m] {
                    self.mrna_progress_internal[m] += 1;
                    self.interior_particles.swap_remove(idx);

                    // Check if motor mRNA (index 1) completed — build new motor
                    if m == 1 && self.mrna_progress_internal[1] >= MRNA_REQUIRED[1] {
                        let spawn_angle = MRNA_ANGLES[1]; // 270° bottom
                        self.motors.push(Motor {
                            x: spawn_angle.cos() * MOTOR_MEMBRANE_RADIUS,
                            y: spawn_angle.sin() * MOTOR_MEMBRANE_RADIUS,
                            charge: 0.0,
                        });
                        self.mrna_progress_internal[1] = 0;
                    }

                    break;
                }
            }
        } else if p_type == 2 {
            // ATP on motor? Find nearest motor within range
            let mut best_motor: Option<usize> = None;
            let mut best_motor_dist = MOTOR_COLLISION_DIST;
            for (i, motor) in self.motors.iter().enumerate() {
                let dx = x - motor.x;
                let dy = y - motor.y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist < best_motor_dist && motor.charge < MAX_ATP {
                    best_motor_dist = dist;
                    best_motor = Some(i);
                }
            }
            if let Some(mi) = best_motor {
                self.interior_particles.swap_remove(idx);
                self.motors[mi].charge = (self.motors[mi].charge + 1.0).min(MAX_ATP);
            }
        }

        // Clear drag state
        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.drag_active = false;
        self.dragged_particle_type = -1;
    }

    #[func]
    fn get_nearest_particle_index(&self, x: f32, y: f32) -> i32 {
        let mut best_idx: i32 = -1;
        let mut best_dist = PICK_RADIUS;
        for (i, p) in self.interior_particles.iter().enumerate() {
            let dx = p.x - x;
            let dy = p.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best_idx = i as i32;
            }
        }
        best_idx
    }

    #[func]
    fn cancel_drag(&mut self) {
        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.dragged_enzyme = false;
        self.dragged_mrna_index = None;
        self.drag_active = false;
        self.dragged_particle_type = -1;
    }

    #[func]
    fn restart(&mut self) {
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.player_radius = MIN_RADIUS;
        self.player_atp = STARTING_ATP;
        self.player_max_atp = MAX_ATP;
        self.player_alive = true;
        self.player_glucose = 0.0;
        self.player_amino_acids = 0.0;
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
        self.player_energy_ratio = STARTING_ATP / MAX_ATP;
        self.motor_charge_display = STARTING_ATP;
        self.atp_particle_count = 0;
        self.glucose_particle_count = 0;
        self.amino_acid_particle_count = 0;
        self.mrna_progress_internal = [0; MRNA_COUNT];
        self.mrna_progress = PackedInt32Array::from(&[0i32; MRNA_COUNT][..]);
        self.interior_view = false;
        self.interior_particles.clear();
        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.dragged_enzyme = false;
        self.dragged_mrna_index = None;
        self.dragged_particle_x = 0.0;
        self.dragged_particle_y = 0.0;
        self.drag_active = false;
        self.dragged_particle_type = -1;
        self.enzyme_interior_x = 0.0;
        self.enzyme_interior_y = 0.0;
        self.enzyme_x = 0.0;
        self.enzyme_y = 0.0;
        self.mrna_xs = PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]);
        self.mrna_ys = PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]);
        self.mrna_types = PackedInt32Array::from(&[0i32, 1, 2][..]);

        // Reset motors to single motor at angle 0
        self.motors = vec![Motor {
            x: MOTOR_ANGLE.cos() * MOTOR_MEMBRANE_RADIUS,
            y: MOTOR_ANGLE.sin() * MOTOR_MEMBRANE_RADIUS,
            charge: STARTING_ATP,
        }];

        for i in 0..self.resources.len() {
            self.respawn_resource(i);
        }
        self.sync_packed_arrays();
        self.sync_interior_arrays();
        self.sync_motor_arrays();
    }
}
