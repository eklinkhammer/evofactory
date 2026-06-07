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
const AMINO_ACID_GROWTH: f32 = 0.5;
const MIN_RADIUS: f32 = 15.0;
const MAX_RADIUS: f32 = 40.0;
const VELOCITY_THRESHOLD: f32 = 5.0;
const INTERIOR_RADIUS: f32 = 200.0;
const ENZYME_RADIUS: f32 = 15.0;
const ENZYME_COLLISION_DIST: f32 = 20.0;
const MOTOR_COLLISION_DIST: f32 = 25.0;
const MOTOR_ANGLE: f32 = 0.0;
const MRNA_COUNT: usize = 3;
const MRNA_DIST: f32 = 70.0;
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
    motor_charge: f32,

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
            motor_charge: STARTING_ATP,

            enzyme_interior_x: 0.0,
            enzyme_interior_y: 0.0,
            enzyme_interior_radius: ENZYME_RADIUS,
            motor_interior_x: MOTOR_ANGLE.cos() * INTERIOR_RADIUS * 0.9,
            motor_interior_y: MOTOR_ANGLE.sin() * INTERIOR_RADIUS * 0.9,
            atp_particle_count: 0,
            glucose_particle_count: 0,
            motor_charge_display: STARTING_ATP,
            mrna_xs: PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]),
            mrna_ys: PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]),
            mrna_types: PackedInt32Array::from(&[0i32, 1, 2][..]),

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

    #[func]
    fn move_player(&mut self, dx: f32, dy: f32) {
        if self.motor_charge <= 0.0 {
            return;
        }
        let speed = 200.0;
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
                match r.resource_type {
                    0 => {
                        self.player_glucose += r.amount;
                        // Spawn interior glucose particle at random position
                        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                        let dist = rng.gen_range(0.0..INTERIOR_RADIUS * 0.85);
                        self.interior_particles.push(InteriorParticle {
                            x: angle.cos() * dist,
                            y: angle.sin() * dist,
                            resource_type: 0,
                        });
                    }
                    1 => {
                        // Amino acids auto-process: growth applies on pickup, no interior particle
                        self.player_amino_acids += r.amount;
                        self.player_radius = (MIN_RADIUS + self.player_amino_acids * AMINO_ACID_GROWTH).min(MAX_RADIUS);
                    }
                    _ => {}
                }
                respawn_indices.push(i);
            }
        }

        for i in respawn_indices {
            self.respawn_resource(i);
        }

        // Movement-based metabolism (uses motor_charge)
        let speed = (self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y).sqrt();
        if speed > VELOCITY_THRESHOLD {
            if self.motor_charge > 0.0 {
                let cost = MOVEMENT_ATP_COST + (self.player_radius - MIN_RADIUS) * SIZE_METABOLISM_FACTOR;
                self.motor_charge -= cost * dt;
                if self.motor_charge < 0.0 {
                    self.motor_charge = 0.0;
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

        // Count particles for HUD
        let mut atp_count = 0;
        let mut glucose_count = 0;
        for p in &self.interior_particles {
            match p.resource_type {
                0 => glucose_count += 1,
                2 => atp_count += 1,
                _ => {}
            }
        }
        self.atp_particle_count = atp_count;
        self.glucose_particle_count = glucose_count;

        // Death check: fully depleted when no motor charge AND no ATP/glucose particles
        if self.motor_charge <= 0.0 && atp_count == 0 && glucose_count == 0 {
            self.motor_charge = 0.0;
            self.player_alive = false;
        }

        // Backward-compatible energy ratio for WorldRenderer cell color
        self.player_atp = self.motor_charge;
        self.player_energy_ratio = self.motor_charge / self.player_max_atp;
        self.motor_charge_display = self.motor_charge;

        self.sync_packed_arrays();
        self.sync_interior_arrays();
    }

    #[func]
    fn try_pick_particle(&mut self, x: f32, y: f32) -> bool {
        let mut best_idx: Option<usize> = None;
        let mut best_dist = PICK_RADIUS;
        for (i, p) in self.interior_particles.iter().enumerate() {
            // Only glucose (0) and ATP (2) are draggable
            if p.resource_type != 0 && p.resource_type != 2 {
                continue;
            }
            let dx = p.x - x;
            let dy = p.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best_idx = Some(i);
            }
        }
        if let Some(idx) = best_idx {
            self.dragged_particle_index = Some(idx);
            self.dragged_particle_x = self.interior_particles[idx].x;
            self.dragged_particle_y = self.interior_particles[idx].y;
            self.drag_active = true;
            self.dragged_particle_type = self.interior_particles[idx].resource_type;
            true
        } else {
            false
        }
    }

    #[func]
    fn drag_particle(&mut self, x: f32, y: f32) {
        if self.dragged_particle_index.is_none() {
            return;
        }
        // Clamp to membrane interior
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

    #[func]
    fn drop_particle(&mut self, x: f32, y: f32) {
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
        } else if p_type == 2 {
            // ATP on motor?
            let motor_x = MOTOR_ANGLE.cos() * INTERIOR_RADIUS * 0.9;
            let motor_y = MOTOR_ANGLE.sin() * INTERIOR_RADIUS * 0.9;
            let dx = x - motor_x;
            let dy = y - motor_y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < MOTOR_COLLISION_DIST && self.motor_charge < MAX_ATP {
                self.interior_particles.swap_remove(idx);
                self.motor_charge = (self.motor_charge + 1.0).min(MAX_ATP);
            }
        }

        // Clear drag state
        self.dragged_particle_index = None;
        self.drag_active = false;
        self.dragged_particle_type = -1;
    }

    #[func]
    fn get_nearest_particle_index(&self, x: f32, y: f32) -> i32 {
        let mut best_idx: i32 = -1;
        let mut best_dist = PICK_RADIUS;
        for (i, p) in self.interior_particles.iter().enumerate() {
            if p.resource_type != 0 && p.resource_type != 2 {
                continue;
            }
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
        self.drag_active = false;
        self.dragged_particle_type = -1;
    }

    #[func]
    fn restart(&mut self) {
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.player_radius = MIN_RADIUS;
        self.motor_charge = STARTING_ATP;
        self.player_atp = STARTING_ATP;
        self.player_alive = true;
        self.player_glucose = 0.0;
        self.player_amino_acids = 0.0;
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
        self.player_energy_ratio = STARTING_ATP / MAX_ATP;
        self.motor_charge_display = STARTING_ATP;
        self.atp_particle_count = 0;
        self.glucose_particle_count = 0;
        self.interior_view = false;
        self.interior_particles.clear();
        self.dragged_particle_index = None;
        self.dragged_particle_x = 0.0;
        self.dragged_particle_y = 0.0;
        self.drag_active = false;
        self.dragged_particle_type = -1;
        self.mrna_xs = PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]);
        self.mrna_ys = PackedFloat32Array::from(&MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]);
        self.mrna_types = PackedInt32Array::from(&[0i32, 1, 2][..]);

        for i in 0..self.resources.len() {
            self.respawn_resource(i);
        }
        self.sync_packed_arrays();
        self.sync_interior_arrays();
    }
}
