use godot::prelude::*;
use rand::Rng;

use crate::types::{InteriorParticle, Zymase, Motor, INTERIOR_RADIUS, MAX_ATP, MOTOR_MEMBRANE_RADIUS, RESOURCE_RADIUS, MRNA_COUNT, CAPACITY_SCALE, MIN_RADIUS};
use crate::crafting::{self, CraftOutput};
use crate::interior;
use crate::rules::{self, Rule};

type CellResource = crate::types::Resource;

const WORLD_BOUND: f32 = 500.0;
const SPAWN_BOUND: f32 = 480.0;
const MIN_RESPAWN_DIST: f32 = 100.0;
const DRIFT_SPEED: f32 = 5.0;
const START_AREA_RADIUS: f32 = 150.0;
const START_AREA_WEIGHT: f32 = 3.0;

const MOVEMENT_ATP_COST: f32 = 0.5;
const SIZE_METABOLISM_FACTOR: f32 = 0.02;
const STARTING_ATP: f32 = 5.0;
const PICK_RADIUS: f32 = 18.0;
const VELOCITY_THRESHOLD: f32 = 5.0;
const ZYMASE_RADIUS: f32 = 15.0;
const MOTOR_ANGLE: f32 = 0.0;
const MRNA_DIST: f32 = 70.0;

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

    resources: Vec<CellResource>,

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

    zymases: Vec<Zymase>,
    #[var]
    zymase_xs: PackedFloat32Array,
    #[var]
    zymase_ys: PackedFloat32Array,
    #[var]
    zymase_buffers: PackedInt32Array,
    #[var]
    zymase_processing_flags: PackedInt32Array,
    #[var]
    zymase_timers: PackedFloat32Array,
    #[var]
    zymase_count: i32,
    dragged_zymase_index: Option<usize>,

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

    #[var]
    zymase_interior_radius: f32,
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

    mrna_processing: [bool; MRNA_COUNT],
    mrna_timers: [f32; MRNA_COUNT],
    #[var]
    mrna_processing_flags: PackedInt32Array,
    #[var]
    mrna_timers_display: PackedFloat32Array,

    expansion_count: i32,

    rules: Vec<Rule>,
    current_suppressions: [bool; MRNA_COUNT],

    #[var]
    regulation_panel_open: bool,
    #[var]
    rule_count: i32,
    #[var]
    rule_descriptions: PackedStringArray,
    #[var]
    rule_firing: PackedInt32Array,
    #[var]
    rule_targets: PackedInt32Array,
    #[var]
    rule_limits: PackedInt32Array,
    #[var]
    mrna_suppressed: PackedInt32Array,

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

            zymases: vec![Zymase { x: 0.0, y: 0.0, buffer: 0, processing: false, timer: 0.0 }],
            zymase_xs: PackedFloat32Array::new(),
            zymase_ys: PackedFloat32Array::new(),
            zymase_buffers: PackedInt32Array::new(),
            zymase_processing_flags: PackedInt32Array::new(),
            zymase_timers: PackedFloat32Array::new(),
            zymase_count: 1,
            dragged_zymase_index: None,

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

            zymase_interior_radius: ZYMASE_RADIUS,
            motor_interior_x: MOTOR_ANGLE.cos() * INTERIOR_RADIUS,
            motor_interior_y: MOTOR_ANGLE.sin() * INTERIOR_RADIUS,
            atp_particle_count: 0,
            glucose_particle_count: 0,
            motor_charge_display: STARTING_ATP,
            mrna_xs: PackedFloat32Array::from(&crafting::MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]),
            mrna_ys: PackedFloat32Array::from(&crafting::MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]),
            mrna_types: PackedInt32Array::from(&[0i32, 1, 2][..]),

            mrna_progress_internal: [0; MRNA_COUNT],
            mrna_progress: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),
            mrna_required: PackedInt32Array::from(&crafting::MRNA_REQUIRED[..]),
            amino_acid_particle_count: 0,

            mrna_processing: [false; MRNA_COUNT],
            mrna_timers: [0.0; MRNA_COUNT],
            mrna_processing_flags: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),
            mrna_timers_display: PackedFloat32Array::from(&[0.0f32; MRNA_COUNT][..]),

            expansion_count: 0,

            rules: rules::default_rules(),
            current_suppressions: [false; MRNA_COUNT],

            regulation_panel_open: false,
            rule_count: 0,
            rule_descriptions: PackedStringArray::new(),
            rule_firing: PackedInt32Array::new(),
            rule_targets: PackedInt32Array::new(),
            rule_limits: PackedInt32Array::new(),
            mrna_suppressed: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),

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

    fn sync_zymase_arrays(&mut self) {
        self.zymase_xs = PackedFloat32Array::new();
        self.zymase_ys = PackedFloat32Array::new();
        self.zymase_buffers = PackedInt32Array::new();
        self.zymase_processing_flags = PackedInt32Array::new();
        self.zymase_timers = PackedFloat32Array::new();

        for e in &self.zymases {
            self.zymase_xs.push(e.x);
            self.zymase_ys.push(e.y);
            self.zymase_buffers.push(e.buffer);
            self.zymase_processing_flags.push(if e.processing { 1 } else { 0 });
            self.zymase_timers.push(e.timer);
        }
        self.zymase_count = self.zymases.len() as i32;
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
        let threshold = START_AREA_WEIGHT / (START_AREA_WEIGHT + 1.0);
        for _ in 0..count {
            let (x, y) = if rng.gen::<f32>() < threshold {
                // Spawn near origin (rejection sampling within circle)
                loop {
                    let cx = rng.gen_range(-START_AREA_RADIUS..START_AREA_RADIUS);
                    let cy = rng.gen_range(-START_AREA_RADIUS..START_AREA_RADIUS);
                    if cx * cx + cy * cy <= START_AREA_RADIUS * START_AREA_RADIUS {
                        break (cx, cy);
                    }
                }
            } else {
                (
                    rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND),
                    rng.gen_range(-SPAWN_BOUND..SPAWN_BOUND),
                )
            };
            let resource_type = rng.gen_range(0..2);
            let amount = 1.0;
            self.resources.push(CellResource {
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

    fn mrna_xs_slice(&self) -> Vec<f32> {
        (0..MRNA_COUNT).map(|i| self.mrna_xs[i] as f32).collect()
    }

    fn mrna_ys_slice(&self) -> Vec<f32> {
        (0..MRNA_COUNT).map(|i| self.mrna_ys[i] as f32).collect()
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
        let counts = interior::count_particles(&self.interior_particles);
        let max_glucose = interior::BASE_MAX_GLUCOSE
            + (CAPACITY_SCALE * (self.expansion_count as f32).sqrt()) as i32;
        let max_amino = interior::BASE_MAX_AMINO
            + (CAPACITY_SCALE * (self.expansion_count as f32).sqrt()) as i32;

        let absorption_events = interior::detect_absorptions(
            self.player_x,
            self.player_y,
            self.player_radius,
            &self.resources,
            counts.glucose,
            max_glucose,
            counts.amino_acid,
            max_amino,
            &mut rng,
        );

        for event in &absorption_events {
            match event.resource_type {
                0 => self.player_glucose += event.amount,
                1 => self.player_amino_acids += event.amount,
                _ => {}
            }
            interior::apply_absorption(&mut self.interior_particles, event, &mut rng);
        }

        // Respawn absorbed resources
        let respawn_indices: Vec<usize> = absorption_events.iter().map(|e| e.resource_index).collect();
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

        // Sync dragged zymase position
        if let Some(ei) = self.dragged_zymase_index {
            if ei < self.zymases.len() {
                self.zymases[ei].x = self.dragged_particle_x;
                self.zymases[ei].y = self.dragged_particle_y;
            }
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

        // Brownian diffusion for interior particles
        interior::diffuse(&mut self.interior_particles, self.dragged_particle_index, dt, &mut rng);

        // Pairwise separation for all interior particles
        interior::separate(&mut self.interior_particles, &mut rng);

        // Re-clamp particles to membrane after separation
        interior::clamp_to_membrane(&mut self.interior_particles);

        // Evaluate gene regulation rules
        self.current_suppressions = rules::evaluate_suppressions(
            &mut self.rules,
            self.motors.len(),
            self.zymases.len(),
            self.expansion_count,
        );

        // Auto-consumption: particles near their target organelles are consumed
        {
            let mrna_xs = self.mrna_xs_slice();
            let mrna_ys = self.mrna_ys_slice();
            let actually_consumed = crafting::auto_consume(
                &self.interior_particles,
                &mut self.zymases,
                &mrna_xs,
                &mrna_ys,
                &mut self.mrna_progress_internal,
                &mut self.mrna_processing,
                &mut self.mrna_timers,
                &mut self.motors,
                self.dragged_particle_index,
                &mut self.player_glucose,
                &self.current_suppressions,
            );

            // Remove consumed particles in reverse order (swap_remove safe)
            for &i in actually_consumed.iter().rev() {
                self.interior_particles.swap_remove(i);
                // Fix up dragged_particle_index if it shifted
                if let Some(ref mut di) = self.dragged_particle_index {
                    if *di == i {
                        self.dragged_particle_index = None;
                    } else if *di == self.interior_particles.len() {
                        // This particle was swapped into position i
                        *di = i;
                    }
                }
            }
        }

        // Zymase timed crafting
        let zymase_outputs = crafting::tick_zymases(&mut self.zymases, dt, &mut rng);
        for output in zymase_outputs {
            if let CraftOutput::SpawnParticle { x, y, resource_type } = output {
                self.interior_particles.push(InteriorParticle { x, y, resource_type });
            }
        }

        // mRNA timed crafting
        {
            let mrna_xs = self.mrna_xs_slice();
            let mrna_ys = self.mrna_ys_slice();
            let mrna_outputs = crafting::tick_mrna(
                &mut self.mrna_processing,
                &mut self.mrna_timers,
                &mut self.mrna_progress_internal,
                &mrna_xs,
                &mrna_ys,
                &self.current_suppressions,
                dt,
            );
            for output in mrna_outputs {
                match output {
                    CraftOutput::SpawnZymase { x, y } => {
                        self.zymases.push(Zymase {
                            x,
                            y,
                            buffer: 0,
                            processing: false,
                            timer: 0.0,
                        });
                    }
                    CraftOutput::SpawnMotor { angle } => {
                        self.motors.push(Motor {
                            x: angle.cos() * MOTOR_MEMBRANE_RADIUS,
                            y: angle.sin() * MOTOR_MEMBRANE_RADIUS,
                            charge: 0.0,
                        });
                    }
                    CraftOutput::GrowCell => {
                        self.expansion_count += 1;
                        self.player_radius =
                            MIN_RADIUS + crafting::GROWTH_SCALE * (self.expansion_count as f32).sqrt();
                    }
                    _ => {}
                }
            }
        }

        // Count particles for HUD
        let counts = interior::count_particles(&self.interior_particles);
        self.atp_particle_count = counts.atp;
        self.glucose_particle_count = counts.glucose;
        self.amino_acid_particle_count = counts.amino_acid;

        // Death check: fully depleted when no motor charge AND no ATP/glucose particles
        let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
        if total_charge <= 0.0 && counts.atp == 0 && counts.glucose == 0 {
            self.player_alive = false;
        }

        // Backward-compatible energy ratio for WorldRenderer cell color
        self.player_atp = total_charge;
        self.player_energy_ratio = total_charge / self.player_max_atp.max(1.0);

        self.sync_packed_arrays();
        self.sync_interior_arrays();
        self.sync_motor_arrays();
        self.sync_zymase_arrays();
        self.sync_mrna_progress();
        self.sync_crafting_state();
        self.sync_rule_arrays();
    }

    fn sync_crafting_state(&mut self) {
        self.mrna_processing_flags = PackedInt32Array::new();
        self.mrna_timers_display = PackedFloat32Array::new();
        for m in 0..MRNA_COUNT {
            self.mrna_processing_flags.push(if self.mrna_processing[m] { 1 } else { 0 });
            self.mrna_timers_display.push(self.mrna_timers[m]);
        }
    }

    fn sync_rule_arrays(&mut self) {
        self.rule_count = self.rules.len() as i32;
        self.rule_descriptions = PackedStringArray::new();
        self.rule_firing = PackedInt32Array::new();
        self.rule_targets = PackedInt32Array::new();
        self.rule_limits = PackedInt32Array::new();

        for rule in &self.rules {
            self.rule_descriptions.push(&GString::from(&rule.description()));
            self.rule_firing.push(if rule.firing { 1 } else { 0 });
            self.rule_targets.push(rule.target.strand_index() as i32);
            self.rule_limits.push(rule.current_limit as i32);
        }

        self.mrna_suppressed = PackedInt32Array::new();
        for i in 0..MRNA_COUNT {
            self.mrna_suppressed.push(if self.current_suppressions[i] { 1 } else { 0 });
        }
    }

    #[func]
    fn toggle_regulation_panel(&mut self) {
        self.regulation_panel_open = !self.regulation_panel_open;
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

        enum PickTarget {
            Particle(usize),
            Motor(usize),
            Zymase(usize),
            Mrna(usize),
        }
        let mut best: Option<PickTarget> = None;

        for (i, p) in self.interior_particles.iter().enumerate() {
            let dx = p.x - x;
            let dy = p.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Particle(i));
            }
        }

        for (i, m) in self.motors.iter().enumerate() {
            let dx = m.x - x;
            let dy = m.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Motor(i));
            }
        }

        for (i, e) in self.zymases.iter().enumerate() {
            let dx = e.x - x;
            let dy = e.y - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Zymase(i));
            }
        }

        for i in 0..MRNA_COUNT {
            let dx = self.mrna_xs[i as usize] - x;
            let dy = self.mrna_ys[i as usize] - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < best_dist {
                best_dist = dist;
                best = Some(PickTarget::Mrna(i));
            }
        }

        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.dragged_zymase_index = None;
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
            Some(PickTarget::Zymase(ei)) => {
                self.dragged_zymase_index = Some(ei);
                self.dragged_particle_x = self.zymases[ei].x;
                self.dragged_particle_y = self.zymases[ei].y;
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
            && self.dragged_zymase_index.is_none()
            && self.dragged_mrna_index.is_none()
        {
            return;
        }

        if self.dragged_motor_index.is_some() {
            let angle = y.atan2(x);
            self.dragged_particle_x = angle.cos() * MOTOR_MEMBRANE_RADIUS;
            self.dragged_particle_y = angle.sin() * MOTOR_MEMBRANE_RADIUS;
        } else {
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

        // Zymase drop: finalize position
        if let Some(ei) = self.dragged_zymase_index {
            if ei < self.zymases.len() {
                self.zymases[ei].x = self.dragged_particle_x;
                self.zymases[ei].y = self.dragged_particle_y;
            }
            self.dragged_zymase_index = None;
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

        // Check drop targets using shared helpers
        if p_type == 0 {
            if let Some(ei) = crafting::find_zymase_target(x, y, &self.zymases) {
                self.player_glucose -= 1.0;
                self.interior_particles.swap_remove(idx);
                crafting::feed_zymase(&mut self.zymases[ei]);
            }
        } else if p_type == 1 {
            let mrna_xs = self.mrna_xs_slice();
            let mrna_ys = self.mrna_ys_slice();
            if let Some(m) = crafting::find_mrna_target(
                x,
                y,
                &mrna_xs,
                &mrna_ys,
                &self.mrna_progress_internal,
                &self.mrna_processing,
                &self.current_suppressions,
            ) {
                crafting::feed_mrna(
                    m,
                    &mut self.mrna_progress_internal,
                    &mut self.mrna_processing,
                    &mut self.mrna_timers,
                );
                self.interior_particles.swap_remove(idx);
            }
        } else if p_type == 2 {
            if let Some(mi) = crafting::find_motor_target(x, y, &self.motors) {
                self.interior_particles.swap_remove(idx);
                crafting::feed_motor(&mut self.motors[mi]);
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
        self.dragged_zymase_index = None;
        self.dragged_mrna_index = None;
        self.drag_active = false;
        self.dragged_particle_type = -1;
    }

    #[func]
    fn restart(&mut self) {
        self.player_x = 0.0;
        self.player_y = 0.0;
        self.player_radius = MIN_RADIUS;
        self.expansion_count = 0;
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
        self.zymases = vec![Zymase { x: 0.0, y: 0.0, buffer: 0, processing: false, timer: 0.0 }];
        self.dragged_zymase_index = None;
        self.mrna_processing = [false; MRNA_COUNT];
        self.mrna_timers = [0.0; MRNA_COUNT];
        self.interior_view = false;
        self.interior_particles.clear();
        self.dragged_particle_index = None;
        self.dragged_motor_index = None;
        self.dragged_mrna_index = None;
        self.dragged_particle_x = 0.0;
        self.dragged_particle_y = 0.0;
        self.drag_active = false;
        self.dragged_particle_type = -1;
        self.mrna_xs = PackedFloat32Array::from(&crafting::MRNA_ANGLES.map(|a| a.cos() * MRNA_DIST)[..]);
        self.mrna_ys = PackedFloat32Array::from(&crafting::MRNA_ANGLES.map(|a| a.sin() * MRNA_DIST)[..]);
        self.mrna_types = PackedInt32Array::from(&[0i32, 1, 2][..]);

        // Reset rules
        self.rules = rules::default_rules();
        self.current_suppressions = [false; MRNA_COUNT];
        self.regulation_panel_open = false;

        // Reset motors to single motor at angle 0
        self.motors = vec![Motor {
            x: MOTOR_ANGLE.cos() * MOTOR_MEMBRANE_RADIUS,
            y: MOTOR_ANGLE.sin() * MOTOR_MEMBRANE_RADIUS,
            charge: STARTING_ATP,
        }];

        let count = self.resources.len() as i32;
        self.resources.clear();
        self.spawn_resources(count);
        self.sync_interior_arrays();
        self.sync_motor_arrays();
        self.sync_zymase_arrays();
    }
}
