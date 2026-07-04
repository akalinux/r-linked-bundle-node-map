use crate::{
    CalculatorTrait, Point, PointBox, ScreenBox, Transform,
    bsp::{IdxBoxAction, IdxBoxIter, OnScreen, PointLookupResult, ScreenIndex, ScreenSlot},
    constants::{DEFAULT_NODE_R, ZERO_TRANSFORM},
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

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct BulkLoad {
    pub link_opts: Vec<LinkOpt>,
    pub bundle_ops: Vec<BunldeOpt>,
    pub node_ops: Vec<NodeOpt>,
    pub lc_ops: Vec<LinkContainerOpt>,
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
}

#[wasm_bindgen]
impl BulkLoad {
    #[wasm_bindgen(constructor)]
    pub fn new(
        link_opts: Vec<LinkOpt>,
        bundle_ops: Vec<BunldeOpt>,
        node_ops: Vec<NodeOpt>,
        lc_ops: Vec<LinkContainerOpt>,
        nodes: Vec<Node>,
        links: Vec<Link>,
        bundles: Vec<Bundle>,
    ) -> Self {
        Self {
            bundle_ops,
            node_ops,
            lc_ops,
            nodes,
            link_opts,
            links,
            bundles,
        }
    }
    pub fn nlb(nodes: Vec<Node>, links: Vec<Link>, bundles: Vec<Bundle>) -> Self {
        Self::new(
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            nodes,
            links,
            bundles,
        )
    }
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
calc_acl!(links, LinkStates);
calc_acl!(backlog, BacklogUpdates);
calc_acl!(animations, HashMap<u64,()>);
calc_acl!(options, Options);
calc_acl!(indexer, ScreenIndex);

macro_rules! calc_bulk {
    ($t:ty,$field:ident) => {
        paste! {
        impl <'c> Calculator {

            pub fn [<$field _opt_bulk>](&mut self,list: Vec<$t>) {
                self.options.$field.reserve(list.len());
                for o in list {
                    self.options.$field.insert(o.id,o);
                }
            }
        }
        }
    };
}
calc_bulk!(NodeOpt, node);
calc_bulk!(LinkContainerOpt, lc);
calc_bulk!(LinkOpt, link);
calc_bulk!(BunldeOpt, bundle);

impl CalculatorTrait for Calculator {}

impl Calculator {
    pub fn on_screen<'s>(&'s self, width: u32, height: u32, t: &Transform) -> OnScreen<'s> {
        self.indexer.on_screen(width, height, t)
    }
}
#[wasm_bindgen]
impl Calculator {
    pub fn current_screen(&self) -> Option<ScreenBox> {
        self.indexer.max_screen()
    }
    pub fn default_screen_block(&self) -> ScreenBox {
        let step = self.indexer.step;
        ScreenBox::new(&ZERO_TRANSFORM, step as u32, step as u32, step)
    }

    pub fn wanted_screens(&self, width: u32, height: u32, t: &Transform) -> Vec<ScreenBox> {
        let mut res = Vec::new();
        let step = self.indexer.step;
        let new = Some(ScreenBox::new(t, width, height, step).getxy_bounds());
        let old;
        match self.indexer.max_screen() {
            Some(s) => old = Some(s.getxy_bounds()),
            _ => old = None,
        }
        for (x, y, state) in IdxBoxIter::new(old, new, step) {
            match state {
                IdxBoxAction::Add => res.push(ScreenBox {
                    width: step as u32,
                    height: step as u32,
                    x,
                    y,
                    step,
                }),
                _ => (),
            }
        }
        return res;
    }
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

    pub fn bulk_load(&mut self, bl: BulkLoad) {
        self.lc_opt_bulk(bl.lc_ops);
        self.node_opt_bulk(bl.node_ops);
        self.link_opt_bulk(bl.link_opts);
        self.bundle_opt_bulk(bl.bundle_ops);
        self.links.bulk = true;
        self.nodes.reserve(bl.nodes.len());

        for node in bl.nodes {
            let id = node.id;
            self.node_add(node);
            self.links.update_links(
                id,
                &mut self.indexer,
                &self.nodes,
                &mut self.options,
                &mut self.animations,
                &mut self.backlog,
            );
        }
        self.links.reserve(bl.links.len());

        for link in bl.links {
            self.link_add(link);
        }

        for bundle in bl.bundles {
            self.bundle_add(bundle);
        }

        self.finish_bulk_load();
    }

    pub fn start_bulk_load(&mut self) {
        self.links.bulk = true;
    }

    pub fn finish_bulk_load(&mut self) {
        self.links.bulk = false;
        self.links.bulk_update(
            self.backlog.links.keys(),
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
        );

        self.options.lc.shrink_to_fit();
        self.options.link.shrink_to_fit();
        self.options.node.shrink_to_fit();
        self.options.bundle.shrink_to_fit();
        self.nodes.shrink_to_fit();
        self.backlog.clear();
        self.links.shrink_to_fit();
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
        if self.links.bulk {
            return res;
        }
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

    pub fn in_point(&self, p: &Point, t: &Transform) -> Option<PointLookupResult> {
        self.indexer.in_point(p, t, &self.nodes, &self.links)
    }
    pub fn move_nodes(&mut self, node_ids: &[u32], p: &Point) {
        let idx = &mut self.indexer;
        let mut ns = Vec::with_capacity(node_ids.len() * 2);
        let mut iter = self.nodes.get_related(node_ids);

        let mut ls = HashMap::new();
        let nl = &self.links.node_links;
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
                    Some(node.index_bound(idx.step)),
                ),
            );
            iter.nodes.update(node);
            match nl.get(&src.id) {
                Some(l) => {
                    ls.reserve(l.len());
                    for i in l.keys() {
                        if !ls.contains_key(i) {
                            ls.insert(*i, ());
                        }
                    }
                }
                _ => (),
            }
        }

        let nodes = &self.nodes;
        let options = &mut self.options;
        let animations = &mut self.animations;
        for lid in ls.keys() {
            let link = self.links.get_mut(&lid).unwrap();
            link.update(nodes, options, animations);
            idx.index(ScreenSlot::Link(*lid), link.screen_index(idx.step, true));
        }
    }
}
