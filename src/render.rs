use std::f32::consts::PI;

use macroquad::prelude::*;
use crate::{*};
use translation::Translatable;

pub fn draw_mode_overlays(state: &State) {
    match &state.mode {
        Mode::Delete(Some(sel)) => {
            let pos = sel.get_center(state);
            let t = pos.translate(state.camera_pos);
            let sz = 15.0;
            draw_rectangle_lines_ex(t.pos.x, t.pos.y, sz, sz, 2.0, DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                color: RED,
                rotation: PI/4.0,
            });

        }

        _ => {}

    }

}

pub fn draw(state: &State) {
    let nodes = &state.nodes;
    let edges = &state.edges;
    for i in 0..=(screen_width() as usize).div_ceil(PX_PER_BAR as usize) {
        let p = vec2((i as f32)*PX_PER_BAR+(state.camera_pos.x/PX_PER_BAR).round()*PX_PER_BAR, 0.0).translate(state.camera_pos).pos;
        draw_line(p.x, 0.0, p.x, screen_height(), 1.0, DARKGRAY);
    }
    for node in nodes.values() {
        let node = node.translate(state.camera_pos);
        draw_circle_lines(node.pos.x, node.pos.y, NODE_RADIUS, 1.5, WHITE);
    }
    for &Edge { nodes: (u, v) } in edges.values() {
        let u = nodes[u].translate(state.camera_pos);
        let v = nodes[v].translate(state.camera_pos);
        let dir = (v.pos - u.pos).normalize();
        let norm = dir.perp();
        let a = u.pos + NODE_RADIUS * dir;
        let b = v.pos - NODE_RADIUS * dir;
        draw_line(a.x, a.y, b.x, b.y, 1.0, WHITE);
        let lb = b - dir * 14.0 - norm * 5.0;
        let rb = b - dir * 14.0 + norm * 5.0;
        draw_line(b.x, b.y, lb.x, lb.y, 1.0, WHITE);
        draw_line(b.x, b.y, rb.x, rb.y, 1.0, WHITE);
    }
    for particle in &state.particles {
        let particle = particle.translate(state.camera_pos);
        let t = 3.0*(state.time-particle.inner.end_time);
        draw_circle(particle.pos.x, particle.pos.y, (t).min(1.0), PINK);
    }
    for &Signal {cur_edge, start_time} in &state.signals {
        let Edge{nodes: (u, v)} = edges[cur_edge];
        let mut dir = nodes[v].pos - nodes[u].pos;
        dir = dir/dir.x;
        let t= state.time - start_time;
        let pos = nodes[u].pos + dir*(t/BAR_TIME)*PX_PER_BAR;
        let t = pos.translate(state.camera_pos);
        draw_circle(t.pos.x, t.pos.y, 5.0, PINK);
    }
    let t= state.mouse_pos.translate(state.camera_pos);
    draw_circle(t.pos.x, t.pos.y, 5.0, RED);
    draw_mode_overlays(state);
}