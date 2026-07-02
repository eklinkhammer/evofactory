pub const INTERIOR_RADIUS: f32 = 200.0;
pub const MAX_ATP: f32 = 5.0;
pub const MOTOR_MEMBRANE_RADIUS: f32 = INTERIOR_RADIUS;
pub const RESOURCE_RADIUS: f32 = 8.0;
pub const MRNA_COUNT: usize = 3;
pub const CAPACITY_SCALE: f32 = 2.0;
pub const MIN_RADIUS: f32 = 15.0;

#[derive(Clone)]
pub struct InteriorParticle {
    pub x: f32,
    pub y: f32,
    pub resource_type: i32,
}

pub struct Resource {
    pub x: f32,
    pub y: f32,
    pub resource_type: i32, // 0 = glucose, 1 = amino acid, 2 = ATP (interior), 3 = nucleotide
    pub amount: f32,
    pub chunk_x: i32,
    pub chunk_y: i32,
}

pub struct Zymase {
    pub x: f32,
    pub y: f32,
    pub buffer: i32,
    pub processing: bool,
    pub timer: f32,
}

pub struct Motor {
    pub x: f32,
    pub y: f32,
    pub charge: f32,
}

pub struct Nucleus {
    pub x: f32,
    pub y: f32,
    pub target_type: i32,   // 0=Zymase, 1=Motor, 2=Membrane
    pub progress: i32,
    pub processing: bool,
    pub timer: f32,
}
