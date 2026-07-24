use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    mem, process,
};

use wasm_bindgen::prelude::*;

use crate::{
    CalculatorTrait, ContainsPoint, FullBox, GetCenter, ImgSrc, Point, PointBox, RenderBox,
    bsp::ScreenSlot, id_compare,
};

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum LabelPosition {
    Top,
    Center,
    Bottom,
}
#[wasm_bindgen(inspectable, getter_with_clone)]
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

impl RenderBox for Node {
    fn id(&self) -> crate::ImgWatchId {
        crate::ImgWatchId::Node(self.id)
    }
    fn width(&self) -> f64 {
        self.w
    }

    fn height(&self) -> f64 {
        self.h
    }

    fn x(&self) -> f64 {
        self.x - self.w * 0.5
    }

    fn y(&self) -> f64 {
        self.y - self.h * 0.5
    }
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
        match self.partial_cmp(other) {
            Some(v) => return v,
            None => process::abort(),
        }
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
        let mut todo = Vec::with_capacity(init.len());
        let mut base_nodes = HashSet::with_capacity(init.len());
        for id in init {
            if known_nodes.contains(id)
                || (nodes.get(*id).is_none() && nodes.get_box(*id).is_none())
            {
                continue;
            }
            known_nodes.insert(*id);
            todo.push(*id);
            base_nodes.insert(*id);
        }
        Self {
            nodes,
            known_nodes,
            known_groups,
            todo,
            base_nodes,
        }
    }
}

impl<'n> Iterator for GetRelatedNodes<'n> {
    type Item = (&'n Node, ScreenSlot);
    fn next(&mut self) -> Option<Self::Item> {
        let todo = &mut self.todo;
        if todo.is_empty() {
            return None;
        }
        let next = todo.pop()?;
        let known_nodes = &mut self.known_nodes;
        let known_groups = &mut self.known_groups;
        let nodes = &self.nodes;
        if !self.base_nodes.contains(&next) {
            let res;
            let ss;
            if let Some(n) = nodes.get_box(next) {
                res = n;
                ss = ScreenSlot::Box(n.id)
            } else {
                res = nodes.get(next)?;
                ss = ScreenSlot::Node(res.id)
            }
            return Some((unsafe { mem::transmute(res) }, ss));
        }
        let groups;
        if let Some(n) = self.nodes.get_box(next) {
            groups = &n.groups
        } else {
            let n = self.nodes.get(next)?;
            groups = &n.groups
        }

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
            for node_id in list.iter() {
                if known_nodes.contains(node_id)
                    || !(nodes.nodes.contains_key(node_id) || nodes.boxes.contains_key(node_id))
                {
                    continue;
                }
                known_nodes.insert(*node_id);
                // we do not want to step into nodes outside of the orginal list
                todo.push(*node_id);
            }
        }
        let node;
        let ss;
        if let Some(n) = nodes.get_box(next) {
            node = n;
            ss = ScreenSlot::Box(node.id);
        } else {
            node = nodes.get(next)?;
            ss = ScreenSlot::Node(node.id);
        }
        return Some((unsafe { mem::transmute(node) }, ss));
    }
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct NodeOpt {
    pub id: u32,
    pub img: String,
    pub color: String,
    pub label_position: LabelPosition,
}

impl ImgSrc for NodeOpt {
    fn img_src(&self) -> String {
        self.img.clone()
    }

    fn box_color(&self) -> String {
        self.color.clone()
    }

