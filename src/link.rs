use std::{collections::HashMap, mem};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, GetCenter, Point, PointBox,
    bsp::{IndexPart, IndexSet},
    calc::Options,
    constants::{
        DEFAULT_ANIMATION, DEFAULT_ANIMATION_DASHES, DEFAULT_ANIMATION_WIDTH_SCALE,
        DEFAULT_BUNDLE_COLOR, DEFAULT_COLOR, DEFAULT_LINK_SCALE, DEFAULT_OPT_NAME,
    },
    node::{Node, NodeStates},
};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Animation {
    Both,  // Animate in both directions
    ToSrc, // Animate towards the src node
    ToDst, // Animate towards the dst node
    None,  // Do not animate
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Link {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub animation: Animation,
    pub label: String,
}
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
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
#[derive(Clone)]
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
            animation_color: String::from(DEFAULT_ANIMATION),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
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
#[derive(Clone)]
pub struct Bundle {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: u32,
    pub links: Vec<u32>,
    pub label: String,
}

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
    mouse_index: IndexPart,
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

#[wasm_bindgen]
#[derive(Copy, Clone)]
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

impl LinkContainer {
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
            let mut cu = self.compute_link_segement(&s, &d, r, src_id, &lc_opt);
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

    pub fn compute_line_box(&self, ne: &Point, points: [&Point; 3]) -> (f64, f64, f64, f64) {
        let mut min_x = ne.x;
        let mut max_x = ne.x;
        let mut min_y = ne.y;
        let mut max_y = ne.y;
        for p in points {
            if max_x < p.x {
                max_x = p.x;
            }
            if max_y < p.y {
                max_y = p.y;
            }
            if min_x > p.x {
                min_x = p.x;
            }
            if min_y > p.y {
                min_y = p.y;
            }
        }
        return (min_x, max_x, min_y, max_y);
    }

    pub fn compute_link_segement(
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
    pub fn mouse_index(&mut self, boundry: i64, needs_new: bool) -> IndexSet {
        let new = self.build_index_bounds(boundry, needs_new);
        let old = mem::replace(&mut self.mouse_index, new.clone());
        return (old, new);
    }
    pub fn build_index_bounds(&self, boundry: i64, needs_new: bool) -> IndexPart {
        match needs_new {
            true => match &self.link_src {
                None => return None,
                Some(cu) => {
                    return Some(cu.cl.index_bound(boundry));
                }
            },
            false => return None,
        }
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
            mouse_index: None,
            screen_index: None,
            opt: 0,
            link_src: None,
        };
    }

    pub fn clear_points(&mut self) {
        self.link_src = None;
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
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

    pub fn add_bundle(&mut self, bundle: Bundle) -> Option<Bundle> {
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

    pub fn remove_link(&mut self, id: u32) -> Option<Link> {
        self.clear_points();

        for (i, l) in self.links.iter().enumerate() {
            if l.id == id {
                let res = self.links.remove(i);
                return Some(res);
            }
        }
        return None;
    }

    pub fn remove_bundle(&mut self, id: u32) -> Option<Bundle> {
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
