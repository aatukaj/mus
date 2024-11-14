// :TODO:
// - ui, macroquad egui??
// - more audio options,
// add/remove nodes and edges
// - stretch on x-axis <- changing translate
use macroquad::prelude::*;
use macroquad::ui;
mod audio;
mod edge;
mod particle;
mod translation;
use edge::*;
use particle::{spawn_particles, Particle};
use slotmap::{new_key_type, SecondaryMap, SlotMap};

use audio::*;

new_key_type! {struct EdgeId; }
new_key_type! {struct NodeId; }
mod util;
use util::IndexOf as _;

mod render;
use render::draw;
mod node;
use node::*;

const NODE_RADIUS: f32 = 14.0;
const PX_PER_BAR: f32 = 150.0;
const BAR_TIME: f32 = 1.0;
const SIGNAL_SPEED: f32 = PX_PER_BAR / BAR_TIME;

struct Signal {
    cur_edge: EdgeId,
    start_time: f32,
}

#[derive(Default)]
struct Adjlist {
    incoming: Vec<(NodeId, EdgeId)>,
    outgoing: Vec<(NodeId, EdgeId)>,
}

#[derive(Default)]
struct State {
    nodes: SlotMap<NodeId, Node>,
    edges: SlotMap<EdgeId, Edge>,
    adj: SecondaryMap<NodeId, Adjlist>,

    signals: Vec<Signal>,
    particles: Vec<Particle>,
    camera_pos: Vec2,
    mouse_pos: Vec2,
    time: f32,
    dt: f32,
    mode: Mode,
    paused: bool,
    hovered_node: Option<NodeId>,
}
enum Selection {
    Node(NodeId),
    Edge(EdgeId),
}
impl Selection {
    fn get_center(&self, state: &State) -> Vec2 {
        match self {
            &Self::Node(node) => state.nodes[node].pos,
            &Self::Edge(edge) => {
                let (u, v) = state.edges[edge].nodes;
                (state.nodes[u].pos + state.nodes[v].pos) / 2.0
            }
        }
    }
}

enum Mode {
    Base { selected_node: Option<NodeId> },
    AddEdge { first: Option<NodeId> },
    Delete(Option<Selection>),
    UpdNode { kind: NodeKind },
}
impl Default for Mode {
    fn default() -> Self {
        return Self::Base {
            selected_node: None,
        };
    }
}
impl State {
    fn add_node(&mut self, node: Node) -> NodeId {
        let id = self.nodes.insert(node);
        self.adj.insert(id, Default::default());
        id
    }
    fn add_edge(&mut self, edge: Edge) -> EdgeId {
        let (u, v) = edge.nodes;
        let id = self.edges.insert(edge);
        self.adj[u].outgoing.push((v, id));
        self.adj[v].incoming.push((u, id));
        id
    }
    fn remove_node(&mut self, node: NodeId) {
        let adjlist = self.adj.remove(node).unwrap();
        for (u, edge) in adjlist.incoming {
            let pos = self.adj[u].outgoing.index_of(&(node, edge)).unwrap();
            self.adj[u].outgoing.swap_remove(pos);
            self.edges.remove(edge);
        }

        for (u, edge) in adjlist.outgoing {
            let pos = self.adj[u].incoming.index_of(&(node, edge)).unwrap();
            self.adj[u].incoming.swap_remove(pos);
            self.edges.remove(edge);
        }

        self.nodes.remove(node);
    }
    fn remove_edge(&mut self, edge: EdgeId) {
        let (u, v) = self.edges.remove(edge).unwrap().nodes;
        let idx = self.adj[u].outgoing.index_of(&(v, edge)).unwrap();
        self.adj[u].outgoing.swap_remove(idx);
        let idx = self.adj[v].incoming.index_of(&(u, edge)).unwrap();
        self.adj[v].incoming.swap_remove(idx);
    }
    fn get_closest_node(&self, pos: Vec2) -> Option<(NodeId, f32)> {
        let mut min_dist = f32::INFINITY;
        let mut closest = None;
        for (id, node) in &self.nodes {
            let d = node.pos.distance_squared(pos);
            if d < min_dist {
                closest = Some(id);
                min_dist = d;
            }
        }
        closest.map(|e| (e, min_dist))
    }
}

