use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    mem,
};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, ContainsPoint, FullBox, GetCenter, Point, PointBox,
    constants::DEFAULT_OPT_NAME, id_compare,
};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum LabelPosition {
    Top,
    Center,
    Bottom,
}
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
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

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.id == other.id {
            return Some(Ordering::Equal);
        } else if self.get_min_x() < other.get_min_x() {
            return Some(Ordering::Less);
        } else if other.get_min_x() < self.get_min_x() {
            return Some(Ordering::Greater);
        } else if self.get_max_x() > other.get_max_x() {
            return Some(Ordering::Less);
        } else if other.get_max_x() > self.get_max_x() {
            return Some(Ordering::Greater);
        } else
        // if we got here.. then both min and max x are equal
        if self.get_min_y() < other.get_min_y() {
            return Some(Ordering::Less);
        } else if other.get_min_y() < self.get_min_y() {
            return Some(Ordering::Greater);
        } else if self.get_max_y() > other.get_max_y() {
            return Some(Ordering::Less);
        } else if other.get_max_y() > self.get_max_y() {
            return Some(Ordering::Less);
        // if we got here.. then x and y axis are equal.. we just sort based on id
        } else if self.id < other.id {
            return Some(Ordering::Less);
        }

        Some(Ordering::Greater)
    }
}
impl Eq for Node {}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
pub struct GetRelatedNodes<'n> {
    pub nodes: &'n mut NodeStates,
    pub known_nodes: HashSet<u32>,
    pub known_groups: HashSet<u32>,
    pub todo: Vec<u32>,
    base_nodes: HashSet<u32>,
}
impl<'n> GetRelatedNodes<'n> {
    pub fn new(init: &[u32], nodes: &'n mut NodeStates) -> Self {
        let mut known_nodes = HashSet::with_capacity(init.len() * 2);
        let known_groups = HashSet::with_capacity(init.len() * 2);
        let mut todo = Vec::with_capacity(init.len() * 4);
        let mut base_nodes = HashSet::with_capacity(init.len());
        for id in init {
            if known_nodes.contains(id) || nodes.get(*id).is_none() {
                continue;
            }
            known_nodes.insert(*id);
            todo.push(*id);
            base_nodes.insert(*id);
        }
        let mut res = Self {
            nodes,
            known_nodes,
            known_groups,
            todo,
            base_nodes,
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
        let nodes = &self.nodes;
        if !self.base_nodes.contains(&next) {
            return Some(unsafe { mem::transmute(nodes.get(next).unwrap()) });
        }
        let groups = &self.nodes.get(next).unwrap().groups;
        todo.reserve(groups.len());
        known_nodes.reserve(groups.len());
        known_groups.reserve(groups.len());

        for group_id in groups {
            if known_groups.contains(group_id) {
                continue;
            }
            known_groups.insert(*group_id);
            let list;
            match nodes.groups.get(group_id) {
                Some(l) => list = l,
                None => continue,
            }
            for node_id in list.keys() {
                if known_nodes.contains(node_id) || !self.nodes.nodes.contains_key(node_id) {
                    continue;
                }
                known_nodes.insert(*node_id);
                // we do not want to step into nodes outside of the orginal list
                todo.push(*node_id);
            }
        }
        return Some(unsafe { mem::transmute(nodes.get(next).unwrap()) });
    }
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
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

impl FullBox for Node {
    fn full_box(&self) -> (Point, Point, Point, Point) {
        let min_x = self.get_min_x();
        let max_x = self.get_max_x();
        let min_y = self.get_min_y();
        let max_y = self.get_max_y();
        return (
            Point { x: min_x, y: min_y }, // nw
            Point { x: max_x, y: min_y }, // ne
            Point { x: min_x, y: max_y }, // sw
            Point { x: max_x, y: max_y }, // se
        );
    }
}

#[wasm_bindgen]
impl Node {
    pub fn get_center(&self) -> Point {
        Point::new(self.x, self.y)
    }
    pub fn in_point(&self, p: &Point) -> bool {
        self.inside_square(&self.get_center(), p, self.w, self.h)
    }

    pub fn x_contains(&self, x: f64) -> bool {
        return !(x < self.get_min_x() || self.get_max_x() < x);
    }

    pub fn y_contains(&self, y: f64) -> bool {
        return !(y < self.get_min_y() || self.get_max_y() < y);
    }

    pub fn overlaps(&self, node: &Self) -> bool {
        ((self.x_contains(node.get_min_x()) || self.x_contains(node.get_max_x()))
            && (self.y_contains(node.get_min_y()) || self.y_contains(node.get_max_y())))
            || ((node.x_contains(self.get_min_x()) || node.x_contains(self.get_max_x()))
                && (node.y_contains(self.get_min_y()) || node.y_contains(self.get_max_y())))
    }

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
    pub node_updates: HashMap<u32, Node>,
    pub nodes: HashMap<u32, Node>,
    pub boxes: HashMap<u32, Node>,
    pub box_updates: HashMap<u32, Node>,
    pub groups: HashMap<u32, HashMap<u32, ()>>,
    pub center: Point,
}

impl NodeStates {
    pub fn node_in_point(&self, id: u32, p: &Point) -> Option<Node> {
        match self.get(id) {
            Some(n) => {
                if n.in_point(p) {
                    return Some(n.clone());
                }
                return None;
            }
            _ => return None,
        }
    }
    pub fn get_related<'n>(&'n mut self, node_ids: &[u32]) -> GetRelatedNodes<'n> {
        return GetRelatedNodes::new(node_ids, self);
    }

