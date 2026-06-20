use crate::{
    Point, PointBox,
    bsp::{IndexSet, Indexer},
    link::{Bundle, BunldeOpt, ContainedBy, Link, LinkContainer, LinkOpt, create_container_id},
    node::{Node, NodeOpt},
};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub struct Options {
    pub options: HashMap<String, LinkOpt>,
    pub bundle_ops: HashMap<String, BunldeOpt>,
    pub node_opts: HashMap<String, NodeOpt>,
}

pub struct BacklogUpdates {
    pub nodes: HashMap<u32, ()>,
    pub links: HashMap<u64, ()>,
}

impl BacklogUpdates {
    pub fn new() -> Self {
        return Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
        };
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
    }
}

pub struct NodeStates {
    updates: HashMap<u32, Node>,
    nodes: HashMap<u32, Node>,
    node_counter: u64,
    order: HashMap<u32, u64>,
}

impl NodeStates {
    pub fn new() -> Self {
        return Self {
            updates: HashMap::new(),
            nodes: HashMap::new(),
            node_counter: 0,
            order: HashMap::new(),
        };
    }

    pub fn insert(&mut self, node: Node) -> Option<Node> {
        self.updates.remove(&node.id);
        let id = node.id;
        let res = self.nodes.insert(node.id, node);
        if res.is_none() {
            self.order.insert(id, self.node_counter);
            self.node_counter += 1;
        }
        return res;
    }

    pub fn remove(&mut self, id: u32) -> Option<Node> {
        self.updates.remove(&id);
        self.order.remove(&id);
        return self.nodes.remove(&id);
    }

    pub fn update(&mut self, node: Node) -> Option<Node> {
        return self.updates.insert(node.id, node);
    }

    pub fn get(&self, id: u32) -> Option<&Node> {
        if let Some(node) = self.updates.get(&id) {
            return Some(node);
        } else if let Some(node) = self.nodes.get(&id) {
            return Some(node);
        }
        return None;
    }

    pub fn get_order(&self, id: u32) -> u64 {
        return *self.order.get(&id).unwrap();
    }
}

#[wasm_bindgen]
pub struct Calculator {
    link_counter: u64, // Sequence in which a link container was added.
    node_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Node instances to LinkContainer instances
    nodes: NodeStates,

    links: HashMap<u64, LinkContainer>,
    link_order: HashMap<u64, u64>,
    link_mouse_bound: i32,
    screen_bound: i32,
    options: Options,
    link_index_mouse: Indexer<u64>,
    link_index_screen: Indexer<u64>,
    node_index_mouse: Indexer<u32>,
    node_mouse_bound: i32,
    node_index_screen: Indexer<u32>,
    backlog: BacklogUpdates,
    drag: bool,
}

macro_rules! update_link {
    ($self:expr,$lc:expr) => {{
        $lc.update(&$self.nodes, &$self.options);
        if $self.drag {
            $self.backlog.links.insert($lc.id, ());
        } else {
            $self
                .link_index_mouse
                .update($lc.id, $lc.mouse_index($self.link_mouse_bound));
        }
        $self
            .link_index_screen
            .update($lc.id, $lc.screen_index($self.screen_bound));
    }};
}

#[wasm_bindgen]
impl Calculator {
    fn add_lc(&mut self, id: u64) {
        if self.links.get(&id).is_none() {
            let next = self.link_counter;
            self.link_counter += 1;
            self.link_order.insert(next, id);
            let lc = LinkContainer::new_id(id);
            self.links.insert(id, lc);
        }
    }
    fn remove_lc(&mut self, id: u64) {
        if let Some(l) = self.links.get(&id) {
            if !l.is_empty() {
                return;
            }
            let (src, dst) = l.get_node_ids();
            for n in [src, dst] {
                let nl = self.node_links.get_mut(&n).unwrap();
                nl.remove(&id);
                if nl.is_empty() {
                    self.node_links.remove(&n);
                }
            }
            self.links.remove(&id);
        }
    }

    pub fn remove_link(&mut self, src: u32, dst: u32, id: u32) -> Option<Link> {
        let lid = create_container_id(src, dst);
        let mut res = None;
        if let Some(lc) = self.links.get_mut(&lid) {
            res = lc.remove_link(id);
            update_link!(self, lc);
        }
        self.remove_lc(lid);
        return res;
    }

    pub fn remove_bundle(&mut self, src: u32, dst: u32, id: u32) -> Option<Bundle> {
        let lid = create_container_id(src, dst);
        let mut res = None;
        if let Some(lc) = self.links.get_mut(&lid) {
            res = lc.remove_bundle(id);
            update_link!(self, lc);
        }
        self.remove_lc(lid);
        return res;
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
        let id = link.get_container_id();
        self.add_lc(id);
        let lc = self.links.get_mut(&id).unwrap();
        let res = lc.add_link(link);
        update_link!(self, lc);

        return res;
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Option<Bundle> {
        let id = bundle.get_container_id();
        self.add_lc(id);

        let lc = self.links.get_mut(&id).unwrap();
        let res = lc.add_bundle(bundle);
        update_link!(self, lc);

        return res;
    }

    pub fn add_node(&mut self, node: Node) -> Option<Node> {
        let id = node.id;
        let new_m = Some(node.index_bound(self.node_mouse_bound));
        let new_s = Some(node.index_bound(self.screen_bound));
        let res = self.nodes.insert(node);
        let mut old_m = None;
        let mut old_s = None;
        if let Some(old_node) = &res {
            old_m = Some(old_node.index_bound(self.node_mouse_bound));
            old_s = Some(old_node.index_bound(self.screen_bound));
        }
        self.index_node(id, (old_m, new_m), (old_s, new_s));

        return res;
    }

    fn index_node(&mut self, id: u32, mi: IndexSet, si: IndexSet) {
        if self.drag {
            self.backlog.nodes.insert(id, ());
        } else {
            self.node_index_mouse.update(id, mi);
        }
        self.node_index_screen.update(id, si);
        let mut known = HashMap::new();
        self.update_links(id, &mut known);
    }

    fn update_links(&mut self, node_id: u32, known: &mut HashMap<u64, ()>) {
        if let Some(nl) = self.node_links.get(&node_id) {
            for lid in nl.keys() {
                if known.contains_key(lid) {
                    continue;
                }
                let link = self.links.get_mut(lid).unwrap();
                update_link!(self, link);
            }
        }
    }

    pub fn remove_node(&mut self, id: u32) -> Option<Node> {
        let res = self.nodes.remove(id);
        if let Some(node) = &res {
            let old_m = Some(node.index_bound(self.node_mouse_bound));
            let old_s = Some(node.index_bound(self.screen_bound));

            self.index_node(id, (old_m, None), (old_s, None));
        }
        return res;
    }

    pub fn move_nodes(&mut self, node_ids: &[u32], p: &Point) {
        let mut ns = HashMap::new();
        let nodes = &mut self.nodes;
        for id in node_ids {
            let new;
            if let Some(node) = nodes.get(*id) {
                new = node.transform(p.x, p.y, 0.0, 0.0);
            } else {
                continue;
            }
            nodes.update(new);
            ns.insert(*id, ());
        }

        let mut known = HashMap::new();
        for id in ns.keys() {
            self.update_links(*id, &mut known);
        }
    }
}
