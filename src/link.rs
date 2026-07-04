use std::{collections::HashMap, mem};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, FullBox, GetCenter, Point, PointBox,
    bsp::{IndexPart, IndexSet, ScreenIndex, ScreenSlot},
    calc::{BacklogUpdates, Options},
    constants::{
        DEFAULT_ANIMATION_COLOR, DEFAULT_ANIMATION_DASHES, DEFAULT_ANIMATION_WIDTH_SCALE,
        DEFAULT_BUNDLE_COLOR, DEFAULT_COLOR, DEFAULT_LINK_SCALE, DEFAULT_OPT_NAME,
    },
    node::{Node, NodeStates},
};

pub struct LinkStates {
    pub bulk: bool,
    pub links: HashMap<u64, LinkContainer>,
    pub updates: HashMap<u64, LinkContainer>,
    pub node_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Node instances to LinkContainer instances
    pub bundle_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Bundle instances to LinkContainer instances
    pub link_links: HashMap<u32, HashMap<u64, ()>>, // mapping of Link instances to LinkContainer instances
}

impl LinkStates {
    pub fn link_contains_point(&self, id: u64, p: &Point) -> Option<LinkContainsType> {
        if let Some(l) = self.get(&id) {
            return l.contains_point(p);
        }
        None
    }
    pub fn new() -> Self {
        Self {
            bulk: false,
            links: HashMap::new(),
            updates: HashMap::new(),
            node_links: HashMap::new(),
            bundle_links: HashMap::new(),
            link_links: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, size: usize) {
        self.links.reserve(size);
        self.node_links.reserve(size);
        self.bundle_links.reserve(size);
        self.link_links.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.links.shrink_to_fit();
        self.updates.shrink_to_fit();
        self.node_links.shrink_to_fit();
        self.bundle_links.shrink_to_fit();
        self.link_links.shrink_to_fit();
    }

    pub fn set_bulk(&mut self, bulk: bool) {
        self.bulk = bulk;
    }

    pub fn bulk_update<'l>(
        &mut self,
        updates: impl Iterator<Item = &'l u64>,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        idx: &mut ScreenIndex,
    ) {
        for lid in updates {
            let mut remove = false;
            match self.get_mut(lid) {
                Some(lc) => {
                    if lc.is_empty() {
                        remove = true;
                        idx.index(ScreenSlot::Link(*lid), lc.screen_index(idx.step, false));
                    } else {
                        lc.update(nodes, ops, animations);
                        idx.index(ScreenSlot::Link(*lid), lc.screen_index(idx.step, true));
                    }
                }
                _ => (),
            }
            if remove {
                self.manage(*lid, false, &mut None);
            }
        }
    }
    pub fn update_links(
        &mut self,
        node_id: u32,
        idx: &mut ScreenIndex,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        backlog: &mut BacklogUpdates,
    ) {
        match self.node_links.get(&node_id) {
            Some(nl) => {
                for i in nl.keys() {
                    let lc;
                    if let Some(l) = self.updates.get_mut(i) {
                        lc = l;
                    } else if let Some(l) = self.links.get_mut(i) {
                        lc = l;
                    } else {
                        continue;
                    }
                    if self.bulk {
                        backlog.links.insert(*i, ());
                    } else {
                        lc.update(nodes, ops, animations);
                        idx.index(ScreenSlot::Link(*i), lc.screen_index(idx.step, true));
                    }
                }
            }
            _ => return,
        }
    }

