use godot::prelude::*;
use rand::Rng;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::collections::HashSet;

use crate::types::{InteriorParticle, Zymase, Motor, Nucleus, INTERIOR_RADIUS, MAX_ATP, MOTOR_MEMBRANE_RADIUS, RESOURCE_RADIUS, MRNA_COUNT, CAPACITY_SCALE, MIN_RADIUS};
use crate::crafting::{self, CraftOutput};
use crate::interior;
use crate::rules::{self, Rule};
use crate::sync;
use crate::tech::{self, Tech};
use crate::autonomy;

type CellResource = crate::types::Resource;

const CHUNK_SIZE: f32 = 200.0;
const CHUNK_MARGIN: f32 = 200.0; // extra margin beyond visible area
const DENSITY_THRESHOLD: f32 = 0.38;
const MAX_RESOURCES_PER_CHUNK: i32 = 8;
const STARTING_BONUS_RADIUS: f32 = 3.0;
const DRIFT_SPEED: f32 = 5.0;

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
    player_nucleotides: f32,

    #[var]
    resource_xs: PackedFloat32Array,
    #[var]
    resource_ys: PackedFloat32Array,
    #[var]
    resource_types: PackedInt32Array,

    #[var]
    resource_radius: f32,

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
    active_chunks: HashSet<(i32, i32)>,
    view_min_wx: f32,
    view_max_wx: f32,
    view_min_wy: f32,
    view_max_wy: f32,

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

    nuclei: Vec<Nucleus>,
    nucleus_unlocked: bool,
    #[var]
    nucleus_count: i32,
    #[var]
    nucleus_xs: PackedFloat32Array,
    #[var]
    nucleus_ys: PackedFloat32Array,
    #[var]
    nucleus_target_types: PackedInt32Array,
    #[var]
    nucleus_progress: PackedInt32Array,
    #[var]
    nucleus_required: PackedInt32Array,
    #[var]
    nucleus_processing_flags: PackedInt32Array,
    #[var]
    nucleus_timers: PackedFloat32Array,
    #[var]
    nucleus_unlocked_flag: bool,

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
    #[var]
    nucleotide_particle_count: i32,

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
    rule_metrics: PackedInt32Array,
    #[var]
    rule_subjects: PackedInt32Array,
    #[var]
    rule_relations: PackedInt32Array,
    #[var]
    rule_thresholds: PackedFloat32Array,
    #[var]
    rule_targets: PackedInt32Array,
    #[var]
    rule_values: PackedFloat32Array,
    #[var]
    rule_enabled: PackedInt32Array,
    #[var]
    rule_firing: PackedInt32Array,
    #[var]
    rule_locked: PackedInt32Array,
    #[var]
    rule_threshold_modes: PackedInt32Array,
    #[var]
    rule_threshold_targets: PackedInt32Array,
    #[var]
    rule_threshold_values: PackedFloat32Array,
    #[var]
    mrna_suppressed: PackedInt32Array,

    techs: Vec<Tech>,

    #[var]
    tech_panel_open: bool,
    #[var]
    tech_selected: i32,
    #[var]
    tech_count: i32,
    #[var]
    tech_names: PackedStringArray,
    #[var]
    tech_descriptions: PackedStringArray,
    #[var]
    tech_progress: PackedFloat32Array,
    #[var]
    tech_completed: PackedInt32Array,

    dragged_particle_index: Option<usize>,
    #[var]
    dragged_particle_x: f32,
    #[var]
    dragged_particle_y: f32,
    #[var]
    drag_active: bool,
    #[var]
    dragged_particle_type: i32,

    // Autonomy state
    autonomous_mode: bool,
    sensor_count: i32,
    seek_target_internal: autonomy::SeekTarget,
    random_walk_timer: f32,
    random_walk_dx: f32,
    random_walk_dy: f32,

    #[var]
    autonomous: bool,
    #[var]
    autonomy_panel_open: bool,
    #[var]
    auto_sensor_count: i32,
    #[var]
    auto_sensor_range: f32,
    #[var]
    auto_seek_target: i32,
    #[var]
    auto_movement_dx: f32,
    #[var]
    auto_movement_dy: f32,
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
            player_nucleotides: 0.0,
            resource_xs: PackedFloat32Array::new(),
            resource_ys: PackedFloat32Array::new(),
            resource_types: PackedInt32Array::new(),
            resource_radius: RESOURCE_RADIUS,
            player_atp: STARTING_ATP,
            player_max_atp: MAX_ATP,
            player_alive: true,
            player_energy_ratio: STARTING_ATP / MAX_ATP,
            velocity_x: 0.0,
            velocity_y: 0.0,
            resources: Vec::new(),
            active_chunks: HashSet::new(),
            view_min_wx: -500.0,
            view_max_wx: 500.0,
            view_min_wy: -500.0,
            view_max_wy: 500.0,
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

            nuclei: Vec::new(),
            nucleus_unlocked: false,
            nucleus_count: 0,
            nucleus_xs: PackedFloat32Array::new(),
            nucleus_ys: PackedFloat32Array::new(),
            nucleus_target_types: PackedInt32Array::new(),
            nucleus_progress: PackedInt32Array::new(),
            nucleus_required: PackedInt32Array::new(),
            nucleus_processing_flags: PackedInt32Array::new(),
            nucleus_timers: PackedFloat32Array::new(),
            nucleus_unlocked_flag: false,

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
            nucleotide_particle_count: 0,

            mrna_processing: [false; MRNA_COUNT],
            mrna_timers: [0.0; MRNA_COUNT],
            mrna_processing_flags: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),
            mrna_timers_display: PackedFloat32Array::from(&[0.0f32; MRNA_COUNT][..]),

            expansion_count: 0,

            rules: rules::default_rules(),
            current_suppressions: [false; MRNA_COUNT],

            regulation_panel_open: false,
            techs: tech::default_techs(),
            tech_panel_open: false,
            tech_selected: 0,
            tech_count: 0,
            tech_names: PackedStringArray::new(),
            tech_descriptions: PackedStringArray::new(),
            tech_progress: PackedFloat32Array::new(),
            tech_completed: PackedInt32Array::new(),
            rule_count: 0,
            rule_metrics: PackedInt32Array::new(),
            rule_subjects: PackedInt32Array::new(),
            rule_relations: PackedInt32Array::new(),
            rule_thresholds: PackedFloat32Array::new(),
            rule_targets: PackedInt32Array::new(),
            rule_values: PackedFloat32Array::new(),
            rule_enabled: PackedInt32Array::new(),
            rule_firing: PackedInt32Array::new(),
            rule_locked: PackedInt32Array::new(),
            rule_threshold_modes: PackedInt32Array::new(),
            rule_threshold_targets: PackedInt32Array::new(),
            rule_threshold_values: PackedFloat32Array::new(),
            mrna_suppressed: PackedInt32Array::from(&[0i32; MRNA_COUNT][..]),

            dragged_particle_index: None,
            dragged_particle_x: 0.0,
            dragged_particle_y: 0.0,
            drag_active: false,
            dragged_particle_type: -1,

            autonomous_mode: false,
            sensor_count: 0,
            seek_target_internal: autonomy::SeekTarget::Nearest,
            random_walk_timer: 0.0,
            random_walk_dx: 0.0,
            random_walk_dy: 0.0,

            autonomous: false,
            autonomy_panel_open: false,
            auto_sensor_count: 0,
            auto_sensor_range: 0.0,
            auto_seek_target: 0,
            auto_movement_dx: 0.0,
            auto_movement_dy: 0.0,
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

    fn sync_nucleus_arrays(&mut self) {
        if self.nuclei.is_empty() {
            self.nucleus_count = 0;
            self.nucleus_unlocked_flag = self.nucleus_unlocked;
            return;
        }

        self.nucleus_xs = PackedFloat32Array::new();
        self.nucleus_ys = PackedFloat32Array::new();
        self.nucleus_target_types = PackedInt32Array::new();
        self.nucleus_progress = PackedInt32Array::new();
        self.nucleus_required = PackedInt32Array::new();
        self.nucleus_processing_flags = PackedInt32Array::new();
        self.nucleus_timers = PackedFloat32Array::new();

        for n in &self.nuclei {
            self.nucleus_xs.push(n.x);
            self.nucleus_ys.push(n.y);
            self.nucleus_target_types.push(n.target_type);
            self.nucleus_progress.push(n.progress);
            self.nucleus_required.push(crafting::MRNA_REQUIRED[n.target_type.clamp(0, 2) as usize]);
            self.nucleus_processing_flags.push(if n.processing { 1 } else { 0 });
            self.nucleus_timers.push(n.timer);
        }
        self.nucleus_count = self.nuclei.len() as i32;
        self.nucleus_unlocked_flag = self.nucleus_unlocked;
    }

    #[func]
    fn cycle_nucleus_target(&mut self, index: i32) {
        let idx = index as usize;
        if idx < self.nuclei.len() && !self.nuclei[idx].processing {
            self.nuclei[idx].target_type = (self.nuclei[idx].target_type + 1) % 3;
            self.nuclei[idx].progress = 0;
        }
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
    fn set_view_bounds(&mut self, min_wx: f32, max_wx: f32, min_wy: f32, max_wy: f32) {
        self.view_min_wx = min_wx;
        self.view_max_wx = max_wx;
        self.view_min_wy = min_wy;
        self.view_max_wy = max_wy;
    }

    fn noise_hash(gx: i32, gy: i32) -> f32 {
        let h = (gx as u32)
            .wrapping_mul(1597334677)
            .wrapping_add((gy as u32).wrapping_mul(3812015801));
        let h = h ^ (h >> 16);
        let h = h.wrapping_mul(2654435769);
        (h & 0x00FF_FFFF) as f32 / 16777216.0
    }

    fn smoothstep(t: f32) -> f32 {
        t * t * (3.0 - 2.0 * t)
    }

    fn value_noise(cx: f32, cy: f32, period: f32) -> f32 {
        let sx = cx / period;
        let sy = cy / period;
        let ix = sx.floor() as i32;
        let iy = sy.floor() as i32;
        let fx = Self::smoothstep(sx - sx.floor());
        let fy = Self::smoothstep(sy - sy.floor());

        let n00 = Self::noise_hash(ix, iy);
        let n10 = Self::noise_hash(ix + 1, iy);
        let n01 = Self::noise_hash(ix, iy + 1);
        let n11 = Self::noise_hash(ix + 1, iy + 1);

        let nx0 = n00 + (n10 - n00) * fx;
        let nx1 = n01 + (n11 - n01) * fx;
        nx0 + (nx1 - nx0) * fy
    }

    fn chunk_density(cx: i32, cy: i32) -> f32 {
        let x = cx as f32;
        let y = cy as f32;
        let n1 = Self::value_noise(x, y, 6.0);
        let n2 = Self::value_noise(x + 100.0, y + 100.0, 2.5);
        (n1 * 0.7 + n2 * 0.3).clamp(0.0, 1.0)
    }

    fn resource_count_for_chunk(cx: i32, cy: i32) -> i32 {
        let density = Self::chunk_density(cx, cy);

        let mut count = if density < DENSITY_THRESHOLD {
            0
        } else {
            let t = (density - DENSITY_THRESHOLD) / (1.0 - DENSITY_THRESHOLD);
            1 + (t * (MAX_RESOURCES_PER_CHUNK - 1) as f32) as i32
        };

        // Starting area bonus
        let dist = ((cx * cx + cy * cy) as f32).sqrt();
        if dist < STARTING_BONUS_RADIUS {
            let bonus = ((1.0 - dist / STARTING_BONUS_RADIUS) * 4.0) as i32;
            count += bonus;
            if count < 3 {
                count = 3;
            }
        }

        count
    }

    fn chunk_seed(cx: i32, cy: i32) -> u64 {
        let a = cx as u64;
        let b = cy as u64;
        a.wrapping_mul(2654435761).wrapping_add(b.wrapping_mul(2246822519))
    }

    fn update_chunks(&mut self) {
        // Compute desired chunks from view bounds + margin
        let mut min_cx = ((self.view_min_wx - CHUNK_MARGIN) / CHUNK_SIZE).floor() as i32;
        let mut max_cx = ((self.view_max_wx + CHUNK_MARGIN) / CHUNK_SIZE).floor() as i32;
        let mut min_cy = ((self.view_min_wy - CHUNK_MARGIN) / CHUNK_SIZE).floor() as i32;
        let mut max_cy = ((self.view_max_wy + CHUNK_MARGIN) / CHUNK_SIZE).floor() as i32;

        // Cap total chunk count to prevent memory exhaustion at extreme zoom
        let player_cx = (self.player_x / CHUNK_SIZE).floor() as i32;
        let player_cy = (self.player_y / CHUNK_SIZE).floor() as i32;
        let span_x = (max_cx - min_cx + 1) as i64;
        let span_y = (max_cy - min_cy + 1) as i64;
        if span_x * span_y > 2500 {
            min_cx = player_cx - 8;
            max_cx = player_cx + 8;
            min_cy = player_cy - 8;
            max_cy = player_cy + 8;
        }

        let mut desired = HashSet::new();
        for cx in min_cx..=max_cx {
            for cy in min_cy..=max_cy {
                desired.insert((cx, cy));
            }
        }

        // Always keep chunks within radius 3 of the player loaded
        // so interior view (high zoom) doesn't unload nearby chunks
        for cx in (player_cx - 3)..=(player_cx + 3) {
            for cy in (player_cy - 3)..=(player_cy + 3) {
                desired.insert((cx, cy));
            }
        }

        // Remove resources belonging to chunks leaving the active set
        let leaving: HashSet<(i32, i32)> = self.active_chunks.difference(&desired).copied().collect();
        if !leaving.is_empty() {
            self.resources.retain(|r| !leaving.contains(&(r.chunk_x, r.chunk_y)));
        }

        // Generate resources for newly active chunks
        let entering: Vec<(i32, i32)> = desired.difference(&self.active_chunks).copied().collect();
        for (cx, cy) in entering {
            let count = Self::resource_count_for_chunk(cx, cy);
            if count == 0 {
                continue;
            }
            let seed = Self::chunk_seed(cx, cy);
            let mut rng = SmallRng::seed_from_u64(seed);
            let base_x = cx as f32 * CHUNK_SIZE;
            let base_y = cy as f32 * CHUNK_SIZE;
            let focus_x = base_x + rng.gen_range(0.25..0.75) * CHUNK_SIZE;
            let focus_y = base_y + rng.gen_range(0.25..0.75) * CHUNK_SIZE;
            for _ in 0..count {
                let ox = (rng.gen_range(0.0_f32..1.0) + rng.gen_range(0.0_f32..1.0) - 1.0) * 0.25 * CHUNK_SIZE;
                let oy = (rng.gen_range(0.0_f32..1.0) + rng.gen_range(0.0_f32..1.0) - 1.0) * 0.25 * CHUNK_SIZE;
                let x = (focus_x + ox).clamp(base_x, base_x + CHUNK_SIZE);
                let y = (focus_y + oy).clamp(base_y, base_y + CHUNK_SIZE);
                let dist = ((cx * cx + cy * cy) as f32).sqrt();
                let resource_type = if dist >= STARTING_BONUS_RADIUS {
                    let t = rng.gen_range(0..3);
                    if t == 2 { 3 } else { t }
                } else {
                    rng.gen_range(0..2)
                };
                self.resources.push(CellResource {
                    x,
                    y,
                    resource_type,
                    amount: 1.0,
                    chunk_x: cx,
                    chunk_y: cy,
                });
            }
        }

        self.active_chunks = desired;
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

        // Update player position (no bounds — infinite world)
        self.player_x += self.velocity_x * dt;
        self.player_y += self.velocity_y * dt;

        self.velocity_x *= damping;
        self.velocity_y *= damping;

        let mut rng = rand::thread_rng();

        // Autonomous movement (chemotaxis or random walk)
        if self.autonomous_mode {
            if self.sensor_count > 0 {
                let (dx, dy) = autonomy::compute_chemotaxis_direction(
                    self.player_x,
                    self.player_y,
                    self.player_radius,
                    self.sensor_count,
                    self.seek_target_internal,
                    &self.resources,
                );
                if dx.abs() > 0.001 || dy.abs() > 0.001 {
                    self.move_player(
                        dx * autonomy::CHEMOTAXIS_STRENGTH,
                        dy * autonomy::CHEMOTAXIS_STRENGTH,
                    );
                    self.auto_movement_dx = dx;
                    self.auto_movement_dy = dy;
                } else {
                    // No resources in range — fall back to random walk
                    self.tick_random_walk(dt, &mut rng);
                }
            } else {
                // No sensors — random walk only
                self.tick_random_walk(dt, &mut rng);
            }
        }

        // Load/unload chunks around player
        self.update_chunks();

        // Drift resources
        for r in &mut self.resources {
            r.x += rng.gen_range(-DRIFT_SPEED..DRIFT_SPEED) * dt;
            r.y += rng.gen_range(-DRIFT_SPEED..DRIFT_SPEED) * dt;
        }

        // Check absorption
        let counts = interior::count_particles(&self.interior_particles);
        let expansion_sqrt = (self.expansion_count as f32).sqrt();
        let max_glucose = interior::BASE_MAX_GLUCOSE + (CAPACITY_SCALE * expansion_sqrt) as i32;
        let max_amino = interior::BASE_MAX_AMINO + (CAPACITY_SCALE * expansion_sqrt) as i32;
        let max_nucleotide = interior::BASE_MAX_NUCLEOTIDE + (CAPACITY_SCALE * expansion_sqrt) as i32;

        let absorption_events = interior::detect_absorptions(
            self.player_x,
            self.player_y,
            self.player_radius,
            &self.resources,
            counts.glucose,
            max_glucose,
            counts.amino_acid,
            max_amino,
            counts.nucleotide,
            max_nucleotide,
            &mut rng,
        );

        for event in &absorption_events {
            match event.resource_type {
                0 => self.player_glucose += event.amount,
                1 => self.player_amino_acids += event.amount,
                3 => self.player_nucleotides += event.amount,
                _ => {}
            }
            interior::apply_absorption(&mut self.interior_particles, event, &mut rng);
        }

        // Remove absorbed resources (sort descending for safe swap_remove)
        let mut remove_indices: Vec<usize> = absorption_events.iter().map(|e| e.resource_index).collect();
        remove_indices.sort_unstable_by(|a, b| b.cmp(a));
        for i in remove_indices {
            self.resources.swap_remove(i);
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
            self.player_radius,
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
                &mut self.nuclei,
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
        self.nucleotide_particle_count = counts.nucleotide;

        // Death check: fully depleted when no motor charge AND no ATP/glucose particles
        // AND no glucose buffered/processing in zymases
        let total_charge: f32 = self.motors.iter().map(|m| m.charge).sum();
        let zymase_has_fuel = self.zymases.iter().any(|z| z.buffer > 0 || z.processing);
        if total_charge <= 0.0 && counts.atp == 0 && counts.glucose == 0 && !zymase_has_fuel {
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

        let tech_ctx = tech::TechContext { max_nucleotide: max_nucleotide };
        tech::tick_techs(&mut self.techs, &mut self.rules, &tech_ctx);

        // Grant sensor when Chemoreceptor tech completes
        if self.sensor_count == 0 {
            if self.techs.get(4).map_or(false, |t| t.completed) {
                self.sensor_count = 1;
            }
        }

        // Unlock nucleus when Programmable Nucleus tech completes
        if !self.nucleus_unlocked {
            if self.techs.get(5).map_or(false, |t| t.completed) {
                self.nucleus_unlocked = true;
                let nx = crafting::NUCLEUS_ANGLE.cos() * MRNA_DIST;
                let ny = crafting::NUCLEUS_ANGLE.sin() * MRNA_DIST;
                self.nuclei.push(Nucleus {
                    x: nx,
                    y: ny,
                    target_type: 0,
                    progress: 0,
                    processing: false,
                    timer: 0.0,
                });
            }
        }

        // Nucleus timed crafting
        let nucleus_outputs = crafting::tick_nuclei(&mut self.nuclei, dt);
        for output in nucleus_outputs {
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

        self.sync_nucleus_arrays();
        self.sync_tech_arrays();
        self.sync_autonomy_state();
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
        let a = sync::build_rule_arrays(&self.rules, &self.current_suppressions);
        self.rule_count = self.rules.len() as i32;
        self.rule_metrics = PackedInt32Array::from(a.metrics.as_slice());
        self.rule_subjects = PackedInt32Array::from(a.subjects.as_slice());
        self.rule_relations = PackedInt32Array::from(a.relations.as_slice());
        self.rule_thresholds = PackedFloat32Array::from(a.thresholds.as_slice());
        self.rule_targets = PackedInt32Array::from(a.targets.as_slice());
        self.rule_values = PackedFloat32Array::from(a.values.as_slice());
        self.rule_enabled = PackedInt32Array::from(a.enabled.as_slice());
        self.rule_firing = PackedInt32Array::from(a.firing.as_slice());
        self.rule_locked = PackedInt32Array::from(a.locked.as_slice());
        self.rule_threshold_modes = PackedInt32Array::from(a.threshold_modes.as_slice());
        self.rule_threshold_targets = PackedInt32Array::from(a.threshold_targets.as_slice());
        self.rule_threshold_values = PackedFloat32Array::from(a.threshold_values.as_slice());
        self.mrna_suppressed = PackedInt32Array::from(a.mrna_suppressed.as_slice());
    }

    fn sync_tech_arrays(&mut self) {
        let a = sync::build_tech_arrays(&self.techs);
        self.tech_count = self.techs.len() as i32;
        self.tech_names = PackedStringArray::new();
        self.tech_descriptions = PackedStringArray::new();
        for name in &a.names {
            self.tech_names.push(&GString::from(name.as_str()));
        }
        for desc in &a.descriptions {
            self.tech_descriptions.push(&GString::from(desc.as_str()));
        }
        self.tech_progress = PackedFloat32Array::from(a.progress.as_slice());
        self.tech_completed = PackedInt32Array::from(a.completed.as_slice());
    }

    #[func]
    fn toggle_tech_panel(&mut self) {
        self.tech_panel_open = !self.tech_panel_open;
    }

    #[func]
    fn select_tech(&mut self, index: i32) {
        if index >= 0 && (index as usize) < self.techs.len() {
            self.tech_selected = index;
        }
    }

    #[func]
    fn toggle_regulation_panel(&mut self) {
        self.regulation_panel_open = !self.regulation_panel_open;
    }

    #[func]
    fn toggle_autonomous_mode(&mut self) {
        self.autonomous_mode = !self.autonomous_mode;
        self.autonomous = self.autonomous_mode;
        if !self.autonomous_mode {
            self.auto_movement_dx = 0.0;
            self.auto_movement_dy = 0.0;
        }
    }

    #[func]
    fn toggle_autonomy_panel(&mut self) {
        self.autonomy_panel_open = !self.autonomy_panel_open;
    }

    #[func]
    fn cycle_seek_target(&mut self) {
        self.seek_target_internal = self.seek_target_internal.next();
        self.auto_seek_target = self.seek_target_internal.as_i32();
    }

    fn tick_random_walk(&mut self, dt: f32, rng: &mut impl Rng) {
        self.random_walk_timer -= dt;
        if self.random_walk_timer <= 0.0 {
            self.random_walk_timer = autonomy::RANDOM_WALK_INTERVAL;
            let angle: f32 = rng.gen_range(0.0..std::f32::consts::TAU);
            self.random_walk_dx = angle.cos();
            self.random_walk_dy = angle.sin();
        }
        self.move_player(
            self.random_walk_dx * autonomy::RANDOM_WALK_STRENGTH,
            self.random_walk_dy * autonomy::RANDOM_WALK_STRENGTH,
        );
        self.auto_movement_dx = self.random_walk_dx;
        self.auto_movement_dy = self.random_walk_dy;
    }

    fn sync_autonomy_state(&mut self) {
        self.autonomous = self.autonomous_mode;
        self.auto_sensor_count = self.sensor_count;
        self.auto_sensor_range = autonomy::sensor_range(self.sensor_count);
        self.auto_seek_target = self.seek_target_internal.as_i32();
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
                self.dragged_particle_type = 100;
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
                self.dragged_particle_type = 101;
                true
            }
            Some(PickTarget::Mrna(i)) => {
                self.dragged_mrna_index = Some(i);
                self.dragged_particle_x = self.mrna_xs[i as usize];
                self.dragged_particle_y = self.mrna_ys[i as usize];
                self.drag_active = true;
                self.dragged_particle_type = 102;
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
        } else if p_type == 3 {
            if let Some(ni) = crafting::find_nucleus_target(x, y, &self.nuclei) {
                self.interior_particles.swap_remove(idx);
                crafting::feed_nucleus(&mut self.nuclei[ni]);
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
    fn cycle_rule_metric(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            let new_metric = self.rules[idx].metric.next();
            self.rules[idx].metric = new_metric;
            if matches!(self.rules[idx].threshold, rules::Threshold::Fixed(_)) {
                self.rules[idx].threshold = rules::default_threshold_for_metric(new_metric);
            }
        }
    }

    #[func]
    fn cycle_rule_subject(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].subject = self.rules[idx].subject.next();
        }
    }

    #[func]
    fn cycle_rule_relation(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].relation = self.rules[idx].relation.next();
        }
    }

    #[func]
    fn cycle_rule_target(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].target = self.rules[idx].target.next();
        }
    }

    #[func]
    fn adjust_rule_threshold(&mut self, i: i32, direction: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            let step = rules::threshold_step(self.rules[idx].metric);
            if let rules::Threshold::Fixed(ref mut v) = self.rules[idx].threshold {
                let delta = step * direction as f32;
                *v = (*v + delta).max(0.0);
            }
        }
    }

    #[func]
    fn set_rule_threshold(&mut self, i: i32, value: f32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].threshold = rules::Threshold::Fixed(value.max(0.0));
        }
    }

    #[func]
    fn set_rule_threshold_variable(&mut self, i: i32, target_idx: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].threshold =
                rules::Threshold::Variable(rules::MrnaTarget::from_index(target_idx as usize));
        }
    }

    #[func]
    fn set_rule_threshold_fixed(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].threshold =
                rules::default_threshold_for_metric(self.rules[idx].metric);
        }
    }

    #[func]
    fn toggle_rule_enabled(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules[idx].enabled = !self.rules[idx].enabled;
        }
    }

    #[func]
    fn add_rule(&mut self) {
        if self.rules.len() < rules::MAX_RULES {
            self.rules.push(rules::Rule {
                metric: rules::Metric::Count,
                subject: rules::MrnaTarget::Zymase,
                relation: rules::Relation::GreaterEqual,
                threshold: rules::default_threshold_for_metric(rules::Metric::Count),
                target: rules::MrnaTarget::Zymase,
                enabled: true,
                firing: false,
                current_value: 0.0,
                current_threshold_value: 0.0,
                locked: false,
            });
        }
    }

    #[func]
    fn remove_rule(&mut self, i: i32) {
        let idx = i as usize;
        if idx < self.rules.len() && !self.rules[idx].locked {
            self.rules.remove(idx);
        }
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
        self.player_nucleotides = 0.0;
        self.velocity_x = 0.0;
        self.velocity_y = 0.0;
        self.player_energy_ratio = STARTING_ATP / MAX_ATP;
        self.motor_charge_display = STARTING_ATP;
        self.atp_particle_count = 0;
        self.glucose_particle_count = 0;
        self.amino_acid_particle_count = 0;
        self.nucleotide_particle_count = 0;
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

        // Reset techs
        self.techs = tech::default_techs();
        self.tech_panel_open = false;
        self.tech_selected = 0;

        // Reset motors to single motor at angle 0
        self.motors = vec![Motor {
            x: MOTOR_ANGLE.cos() * MOTOR_MEMBRANE_RADIUS,
            y: MOTOR_ANGLE.sin() * MOTOR_MEMBRANE_RADIUS,
            charge: STARTING_ATP,
        }];

        // Reset nuclei
        self.nuclei.clear();
        self.nucleus_unlocked = false;

        // Reset autonomy
        self.autonomous_mode = false;
        self.sensor_count = 0;
        self.seek_target_internal = autonomy::SeekTarget::Nearest;
        self.random_walk_timer = 0.0;
        self.random_walk_dx = 0.0;
        self.random_walk_dy = 0.0;
        self.autonomous = false;
        self.autonomy_panel_open = false;
        self.auto_sensor_count = 0;
        self.auto_sensor_range = 0.0;
        self.auto_seek_target = 0;
        self.auto_movement_dx = 0.0;
        self.auto_movement_dy = 0.0;

        self.resources.clear();
        self.active_chunks.clear();
        self.update_chunks();
        self.sync_packed_arrays();
        self.sync_interior_arrays();
        self.sync_motor_arrays();
        self.sync_zymase_arrays();
    }
}
