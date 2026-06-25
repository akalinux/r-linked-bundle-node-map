use crate::{
    CalculatorTrait, Point, Transform,
    bsp::Indexers,
    constants::DEFAULT_OPT_NAME,
    link::{
        Bundle, BunldeOpt, ContainedBy, Link, LinkContainer, LinkContainerOpt, LinkOpt,
        create_container_id,
    },
    node::{Node, NodeOpt, NodeStates},
};
use std::collections::HashMap;
use std::mem;
use wasm_bindgen::prelude::*;

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
#[wasm_bindgen(inspectable)]
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

#[wasm_bindgen]
pub struct Calculator {
    node_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Node instances to LinkContainer instances
    nodes: NodeStates,

    backlog: BacklogUpdates,
    links: HashMap<u64, LinkContainer>,
    animations: HashMap<u64, ()>,
    options: Options,
    transform: Transform,
    indexer: Indexers,
}

macro_rules! cul_lc {
    ($self:ident,$lid:ident) => {{
        if $self.links.get(&$lid).unwrap().is_empty() {
            let lc = $self.links.remove(&$lid).unwrap();
            let (src, dst) = lc.get_node_ids();
            for id in [src, dst] {
                let t = $self.node_links.get_mut(&id).unwrap();
                if t.is_empty() {
                    $self.node_links.remove(&id);
                }
            }
        } else {
            let lc = $self.links.get_mut(&$lid).unwrap();
            lc.update(&$self.nodes, &mut $self.options);
            $self.indexer.add_link(lc);
        }
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

    pub fn add_node(&mut self, node: Node) -> Option<Node> {
        if let Some(node) = self.nodes.get(node.id) {
            self.indexer.clear_node(node);
        }
        self.indexer.add_node(&node);
        let mut known = HashMap::new();
        self.reindex_node_links(node.id, &mut known);

        return self.nodes.insert(node);
    }
    fn reindex_node_links(&mut self, node_id: u32, known: &mut HashMap<u64, ()>) {
        if !self.node_links.contains_key(&node_id) {
            return;
        }

        for lid in self.node_links.get(&node_id).unwrap().keys() {
            let link = self.links.get_mut(lid).unwrap();
            if known.contains_key(lid) {
                continue;
            }
            known.insert(*lid, ());
            link.update(&self.nodes, &mut self.options);
        }
    }

    pub fn remove_node(&mut self, id: u32) -> Option<Node> {
        let res = self.nodes.remove(id);
        if let Some(node) = &res {
            let mut known = HashMap::new();
            self.reindex_node_links(node.id, &mut known);
        }

        return res;
    }

    fn manage_lc(&mut self, link_id: u64) -> &mut LinkContainer {
        if self.links.contains_key(&link_id) {
            return self.links.get_mut(&link_id).unwrap();
        }

        let lc = LinkContainer::new_id(link_id);
        self.links.insert(link_id, lc);
        return self.links.get_mut(&link_id).unwrap();
    }

    pub fn remove_link(&mut self, src: u32, dst: u32, id: u32) -> Option<Link> {
        let res;
        let lid = create_container_id(src, dst);

        {
            let lc = self.manage_lc(lid);
            res = lc.remove_link(id);
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn remove_bundle(&mut self, src: u32, dst: u32, id: u32) -> Option<Bundle> {
        let lid = create_container_id(src, dst);
        let res;
        {
            let lc = self.manage_lc(lid);
            res = lc.remove_bundle(id);
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
        let lid = link.get_container_id();
        let res;
        {
            let lc = self.manage_lc(lid);
            res = lc.add_link(link);
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Option<Bundle> {
        let lid = bundle.get_container_id();
        let res;
        {
            let lc = self.manage_lc(lid);
            res = lc.add_bundle(bundle);
        }
        cul_lc!(self, lid);
        return res;
    }

    fn move_related_nodes(
        node_id: &u32,
        nodes: &mut NodeStates,
        known: &mut HashMap<u32, ()>,
        p: &Point,
        idx: &mut Indexers,
    ) {
        if known.contains_key(node_id) {
            return;
        }
        let node;
        if let Some(n) = nodes.get(*node_id) {
            idx.clear_node(n);

            node = n.transform(p.x, p.y, 0.0, 0.0);
        } else {
            return;
        }
        nodes.update(node.transform(p.x, p.y, 0.0, 0.0));
        known.insert(*node_id, ());
        idx.index_screen_node(&node);
        for node_id in &node.linked {
            Self::move_related_nodes(node_id, nodes, known, p, idx);
        }
    }

    pub fn move_nodes(&mut self, node_ids: &[u32], p: &Point) {
        let mut ns = HashMap::new();
        let idx = &mut self.indexer;
        {
            let nodes = &mut self.nodes;
            for id in node_ids {
                Self::move_related_nodes(id, nodes, &mut ns, p, idx);
            }
        }

        let backlog = &mut self.backlog;
        let mut ls = HashMap::new();
        for id in ns.keys() {
            if !backlog.nodes.contains_key(id) {
                backlog.nodes.insert(*id, ());
            }
            if let Some(links) = self.node_links.get(id) {
                for lid in links.keys() {
                    if ls.contains_key(lid) {
                        continue;
                    }
                    let link = self.links.get_mut(lid).unwrap();
                    ls.insert(*lid, ());
                    idx.clear_link(link);
                    idx.index_screen_link(link);
                    if !backlog.links.contains_key(lid) {
                        backlog.links.insert(*lid, ());
                    }
                }
            }
        }
    }
}
