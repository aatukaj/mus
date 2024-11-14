use macroquad::math::Vec2;

use crate::{particle::Particle, Node};

pub struct Translation<'a, T> {
    pub pos: Vec2,
    pub inner: &'a T,
}
pub trait Translatable: Sized {
    fn translate(&self, camera_pos: Vec2) -> Translation<'_, Self>;

}
impl Translatable for Node {
    fn translate(&self, camera_pos: Vec2) -> Translation<'_, Self> {
        return Translation {
            pos: self.pos - camera_pos,
            inner: self
        }
        
    }
}
impl Translatable for Vec2 {
    fn translate(&self, camera_pos: Vec2) -> Translation<'_, Self> {
        return Translation { pos: *self-camera_pos, inner: self }
    }
}
impl Translatable for Particle {
    fn translate(&self, camera_pos: Vec2) -> Translation<'_, Self> {
        return Translation {
            pos: self.pos-camera_pos,
            inner: self,
        }
    }
}