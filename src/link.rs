use std::{mem, ops::RangeInclusive};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, GetCenter, Point, PointBox,
    calc::{NodeStates, Options},
    constants::{
        DEFAULT_ANIMATION, DEFAULT_ANIMATION_DASHES, DEFAULT_ANIMATION_WIDTH_SCALE,
        DEFAULT_BUNDLE_COLOR, DEFAULT_COLOR, DEFAULT_LINK_SCALE, DEFAULT_OPT_NAME,
    },
    node::Node,
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
pub struct Link {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: String,
    pub animation: Animation,
    pub label: String,
}
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct LinkContainerOpt {
    pub id: String,
    pub scale: f64,
    pub animation_scale: f64,
}

impl LinkContainerOpt {
    pub fn defaults() -> Self {
        return Self {
            id: String::from("default"),
            scale: DEFAULT_LINK_SCALE,
            animation_scale: DEFAULT_ANIMATION_WIDTH_SCALE,
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct LinkOpt {
    pub id: String,
    pub color: String,
    pub animation_color: String,
    pub animation_dashes: Vec<f64>,
}

impl LinkOpt {
    pub fn defaults() -> Self {
        return Self {
            id: String::from(DEFAULT_OPT_NAME),
            color: String::from(DEFAULT_COLOR),
            animation_color: String::from(DEFAULT_ANIMATION),
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct BunldeOpt {
    pub id: String,
    pub color: String,
    pub img: String,
}

impl BunldeOpt {
    pub fn defaults() -> Self {
        return Self {
            id: String::from(DEFAULT_OPT_NAME),
            color: String::from(DEFAULT_BUNDLE_COLOR),
            img: String::from(""),
        };
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct Bundle {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub opt: String,
    pub links: Vec<u32>,
    pub label: String,
}
pub trait ContainedBy {
    fn get_container_id(&self) -> u64 {
        return create_container_id(self.src(), self.dst());
    }
    fn src(&self) -> u32;
    fn dst(&self) -> u32;
}

impl ContainedBy for Bundle {
    fn src(&self) -> u32 {
        return self.src;
    }
    fn dst(&self) -> u32 {
        return self.dst;
    }
}

impl ContainedBy for Link {
    fn src(&self) -> u32 {
        return self.src;
    }
    fn dst(&self) -> u32 {
        return self.dst;
    }
}

pub struct LinkContainer {
    pub links: Vec<Link>,
    pub bundles: Vec<Bundle>,
    pub id: u64,
    pub opt: String,
    pub mouse_index: Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
    pub screen_index: Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
    pub src_point: Option<Point>,
    pub dst_point: Option<Point>,
    pub r: Option<f64>,
    pub cl: Option<ComputedLinks>,
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

pub struct ComputedLink {
    pub src: Point,
    pub dst: Point,
}

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
    pub fn update(&mut self, nodes: &NodeStates, ops: &mut Options) {
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

        let r = self.get_min_r(src, dst);
        if let Some(a) = &self.src_point
            && let Some(b) = &self.dst_point
            && let Some(cmp_r) = self.r
        {
            let s = src.get_center();
            let d = dst.get_center();
            if *a == s && *b == d && cmp_r == r {
                // we are all ready up to date!
                return;
            }
            self.src_point = Some(s);
            self.dst_point = Some(d);
            // todo
            //let lc_opt = ops.get_lc(&self.opt);
        }

        if !self.links.is_empty() {}
    }

    pub fn compute_link_segement(
        &self,
        src: &Point,
        dst: &Point,
        r: f64,
        src_id: u32,
        link_scale: f64,
    ) -> ComputedLinks {
        let mut links = Vec::new();
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
        let mut min_x = ne.x;
        let mut max_x = ne.x;
        let mut min_y = ne.y;
        let mut max_y = ne.y;
        for p in [nw, sw, se] {
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

        ne = self.get_xy(ne.x, ne.y, r, base_angle + 180.0);
        nw = self.get_xy(nw.x, nw.y, r, base_angle);
        let (width, step, init_step) =
            self.compute_line_width(link_scale, r * 2.0, self.links.len());
        let mut animations = Vec::new();
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

        return ComputedLinks {
            width,
            links,
            min_x,
            max_x,
            min_y,
            max_y,
            animations,
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
        if nodes == 1 {
            offset = 0;
        } else {
            offset = 1
        }
        return 2 * nodes - offset;
    }

    pub fn compute_bunlde_points(&self, src: &Point, dst: &Point, bundles: usize) -> Vec<Point> {
        let center = src.compute_center(dst);
        if bundles < 4 {
            // quick and dirty optimization for up to 3 bundles..
            match bundles {
                // just dead center
                1 => return Vec::from([center]),
                // left of start, right of end
                2 => return Vec::from([src.compute_center(&center), dst.compute_center(&center)]),
                // left of start, cetner, right of end.
                3 => {
                    return Vec::from([
                        src.compute_center(&center),
                        center,
                        dst.compute_center(&center),
                    ]);
                }
                _ => (),
            }
        }
        // From here on out it is simply cheaper to compute the distance and plot each point.
        let mut sets = Vec::new();
        let distance = self.get_distance(src.x, src.y, dst.x, dst.y);
        let bc = self.compute_node_scale(bundles);
        let width = distance / ((bc as f64) + 2.0);

        let angle = self.get_angle(src.x, src.y, dst.x, dst.y) + 180.0;
        for i in 0..bundles {
            let r = width + ((i as f64) * 2.0 * width);
            sets.push(self.get_xy(src.x, src.y, r, angle));
        }

        return sets;
    }
    pub fn mouse_index(
        &mut self,
        idx: i32,
    ) -> (
        Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
        Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
    ) {
        return (None, None);
    }

    pub fn screen_index(
        &mut self,
        idx: i32,
    ) -> (
        Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
        Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
    ) {
        return (None, None);
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
            opt: String::from(DEFAULT_OPT_NAME),
            src_point: None,
            dst_point: None,
            r: None,
            cl: None,
        };
    }

    pub fn clear_points(&mut self) {
        self.src_point = None;
        self.dst_point = None;
        self.r = None;
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
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
