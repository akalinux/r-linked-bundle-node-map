use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, ContainsPoint, GetCenter, Point, PointBox, constants::DEFAULT_OPT_NAME,
};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum LabelPosition {
    Top,
    Bottom,
    Center,
}
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub w: f64,
    pub id: u32,
    pub label: String,
    pub opt: String,
    pub linked: Vec<u32>,
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct NodeOpt {
    pub id: String,
    pub img: String,
    pub color: String,
    pub layer: i8,
    pub label: LabelPosition,
}

impl NodeOpt {
    pub fn defaults() -> Self {
        return Self {
            id: String::from(DEFAULT_OPT_NAME),
            img: String::from(""),
            color: String::from("DEFAULT_COLOR"),
            layer: 0,
            label: LabelPosition::Top,
        };
    }
}

#[wasm_bindgen]
impl Node {
    pub fn get_min_r(&self) -> f64 {
        if self.w < self.h {
            return self.w;
        }
        return self.h;
    }
    #[wasm_bindgen(constructor)]
    pub fn new(
        x: f64,
        y: f64,
        w: f64,
        h: f64,
        id: u32,
        label: String,
        opt: String,
        linked: Vec<u32>,
    ) -> Self {
        return Self {
            x,
            y,
            w,
            h,
            id,
            label,
            opt,
            linked,
        };
    }

    pub fn transform(&self, x: f64, y: f64, w: f64, h: f64) -> Self {
        return Self::new(
            self.x + x,
            self.y + y,
            self.w + w,
            self.h + h,
            self.id,
            String::from(&self.label),
            String::from(&self.opt),
            self.linked.clone(),
        );
    }
}

impl GetCenter for Node {
    fn get_center(&self) -> Point {
        return Point {
            x: self.x,
            y: self.y,
        };
    }
}

impl ContainsPoint for Node {
    fn contains_point(&self, p: &Point) -> bool {
        return self.inside_square(p, &self.get_center(), self.w, self.h);
    }
}
impl CalculatorTrait for Node {}
impl PointBox for Node {
    fn get_min_x(&self) -> f64 {
        return self.x - self.w * 0.5;
    }

    fn get_max_x(&self) -> f64 {
        return self.x + self.w * 0.5;
    }

    fn get_max_y(&self) -> f64 {
        return self.y + self.h * 0.5;
    }

    fn get_min_y(&self) -> f64 {
        return self.y - self.h * 0.5;
    }
}

pub struct NodeStates {
    updates: HashMap<u32, Node>,
    nodes: HashMap<u32, Node>,
}

impl NodeStates {
    pub fn new() -> Self {
        return Self {
            updates: HashMap::new(),
            nodes: HashMap::new(),
        };
    }

    pub fn insert(&mut self, node: Node) -> Option<Node> {
        self.updates.remove(&node.id);
        return self.nodes.insert(node.id, node);
    }

    pub fn remove(&mut self, id: u32) -> Option<Node> {
        self.updates.remove(&id);
        return self.nodes.remove(&id);
    }
    pub fn update(&mut self, node: Node) -> Option<Node> {
        return self.updates.insert(node.id, node);
    }

    pub fn get(&self, id: u32) -> Option<&Node> {
        if let Some(node) = self.updates.get(&id) {
            return Some(node);
        } else if let Some(node) = self.nodes.get(&id) {
            return Some(node);
        }
        return None;
    }
}
