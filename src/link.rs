use std::{collections::HashMap, mem, ops::RangeInclusive};

use wasm_bindgen::prelude::*;

use crate::{
    GetCenter, Point,
    calc::{NodeStates, Options},
    constants::{
        DEFAULT_ANIMATION, DEFAULT_ANIMATION_DASHES, DEFAULT_BUNDLE_COLOR, DEFAULT_COLOR,
        DEFAULT_LINK_SCALE, DEFAULT_OPT_NAME,
    },
    node::Node,
};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum Animation {
    Both, // Animate in both directions
    Src,  // Animate towards the src node
    Dst,  // Animate towards the dst node
    None, // Do not animate
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
}

impl LinkContainerOpt {
    pub fn defaults() -> Self {
        return Self {
            id: String::from("default"),
            scale: DEFAULT_LINK_SCALE,
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
    pub mouse_index: Option<(RangeInclusive<u32>, RangeInclusive<u32>)>,
    pub screen_index: Option<(RangeInclusive<u32>, RangeInclusive<u32>)>,
    pub src_point: Option<Point>,
    pub dst_point: Option<Point>,
    pub r: Option<f64>,
    pub cl: Option<ComputedLinks>,
}

pub struct ComputedLinks {
    pub width: f64,
    pub links: Vec<(Point, Point)>,
    pub distance: Option<f64>,
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
            dst = s;
        } else {
            return;
        }

        let lc_opt = ops.get_lc(&self.opt);
        let r = self.get_min_r(src, dst) * lc_opt.scale;
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
        }

        if !self.links.is_empty() {}
    }

    pub fn compute_link_segement(&self, src: &Point, dst: &Point, r: f64) -> ComputedLinks {
        let links = Vec::new();
        let width = self.compute_line_width(r);
        // Apply the shifting offset to create movement
        // ctx.lineDashOffset = offset;
        let mut distance = None;
        return ComputedLinks {
            width,
            links,
            distance,
        };
    }
    pub fn compute_line_width(&self, r: f64) -> f64 {
        return r / (r * 2.0 + 1.0);
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