    fn watch_id(&self) -> crate::ImgWatchId {
        crate::ImgWatchId::Node(self.id)
    }
}

#[wasm_bindgen]
impl NodeOpt {
    #[wasm_bindgen(constructor)]
    pub fn new(id: u32, img: String, color: String, label_position: LabelPosition) -> Self {
        Self {
            id,
            img,
            color,
            label_position,
        }
    }
}

impl NodeOpt {
    pub fn defaults() -> Self {
        return Self {
            id: 0,
            img: String::from(""),
            color: String::from("DEFAULT_COLOR"),
            label_position: LabelPosition::Top,
        };
    }
}

impl Node {
    pub fn get_center(&self) -> Point {
        Point::new(self.x, self.y)
    }
    pub fn full_box(&self) -> FullBox {
        let min_x = self.get_min_x();
        let max_x = self.get_max_x();
        let min_y = self.get_min_y();
        let max_y = self.get_max_y();
        (
            Point { x: min_x, y: min_y }, // nw
            Point { x: max_x, y: min_y }, // ne
            Point { x: min_x, y: max_y }, // sw
            Point { x: max_x, y: max_y }, // se
        )
    }
}

#[wasm_bindgen]
impl Node {
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
}
impl Node {
    pub fn in_point(&self, p: &Point) -> bool {
        self.inside_square(&self.get_center(), p, self.w, self.h)
    }

    pub fn to_point(&mut self, p: &Point) {
        self.x = p.x;
        self.y = p.y;
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
    /// Clones the non geometry settings into this node.
    pub fn merge(&mut self, node: &Self) {
        self.label = node.label.clone();
        self.opt = node.opt;
        self.groups = node.groups.clone();
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
    pub groups: HashMap<u32, HashSet<u32>>,
    pub center: Point,
}

impl NodeStates {
    pub fn node_in_point<'r>(&'r self, id: u32, p: &Point) -> Option<&'r Node> {
        match self.get(id) {
            Some(n) => {
                if n.in_point(p) {
                    return Some(n);
                }
            }
            _ => (),
        };
        None
    }

    pub fn box_in_point<'r>(&'r self, id: u32, p: &Point) -> Option<&'r Node> {
        match self.get_box(id) {
            Some(n) => {
                if n.in_point(p) {
                    return Some(n);
                }
            }
            _ => (),
        };
        None
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
                g.insert(node_id);
            } else {
                let g = HashSet::from([(node_id)]);
                self.groups.insert(*group, g);
            }
        }
    }

    pub fn insert(&mut self, node: Node, merge: bool) -> Option<Node> {
        if self.boxes.contains_key(&node.id) {
            return None;
        }
        let res = self.node_updates.remove(&node.id);
        if let Some(mut n) = res {
            self.clear_node_grps(node.id, &n.groups);
            if merge {
                n.merge(&node);
                self.node_updates.insert(n.id, n);
            } else {
                self.center.x -= n.x;
                self.center.y -= n.y;
                self.center.x += node.x;
                self.center.y += node.y;
            }
        } else {
            self.center.x += node.x;
            self.center.y += node.y;
        }
        self.append_node_grps(node.id, &node.groups);
        return self.nodes.insert(node.id, node);
    }

    pub fn insert_box(&mut self, node: Node, merge: bool) -> Option<Node> {
        if self.nodes.contains_key(&node.id) {
            return None;
        }
        let res = self.box_updates.remove(&node.id);
        if let Some(mut n) = res {
            self.clear_node_grps(node.id, &n.groups);
            if merge {
                n.merge(&node);
                self.box_updates.insert(n.id, n);
            }
        }
        self.append_node_grps(node.id, &node.groups);
        return self.boxes.insert(node.id, node);
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
    pub fn get_updates<'s>(&self) -> (Vec<&Node>, Vec<&Node>) {
        let mut nodes = Vec::with_capacity(self.node_updates.len());
        let mut boxes = Vec::with_capacity(self.box_updates.len());
        for node in self.node_updates.values() {
            nodes.push(node);
        }
        for node in self.box_updates.values() {
            boxes.push(node);
        }
        (nodes, boxes)
    }
    pub fn update(&mut self, node: Node) -> Option<Node> {
        if self.boxes.contains_key(&node.id) {
            return self.box_updates.insert(node.id, node);
        } else if let Some(node) = self.node_updates.get(&node.id) {
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
        }
        self.nodes.get(&id)
    }
    pub fn get_box(&self, id: u32) -> Option<&Node> {
        if let Some(node) = self.box_updates.get(&id) {
            return Some(node);
        }
        self.boxes.get(&id)
    }
}

id_compare!(Node, NodeOpt);
