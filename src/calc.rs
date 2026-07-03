use crate::{
    CalculatorTrait, Point, PointBox,
    bsp::{ScreenIndex, ScreenSlot},
    constants::DEFAULT_NODE_R,
    link::{Bundle, BunldeOpt, Link, LinkContainerOpt, LinkOpt, LinkStates},
    node::{Node, NodeOpt, NodeStates},
};
use pastey::paste;
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
    links: LinkStates,
    nodes: NodeStates,
    backlog: BacklogUpdates,
    animations: HashMap<u64, ()>,
    options: Options,
    indexer: ScreenIndex,
}

macro_rules! calc_acl {
    ($field:ident,$t:ty) => {
        paste! {
            impl<'c> Calculator {
                pub fn [<$field>](&'c self) -> &'c $t {
                    return &self.$field
                }

                pub fn [<$field _mut>](&'c mut self) -> &'c mut $t {
                    return &mut self.$field
                }
            }
        }
    };
}

calc_acl!(nodes, NodeStates);
calc_acl!(backlog, BacklogUpdates);
calc_acl!(animations, HashMap<u64,()>);
calc_acl!(options, Options);
calc_acl!(indexer, ScreenIndex);

impl CalculatorTrait for Calculator {}

#[wasm_bindgen]
impl Calculator {
    pub fn new_with_settings(screen_mouse_b: i64, size: usize) -> Self {
        return Self {
            links: LinkStates::new(),
            animations: HashMap::with_capacity(size),
            nodes: NodeStates::new(size),
            backlog: BacklogUpdates::new(size),
            indexer: ScreenIndex::new(screen_mouse_b),
            options: Options::new(),
        };
    }

    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mouse_b = (DEFAULT_NODE_R as i64) * 4;
        let link_b = mouse_b * 4;
        let screen_b: i64 = link_b * 4;
        return Self::new_with_settings(screen_b, 256);
    }

    pub fn node_add(&mut self, node: Node) -> Option<Node> {
        let old;
        match self.nodes.get(node.id) {
            Some(node) => old = Some(node.index_bound(self.indexer.step)),
            None => old = None,
        };

        self.indexer.index(
            ScreenSlot::Node(node.id),
            (old, Some(node.index_bound(self.indexer.step))),
        );
        let node_id = node.id;
        let res = self.nodes.insert(node);
        self.links.update_links(
            node_id,
            &mut self.indexer,
            &mut self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.backlog,
        );
        return res;
    }

    pub fn node_remove(&mut self, id: u32) {
        let res = self.nodes.remove(id);
        if let Some(node) = &res {
            self.indexer.index(
                ScreenSlot::Node(id),
                (Some(node.index_bound(self.indexer.step)), None),
            );
            self.links.update_links(
                id,
                &mut self.indexer,
                &mut self.nodes,
                &mut self.options,
                &mut self.animations,
                &mut self.backlog,
            );
        }
    }

    pub fn link_remove(&mut self, id: u32) {
        self.links.link_remove(
            id,
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
            &mut self.backlog,
        )
    }

    pub fn bundle_remove(&mut self, id: u32) {
        self.links.bundle_remove(
            id,
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
            &mut self.backlog,
        )
    }

    pub fn link_add(&mut self, link: Link) {
        self.links.link_add(
            link,
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
            &mut self.backlog,
        );
    }

    pub fn group_add(&mut self, id: u32, nodes: &[u32]) -> Option<Vec<u32>> {
        return self.nodes.group_add(id, nodes);
    }

    pub fn group_remove(&mut self, id: u32) -> Option<Vec<u32>> {
        return self.nodes.group_remove(id);
    }

    pub fn bundle_add(&mut self, bundle: Bundle) {
        self.links.bundle_add(
            bundle,
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
            &mut self.backlog,
        );
    }

    pub fn move_nodes(&mut self, node_ids: &[u32], p: &Point) {
        let idx = &mut self.indexer;
        let mut ns = Vec::with_capacity(node_ids.len() * 2);
        let mut iter = self.nodes.get_related(node_ids);

        loop {
            let src;
            match iter.next() {
                Some(n) => src = n,
                _ => break,
            }
            ns.push(src.id);
            let node = src.transform(p.x, p.y, 0.0, 0.0);
            idx.index(
                ScreenSlot::Node(src.id),
                (
                    Some(src.index_bound(idx.step)),
                    Some(src.index_bound(idx.step)),
                ),
            );
            iter.nodes.update(node);
        }

        let mut ls = HashMap::new();
        for id in ns {
            let mut links: Vec<u64>;
            match self.links.node_links.get(&id) {
                Some(l) => {
                    links = Vec::with_capacity(l.len());
                    for i in l.keys() {
                        links.push(*i);
                    }
                }
                None => continue,
            }
            for lid in links {
                if ls.contains_key(&lid) {
                    continue;
                }
                let link = self.links.get_mut(&lid).unwrap();
                ls.insert(lid, ());
                idx.index(ScreenSlot::Link(lid), link.screen_index(idx.step, true));
            }
        }
    }
}
