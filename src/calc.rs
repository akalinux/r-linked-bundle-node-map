use crate::{
    CalculatorTrait, Point, PointBox, ScreenBox, Transform,
    bsp::{IdxBoxAction, IdxBoxIter, OnScreen, PointLookupResult, ScreenIndex, ScreenSlot},
    constants::{DEFAULT_NODE_R, ZERO_TRANSFORM},
    link::{Bundle, BundleOpt, Link, LinkContainerOpt, LinkOpt, LinkStates},
    node::{Node, NodeOpt, NodeStates},
};
use pastey::paste;
use std::collections::{HashMap, HashSet};
use std::mem;
use wasm_bindgen::prelude::*;

pub struct BacklogUpdates {
    pub nodes: HashSet<u32>,
    pub links: HashSet<u64>,
}
impl BacklogUpdates {
    pub fn new() -> Self {
        return Self {
            nodes: HashSet::new(),
            links: HashSet::new(),
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
                let v = unsafe { self.$field.get(&0).unwrap_unchecked() };
                return unsafe { mem::transmute(v) };
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
    pub bundle: HashMap<u32, BundleOpt>,
    pub node: HashMap<u32, NodeOpt>,
    pub lc: LinkContainerOpt,
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone)]
pub struct BulkLoad {
    pub merge: bool,
    pub link_opts: Vec<LinkOpt>,
    pub bundle_ops: Vec<BundleOpt>,
    pub node_ops: Vec<NodeOpt>,
    pub lc_ops: LinkContainerOpt,
    pub nodes: Vec<Node>,
    pub boxes: Vec<Node>,
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
}

#[wasm_bindgen]
impl BulkLoad {
    #[wasm_bindgen(constructor)]
    pub fn new(
        merge: bool,
        links: Vec<Link>,
        link_opts: Vec<LinkOpt>,
        bundle_ops: Vec<BundleOpt>,
        lc_ops: LinkContainerOpt,
        node_ops: Vec<NodeOpt>,
        nodes: Vec<Node>,
        boxes: Vec<Node>,
        bundles: Vec<Bundle>,
    ) -> Self {
        Self {
            merge,
            bundle_ops,
            node_ops,
            lc_ops,
            nodes,
            boxes,
            link_opts,
            links,
            bundles,
        }
    }
    pub fn nlb(nodes: Vec<Node>, links: Vec<Link>, bundles: Vec<Bundle>) -> Self {
        Self::new(
            true,
            links,
            Vec::new(),
            Vec::new(),
            LinkContainerOpt::defaults(),
            Vec::new(),
            nodes,
            Vec::new(),
            bundles,
        )
    }
}
impl Options {
    pub fn get_lc(&self) -> &LinkContainerOpt {
        &self.lc
    }
    pub fn new() -> Self {
        return Self {
            link: HashMap::new(),
            bundle: HashMap::new(),
            node: HashMap::new(),
            lc: LinkContainerOpt::defaults(),
        };
    }
}
build_opts!(LinkOpt, link, get_link, set_link, rm_link);
build_opts!(BundleOpt, bundle, get_bundle, set_bundle, rm_bundle);
build_opts!(NodeOpt, node, get_node, set_node, rm_node);
//build_opts!(LinkContainerOpt, lc, get_lc, set_lc, rm_lc);

#[wasm_bindgen]
pub struct Calculator {
    #[wasm_bindgen(skip)]
    pub links: LinkStates,
    #[wasm_bindgen(skip)]
    pub nodes: NodeStates,
    #[wasm_bindgen(skip)]
    pub backlog: BacklogUpdates,
    #[wasm_bindgen(skip)]
    pub animations: HashSet<u64>,
    #[wasm_bindgen(skip)]
    pub options: Options,
    #[wasm_bindgen(skip)]
    pub indexer: ScreenIndex,
}

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
//calc_bulk!(LinkContainerOpt, lc);
calc_bulk!(LinkOpt, link);
calc_bulk!(BundleOpt, bundle);

#[wasm_bindgen(inspectable, getter_with_clone)]
pub struct MouseImpacted {
    pub nodes: Vec<u32>,
    pub boxes: Vec<u32>,
    pub bundles: Vec<u32>,
    pub links: Vec<u32>,
}

#[wasm_bindgen]
pub enum MouseEvent {
    CanvasMove(Transform),
    CanvasScale(Transform),
    Moved(MouseImpacted),
    MouseOver(MouseImpacted),
}
impl CalculatorTrait for Calculator {}

impl Calculator {
    pub fn in_point<'r>(&'r self, p: &Point, t: &Transform) -> PointLookupResult<'r> {
        self.indexer.in_point(p, t, &self.nodes, &self.links)
    }
    pub fn move_nodes(&mut self, node_ids: &[u32], p: &Point, use_groups: bool) -> MouseImpacted {
        let idx = &mut self.indexer;
        let mut boxes = Vec::new();
        let mut nodes = Vec::new();

        let mut ls = HashSet::new();
        let nl = &self.links.node_links;
        if use_groups {
            let mut iter = self.nodes.get_related(node_ids);
            loop {
                let (src, ss);
                match iter.next() {
                    Some(n) => (src, ss) = n,
                    _ => break,
                };
                match &ss {
                    ScreenSlot::Node(_) => nodes.push(src.id),
                    ScreenSlot::Box(_) => boxes.push(src.id),
                    _ => (),
                }
                iter.nodes
                    .update(Self::node_updates(idx, p, src, nl, &mut ls, ss));
            }
        } else {
            let mut known = HashSet::with_capacity(node_ids.len());
            for id in node_ids {
                if known.contains(id) {
                    continue;
                }
                known.insert(*id);
                let node;
                if let Some(src) = self.nodes.get_box(*id) {
                    boxes.push(*id);
                    node = Self::node_updates(idx, p, src, nl, &mut ls, ScreenSlot::Box(*id));
                } else if let Some(src) = self.nodes.get(*id) {
                    nodes.push(*id);
                    node = Self::node_updates(idx, p, src, nl, &mut ls, ScreenSlot::Node(*id));
                } else {
                    continue;
                }
                self.nodes.update(node);
            }
        }

        let ns = &self.nodes;
        let options = &mut self.options;
        let animations = &mut self.animations;
        for lid in ls.iter() {
            let link = unsafe { self.links.get_mut(lid).unwrap_unchecked() };
            link.update(ns, options, animations);
            idx.index(ScreenSlot::Link(*lid), link.screen_index(idx.step, true));
        }
        MouseImpacted {
            nodes,
            boxes,
            bundles: vec![],
            links: vec![],
        }
    }
    pub fn on_screen<'s>(&'s self, width: u32, height: u32, t: &Transform) -> OnScreen<'s> {
        self.indexer.on_screen(width, height, t)
    }
    pub fn default_screen_block(&self) -> ScreenBox {
        let step = self.indexer.step;
        ScreenBox::new(&ZERO_TRANSFORM, step as u32, step as u32, step)
    }

    pub fn get_src_dst_center(&self, src: u32, dst: u32) -> Point {
        let a = unsafe { self.nodes.get(src).unwrap_unchecked() };
        let b = unsafe { self.nodes.get(dst).unwrap_unchecked() };
        Point {
            x: (a.x + b.x) * 0.5,
            y: (a.y + b.y) * 0.5,
        }
    }
    fn node_updates(
        idx: &mut ScreenIndex,
        p: &Point,
        src: &Node,
        nl: &HashMap<u32, HashSet<u64>>,
        ls: &mut HashSet<u64>,
        ss: ScreenSlot,
    ) -> Node {
        let node = src.transform(p.x, p.y, 0.0, 0.0);
        idx.index(
            ss,
            (
                Some(src.index_bound(idx.step)),
                Some(node.index_bound(idx.step)),
            ),
        );
        match nl.get(&src.id) {
            Some(l) => {
                ls.reserve(l.len());
                for i in l.iter() {
                    if !ls.contains(i) {
                        ls.insert(*i);
                    }
                }
            }
            _ => (),
        };
        node
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
}
#[wasm_bindgen]
impl Calculator {
    pub fn new_with_settings(screen_mouse_b: i64, size: usize) -> Self {
        return Self {
            links: LinkStates::new(),
            animations: HashSet::with_capacity(size),
            nodes: NodeStates::new(size),
            backlog: BacklogUpdates::new(),
            indexer: ScreenIndex::new(screen_mouse_b),
            options: Options::new(),
        };
    }

