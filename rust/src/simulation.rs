use godot::prelude::*;
use rand::Rng;

const RESOURCE_RADIUS: f32 = 8.0;
const WORLD_BOUND: f32 = 500.0;
const SPAWN_BOUND: f32 = 480.0;
const MIN_RESPAWN_DIST: f32 = 100.0;
const DRIFT_SPEED: f32 = 5.0;

const MOVEMENT_ATP_COST: f32 = 0.5;
const SIZE_METABOLISM_FACTOR: f32 = 0.02;
const GLUCOSE_TO_ATP: f32 = 4.0;
const STARTING_ATP: f32 = 20.0;
const MAX_ATP: f32 = 50.0;
const AMINO_ACID_GROWTH: f32 = 0.5;
const MIN_RADIUS: f32 = 15.0;
const MAX_RADIUS: f32 = 40.0;
const VELOCITY_THRESHOLD: f32 = 5.0;

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
        }
    }
}

#[godot_api]
impl Simulation {
    #[func]
    fn move_player(&mut self, dx: f32, dy: f32) {
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
                        self.player_atp = (self.player_atp + r.amount * GLUCOSE_TO_ATP).min(MAX_ATP);
                    }
                    1 => {
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

        // Movement-based metabolism
        let speed = (self.velocity_x * self.velocity_x + self.velocity_y * self.velocity_y).sqrt();
        if speed > VELOCITY_THRESHOLD {
            let cost = MOVEMENT_ATP_COST + (self.player_radius - MIN_RADIUS) * SIZE_METABOLISM_FACTOR;
            self.player_atp -= cost * dt;
        }

        // Death check
        if self.player_atp <= 0.0 {
            self.player_atp = 0.0;
            self.player_alive = false;
        }

        // Update energy ratio
        self.player_energy_ratio = self.player_atp / self.player_max_atp;

        self.sync_packed_arrays();
    }

    #[func]
    fn restart(&mut self) {
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.player_radius = MIN_RADIUS;
        self.player_atp = STARTING_ATP;
        self.player_alive = true;
        self.player_glucose = 0.0;
        self.player_amino_acids = 0.0;
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
        self.player_energy_ratio = STARTING_ATP / MAX_ATP;

        for i in 0..self.resources.len() {
            self.respawn_resource(i);
        }
        self.sync_packed_arrays();
    }
}