    pub fn new(size: usize) -> Self {
        return Self {
            boxes: HashMap::new(),
            box_updates: HashMap::new(),
            node_updates: HashMap::new(),
            nodes: HashMap::with_capacity(size),
            groups: HashMap::new(),
            center: Point { x: 0.0, y: 0.0 },
        };
    }
    pub fn reserve(&mut self, size: usize) {
        self.nodes.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    pub fn get_node_changes(&self) -> Vec<Node> {
        let mut nodes = Vec::with_capacity(self.node_updates.len());

        for src in self.node_updates.values() {
            nodes.push(src.clone());
        }
        return nodes;
    }
    pub fn node_count(&self) -> usize {
        return self.nodes.len();
    }

    fn clear_node_grps(&mut self, node_id: u32, groups: &[u32]) {
        for group in groups {
            let rm;
            if let Some(src) = self.groups.get_mut(group) {
                src.remove(&node_id);
                rm = src.is_empty()
            } else {
                rm = false;
            }
            if rm {
                self.groups.remove(group);
            }
        }
    }
    fn append_node_grps(&mut self, node_id: u32, groups: &[u32]) {
        for group in groups {
            if let Some(g) = self.groups.get_mut(group) {
                g.insert(node_id, ());
            } else {
                let g = HashMap::from([(node_id, ())]);
                self.groups.insert(*group, g);
            }
        }
    }

    pub fn insert(&mut self, node: Node) -> Option<Node> {
        if self.boxes.contains_key(&node.id) {
            panic!("Node: {}, is all ready listed as a box", node.id);
        }
        let res = self.node_updates.remove(&node.id);
        if let Some(n) = res {
            self.center.x -= n.x;
            self.center.y -= n.y;
            self.clear_node_grps(node.id, &n.groups);
        }
        self.center.x += node.x;
        self.center.y += node.y;
        self.append_node_grps(node.id, &node.groups);
        return self.nodes.insert(node.id, node);
    }

    pub fn insert_box(&mut self, node: Node) -> Option<Node> {
        if self.nodes.contains_key(&node.id) {
            panic!("Box: {}, is all ready listed as a: node", node.id);
        }
        let res = self.box_updates.remove(&node.id);
        if let Some(n) = res {
            self.clear_node_grps(node.id, &n.groups);
        }
        self.append_node_grps(node.id, &node.groups);
        return self.nodes.insert(node.id, node);
    }

    pub fn remove_box(&mut self, id: u32) -> Option<Node> {
        self.box_updates.remove(&id);
        let res = self.boxes.remove(&id);

        if let Some(n) = &res {
            self.clear_node_grps(n.id, &n.groups);
        }
        return res;
    }

    pub fn remove(&mut self, id: u32) -> Option<Node> {
        self.node_updates.remove(&id);
        let res = self.nodes.remove(&id);

        if let Some(n) = &res {
            self.center.x -= n.x;
            self.center.y -= n.y;
            self.clear_node_grps(n.id, &n.groups);
        }
        return res;
    }

    pub fn get_center(&self) -> Point {
        if self.nodes.is_empty() {
            return Point { x: 0.0, y: 0.0 };
        }
        return Point {
            x: self.center.x / self.nodes.len() as f64,
            y: self.center.y / self.nodes.len() as f64,
        };
    }
    pub fn update(&mut self, node: Node) -> Option<Node> {
        if let Some(node) = self.node_updates.get(&node.id) {
            self.center.x -= node.x;
            self.center.y -= node.y;
        } else if let Some(node) = self.nodes.get(&node.id) {
            self.center.x -= node.x;
            self.center.y -= node.y;
        }
        self.center.x += node.x;
        self.center.y += node.y;
        return self.node_updates.insert(node.id, node);
    }

    pub fn get(&self, id: u32) -> Option<&Node> {
        if let Some(node) = self.node_updates.get(&id) {
            return Some(node);
        } else if let Some(node) = self.nodes.get(&id) {
            return Some(node);
        }
        return None;
    }
    pub fn get_box(&self, id: u32) -> Option<&Node> {
        if let Some(node) = self.box_updates.get(&id) {
            return Some(node);
        } else if let Some(node) = self.boxes.get(&id) {
            return Some(node);
        }
        return None;
    }
}

id_compare!(Node, NodeOpt);
