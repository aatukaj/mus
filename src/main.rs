
// :TODO:
// - ui, macroquad egui??
// - more audio options,
// add/remove nodes and edges
// - stretch on x-axis <- changing translate
use macroquad::prelude::*;
mod audio;
mod particle;
mod translation;
use particle::{spawn_particles, Particle};
use slotmap::{SlotMap, SecondaryMap, DefaultKey, new_key_type};

new_key_type! {struct EdgeId; }
new_key_type! {struct NodeId; }
mod util;
use util::IndexOf as _;

mod render;
use render::draw;

const NODE_RADIUS: f32 = 14.0;
const PX_PER_BAR: f32 = 150.0;
const BAR_TIME: f32 = 10.0;
const SIGNAL_SPEED: f32 = PX_PER_BAR/BAR_TIME;

struct Signal {
    cur_edge: EdgeId,
    start_time: f32,
}

struct Node {
    pos: Vec2,
}
impl Node {
    fn new(x: f32, y: f32) -> Self {
        Self {
            pos: vec2(x, y),
        }
    }
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
                (state.nodes[u].pos+state.nodes[v].pos)/2.0
            }
        }

    }

}

enum Mode {
    Base {selected_node: Option<NodeId>, },
    AddEdge {first: Option<NodeId>, },
    AddNode,
    Delete(Option<Selection>),
}
impl Default for Mode {
    fn default() -> Self {
        return Self::Base { selected_node: None };
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

}

struct Edge {
    nodes: (NodeId, NodeId),
}
impl Edge {
    fn new(u: NodeId, v: NodeId) -> Self {
        Self { nodes: (u, v) }
    }
    fn distance_squared(&self, pos: Vec2, nodes: &SlotMap<NodeId, Node>, clamp: f32) -> f32 {
        let  a = nodes[self.nodes.0].pos;
        let  b=  nodes[self.nodes.1].pos;
        let l_sq = a.distance_squared(b);
        if l_sq==0.0 { return a.distance_squared(pos);}
        let t = ((pos-a).dot(b-a) / l_sq).clamp(clamp/2.0,1.0-clamp/2.0);
        let proj = a+t*(b-a);
        proj.distance_squared(pos)
    }
}
fn handle_input(state: &mut State) {
    let cam_speed = 100.0;
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
    match &state.mode {
        Mode::Base { selected_node } => {
            if is_key_pressed(KeyCode::X) {
                state.mode = Mode::Delete(None);
            }
            else if is_mouse_button_down(MouseButton::Left) && selected_node.is_none(){
                for i in state.nodes.keys() {
                    let node = &state.nodes[i];
                    if node.pos.distance_squared(state.mouse_pos) <= NODE_RADIUS.powi(2) {
                        state.mode = Mode::Base { selected_node: Some(i) };
                        break;
                    }
                }
            }
            if is_mouse_button_released(MouseButton::Left) {
                state.mode = Mode::Base { selected_node: None };
            }
        }
        Mode::Delete(sel) => {
            if is_mouse_button_pressed(MouseButton::Left) {
                match sel {
                    Some(Selection::Node(id)) => state.remove_node(*id),
                    Some(Selection::Edge(id)) => state.remove_edge(*id),
                    None => {},
                }
                state.mode = Mode::Delete(None)
            }
            if is_key_pressed(KeyCode::X) {
                state.mode = Mode::Base { selected_node: None }
            }
        }
        _ => {

        }

    }

}
#[macroquad::main("moi")]
async fn main() {
    let tx = audio::setup();
    rand::srand(macroquad::miniquad::date::now() as _);
    println!("{}", rand::gen_range(0, 100));
    let mut state= State::default();
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
    let id = state.add_edge(Edge::new(ids[5], ids[0]));
    state.mode = Mode::Base {
        selected_node: None,
    };

    // let mut nodes = vec![Node::new(10.0, 10.0), Node::new(100.0, 100.0)];
    // let edges = vec![Edge::new(0, 1)];
    let mut elapsed = 100.0;
    loop {
        let dt= get_frame_time();
        let m_pos = Vec2::from(mouse_position());
        state.mouse_pos = m_pos + state.camera_pos;
        state.time += dt;
        state.dt = dt;
        elapsed += get_frame_time();
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
                    let d= node.pos.distance_squared(state.mouse_pos);
                    if d<min_dist {
                        new_sel = Some(Selection::Node(id));
                        min_dist = d;
                    }
                }
                let node_dist = min_dist;
                for (id, edge) in &state.edges {
                    let d = edge.distance_squared(state.mouse_pos, &state.nodes, 0.5);
                    if d < 4.0*node_dist && d<min_dist {
                        min_dist = d;
                        new_sel = Some(Selection::Edge(id));
                    } 
                }
                if min_dist>100.0*100.0 {new_sel = None;}
                state.mode = Mode::Delete(new_sel);
            }

            _ => {},

        }

        for i in (0..state.signals.len()).rev() {
            if !state.edges.contains_key(state.signals[i].cur_edge) {
                state.signals.swap_remove(i);
                continue;
            }
            let (u, v) =state.edges[state.signals[i].cur_edge].nodes;
            let dx = state.nodes[v].pos.x - state.nodes[u].pos.x;
            if dx<0.0 {
                state.signals[i].start_time += dt;
                continue; 
            }
            let t = state.time - state.signals[i].start_time;

            if t*SIGNAL_SPEED<dx { continue; }
            tx.send(audio::Command::PlaySound).unwrap();
            state.signals[i].start_time += dx/SIGNAL_SPEED;
            // let next = state.adj_list[v].choose();
            // println!("{}", state.adj_list[v].len());
            let start_time = state.signals[i].start_time;
            let cur = state.signals[i].cur_edge;
            let to = state.edges[cur].nodes.1;
            let pos = state.nodes[to].pos;
            spawn_particles(&mut state, pos);
            for &(_, e) in &state.adj[to].outgoing {
                state.signals.push({Signal {
                    start_time,
                    cur_edge: e
                }});
            }

            state.signals.swap_remove(i);
        }
        for i in (0..state.particles.len()).rev() {
            if state.particles[i].end_time<=state.time {
                state.particles.swap_remove(i);
            } else {
                state.particles[i].update(dt);
            }
        }

        if elapsed >= 5.0 {
            elapsed = 0.0;
            state.signals.push(Signal {
                cur_edge: id,
                start_time: state.time,
            });
        }
        clear_background(BLACK);
        draw(&state);
      
        next_frame().await;
    }
}