    pub fn bulk_load(&mut self, bl: BulkLoad) {
        self.options.lc = bl.lc_ops;
        self.node_opt_bulk(bl.node_ops);
        self.link_opt_bulk(bl.link_opts);
        self.bundle_opt_bulk(bl.bundle_ops);
        self.links.bulk = true;
        self.nodes.reserve(bl.nodes.len());
        let merge = bl.merge;

        for node in bl.nodes {
            let id = node.id;
            self.node_add(node, merge);
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

        for b in bl.boxes {
            self.box_add(b, merge);
        }
        self.finish_bulk_load();
    }

    pub fn start_bulk_load(&mut self) {
        self.links.bulk = true;
    }

    pub fn finish_bulk_load(&mut self) {
        self.links.bulk = false;
        self.links.bulk_update(
            self.backlog.links.iter(),
            &self.nodes,
            &mut self.options,
            &mut self.animations,
            &mut self.indexer,
        );

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
    pub fn box_add(&mut self, node: Node, merge: bool) -> Option<Node> {
        let new = Some(node.index_bound(self.indexer.step));
        let id = node.id;
        let res = self.nodes.insert_box(node, merge);
        let old;
        match &res {
            Some(o) => old = Some(o.index_bound(self.indexer.step)),
            None => old = None,
        };
        self.indexer.index(ScreenSlot::Box(id), (old, new));

        res
    }

    pub fn box_remove(&mut self, id: u32) -> Option<Node> {
        let res = self.nodes.remove_box(id);
        if let Some(node) = &res {
            self.indexer.index(
                ScreenSlot::Box(id),
                (Some(node.index_bound(self.indexer.step)), None),
            );
        }
        res
    }
    pub fn node_add(&mut self, node: Node, merge: bool) -> Option<Node> {
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
        let res = self.nodes.insert(node, merge);
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
}
