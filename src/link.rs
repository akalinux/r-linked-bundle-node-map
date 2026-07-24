use std::{
    collections::{HashMap, HashSet},
    mem, process,
};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, FullBox, GetCenter, ImgSrc, Point, PointBox, RenderBox,
    bsp::{IndexPart, IndexSet, ScreenIndex, ScreenSlot},
    calc::{BacklogUpdates, Options},
    constants::{
        DEFAULT_ANIMATION_COLOR, DEFAULT_ANIMATION_WIDTH_SCALE, DEFAULT_BUNDLE_COLOR,
        DEFAULT_COLOR, DEFAULT_LINK_SCALE,
    },
    id_compare,
    node::{Node, NodeStates},
};
pub struct LinkStates {
    pub bulk: bool,
    pub links: HashMap<u64, LinkContainer>,
    //pub updates: HashMap<u64, LinkContainer>,
    pub node_links: HashMap<u32, HashSet<u64>>, // mapping of Node instances to LinkContainer instances
    pub bundle_links: HashMap<u32, HashSet<u64>>, // mapping of Bundle instances to LinkContainer instances
    pub link_links: HashMap<u32, HashSet<u64>>, // mapping of Link instances to LinkContainer instances
}

impl LinkStates {
    pub fn link_contains_point<'r>(&'r self, id: u64, p: &Point) -> LinkContainsType<'r> {
        if let Some(l) = self.get(&id) {
            return l.contains_point(p);
        }
        LinkContainsType::NoMatch
    }
    pub fn new() -> Self {
        Self {
            bulk: false,
            links: HashMap::new(),
            //updates: HashMap::new(),
            node_links: HashMap::new(),
            bundle_links: HashMap::new(),
            link_links: HashMap::new(),
        }
    }
    pub fn reserve(&mut self, size: usize) {
        self.links.reserve(size);
        self.node_links.reserve(size * 2);
        self.bundle_links.reserve(size);
        self.link_links.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.links.shrink_to_fit();
        //self.updates.shrink_to_fit();
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
        animations: &mut HashSet<u64>,
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
        animations: &mut HashSet<u64>,
        backlog: &mut BacklogUpdates,
    ) {
        if let Some(nl) = self.node_links.get(&node_id) {
            for i in nl.iter() {
                match self.links.get_mut(i) {
                    Some(lc) => match self.bulk {
                        true => {
                            backlog.links.insert(*i);
                        }
                        false => {
                            lc.update(nodes, ops, animations);
                            idx.index(ScreenSlot::Link(*i), lc.screen_index(idx.step, true));
                        }
                    },
                    _ => (),
                }
            }
        }
    }

    fn manage_nl(&mut self, id: u64, add: bool) {
        let (src, dst) = unsafe { mem::transmute::<u64, (u32, u32)>(id) };
        for n in [src, dst] {
            let mut empty = true;
            if let Some(nl) = self.node_links.get_mut(&n) {
                if add {
                    empty = false;
                    nl.insert(id);
                } else {
                    nl.remove(&id);
                    empty = nl.is_empty()
                }
            }
            match !add && empty {
                true => self.node_links.remove(&n),
                false => self.node_links.insert(n, HashSet::from([id])),
            };
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
                    true => (),
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
        self.links.get(id)
    }

    pub fn get_mut<'l>(&'l mut self, id: &u64) -> Option<&'l mut LinkContainer> {
        self.links.get_mut(id)
    }

    pub fn get_mut_for_change<'l>(&'l mut self, id: &u64) -> Option<&'l mut LinkContainer> {
        self.manage(*id, true, &mut None)
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
                    l.insert(lid);
                    empty = false;
                }
                None => {
                    cross.insert(id, HashSet::from([lid]));
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
        animations: &mut HashSet<u64>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) -> &'l mut LinkContainer {
        self.manage_cross_link(true, link.id, link.src, link.dst, true);
        let bulk = self.bulk;
        let id = link.link_id();
        self.manage_nl(id, true);
        let lc;
        match self.manage(link.link_id(), true, &mut None) {
            Some(v) => lc = v,
            None => process::abort(),
        };
        lc.link_add(link);
        if bulk {
            backlog.links.insert(id);
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
        animations: &mut HashSet<u64>,
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
        for lid in links.iter() {
            let src;
            let dst;

            match self.links.get_mut(&lid) {
                Some(link) => {
                    link.link_remove(id);
                    (src, dst) = link.get_node_ids();
                }
                None => process::abort(),
            }
            self.manage_nl(*lid, false);
            let mut rm = None;
            self.manage(*lid, false, &mut rm);
            if bulk {
                backlog.links.insert(*lid);
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
        /*if let Some(l) = self.updates.remove(&id) {
            link = l;
            self.links.remove(&id);
        } else */
        if let Some(l) = self.links.remove(&id) {
            link = l;
        } else {
            return;
        }
        idx.index(ScreenSlot::Link(id), link.screen_index(idx.step, false));
        // need to clean up all relations as well.

        for b in link.bundles.iter() {
            match self.bundle_links.get_mut(&b.id) {
                Some(l) => {
                    l.remove(&id);
                    if l.is_empty() {
                        self.bundle_links.remove(&b.id);
                    }
                }
                _ => (),
            }
        }
        for l in link.links.iter() {
            match self.link_links.get_mut(&l.id) {
                Some(ls) => {
                    ls.remove(&id);
                    if ls.is_empty() {
                        self.link_links.remove(&l.id);
                    }
                }
                _ => (),
            }
        }
        let (src, dst) = link.get_node_ids();
        for node_id in [src, dst] {
            match self.node_links.get_mut(&node_id) {
                Some(nl) => {
                    nl.remove(&id);
                    if nl.is_empty() {
                        self.node_links.remove(&node_id);
                    }
                }
                _ => (),
            }
        }
    }

    pub fn bundle_add<'l>(
        &'l mut self,
        bunlde: Bundle,
        nodes: &NodeStates,
        ops: &mut Options,
        animations: &mut HashSet<u64>,
        idx: &mut ScreenIndex,
        backlog: &mut BacklogUpdates,
    ) -> &'l mut LinkContainer {
        let bulk = self.bulk;
        self.manage_cross_link(true, bunlde.id, bunlde.src, bunlde.dst, false);
        let lc;
        match self.manage(bunlde.link_id(), true, &mut None) {
            Some(l) => lc = l,
            None => process::abort(),
        }
        lc.bundle_add(bunlde);
        if bulk {
            backlog.links.insert(lc.id);
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
        animations: &mut HashSet<u64>,
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
        for lid in bundles.iter() {
            let src;
            let dst;

            match self.links.get_mut(&lid) {
                Some(link) => {
                    link.bundle_remove(id);

                    (src, dst) = link.get_node_ids();
                }
                None => process::abort(),
            }
            let mut rm = None;
            self.manage(*lid, false, &mut rm);
            if bulk {
                backlog.links.insert(*lid);
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

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
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
}

impl Link {
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone)]
pub struct LinkContainerOpt {
    pub scale: f64,
    pub animation_scale: f64,
}

#[wasm_bindgen]
impl LinkContainerOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(scale: f64, animation_scale: f64) -> Self {
        Self {
            scale,
            animation_scale,
        }
    }
}
impl LinkContainerOpt {
    pub fn defaults() -> Self {
        return Self {
            scale: DEFAULT_LINK_SCALE,
            animation_scale: DEFAULT_ANIMATION_WIDTH_SCALE,
        };
    }
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct LinkOpt {
    pub id: u32,
    pub color: String,
    pub animation_color: String,
}

#[wasm_bindgen]
impl LinkOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, color: String, animation_color: String) -> Self {
        Self {
            id,
            color,
            animation_color,
        }
    }
}
impl LinkOpt {
    pub fn defaults() -> Self {
        Self {
            id: 0,
            color: String::from(DEFAULT_COLOR),
            animation_color: String::from(DEFAULT_ANIMATION_COLOR),
        }
    }
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct BundleOpt {
    pub id: u32,
    pub color: String,
    pub img: String,
}
impl ImgSrc for BundleOpt {
    fn img_src(&self) -> String {
        self.img.clone()
    }

