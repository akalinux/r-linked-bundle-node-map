use crate::{
    CalculatorTrait, Point, PointBox, Transform,
    bsp::{IndexSet, Indexer},
    constants::DEFAULT_OPT_NAME,
    link::{
        Bundle, BunldeOpt, ContainedBy, Link, LinkContainer, LinkContainerOpt, LinkOpt,
        create_container_id,
    },
    node::{Node, NodeOpt},
};
use std::collections::HashMap;
use std::mem;
use wasm_bindgen::prelude::*;

macro_rules! build_opts {
    ($t:ty,$field:ident,$get:ident,$set:ident,$del:ident) => {
        impl Options {
            pub fn $set(&mut self, opt: $t) -> Option<$t> {
                return self.$field.insert(String::from(&opt.id), opt);
            }
            pub fn $get<'a>(&mut self, id: &String) -> &'a $t {
                if let Some(v) = self.$field.get(id) {
                    return unsafe { mem::transmute(v) };
                } else if let Some(v) = self.$field.get(&String::from(DEFAULT_OPT_NAME)) {
                    return unsafe { mem::transmute(v) };
                }
                // not even the default option exists!
                let opt = <$t>::defaults();
                self.$field.insert(opt.id.clone(), opt);
                return unsafe {
                    mem::transmute(self.$field.get(&String::from(DEFAULT_OPT_NAME)).unwrap())
                };
            }
            pub fn $del(&mut self, id: &String) -> Option<$t> {
                return self.$field.remove(id);
            }
        }
        impl Calculator {
            pub fn $set(&mut self, opt: $t) -> Option<$t> {
                return self.options.$set(opt);
            }
            pub fn $get(&mut self, id: &String) -> &$t {
                return self.options.$get(id);
            }
            pub fn $del(&mut self, id: &String) -> Option<$t> {
                return self.options.$del(id);
            }
        }
    };
}

pub struct Options {
    pub link: HashMap<String, LinkOpt>,
    pub bundle: HashMap<String, BunldeOpt>,
    pub node: HashMap<String, NodeOpt>,
    pub lc: HashMap<String, LinkContainerOpt>,
}
impl Options {
    pub fn new() -> Self {
        return Self {
            link: HashMap::new(),
            bundle: HashMap::new(),
            node: HashMap::new(),
            lc: HashMap::new(),
        };
    }
}
build_opts!(LinkOpt, link, get_link, set_link, rm_link);
build_opts!(BunldeOpt, bundle, get_bundle, set_bundle, rm_bundle);
build_opts!(NodeOpt, node, get_node, set_node, rm_node);
build_opts!(LinkContainerOpt, lc, get_lc, set_lc, rm_lc);
#[wasm_bindgen]
pub struct Move {
    pub start: Point,
}

#[wasm_bindgen]
impl Move {
    #[wasm_bindgen(constructor)]
    pub fn new(p: &Point) -> Self {
        let res = Self { start: *p };

        return res;
    }
    pub fn stop(&mut self, p: &Point) -> Point {
        let diff = Point {
            x: p.x - self.start.x,
            y: p.y - self.start.y,
        };
        self.start = *p;

        return diff;
    }
}

impl CalculatorTrait for Move {}

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
    transform: Transform,
}

macro_rules! update_link {
    ($self:expr,$lc:expr) => {{
        $lc.update(&$self.nodes, &mut $self.options);
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
    pub fn get_transform(&self) -> Transform {
        return self.transform;
    }
    pub fn set_transform(&mut self, t: &Transform) {
        self.transform = *t;
    }
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
