use crate::{
    Point, PointBox,
    bsp::{IndexSet, Indexer},
    link::{Bundle, BunldeOpt, ContainedBy, Link, LinkContainer, LinkOpt, create_container_id},
    node::Node,
};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

pub struct Options {
    pub options: HashMap<String, LinkOpt>,
    pub bundle_ops: HashMap<String, BunldeOpt>,
}

#[wasm_bindgen]
pub struct Calculator {
    node_counter: u64,
    node_order: HashMap<u32, u64>,
    nodes: HashMap<u32, Node>,
    node_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Node instances to LinkContainer instances
    node_u: HashMap<u32, Node>,                 // Updates to nodes

    links: HashMap<u64, LinkContainer>,
    link_counter: u64,
    link_order: HashMap<u64, u64>,
    link_mouse_bound: i32,
    screen_bound: i32,
    options: Options,
    link_index_mouse: Indexer<u64>,
    link_index_screen: Indexer<u64>,
    node_index_mouse: Indexer<u32>,
    node_mouse_bound: i32,
    node_index_screen: Indexer<u32>,
}

macro_rules! update_link {
    ($self:expr,$lc:expr,$lid:expr) => {{
        $lc.update(&$self.nodes, &$self.options);
        $self
            .link_index_mouse
            .update($lid, $lc.mouse_index($self.link_mouse_bound));
        $self
            .link_index_screen
            .update($lid, $lc.screen_index($self.screen_bound));
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
            update_link!(self, lc, lid);
            /*
            lc.update(&self.nodes, &self.options);
            self.link_index_mouse
                .update(lid, lc.mouse_index(self.link_mouse_bound));
            self.link_index_screen
                .update(lid, lc.screen_index(self.screen_bound));
            */
        }
        self.remove_lc(lid);
        return res;
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
        let id = link.get_container_id();
        self.add_lc(id);
        let lc = self.links.get_mut(&id).unwrap();
        let res = lc.add_link(link);
        update_link!(self, lc, id);
        /*
        lc.update(&self.nodes, &self.options);
        self.link_index_mouse
            .update(id, lc.mouse_index(self.link_mouse_bound));
        self.link_index_screen
            .update(id, lc.screen_index(self.screen_bound));
        */
        return res;
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Option<Bundle> {
        let id = bundle.get_container_id();
        self.add_lc(id);

        let lc = self.links.get_mut(&id).unwrap();
        let res = lc.add_bundle(bundle);
        lc.update(&self.nodes, &self.options);
        self.link_index_mouse
            .update(id, lc.mouse_index(self.link_mouse_bound));
        self.link_index_screen
            .update(id, lc.screen_index(self.screen_bound));

        return res;
    }

    pub fn add_node(&mut self, node: Node) -> Option<Node> {
        self.node_u.remove(&node.id);
        let id = node.id;
        let new_m = Some(node.index_bound(self.node_mouse_bound));
        let new_s = Some(node.index_bound(self.screen_bound));
        let res = self.nodes.insert(node.id, node);
        let mut old_m = None;
        let mut old_s = None;
        if let Some(old_node) = &res {
            old_m = Some(old_node.index_bound(self.node_mouse_bound));
            old_s = Some(old_node.index_bound(self.screen_bound));
        } else {
            let next = self.node_counter;
            self.node_counter += 1;
            self.node_order.insert(id, next);
        }
        self.index_node(id, (old_m, new_m), (old_s, new_s));

        return res;
    }

    fn index_node(&mut self, id: u32, mi: IndexSet, si: IndexSet) {
        self.node_index_mouse.update(id, mi);
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
                link.update(&self.nodes, &self.options);
                self.link_index_mouse
                    .update(*lid, link.mouse_index(self.link_mouse_bound));
                self.link_index_screen
                    .update(*lid, link.screen_index(self.screen_bound));
            }
        }
    }

    pub fn remove_node(&mut self, id: u32) -> Option<Node> {
        self.node_u.remove(&id);
        self.node_order.remove(&id);
        let res = self.nodes.remove(&id);
        if let Some(node) = &res {
            let old_m = Some(node.index_bound(self.node_mouse_bound));
            let old_s = Some(node.index_bound(self.screen_bound));

            self.index_node(id, (old_m, None), (old_s, None));
        }
        return res;
    }

    pub fn move_nodes(&mut self, nodes: &[u32], p: &Point) {
        let mut ns = HashMap::new();
        for id in nodes {
            let n;
            if let Some(node) = self.node_u.get(id) {
                n = node;
            } else if let Some(node) = self.nodes.get(id) {
                n = node;
            } else {
                continue;
            }
            ns.insert(n.id, ());

            self.node_u.insert(*id, n.transform(p.x, p.y, 0.0, 0.0));
        }

        let mut known = HashMap::new();
        for id in ns.keys() {
            self.update_links(*id, &mut known);
        }
    }
}

impl Calculator {
    pub fn get_current_node(&self, id: u32) -> Option<&Node> {
        let n;
        if let Some(node) = self.node_u.get(&id) {
            n = node;
        } else if let Some(node) = self.nodes.get(&id) {
            n = node;
        } else {
            return None;
        }
        return Some(n);
    }
}