    fn box_color(&self) -> String {
        self.color.clone()
    }
    fn watch_id(&self) -> crate::ImgWatchId {
        crate::ImgWatchId::Bundle(self.id)
    }
}
#[wasm_bindgen]
impl BundleOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, color: String, img: String) -> Self {
        Self { id, color, img }
    }
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            color: String::from(DEFAULT_BUNDLE_COLOR),
            img: String::from(""),
        };
    }
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
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
}
impl Bundle {
    pub fn link_id(&self) -> u64 {
        create_container_id(self.src, self.dst)
    }
}

impl CalculatorTrait for Bundle {}

pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub id: u64,
    pub screen_index: IndexPart,
    pub link_src: Option<LinkSource>,
}

pub struct LinkSource {
    pub src_point: Point,
    pub dst_point: Point,
    pub r: f64,
    pub cl: ComputedLinks,
}

impl CalculatorTrait for LinkContainer {}

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
pub struct BundleRenderBox {
    x: f64,
    y: f64,
    s: f64,
    id: u32,
}
impl BundleRenderBox {
    pub fn new(p: &Point, s: f64, id: u32) -> Self {
        Self {
            x: p.x,
            y: p.y,
            s,
            id,
        }
    }
}
impl RenderBox for BundleRenderBox {
    fn width(&self) -> f64 {
        self.s
    }

