use std::f32::consts::PI;

use macroquad::{math::Vec2, rand};

use crate::State;

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub end_time: f32,
}
impl Particle {
    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel*dt;
    }

}

pub fn spawn_particles(state: &mut State, pos: Vec2) {
    for _ in 0..20 {
        let dir = Vec2::from_angle(rand::gen_range(0.0, 2.0*PI));
        state.particles.push(Particle {
            pos,
            vel: dir * rand::gen_range(17.0, 25.0),
            end_time: state.time + 2.0,
        });

    }

}