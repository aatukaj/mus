use std::f32::consts::PI;

use crate::*;
use translation::Translatable;

const GHOST_COLOR: Color = Color {
    a: 0.5,
    ..LIGHTGRAY
};
pub fn draw_mode_overlays(state: &State) {
    if matches!(state.mode, Mode::AddEdge { .. } | Mode::UpdNode { .. }) {
        if let Some(id) = state.hovered_node {
            let t = state.nodes[id].translate(state.camera_pos);
            let sz = 15.0;
            draw_rectangle_lines_ex(
                t.pos.x,
                t.pos.y,
                sz,
                sz,
                2.0,
                DrawRectangleParams {
                    offset: vec2(0.5, 0.5),
                    color: GHOST_COLOR,
                    rotation: PI / 4.0,
                },
            );
        }
    }
    if matches!(state.mode, Mode::AddEdge { .. }) {
        if state.hovered_node.is_none() {
            draw_node(
                state.mouse_pos.translate(state.camera_pos).pos,
                GHOST_COLOR,
                None,
                state,
            );
        }
    }
    match &state.mode {
        Mode::Delete(Some(sel)) => {
            let pos = sel.get_center(state);
            let t = pos.translate(state.camera_pos);
            let sz = 15.0;
            draw_rectangle_lines_ex(
                t.pos.x,
                t.pos.y,
                sz,
                sz,
                2.0,
                DrawRectangleParams {
                    offset: vec2(0.5, 0.5),
                    color: RED,
                    rotation: PI / 4.0,
                },
            );
        }
        Mode::AddEdge { first } => {
            if let Some(first_id) = *first {
                let a = state.nodes[first_id].pos.translate(state.camera_pos);
                let b = state
                    .hovered_node
                    .map(|id| state.nodes[id].pos)
                    .unwrap_or(state.mouse_pos);

                draw_arrow(a.pos, b.translate(state.camera_pos).pos, GHOST_COLOR);
            }
        }
        Mode::UpdNode { kind } => {
            let pos = state
                .hovered_node
                .map(|id| state.nodes[id].pos)
                .unwrap_or(state.mouse_pos)
                .translate(state.camera_pos)
                .pos;
            draw_node(pos, GHOST_COLOR, Some(kind), state);
        }

        _ => {}
    }
}
pub fn draw_node(pos: Vec2, color: Color, kind: Option<&NodeKind>, state: &State) {
    draw_circle_lines(pos.x, pos.y, NODE_RADIUS, 1.5, color);
    if let Some(kind) = kind {
        match kind {
            NodeKind::Sample(idx) => {
                let text = idx.to_string();
                let dims = measure_text(&text, None, 22, 1.0);
                draw_text(
                    &text,
                    pos.x - dims.width / 2.0,
                    pos.y + dims.height / 2.0,
                    22.0,
                    color,
                );
            }
            NodeKind::Spawner {
                bar_delay,
                next_spawn,
            } => {
                let t = (next_spawn - state.time) / (bar_delay * BAR_TIME);
                draw_arc(
                    pos.x,
                    pos.y,
                    10,
                    NODE_RADIUS - 6.0,
                    90.0,
                    4.0,
                    t * 360.0,
                    GREEN,
                );
            }
            _ => {}
        }
    }
}
pub fn draw_arrow(u: Vec2, v: Vec2, color: Color) {
    let dir = (v - u).normalize();
    let norm = dir.perp();
    let a = u + NODE_RADIUS * dir;
    let b = v - NODE_RADIUS * dir;
    draw_line(a.x, a.y, b.x, b.y, 1.0, color);
    let lb = b - dir * 14.0 - norm * 5.0;
    let rb = b - dir * 14.0 + norm * 5.0;
    draw_line(b.x, b.y, lb.x, lb.y, 1.0, color);
    draw_line(b.x, b.y, rb.x, rb.y, 1.0, color);
}

pub fn draw(state: &State) {
    let nodes = &state.nodes;
    let edges = &state.edges;
    for i in 0..=(screen_width() as usize).div_ceil(PX_PER_BAR as usize) {
        let p = vec2(
            (i as f32) * PX_PER_BAR + (state.camera_pos.x / PX_PER_BAR).round() * PX_PER_BAR,
            0.0,
        )
        .translate(state.camera_pos)
        .pos;
        draw_line(p.x, 0.0, p.x, screen_height(), 1.0, DARKGRAY);
    }
    for node in nodes.values() {
        let node = node.translate(state.camera_pos);
        draw_node(node.pos, WHITE, Some(&node.inner.kind), state);
    }
    for &Edge { nodes: (u, v) } in edges.values() {
        let u = nodes[u].translate(state.camera_pos);
        let v = nodes[v].translate(state.camera_pos);
        draw_arrow(u.pos, v.pos, WHITE);
    }
    for particle in &state.particles {
        let particle = particle.translate(state.camera_pos);
        let t = 3.0 * (state.time - particle.inner.end_time);
        draw_circle(particle.pos.x, particle.pos.y, (t).min(1.0), PINK);
    }
    for &Signal {
        cur_edge,
        start_time,
    } in &state.signals
    {
        let Edge { nodes: (u, v) } = edges[cur_edge];
        let mut dir = nodes[v].pos - nodes[u].pos;
        dir = dir / dir.x;
        let t = state.time - start_time;
        let pos = nodes[u].pos + dir * (t / BAR_TIME) * PX_PER_BAR;
        let t = pos.translate(state.camera_pos);
        draw_circle(t.pos.x, t.pos.y, 5.0, PINK);
    }
    draw_mode_overlays(state);
}

