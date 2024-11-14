use macroquad::math::Vec2;
use slotmap::SlotMap;

use crate::{Node, NodeId};

pub struct Edge {
    pub nodes: (NodeId, NodeId),
}
impl Edge {
    pub fn new(u: NodeId, v: NodeId) -> Self {
        assert!(u != v);
        Self { nodes: (u, v) }
    }
    pub fn distance_squared(&self, pos: Vec2, nodes: &SlotMap<NodeId, Node>, clamp: f32) -> f32 {
        let a = nodes[self.nodes.0].pos;
        let b = nodes[self.nodes.1].pos;
        let l_sq = a.distance_squared(b);
        if l_sq == 0.0 {
            return a.distance_squared(pos);
        }
        let t = ((pos - a).dot(b - a) / l_sq).clamp(clamp / 2.0, 1.0 - clamp / 2.0);
        let proj = a + t * (b - a);
        proj.distance_squared(pos)
    }
}