    fn height(&self) -> f64 {
        self.s
    }

    fn x(&self) -> f64 {
        self.x - self.s * 0.5
    }

    fn y(&self) -> f64 {
        self.y - self.s * 0.5
    }
    fn id(&self) -> crate::ImgWatchId {
        crate::ImgWatchId::Bundle(self.id)
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

#[derive(Copy, Clone)]
pub struct ComputedLink {
    pub src: Point,
    pub dst: Point,
    pub opt: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct AnimatedLink {
    pub src: Point,
    pub dst: Point,
    pub width: f64,
    pub opt: u32,
}
#[inline(never)]
pub(crate) fn create_container_id(src: u32, dst: u32) -> u64 {
    let id: u64;
    if src < dst {
        id = unsafe { mem::transmute::<(u32, u32), u64>((src, dst)) }
    } else {
        id = unsafe { mem::transmute::<(u32, u32), u64>((dst, src)) }
    }
    return id;
}

#[derive(Clone, Debug, PartialEq)]
pub enum LinkContainsType<'r> {
    Bundle(&'r Bundle),
    Link(&'r Link),
    NoMatch,
}
impl<'r> LinkContainsType<'r> {
    pub fn is_none(self) -> bool {
        match self {
            LinkContainsType::NoMatch => true,
            _ => false,
        }
    }
}
impl LinkContainer {
    pub fn get_bundle_box(&self, id: u32) -> Option<BundleRenderBox> {
        if let Some(cl) = &self.link_src {
            for pos in 0..self.bundles.len() {
                let b = &self.bundles[pos];
                if id == b.id {
                    let p = cl.cl.bundles[pos];
                    return Some(BundleRenderBox {
                        x: p.x,
                        y: p.y,
                        s: cl.cl.width,
                        id,
                    });
                }
            }
        }
        None
    }
    pub fn get_link_render(&self, id: u32) -> Option<(Point, Point, f64, u32)> {
        if let Some(lc) = &self.link_src {
            for pos in 0..self.links.len() {
                let link = &self.links[pos];
                if id == link.id {
                    let set = lc.cl.links[pos];
                    return Some((set.src, set.dst, lc.cl.width, set.opt));
                }
            }
        }
        None
    }

    pub fn contains_point<'r>(&'r self, p: &Point) -> LinkContainsType<'r> {
        let cu;
        if let Some(c) = &self.link_src {
            cu = c;
        } else {
            return LinkContainsType::NoMatch;
        }
        // first check the bundles
        let cl = &cu.cl;
        for i in 0..cl.bundles.len() {
            let bp = &cl.bundles[i];
            if self.inside_circle(p, bp, cu.r) {
                return LinkContainsType::Bundle(&self.bundles[i]);
            }
        }
        for i in 0..cl.links.len() {
            let pb = &cl.links[i];
            let lb = self.full_box(&pb.src, &pb.dst, cl.width);
            if self.inside_box(&lb, p) {
                return LinkContainsType::Link(&self.links[i]);
            }
        }

        return LinkContainsType::NoMatch;
    }

    pub fn full_box(&self, src: &Point, dst: &Point, width: f64) -> FullBox {
        let r = width * 0.5;
        let base_angle = src.get_angle(src.x, src.y, dst.x, dst.y);
        let angle_north = base_angle + 90.0;
        let angle_south = angle_north + 180.0;

        // true outer points
        let ne = src.get_xy(src.x, src.y, r, angle_north);
        let nw = src.get_xy(dst.x, dst.y, r, angle_north);
        let se = src.get_xy(src.x, src.y, r, angle_south);
        let sw = src.get_xy(dst.x, dst.y, r, angle_south);
        return (ne, nw, se, sw);
    }

    pub fn get_min_r(&self, s: &Node, d: &Node) -> f64 {
        let a = s.get_min_r();
        let b = d.get_min_r();
        if a < b {
            return a;
        }
        return b;
    }
    pub fn update(&mut self, nodes: &NodeStates, ops: &mut Options, animations: &mut HashSet<u64>) {
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

        let lc_opt = ops.get_lc();
        if !self.is_empty() {
            // this code is temporary.. will need to upgrade it in order to support arches and elbows.
            let mut cu = self.compute_link_segement_line(&s, &d, r, src_id, &lc_opt);
            if !cu.animations.is_empty() {
                animations.insert(self.id);
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

        let mut animations = Vec::new();
        let bundles = Vec::new();
        if self.links.is_empty() {
            return ComputedLinks {
                width: r * 2.0,
                links,
                min_x,
                max_x,
                min_y,
                max_y,
                animations,
                bundles,
            };
        }

        let (width, step, init_step) =
            self.compute_line_width(link_opt.scale, r * 2.0, self.links.len());

        for (i, link) in self.links.iter().enumerate() {
            let inc_by = init_step + step * (i as f64);
            let start = self.get_xy(ne.x, ne.y, inc_by, angle_south);
            let end = self.get_xy(nw.x, nw.y, inc_by, angle_south);
            let clink;
            if src_id == link.src {
                clink = ComputedLink {
                    src: start,
                    dst: end,
                    opt: link.opt,
                }
            } else {
                clink = ComputedLink {
                    src: end,
                    dst: start,
                    opt: link.opt,
                }
            }
            self.compute_animation(
                link,
                &clink,
                &mut animations,
                width,
                angle_north,
                angle_south,
            );

            links.push(clink);
        }

        return ComputedLinks {
            width,
            links,
            min_x,
            max_x,
            min_y,
            max_y,
            animations,
            bundles,
        };
    }

    pub fn compute_animation(
        &self,
        link: &Link,
        clink: &ComputedLink,
        animations: &mut Vec<AnimatedLink>,
        width: f64,
        angle_north: f64,
        angle_south: f64,
    ) {
        match link.animation {
            Animation::Both => {
                let (aw, _, init_step) = self.compute_line_width(1.0, width, 2);

                animations.push(AnimatedLink {
                    src: self.get_xy(clink.src.x, clink.src.y, init_step, angle_north),
                    dst: self.get_xy(clink.dst.x, clink.dst.y, init_step, angle_north),
                    width: aw,
                    opt: link.opt,
                });
                animations.push(AnimatedLink {
                    src: self.get_xy(clink.dst.x, clink.dst.y, init_step, angle_south),
                    dst: self.get_xy(clink.src.x, clink.src.y, init_step, angle_south),
                    width: aw,
                    opt: link.opt,
                });
            }
            Animation::ToSrc => {
                let (aw, _, _) = self.compute_line_width(1.0, width, 1);
                animations.push(AnimatedLink {
                    src: clink.dst,
                    dst: clink.src,
                    width: aw,
                    opt: link.opt,
                });
            }
            Animation::ToDst => {
                let (aw, _, _) = self.compute_line_width(1.0, width, 1);
                animations.push(AnimatedLink {
                    src: clink.src,
                    dst: clink.dst,
                    width: aw,
                    opt: link.opt,
                });
            }
            _ => (),
        }
    }
    pub fn compute_line_width(&self, link_scale: f64, r: f64, nodes: usize) -> (f64, f64, f64) {
        //let lc = self.compute_node_scale(nodes) as f64;
        let offset;
        match nodes {
            0 => offset = 0,
            1 => offset = 0,
            _ => offset = 1,
        }
        let lc = (2 * nodes - offset) as f64;
        let scaled = r * link_scale;
        let width = scaled / lc;
        let step = scaled / (nodes as f64);
        return (width, step, step * 0.5);
    }

    pub fn compute_bunlde_points(
        &self,
        src: &Point,
        dst: &Point,
        bundles: usize,
        points: &mut Vec<Point>,
    ) {
        points.reserve(bundles);
        let distance = self.get_distance(src.x, src.y, dst.x, dst.y);
        let scale = (bundles * 2) as f64;
        let size = distance / scale;

        let angle = self.get_angle(src.x, src.y, dst.x, dst.y) + 180.0;
        for i in (1..bundles * 2).step_by(2) {
            let r = size * i as f64;
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
            link_src: None,
        };
    }

    pub fn clear_points(&mut self) {
        self.link_src = None;
    }

    pub fn link_add(&mut self, link: Link) -> Option<Link> {
        if link.src == link.dst {
            return None;
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
            return None;
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

id_compare!(Link, Bundle, LinkOpt, BundleOpt, LinkContainer);
