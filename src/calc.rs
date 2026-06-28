use crate::{
    CalculatorTrait, Point, Transform,
    bsp::Indexers,
    constants::DEFAULT_NODE_R,
    link::{
        Bundle, BunldeOpt, Link, LinkContainer, LinkContainerOpt, LinkOpt, SrcDstIs,
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
    pub fn new(size: usize) -> Self {
        return Self {
            nodes: HashMap::with_capacity(size),
            links: HashMap::with_capacity(size),
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
                return self.$field.insert(opt.id, opt);
            }
            pub fn $get<'a>(&mut self, id: &u32) -> &'a $t {
                if let Some(v) = self.$field.get(id) {
                    return unsafe { mem::transmute(v) };
                } else if let Some(v) = self.$field.get(&0) {
                    return unsafe { mem::transmute(v) };
                }
                // not even the default option exists!
                let opt = <$t>::defaults();
                self.$field.insert(opt.id.clone(), opt);
                return unsafe { mem::transmute(self.$field.get(&0).unwrap()) };
            }
            pub fn $del(&mut self, id: &u32) -> Option<$t> {
                return self.$field.remove(id);
            }
        }
        impl Calculator {
            pub fn $set(&mut self, opt: $t) -> Option<$t> {
                return self.options.$set(opt);
            }
            pub fn $get(&mut self, id: &u32) -> &$t {
                return self.options.$get(id);
            }
            pub fn $del(&mut self, id: &u32) -> Option<$t> {
                return self.options.$del(id);
            }
        }
    };
}

pub struct Options {
    pub link: HashMap<u32, LinkOpt>,
    pub bundle: HashMap<u32, BunldeOpt>,
    pub node: HashMap<u32, NodeOpt>,
    pub lc: HashMap<u32, LinkContainerOpt>,
}
impl Options {
    pub fn new() -> Self {
        return Self {
            link: HashMap::with_capacity(10),
            bundle: HashMap::with_capacity(10),
            node: HashMap::with_capacity(10),
            lc: HashMap::with_capacity(10),
        };
    }
}
build_opts!(LinkOpt, link, get_link, set_link, rm_link);
build_opts!(BunldeOpt, bundle, get_bundle, set_bundle, rm_bundle);
build_opts!(NodeOpt, node, get_node, set_node, rm_node);
build_opts!(LinkContainerOpt, lc, get_lc, set_lc, rm_lc);

#[wasm_bindgen]
pub struct Calculator {
    node_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Node instances to LinkContainer instances
    bundle_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Bundle instances to LinkContainer instances
    link_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Link instances to LinkContainer instances
    nodes: NodeStates,

    backlog: BacklogUpdates,
    links: HashMap<u64, LinkContainer>,
    animations: HashMap<u64, ()>,
    options: Options,
    transform: Transform,
    indexer: Indexers,
}

impl CalculatorTrait for Calculator {}

macro_rules! manage_linked {
    ($self:ident,$field:ident,$add:expr,$el:expr) => {
        let lid = $el.get_container_id();
        if $add {
            match $self.$field.get_mut(&$el.id) {
                Some(f) => {
                    f.insert(lid, ());
                }
                None => {
                    let mut hm = HashMap::new();
                    hm.insert(lid, ());
                    $self.$field.insert($el.id, hm);
                }
            }
        } else {
            let mut rm = false;
            match $self.$field.get_mut(&$el.id) {
                Some(f) => {
                    f.remove(&lid);
                    if f.is_empty() {
                        rm = true;
                    }
                }
                None => (),
            }
            if rm {
                $self.$field.remove(&$el.id);
            }
        }
    };
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
            lc.update(&$self.nodes, &mut $self.options, &mut $self.animations);
            $self.indexer.add_link(lc);
        }
    }};
}

#[wasm_bindgen]
impl Calculator {
    #[wasm_bindgen(constructor)]
    pub fn new(node_mouse_b: i64, link_mouse_b: i64, screen_mouse_b: i64, size: usize) -> Self {
        return Self {
            links: HashMap::with_capacity(size),
            animations: HashMap::with_capacity(size),
            nodes: NodeStates::new(size),
            backlog: BacklogUpdates::new(size),
            indexer: Indexers::new(node_mouse_b, link_mouse_b, screen_mouse_b, size),
            options: Options::new(),
            transform: Transform {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            },
            node_links: HashMap::with_capacity(size),
            bundle_links: HashMap::with_capacity(size),
            link_links: HashMap::with_capacity(size),
        };
    }
    pub fn get_node_changes(&self) -> Vec<Node> {
        return self.nodes.get_node_changes();
    }

    pub fn from_defaults() -> Self {
        let mouse_b = (DEFAULT_NODE_R as i64) * 4;
        let link_b = mouse_b * 4;
        let screen_b = link_b * 4;
        return Self::new(mouse_b, link_b, screen_b, 256);
    }
    pub fn get_transform(&self) -> Transform {
        return self.transform;
    }
    pub fn set_transform(&mut self, t: &Transform) {
        self.transform = *t;
    }

    pub fn node_add(&mut self, node: Node) -> Option<Node> {
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
            link.update(&self.nodes, &mut self.options, &mut self.animations);
        }
    }

    pub fn node_remove(&mut self, id: u32) -> Option<Node> {
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

    pub fn link_remove(&mut self, src: u32, dst: u32, id: u32) -> Option<Link> {
        let res;
        let lid = create_container_id(src, dst);

        {
            let lc = self.manage_lc(lid);
            res = lc.remove_link(id);
            if let Some(link) = &res {
                manage_linked!(self, bundle_links, false, link);
            }
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn bundle_remove(&mut self, src: u32, dst: u32, id: u32) -> Option<Bundle> {
        let lid = create_container_id(src, dst);
        let res;
        {
            let lc = self.manage_lc(lid);
            res = lc.remove_bundle(id);
            if let Some(bundle) = &res {
                manage_linked!(self, bundle_links, false, bundle);
            }
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn link_add(&mut self, link: Link) -> Option<Link> {
        let lid = link.get_container_id();
        manage_linked!(self, link_links, true, link);
        let res;
        {
            let lc = self.manage_lc(lid);
            res = lc.add_link(link);
        }
        cul_lc!(self, lid);
        return res;
    }

    pub fn bundle_add(&mut self, bundle: Bundle) -> Option<Bundle> {
        let lid = bundle.get_container_id();
        let res;
        manage_linked!(self, bundle_links, true, bundle);
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
