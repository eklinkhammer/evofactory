use godot::prelude::*;

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

    velocity_x: f32,
    velocity_y: f32,
}

#[godot_api]
impl INode for Simulation {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            player_x: 0.0,
            player_y: 0.0,
            player_radius: 15.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
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
    fn tick(&mut self, delta: f64) {
        let dt = delta as f32;
        let damping = 0.9_f32.powf(dt * 60.0);
        let bound = 500.0;

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
    }
}
