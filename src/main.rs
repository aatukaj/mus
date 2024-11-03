// seperate thread for audio, block processing maybe, but still make it kinda dynamic
use macroquad::prelude::*;
mod audio;

struct Node {
    pos: Vec2,
    total_force: Vec2,
    vel: Vec2,
    locked: bool,
}
impl Node {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
            total_force: vec2(0.0, 0.0),
            vel: vec2(0.0, 0.0),
            locked: false,
        }
    }
    fn update(&mut self, dt: f32) {
        self.vel -= self.vel * SLOWNESS * dt;
        if !self.locked {
            self.pos += self.vel * dt;
        }

        self.vel += self.total_force / NODE_MASS * dt;
        self.total_force = vec2(0.0, 0.0);
    }
}
struct State {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    mouse_pos: Vec2,
    selected_node: Option<NodeId>,
}

type NodeId = usize;

const EDGE_STIFFNESS: f32 = 1.2;
const EDGE_BASE_LENGTH: f32 = 30.0;
const NODE_MASS: f32 = 1.0;
const SLOWNESS: f32 = 0.97;
const DISPERSION_AMT: f32 = 1500000.0; // idk
const NODE_RADIUS: f32 = 14.0;

struct Edge {
    nodes: (NodeId, NodeId),
}
impl Edge {
    fn new(u: NodeId, v: NodeId) -> Self {
        Self { nodes: (u, v) }
    }
}
fn apply_dispersions(nodes: &mut [Node]) {
    let len = nodes.len();
    for u in 0..len {
        for v in (u + 1)..len {
            let dist = nodes[u].pos.distance_squared(nodes[v].pos);
            let force = (nodes[v].pos - nodes[u].pos).normalize() * (DISPERSION_AMT / dist); // i->j
            nodes[v].total_force += force;
            nodes[u].total_force -= force;
        }
    }
}
fn apply_edge_forces(nodes: &mut [Node], edges: &[Edge]) {
    for &Edge { nodes: (u, v) } in edges {
        let dif = nodes[u].pos.distance(nodes[v].pos) - EDGE_BASE_LENGTH;
        let force = (nodes[v].pos - nodes[u].pos).normalize() * dif * EDGE_STIFFNESS; // Hooke's law
                                                                                      // u-> v force
        nodes[v].total_force -= force;
        nodes[u].total_force += force;
    }
}

fn update(nodes: &mut [Node], delta_time: f32) {
    for node in nodes.iter_mut() {
        node.update(delta_time);
    }
}
fn draw(nodes: &[Node], edges: &[Edge]) {
    for node in nodes {
        draw_circle_lines(node.pos.x, node.pos.y, NODE_RADIUS, 1.5, WHITE);
        let sz = 5.0;
        if node.locked {
            draw_rectangle_lines(
                node.pos.x - sz / 2.0,
                node.pos.y - sz / 2.0,
                sz,
                sz,
                2.0,
                RED,
            );
        }
    }
    for &Edge { nodes: (u, v) } in edges {
        let dir = (nodes[v].pos - nodes[u].pos).normalize();
        let norm = dir.perp();
        let a = nodes[u].pos + NODE_RADIUS * dir;
        let b = nodes[v].pos - NODE_RADIUS * dir;
        draw_line(a.x, a.y, b.x, b.y, 1.0, WHITE);
        let lb = b - dir * 14.0 - norm * 5.0;
        let rb = b - dir * 14.0 + norm * 5.0;
        draw_line(b.x, b.y, lb.x, lb.y, 1.0, WHITE);
        draw_line(b.x, b.y, rb.x, rb.y, 1.0, WHITE);
    }
}

#[macroquad::main("music")]
async fn main() {
    audio::setup();
    let mut nodes = vec![
        Node::new(10.0, 10.0),
        Node::new(50.0, 200.0),
        Node::new(100.0, 100.0),
        Node::new(200.0, 200.0),
        Node::new(300.0, 200.0),
    ];

    let edges = vec![
        Edge::new(0, 1),
        Edge::new(1, 2),
        Edge::new(2, 3),
        Edge::new(3, 0),
        Edge::new(0, 4),
        Edge::new(1, 4),
        Edge::new(2, 4),
        Edge::new(3, 4),
    ];
    // let mut nodes = vec![Node::new(10.0, 10.0), Node::new(100.0, 100.0)];
    // let edges = vec![Edge::new(0, 1)];
    let mut elapsed = 0.0;
    let mut selected_node: Option<NodeId> = None;
    loop {
        let m_pos = Vec2::from(mouse_position());
        elapsed += get_frame_time();
        if is_mouse_button_down(MouseButton::Left) {
            for i in 0..nodes.len() {
                let node = &nodes[i];
                if node.pos.distance_squared(m_pos) <= NODE_RADIUS.powi(2) {
                    selected_node = Some(i);
                    break;
                }
            }
        }
        if is_mouse_button_released(MouseButton::Left) {
            selected_node = None;
        }
        if is_key_pressed(KeyCode::R) {
            if let Some(i) = selected_node {
                nodes[i].locked = !nodes[i].locked;
            } else {
                for i in 0..nodes.len() {
                    let node = &nodes[i];
                    if node.pos.distance_squared(m_pos) <= NODE_RADIUS.powi(2) {
                        nodes[i].locked = !nodes[i].locked;
                    }
                }
            }
        }
        if is_key_pressed(KeyCode::Space) {
            let lock = !nodes.iter().any(|node| node.locked);
            for node in nodes.iter_mut() {
                node.locked = lock;
            }
        }
        if elapsed >= 1.0 / 30.0 {
            apply_dispersions(&mut nodes);
            apply_edge_forces(&mut nodes, &edges);
            if let Some(v) = selected_node {
                let force = 2.0 * (m_pos - nodes[v].pos);
                nodes[v].total_force += force;
            }

            update(&mut nodes, elapsed);
            elapsed = 0.0;
        }
        clear_background(BLACK);
        draw(&nodes, &edges);
        next_frame().await;
    }
}