fn handle_input(state: &mut State) {
    let cam_speed = 150.0;
    if is_key_down(KeyCode::A) {
        state.camera_pos.x -= cam_speed * state.dt;
    }
    if is_key_down(KeyCode::D) {
        state.camera_pos.x += cam_speed * state.dt;
    }
    if is_key_down(KeyCode::W) {
        state.camera_pos.y -= cam_speed * state.dt;
    }
    if is_key_down(KeyCode::S) {
        state.camera_pos.y += cam_speed * state.dt;
    }
    if is_key_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }
    if is_key_pressed(KeyCode::Escape) {
        state.mode = Mode::Base {
            selected_node: None,
        };
    }
    if is_mouse_button_pressed(MouseButton::Right) {
        state.mode = Mode::Base {
            selected_node: None,
        };
    }
    if is_key_pressed(KeyCode::X) {
        if matches!(state.mode, Mode::Delete(_)) {
            state.mode = Mode::Base {
                selected_node: None,
            };
        } else {
            state.mode = Mode::Delete(None);
        }
    }
    match &state.mode {
        Mode::Base { selected_node } => {
            if is_key_pressed(KeyCode::Key1) {
                state.mode = Mode::AddEdge { first: None };
            } else if is_mouse_button_down(MouseButton::Left) && selected_node.is_none() {
                for i in state.nodes.keys() {
                    let node = &state.nodes[i];
                    if node.pos.distance_squared(state.mouse_pos) <= NODE_RADIUS.powi(2) {
                        state.mode = Mode::Base {
                            selected_node: Some(i),
                        };
                        break;
                    }
                }
            }
            if is_mouse_button_released(MouseButton::Left) {
                state.mode = Mode::Base {
                    selected_node: None,
                };
            }
        }
        Mode::Delete(sel) => {
            if is_mouse_button_pressed(MouseButton::Left) {
                match sel {
                    Some(Selection::Node(id)) => state.remove_node(*id),
                    Some(Selection::Edge(id)) => state.remove_edge(*id),
                    None => {}
                }
                state.mode = Mode::Delete(None)
            }
        }
        Mode::AddEdge { first } => {
            if is_mouse_button_pressed(MouseButton::Left) {
                match *first {
                    Some(first_id) => {
                        let id;
                        if let Some(h) = state.hovered_node {
                            id = h;
                        } else {
                            id = state.add_node(Node::new(state.mouse_pos.x, state.mouse_pos.y));
                        }
                        state.add_edge(Edge::new(first_id, id));
                        state.mode = Mode::AddEdge { first: Some(id) };
                    }
                    None => {
                        if let Some(id) = state.hovered_node {
                            state.mode = Mode::AddEdge { first: Some(id) };
                        } else {
                            let id =
                                state.add_node(Node::new(state.mouse_pos.x, state.mouse_pos.y));
                            state.mode = Mode::AddEdge { first: Some(id) };
                        }
                    }
                }
            } else if is_mouse_button_pressed(MouseButton::Right) {
                state.mode = Mode::AddEdge { first: None };
            } else if is_key_pressed(KeyCode::Key1) {
                state.mode = Mode::Base {
                    selected_node: None,
                };
            }
        }
        Mode::UpdNode { kind } => {
            if is_mouse_button_pressed(MouseButton::Left) {
                if let Some(id) = state.hovered_node {
                    state.nodes[id].kind = kind.clone();
                } else {
                    let kind = kind.clone();
                    let new_node = state.add_node(Node::new(state.mouse_pos.x, state.mouse_pos.y));
                    state.nodes[new_node].kind = kind;
                }
            }
        }
        _ => {}
    }
}
#[macroquad::main("moi")]
async fn main() {
    rand::srand(macroquad::miniquad::date::now() as _);
    println!("{}", rand::gen_range(0, 100));
    let mut state = State::default();
    let audio_system = Audio::new(load_samples().await);
    let ids = vec![
        state.add_node(Node::new(10.0, 10.0)),
        state.add_node(Node::new(50.0, 200.0)),
        state.add_node(Node::new(100.0, 100.0)),
        state.add_node(Node::new(200.0, 250.0)),
        state.add_node(Node::new(300.0, 200.0)),
        state.add_node(Node::new(-50.0, -50.0)),
    ];

    state.add_edge(Edge::new(ids[0], ids[1]));
    state.add_edge(Edge::new(ids[1], ids[2]));
    state.add_edge(Edge::new(ids[2], ids[3]));
    state.add_edge(Edge::new(ids[0], ids[4]));
    state.add_edge(Edge::new(ids[1], ids[4]));
    state.add_edge(Edge::new(ids[2], ids[4]));
    state.add_edge(Edge::new(ids[5], ids[0]));
    state.mode = Mode::Base {
        selected_node: None,
    };
    state.nodes[ids[5]].kind = NodeKind::Spawner {
        bar_delay: 4.0,
        next_spawn: 0.0,
    };
    let mut next_time_spawn = 0.0;

    // let mut nodes = vec![Node::new(10.0, 10.0), Node::new(100.0, 100.0)];
    // let edges = vec![Edge::new(0, 1)];
    loop {
        let dt = get_frame_time();
        let m_pos = Vec2::from(mouse_position());
        state.mouse_pos = m_pos + state.camera_pos;
        if !state.paused {
            state.time += dt;
        };
        state.dt = dt;
        if let Some((id, _)) = state
            .get_closest_node(state.mouse_pos)
            .filter(|e| e.1 <= 4.0 * NODE_RADIUS * NODE_RADIUS)
        {
            state.hovered_node = Some(id);
        } else {
            state.hovered_node = None;
        }
        for (id, node) in &mut state.nodes {
            if let NodeKind::Spawner {
                bar_delay,
                next_spawn,
            } = &mut node.kind
            {
                if *next_spawn <= state.time {
                    for (_, edge_id) in &state.adj[id].outgoing {
                        state.signals.push(Signal {
                            cur_edge: *edge_id,
                            start_time: *next_spawn,
                        })
                    }
                    *next_spawn += *bar_delay * BAR_TIME;
                    next_time_spawn = *next_spawn;
                }
            }
        }
        handle_input(&mut state);
        match &state.mode {
            Mode::Base { selected_node } => {
                if let Some(v) = selected_node {
                    state.nodes[*v].pos = state.mouse_pos;
                }
            }
            Mode::Delete(_) => {
                let mut min_dist = f32::INFINITY;
                let mut new_sel = None;
                for (id, node) in &state.nodes {
                    let d = node.pos.distance_squared(state.mouse_pos);
                    if d < min_dist {
                        new_sel = Some(Selection::Node(id));
                        min_dist = d;
                    }
                }
                let node_dist = min_dist;
                for (id, edge) in &state.edges {
                    let d = edge.distance_squared(state.mouse_pos, &state.nodes, 0.5);
                    if d < 4.0 * node_dist && d < min_dist {
                        min_dist = d;
                        new_sel = Some(Selection::Edge(id));
                    }
                }
                if min_dist > 100.0 * 100.0 {
                    new_sel = None;
                }
                state.mode = Mode::Delete(new_sel);
            }

            _ => {}
        }

        for i in (0..state.signals.len()).rev() {
            if !state.edges.contains_key(state.signals[i].cur_edge) {
                state.signals.swap_remove(i);
                continue;
            }
            let (u, v) = state.edges[state.signals[i].cur_edge].nodes;
            let dx = state.nodes[v].pos.x - state.nodes[u].pos.x;
            if dx < 0.0 {
                state.signals[i].start_time += dt;
                continue;
            }
            let t = state.time - state.signals[i].start_time;

            if t * SIGNAL_SPEED < dx {
                continue;
            }
            state.signals[i].start_time += dx / SIGNAL_SPEED;
            // let next = state.adj_list[v].choose();
            // println!("{}", state.adj_list[v].len());
            let start_time = state.signals[i].start_time;
            let cur = state.signals[i].cur_edge;
            let to = state.edges[cur].nodes.1;
            match state.nodes[to].kind {
                NodeKind::Sample(idx) => audio_system.play(idx),
                _ => {}
            }
            let pos = state.nodes[to].pos;
            spawn_particles(&mut state, pos);
            for &(_, e) in &state.adj[to].outgoing {
                state.signals.push({
                    Signal {
                        start_time,
                        cur_edge: e,
                    }
                });
            }

            state.signals.swap_remove(i);
        }
        for i in (0..state.particles.len()).rev() {
            if state.particles[i].end_time <= state.time {
                state.particles.swap_remove(i);
            } else {
                state.particles[i].update(dt);
            }
        }
        clear_background(BLACK);
        draw(&state);
        for i in 0..10 {
            if ui::root_ui().button(None, i.to_string()) {
                state.mode = Mode::UpdNode {
                    kind: NodeKind::Sample(i),
                };
                audio_system.play(i);
            }
        }
        let mut spawner = |i: usize| {
            if ui::root_ui().button(None, "S".to_string() + &i.to_string()) {
                state.mode = Mode::UpdNode {
                    kind: NodeKind::Spawner {
                        bar_delay: i as f32,
                        next_spawn: state.time.max(next_time_spawn),
                    },
                };
            }
        };
        spawner(1);
        spawner(2);
        spawner(3);
        spawner(4);
        spawner(8);

        next_frame().await;
    }
}
