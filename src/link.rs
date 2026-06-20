use std::{collections::HashMap, mem, ops::RangeInclusive};

use wasm_bindgen::prelude::*;

use crate::{
    calc::{NodeStates, Options},
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
pub struct LinkOpt {
    pub id: String,
    pub color: String,
    pub animation_color: String,
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct BunldeOpt {
    pub id: String,
    pub color: String,
    pub src: String,
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
    pub animations: Vec<u32>,
    pub id: u64,
    pub mouse_index: Option<(RangeInclusive<u32>, RangeInclusive<u32>)>,
    pub screen_index: Option<(RangeInclusive<u32>, RangeInclusive<u32>)>,
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
    pub fn update(&mut self, nodes: &NodeStates, ops: &Options) {}
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
            animations: Vec::new(),
        };
    }

    pub fn is_animated(&self) -> bool {
        return !self.animations.is_empty();
    }

    pub fn add_link(&mut self, link: Link) -> Option<Link> {
        let mut animated = true;
        match link.animation {
            Animation::None => animated = false,
            _ => (),
        }
        for id in self.animations.iter() {
            if *id == link.id {
                if animated {
                    animated = false;
                    break;
                }
            }
        }
        if animated {
            self.animations.push(link.id);
        }
        for (id, l) in self.links.iter().enumerate() {
            if l.id == link.id {
                return Some(mem::replace(&mut self.links[id], link));
            }
        }
        self.links.push(link);
        return None;
    }

    pub fn add_bundle(&mut self, bundle: Bundle) -> Option<Bundle> {
        for (id, b) in self.bundles.iter().enumerate() {
            if b.id == b.id {
                return Some(mem::replace(&mut self.bundles[id], bundle));
            }
        }
        self.bundles.push(bundle);
        return None;
    }

    pub fn remove_link(&mut self, id: u32) -> Option<Link> {
        for (i, aid) in self.animations.iter().enumerate() {
            if *aid == id {
                self.animations.remove(i);
                break;
            }
        }
        for (i, l) in self.links.iter().enumerate() {
            if l.id == id {
                let res = self.links.remove(i);
                return Some(res);
            }
        }
        return None;
    }

    pub fn remove_bundle(&mut self, id: u32) -> Option<Bundle> {
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