    fn manage_nl(&mut self, id: u64, add: bool) {
        let (src, dst) = unsafe { mem::transmute::<u64, (u32, u32)>(id) };
        for n in [src, dst] {
            let mut empty = true;
            match self.node_links.get_mut(&n) {
                Some(nl) => {
                    if add {
                        empty = false;
                        nl.insert(id, ());
                    } else {
                        nl.remove(&id);
                        empty = nl.is_empty()
                    }
                }
                None => {}
            };
            if !add && empty {
                self.node_links.remove(&n);
            }
        }
    }
    fn manage<'l>(
        &'l mut self,
        id: u64,
        add: bool,
        old: &mut Option<LinkContainer>,
    ) -> Option<&'l mut LinkContainer> {
        let exists;
        let empty;
        match self.links.get(&id) {
            Some(lc) => {
                exists = true;
                empty = lc.is_empty()
            }
            None => {
                exists = false;
                empty = true
            }
        }
        match add {
            true => {
                match exists {
                    true => {
                        if let Some(l) = self.updates.remove(&id) {
                            self.links.insert(id, l);
                        };
                    }
                    false => {
                        self.links.insert(id, LinkContainer::new_id(id));
                    }
                };

                return self.links.get_mut(&id);
            }
            false => {
                match exists {
                    true => match empty {
                        true => {
                            if !self.bulk {
                                if let Some(res) = self.updates.remove(&id) {
                                    *old = Some(res);
                                }
                                if old.is_none()
                                    && let Some(res) = self.links.remove(&id)
                                {
                                    *old = Some(res);
                                }
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                };
                return None;
            }
        };
    }
    pub fn get<'l>(&'l self, id: &u64) -> Option<&'l LinkContainer> {
        match self.updates.get(id) {
            Some(l) => return Some(l),
            _ => (),
        };
        return self.links.get(id);
    }

    pub fn get_mut<'l>(&'l mut self, id: &u64) -> Option<&'l mut LinkContainer> {
        match self.updates.get_mut(id) {
            Some(l) => return Some(l),
            _ => (),
        };
        return self.links.get_mut(id);
    }

    pub fn get_mut_for_change<'l>(&'l mut self, id: &u64) -> Option<&'l mut LinkContainer> {
        return self.manage(*id, true, &mut None);
    }

    pub fn manage_cross_link(&mut self, add: bool, id: u32, src: u32, dst: u32, link: bool) {
        let cross;
        if link {
            cross = &mut self.link_links;
        } else {
            cross = &mut self.bundle_links;
        }
        let empty;
        let lid = create_container_id(src, dst);
        match add {
            true => match cross.get_mut(&id) {
                Some(l) => {
                    l.insert(lid, ());
                    empty = false;
                }
                None => {
                    cross.insert(id, HashMap::from([(lid, ())]));
                    empty = false;
                }
            },
            false => match cross.get_mut(&id) {
                Some(l) => {
                    l.remove(&lid);
                    empty = l.is_empty();
                }
                None => {
                    empty = true;
                }
            },
        }

        if !add && empty {
            cross.remove(&id);
        }
    }
    pub fn link_add<'l>(
        &'l mut self,
        link: Link,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) -> &'l mut LinkContainer {
        self.manage_cross_link(true, link.id, link.src, link.dst, true);
        let bulk = self.bulk;
        let id = link.get_container_id();
        self.manage_nl(id, true);
        let lc = self
            .manage(link.get_container_id(), true, &mut None)
            .unwrap();
        lc.link_add(link);
        if bulk {
            backlog.links.insert(id, ());
        } else {
            lc.update(nodes, ops, animations);
            idx.index(ScreenSlot::Link(id), lc.screen_index(idx.step, true));
        }
        return lc;
    }
    pub fn link_remove(
        &mut self,
        id: u32,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) {
        let bulk = self.bulk;
        let links;
        if let Some(l) = self.link_links.remove(&id) {
            links = l;
        } else {
            return;
        }
        for lid in links.keys() {
            let src;
            let dst;

            {
                let link = self.links.get_mut(&lid).unwrap();
                link.link_remove(id);

                (src, dst) = link.get_node_ids();
            }
            self.manage_nl(*lid, false);
            let mut rm = None;
            self.manage(*lid, false, &mut rm);
            if bulk {
                backlog.links.insert(*lid, ());
            } else {
                if let Some(lc) = rm {
                    idx.index(ScreenSlot::Link(lc.id), (lc.screen_index.clone(), None));
                } else if let Some(lc) = self.links.get_mut(lid) {
                    lc.update(nodes, ops, animations);
                    idx.index(ScreenSlot::Link(lc.id), lc.screen_index(idx.step, true));
                }
            }
            self.manage_cross_link(false, id, src, dst, true);
        }
    }

    pub fn drop_link(&mut self, id: u64, idx: &mut ScreenIndex) {
        let mut link;
        if let Some(l) = self.updates.remove(&id) {
            link = l;
            self.links.remove(&id);
        } else if let Some(l) = self.links.remove(&id) {
            link = l;
        } else {
            return;
        }
        idx.index(ScreenSlot::Link(id), link.screen_index(idx.step, false));
        // need to clean up all relations as well.

        for b in link.bundles.iter() {
            let l = self.bundle_links.get_mut(&b.id).unwrap();
            l.remove(&id);
            if l.is_empty() {
                self.bundle_links.remove(&b.id);
            }
        }
        for l in link.links.iter() {
            let ls = self.link_links.get_mut(&l.id).unwrap();
            ls.remove(&id);
            if ls.is_empty() {
                self.link_links.remove(&l.id);
            }
        }
        let (src, dst) = link.get_node_ids();
        for node_id in [src, dst] {
            let nl = self.node_links.get_mut(&node_id).unwrap();
            nl.remove(&id);
            if nl.is_empty() {
                self.node_links.remove(&node_id);
            }
        }
    }

    pub fn bundle_add<'l>(
        &'l mut self,
        bunlde: Bundle,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) -> &'l mut LinkContainer {
        let bulk = self.bulk;
        self.manage_cross_link(true, bunlde.id, bunlde.src, bunlde.dst, false);
        let lc = self
            .manage(bunlde.get_container_id(), true, &mut None)
            .unwrap();
        lc.bundle_add(bunlde);
        if bulk {
            backlog.links.insert(lc.id, ());
        } else {
            lc.update(nodes, ops, animations);
            idx.index(ScreenSlot::Link(lc.id), lc.screen_index(idx.step, true));
        }
        return lc;
    }
    pub fn bundle_remove(
        &mut self,
        id: u32,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) {
        let bulk = self.bulk;
        let bundles;
        if let Some(l) = self.bundle_links.remove(&id) {
            bundles = l;
        } else {
            return;
        }
        for lid in bundles.keys() {
            let src;
            let dst;

            {
                let link = self.links.get_mut(&lid).unwrap();
                link.bundle_remove(id);

                (src, dst) = link.get_node_ids();
            }
            let mut rm = None;
            self.manage(*lid, false, &mut rm);
            if bulk {
                backlog.links.insert(*lid, ());
            } else {
                if let Some(lc) = rm {
                    idx.index(ScreenSlot::Link(*lid), (lc.screen_index.clone(), None));
                } else if let Some(lc) = self.get_mut(lid) {
                    lc.update(nodes, ops, animations);
                    idx.index(ScreenSlot::Link(lc.id), lc.screen_index(idx.step, true));
                }
            }
            self.manage_cross_link(false, id, src, dst, false);
        }
        return;
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Animation {
    Both,  // Animate in both directions
    ToSrc, // Animate towards the src node
    ToDst, // Animate towards the dst node
    None,  // Do not animate
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct Link {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub animation: Animation,
    pub label: String,
}

#[wasm_bindgen]
impl Link {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, src: u32, dst: u32, opt: u32, animation: Animation, label: String) -> Self {
        Self {
            id,
            src,
            dst,
            opt,
            animation,
            label,
        }
    }

    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct LinkContainerOpt {
    pub id: u32,
    pub label: String,
    pub scale: f64,
    pub animation_scale: f64,
}

impl LinkContainerOpt {
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            label: String::from(DEFAULT_OPT_NAME),
            scale: DEFAULT_LINK_SCALE,
            animation_scale: DEFAULT_ANIMATION_WIDTH_SCALE,
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct LinkOpt {
    pub id: u32,
    pub label: String,
    pub color: String,
    pub animation_color: String,
    pub animation_dashes: Vec<f64>,
}

impl LinkOpt {
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            label: String::from(DEFAULT_OPT_NAME),
            color: String::from(DEFAULT_COLOR),
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct BunldeOpt {
    pub id: u32,
    pub label: String,
    pub color: String,
    pub img: String,
}

impl BunldeOpt {
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            label: String::from(DEFAULT_OPT_NAME),
            color: String::from(DEFAULT_BUNDLE_COLOR),
            img: String::from(""),
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub links: Vec<u32>,
    pub label: String,
}

#[wasm_bindgen]
impl Bundle {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, src: u32, dst: u32, opt: u32, links: Vec<u32>, label: String) -> Self {
        Self {
            id,
            src,
            dst,
            label,
            links,
            opt,
        }
    }
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

impl CalculatorTrait for Bundle {}

pub trait SrcDstIs {
    fn get_container_id(&self) -> u64 {
        return create_container_id(self.src(), self.dst());
    }
    fn src(&self) -> u32;
    fn dst(&self) -> u32;
}

impl SrcDstIs for Bundle {
    fn src(&self) -> u32 {
        return self.src;
    }
    fn dst(&self) -> u32 {
        return self.dst;
    }
}

impl SrcDstIs for Link {
    fn src(&self) -> u32 {
        return self.src;
    }
    fn dst(&self) -> u32 {
        return self.dst;
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub id: u64,
    pub opt: u32,
    screen_index: IndexPart,
    link_src: Option<LinkSource>,
}

#[derive(Clone)]
pub struct LinkSource {
    pub src_point: Point,
    pub dst_point: Point,
    pub r: f64,
    pub cl: ComputedLinks,
}

impl CalculatorTrait for LinkContainer {}

#[derive(Clone)]
pub struct ComputedLinks {
    pub width: f64,
    pub links: Vec<ComputedLink>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub animations: Vec<AnimatedLink>,
    pub bundles: Vec<Point>,
}
impl FullBox for ComputedLinks {
    fn full_box(&self) -> (Point, Point, Point, Point) {
        let min_x = self.min_x;
        let max_x = self.max_x;
        let min_y = self.min_y;
        let max_y = self.max_y;
        return (
            Point { x: min_x, y: min_y }, // nw
            Point { x: max_x, y: min_y }, // ne
            Point { x: min_x, y: max_y }, // sw
            Point { x: max_x, y: max_y }, // se
        );
    }
}
impl PointBox for ComputedLinks {
    fn get_min_x(&self) -> f64 {
        return self.min_x;
    }

    fn get_max_x(&self) -> f64 {
        return self.max_x;
    }

    fn get_max_y(&self) -> f64 {
        return self.max_y;
    }

    fn get_min_y(&self) -> f64 {
        return self.min_y;
    }
}

impl GetCenter for ComputedLinks {
    fn get_center(&self) -> Point {
        let x = (self.max_x + self.min_x) * 0.5;
        let y = (self.max_y + self.min_y) * 0.5;
        return Point { x, y };
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub struct ComputedLink {
    pub src: Point,
    pub dst: Point,
}

#[derive(Clone, Copy, Debug)]
pub struct LinkBox {
    ne: Point,
    nw: Point,
    se: Point,
    sw: Point,
}

impl FullBox for LinkBox {
    fn full_box(&self) -> (Point, Point, Point, Point) {
        (self.ne, self.nw, self.se, self.sw)
    }
}

impl CalculatorTrait for LinkBox {}

impl LinkBox {
    pub fn new(src: &Point, dst: &Point, width: f64) -> Self {
        let r = width * 0.5;
        let base_angle = src.get_angle(src.x, src.y, dst.x, dst.y);
        let angle_north = base_angle + 90.0;
        let angle_south = angle_north + 180.0;

        // true outer points
        let ne = src.get_xy(src.x, src.y, r, angle_north);
        let nw = src.get_xy(dst.x, dst.y, r, angle_north);
        let se = src.get_xy(src.x, src.y, r, angle_south);
        let sw = src.get_xy(dst.x, dst.y, r, angle_south);
        return Self { ne, nw, se, sw };
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AnimatedLink {
    pub src: Point,
    pub dst: Point,
    pub width: f64,
}
pub(crate) fn create_container_id(src: u32, dst: u32) -> u64 {
    let id: u64;
    if src < dst {
        id = unsafe { mem::transmute::<(u32, u32), u64>((src, dst)) }
    } else {
        id = unsafe { mem::transmute::<(u32, u32), u64>((dst, src)) }
    }
    return id;
}

#[wasm_bindgen]
#[derive(Clone, Debug, PartialEq)]
pub enum LinkContainsType {
    Bundle(Bundle),
    Link(Link),
}

impl LinkContainer {
    pub fn contains_point(&self, p: &Point) -> Option<LinkContainsType> {
        let cu;
        if let Some(c) = &self.link_src {
            cu = c;
        } else {
            return None;
        }
        // first check the bundles
        let cl = &cu.cl;
        for i in 0..cl.bundles.len() {
            let bp = &cl.bundles[i];
            if self.inside_circle(p, bp, cu.r) {
                return Some(LinkContainsType::Bundle(self.bundles[i].clone()));
            }
        }
        for i in 0..cl.links.len() {
            let pb = &cl.links[i];
            let lb = LinkBox::new(&pb.src, &pb.dst, cl.width);
            if self.inside_box(&lb, p) {
                return Some(LinkContainsType::Link(self.links[i].clone()));
            }
        }

        return None;
    }
    pub fn get_min_r(&self, s: &Node, d: &Node) -> f64 {
        let a = s.get_min_r();
        let b = d.get_min_r();
        if a < b {
            return a;
        }
        return b;
    }
    pub fn update(
        &mut self,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashMap<u64, ()>,
    ) {
        let (src_id, dst_id) = self.get_node_ids();
        let src;
        let dst;
        if let Some(s) = nodes.get(src_id)
            && let Some(d) = nodes.get(dst_id)
        {
            src = s;
            dst = d;
        } else {
            return;
        }

        let s = src.get_center();
        let d = dst.get_center();
        let r = self.get_min_r(src, dst);
        if let Some(state) = &self.link_src {
            if state.src_point == s && state.dst_point == d && state.r == r {
                // we are all ready up to date!
                return;
            }
        }
        animations.remove(&self.id);

        let lc_opt = ops.get_lc(&self.opt);
        if !self.is_empty() {
            // this code is temporary.. will need to upgrade it in order to support arches and elbows.
            let mut cu = self.compute_link_segement_line(&s, &d, r, src_id, &lc_opt);
            if !cu.animations.is_empty() {
                animations.insert(self.id, ());
            }
            self.compute_bunlde_points(&s, &d, self.bundles.len(), &mut cu.bundles);
            let src = LinkSource {
                src_point: s,
                dst_point: d,
                r: r,
                cl: cu,
            };
            self.link_src = Some(src);
        }
    }

    pub fn compute_link_segement_line(
        &self,
        src: &Point,
        dst: &Point,
        r: f64,
        src_id: u32,
        link_opt: &LinkContainerOpt,
    ) -> ComputedLinks {
        let mut links = Vec::with_capacity(self.links.len());
        // Apply the shifting offset to create movement
        // ctx.lineDashOffset = offset;

        let base_angle = self.get_angle(src.x, src.y, dst.x, dst.y);
        let angle_north = base_angle + 90.0;
        let angle_south = angle_north + 180.0;

        // true outer points
        let mut ne = self.get_xy(src.x, src.y, r, angle_north);
        let mut nw = self.get_xy(dst.x, dst.y, r, angle_north);
        let se = self.get_xy(src.x, src.y, r, angle_south);
        let sw = self.get_xy(dst.x, dst.y, r, angle_south);

        let (min_x, max_x, min_y, max_y) = self.compute_line_box(&ne, [&nw, &sw, &se]);

        ne = self.get_xy(ne.x, ne.y, r, base_angle + 180.0);
        nw = self.get_xy(nw.x, nw.y, r, base_angle);

        if self.links.is_empty() {
            return ComputedLinks {
                width: r * 2.0,
                links,
                min_x,
                max_x,
                min_y,
                max_y,
                animations: Vec::new(),
                bundles: Vec::new(),
            };
        }

        let (width, step, init_step) =
            self.compute_line_width(link_opt.scale, r * 2.0, self.links.len());

        // assume wost case.
        let mut animations = Vec::with_capacity(self.links.len());
        for (i, link) in self.links.iter().enumerate() {
            let inc_by = init_step + step * (i as f64);
            let start = self.get_xy(ne.x, ne.y, inc_by, angle_south);
            let end = self.get_xy(nw.x, nw.y, inc_by, angle_south);
            let clink;
            if src_id == link.src {
                clink = ComputedLink {
                    src: start,
                    dst: end,
                }
            } else {
                clink = ComputedLink {
                    src: end,
                    dst: start,
                }
            }
            match link.animation {
                Animation::None => (),
                Animation::Both => {
                    let (aw, _, init_step) = self.compute_line_width(1.0, width, 2);

                    animations.push(AnimatedLink {
                        src: self.get_xy(clink.src.x, clink.src.y, init_step, angle_north),
                        dst: self.get_xy(clink.dst.x, clink.dst.y, init_step, angle_north),
                        width: aw,
                    });
                    animations.push(AnimatedLink {
                        src: self.get_xy(clink.dst.x, clink.dst.y, init_step, angle_south),
                        dst: self.get_xy(clink.src.x, clink.src.y, init_step, angle_south),
                        width: aw,
                    });
                }
                Animation::ToSrc => {
                    let (aw, _, _) = self.compute_line_width(1.0, width, 1);
                    animations.push(AnimatedLink {
                        src: clink.dst,
                        dst: clink.src,
                        width: aw,
                    });
                }
                Animation::ToDst => {
                    let (aw, _, _) = self.compute_line_width(1.0, width, 1);
                    animations.push(AnimatedLink {
                        src: clink.src,
                        dst: clink.dst,
                        width: aw,
                    });
                }
            }
            links.push(clink);
        }

        // free any unused memory
        animations.shrink_to_fit();
        return ComputedLinks {
            width,
            links,
            min_x,
            max_x,
            min_y,
            max_y,
            animations,
            bundles: Vec::new(),
        };
    }
    pub fn compute_line_width(&self, link_scale: f64, r: f64, nodes: usize) -> (f64, f64, f64) {
        let lc = self.compute_node_scale(nodes) as f64;
        let scaled = r * link_scale;
        let width = scaled / lc;
        let step = scaled / (nodes as f64);
        return (width, step, step * 0.5);
    }

    pub fn compute_node_scale(&self, nodes: usize) -> usize {
        let offset;
        match nodes {
            0 => return 0,
            1 => offset = 0,
            _ => offset = 1,
        }
        return 2 * nodes - offset;
    }

    pub fn compute_bunlde_points(
        &self,
        src: &Point,
        dst: &Point,
        bundles: usize,
        points: &mut Vec<Point>,
    ) {
        let center = src.compute_center(dst);
        points.reserve(bundles);
        if bundles < 4 {
            // quick and dirty optimization for up to 3 bundles..
            match bundles {
                // just dead center
                1 => {
                    points.extend_from_slice(&[center]);
                    return;
                }
                // left of start, right of end
                2 => {
                    points.extend_from_slice(&[
                        src.compute_center(&center),
                        dst.compute_center(&center),
                    ]);
                    return;
                }
                // left of start, cetner, right of end.
                3 => {
                    points.extend_from_slice(&[
                        src.compute_center(&center),
                        center,
                        dst.compute_center(&center),
                    ]);
                    return;
                }
                _ => (),
            }
        }
        // From here on out it is simply cheaper to compute the distance and plot each point.
        let distance = self.get_distance(src.x, src.y, dst.x, dst.y);
        let bc = self.compute_node_scale(bundles);
        let width = distance / ((bc as f64) + 2.0);

        let angle = self.get_angle(src.x, src.y, dst.x, dst.y) + 180.0;
        for i in 0..bundles {
            let r = width + ((i as f64) * 2.0 * width);
            points.push(self.get_xy(src.x, src.y, r, angle));
        }
    }

    pub fn build_index_bounds(&self, boundry: i64, needs_new: bool) -> IndexPart {
        match needs_new {
            true => match &self.link_src {
                Some(cu) => return Some(cu.cl.index_bound(boundry)),
                _ => (),
            },
            _ => (),
        }
        None
    }

    pub fn screen_index(&mut self, boundry: i64, needs_new: bool) -> IndexSet {
        let new = self.build_index_bounds(boundry, needs_new);
        let old = mem::replace(&mut self.screen_index, new.clone());
        return (old, new);
    }
    pub fn is_empty(&self) -> bool {
        return self.links.is_empty() && self.bundles.is_empty();
    }

    pub fn new(src: u32, dst: u32) -> Self {
        return Self::new_id(create_container_id(src, dst));
    }
    pub fn new_id(id: u64) -> Self {
        return Self {
            id,
            links: Vec::new(),
            bundles: Vec::new(),
            screen_index: None,
            opt: 0,
            link_src: None,
        };
    }

    pub fn clear_points(&mut self) {
        self.link_src = None;
    }

    pub fn link_add(&mut self, link: Link) -> Option<Link> {
        if link.src == link.dst {
            panic!("Link.src and Link.dst cannot be the same!");
        }
        self.clear_points();
        for (id, l) in self.links.iter().enumerate() {
            if l.id == link.id {
                return Some(mem::replace(&mut self.links[id], link));
            }
        }
        self.links.push(link);
        return None;
    }

    pub fn bundle_add(&mut self, bundle: Bundle) -> Option<Bundle> {
        if bundle.src == bundle.dst {
            panic!("Bundle.src and Bundle.dst cannot be the same!");
        }
        self.clear_points();
        for (id, b) in self.bundles.iter().enumerate() {
            if b.id == b.id {
                return Some(mem::replace(&mut self.bundles[id], bundle));
            }
        }
        self.bundles.push(bundle);
        return None;
    }

    pub fn link_remove(&mut self, id: u32) -> Option<Link> {
        self.clear_points();

        for (i, l) in self.links.iter().enumerate() {
            if l.id == id {
                let res = self.links.remove(i);
                return Some(res);
            }
        }
        return None;
    }

    pub fn bundle_remove(&mut self, id: u32) -> Option<Bundle> {
        self.clear_points();
        for (i, l) in self.bundles.iter().enumerate() {
            if l.id == id {
                let res = self.bundles.remove(i);
                return Some(res);
            }
        }
        return None;
    }

    pub fn get_node_ids(&self) -> (u32, u32) {
        return unsafe { mem::transmute::<u64, (u32, u32)>(self.id) };
    }
}
