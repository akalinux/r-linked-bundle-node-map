use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, ContainsPoint, GetCenter, Point, PointBox, constants::DEFAULT_OPT_NAME,
};

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum LabelPosition {
    Top,
    Center,
    Bottom,
}
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub w: f64,
    pub id: u32,
    pub label: String,
    pub opt: u32,
    pub groups: Vec<u32>,
}

pub struct GetRelatedNodes<'n> {
    nodes: &'n NodeStates,
    known_nodes: HashMap<u32, ()>,
    known_groups: HashMap<u32, ()>,
    todo: Vec<u32>,
}
impl<'n> GetRelatedNodes<'n> {
    pub fn new(init: &[u32], nodes: &'n NodeStates) -> Self {
        let mut known_nodes = HashMap::with_capacity(init.len() * 2);
        let known_groups = HashMap::with_capacity(init.len() * 2);
        let mut todo = Vec::with_capacity(init.len() * 4);
        for id in init {
            if known_nodes.contains_key(id) || nodes.get(*id).is_none() {
                continue;
            }
            known_nodes.insert(*id, ());
            todo.push(*id)
        }
        let mut res = Self {
            nodes,
            known_nodes,
            known_groups,
            todo,
        };
        res.todo.reserve(init.len() * 2);
        return res;
    }
}

impl<'n> Iterator for GetRelatedNodes<'n> {
    type Item = &'n Node;
    fn next(&mut self) -> Option<Self::Item> {
        let todo = &mut self.todo;
        if todo.is_empty() {
            return None;
        }
        let next = todo.pop().unwrap();
        let known_nodes = &mut self.known_nodes;
        let known_groups = &mut self.known_groups;
        let nodes = self.nodes;
        let groups = &self.nodes.get(next).unwrap().groups;
        todo.reserve(groups.len());
        known_nodes.reserve(groups.len());
        known_groups.reserve(groups.len());

        for group_id in groups {
            if known_groups.contains_key(group_id) {
                continue;
            }
            known_groups.insert(*group_id, ());
            let list;
            match nodes.groups.get(group_id) {
                Some(l) => list = l,
                None => continue,
            }
            for node_id in list {
                if known_nodes.contains_key(node_id) || !self.nodes.nodes.contains_key(node_id) {
                    continue;
                }
                known_nodes.insert(*node_id, ());
                todo.push(*node_id);
            }
        }
        return Some(nodes.get(next).unwrap());
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct NodeOpt {
    pub id: u32,
    pub label: String,
    pub img: String,
    pub color: String,
    pub label_position: LabelPosition,
}

impl NodeOpt {
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            label: String::from(DEFAULT_OPT_NAME),
            img: String::from(""),
            color: String::from("DEFAULT_COLOR"),
            label_position: LabelPosition::Top,
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
        opt: u32,
        groups: Vec<u32>,
    ) -> Self {
        return Self {
            x,
            y,
            w,
            h,
            id,
            label,
            opt,
            groups,
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
            self.opt,
            self.groups.clone(),
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
    pub updates: HashMap<u32, Node>,
    pub nodes: HashMap<u32, Node>,
    pub groups: HashMap<u32, Vec<u32>>,
}

impl NodeStates {
    pub fn get_related<'n>(&'n self, node_ids: &[u32]) -> GetRelatedNodes<'n> {
        return GetRelatedNodes::new(node_ids, self);
    }

    pub fn group_add(&mut self, id: u32, nodes: &[u32]) -> Option<Vec<u32>> {
        self.groups.insert(id, Vec::from(nodes))
    }
    pub fn group_remove(&mut self, id: u32) -> Option<Vec<u32>> {
        self.groups.remove(&id)
    }
    pub fn new(size: usize) -> Self {
        return Self {
            updates: HashMap::new(),
            nodes: HashMap::with_capacity(size),
            groups: HashMap::new(),
        };
    }
    pub fn reserve(&mut self, size: usize) {
        self.nodes.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    pub fn get_node_changes(&self) -> Vec<Node> {
        let mut nodes = Vec::with_capacity(self.updates.len());

        for src in self.updates.values() {
            nodes.push(src.clone());
        }
        return nodes;
    }
    pub fn node_count(&self) -> usize {
        return self.nodes.len();
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
