use macroquad::math::{vec2, Vec2};

pub struct Node {
    pub pos: Vec2,
    pub kind: NodeKind,
}
impl Node {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            kind: NodeKind::Default,
        }
    }
}
#[derive(Clone)]
pub enum NodeKind {
    Default,
    Spawner { bar_delay: f32, next_spawn: f32 },
    Sample(usize),
}
