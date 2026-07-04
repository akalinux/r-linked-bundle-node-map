use wasm_bindgen::prelude::wasm_bindgen;

use crate::{
    CalculatorTrait, Point, ScreenBox, Transform,
    link::{Bundle, Link, LinkContainsType, LinkStates},
    node::{Node, NodeStates},
};
use std::{
    collections::{BTreeMap, HashMap},
    mem,
    ops::RangeInclusive,
};

pub type IndexXY = (RangeInclusive<i64>, RangeInclusive<i64>);
pub type IndexPart = Option<IndexXY>;
pub type IndexSet = (IndexPart, IndexPart);

pub struct ScreenBoundY {
    pub nodes: BTreeMap<u32, ()>,
    pub links: BTreeMap<u64, ()>,
}

impl ScreenBoundY {
    pub fn new() -> Self {
        Self {
            nodes: BTreeMap::new(),
            links: BTreeMap::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        return self.nodes.is_empty() && self.links.is_empty();
    }
    pub fn add(&mut self, t: &ScreenSlot) {
        match t {
            ScreenSlot::Link(l) => self.links.insert(*l, ()),
            ScreenSlot::Node(n) => self.nodes.insert(*n, ()),
        };
    }
    pub fn remove(&mut self, t: &ScreenSlot) {
        match t {
            ScreenSlot::Link(l) => self.links.remove(l),
            ScreenSlot::Node(n) => self.nodes.remove(n),
        };
    }
}

pub struct ScreenIndex {
    pub x: BTreeMap<i64, BTreeMap<i64, ScreenBoundY>>,
    pub step: i64,
}

enum IdxBoxIterSection {
    Old,
    New,
    Done,
}
pub struct IdxBoxIter {
    old: IndexPart,
    new: IndexPart,
    step: i64,
    next: Option<(i64, i64, IdxBoxIterSection)>,
}

impl IdxBoxIter {
    pub fn new(old: IndexPart, new: IndexPart, step: i64) -> Self {
        if let Some((x, y)) = &old {
            if let Some((cx, cy)) = &new {
                if x == cx && y == cy {
                    return Self {
                        old,
                        new,
                        step,
                        next: None,
                    };
                }
            }
            Self {
                old: old.clone(),
                new,
                step,
                next: Some((*x.start(), *y.start(), IdxBoxIterSection::Old)),
            }
        } else if let Some((x, y)) = &new {
            Self {
                old,
                new: new.clone(),
                step,
                next: Some((*x.start(), *y.start(), IdxBoxIterSection::New)),
            }
        } else {
            Self {
                old,
                new,
                step,
                next: None,
            }
        }
    }
    fn h_next(x: &mut i64, y: &mut i64, s: &mut IdxBoxIterSection, b: &IndexPart) {
        match s {
            IdxBoxIterSection::Old => {
                if let Some(n) = b {
                    *x = *n.0.start();
                    *y = *n.1.start();
                    *s = IdxBoxIterSection::New;
                } else {
                    *s = IdxBoxIterSection::Done
                }
            }
            _ => *s = IdxBoxIterSection::Done,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum IdxBoxAction {
    Add,
    Remove,
}
impl Iterator for IdxBoxIter {
    type Item = (i64, i64, IdxBoxAction);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.next {
                Some((cx, cy, s)) => {
                    let a;
                    let b;
                    let t;
                    match s {
                        IdxBoxIterSection::Old => {
                            if let Some(d) = &self.old {
                                a = d;
                            } else {
                                return None;
                            }
                            b = &self.new;
                            t = IdxBoxAction::Remove;
                        }
                        IdxBoxIterSection::New => {
                            if let Some(d) = &self.new {
                                a = d;
                            } else {
                                return None;
                            }
                            b = &self.old;
                            t = IdxBoxAction::Add;
                        }
                        IdxBoxIterSection::Done => return None,
                    }
                    let x = *cx;
                    let y = *cy;
                    match b {
                        Some((cmp_x, cmp_y)) => {
                            if cmp_x.contains(&x) && cmp_y.contains(&y) {
                                *cx = cmp_x.end() + self.step;
                                continue;
                            }
                        }
                        None => {}
                    }
                    if x > *a.0.end() {
                        *cx = *a.0.start();
                        *cy += self.step;
                        continue;
                    } else if y > *a.1.end() {
                        Self::h_next(cx, cy, s, b);
                        continue;
                    }
                    *cx += self.step;
                    return Some((x, y, t));
                }
                None => return None,
            }
        }
    }
}
pub struct OnScreen<'s> {
    idx: &'s ScreenIndex,
    x: RangeInclusive<i64>,
    y: RangeInclusive<i64>,
    cx: i64,
    cy: i64,
    next: Option<(Vec<u32>, Vec<u64>, ScreenBox)>,
    known_nodes: HashMap<u32, ()>,
    known_links: HashMap<u64, ()>,
}

impl<'s> Iterator for OnScreen<'s> {
    type Item = (Vec<u32>, Vec<u64>, ScreenBox);
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() {
            return None;
        }

        loop {
            let x = self.cx;
            let y = self.cy;
            self.cy += self.idx.step;
            if self.cy > *self.y.end() {
                self.cy = *self.y.start();
                self.cx += self.idx.step;
            }
            if x > *self.x.end() {
                return mem::replace(&mut self.next, None);
            }

            match self.idx.x.get(&x) {
                Some(idx) => match idx.get(&y) {
                    Some(idy) => {
                        let kl = &mut self.known_links;
                        let kn = &mut self.known_nodes;
                        kl.reserve(idy.links.len());
                        kn.reserve(idy.nodes.len());
                        let mut links = Vec::with_capacity(idy.links.len());
                        let mut nodes = Vec::with_capacity(idy.nodes.len());
                        for lid in idy.links.keys() {
                            if kl.contains_key(lid) {
                                continue;
                            }
                            kl.insert(*lid, ());
                            links.push(*lid);
                        }
                        for id in idy.nodes.keys() {
                            if kn.contains_key(id) {
                                continue;
                            }
                            kn.insert(*id, ());
                            nodes.push(*id);
                        }
                        if nodes.is_empty() && links.is_empty() {
                            continue;
                        }
                        return mem::replace(
                            &mut self.next,
                            Some((nodes, links, ScreenBox::from_step(x, y, self.idx.step))),
                        );
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
}
impl<'s> OnScreen<'s> {
    pub fn new(idx: &'s ScreenIndex, t: &Transform, width: u32, height: u32) -> Self {
        let view;
        let sb;
        match idx.max_screen() {
            Some(s) => sb = s,
            None => return Self::no_screen(idx),
        }
        let cmp = ScreenBox::new(t, width, height, idx.step);
        match sb.contains(&cmp) {
            Some(v) => view = v,
            None => return Self::no_screen(idx),
        }
        let (x, y) = view.getxy_bounds();
        let known_nodes = HashMap::new();
        let known_links = HashMap::new();
        let mut res = Self {
            idx,
            cx: *x.start(),
            cy: *y.start(),
            x,
            y,
            known_links,
            known_nodes,
            next: Some((Vec::new(), Vec::new(), ScreenBox::empty())),
        };
        // need our first pass to ensure the data is populated.
        res.next();

        return res;
    }
    fn no_screen(idx: &'s ScreenIndex) -> Self {
        Self {
            idx,
            x: 0..=0,
            y: 0..=0,
            cy: 0,
            cx: 0,
            next: None,
            known_links: HashMap::new(),
            known_nodes: HashMap::new(),
        }
    }
}

pub enum ScreenSlot {
    Node(u32),
    Link(u64),
}

#[wasm_bindgen]
#[derive(Debug, PartialEq)]
pub enum PointLookupResult {
    Node(Node),
    Link(Link),
    Bundle(Bundle),
}
impl ScreenIndex {
    pub fn new(step: i64) -> Self {
        return Self {
            x: BTreeMap::new(),
            step,
        };
    }

    pub fn in_point(
        &self,
        p: &Point,
        t: &Transform,
        n: &NodeStates,
        l: &LinkStates,
    ) -> Option<PointLookupResult> {
        let tp = p.to_map_xy(p, t);
        let ip = tp.to_index_point(self.step);
        if let Some(y) = self.x.get(&ip.0)
            && let Some(r) = y.get(&ip.1)
        {
            for node_id in r.nodes.keys() {
                if let Some(node) = n.node_in_point(*node_id, &tp) {
                    return Some(PointLookupResult::Node(node));
                }
            }
            for link_id in r.links.keys() {
                if let Some(r) = l.link_contains_point(*link_id, p) {
                    match r {
                        LinkContainsType::Bundle(b) => return Some(PointLookupResult::Bundle(b)),
                        LinkContainsType::Link(l) => return Some(PointLookupResult::Link(l)),
                    }
                }
            }
        }

        None
    }
    pub fn max_screen(&self) -> Option<ScreenBox> {
        let idx_x = &self.x;
        if idx_x.is_empty() {
            return None;
        }
        let (start_x, y_t) = idx_x.first_key_value().unwrap();
        let (start_y, _) = y_t.first_key_value().unwrap();
        let (end_x, y_t) = idx_x.last_key_value().unwrap();
        let (end_y, _) = y_t.last_key_value().unwrap();
        let end_x = *end_x + self.step;
        let end_y = *end_y + self.step;
        let width = (end_x - start_x) as u32;
        let height = (end_y - start_y) as u32;
        return Some(ScreenBox {
            x: *start_x,
            y: *start_y,
            width,
            height,
            step: self.step,
        });
    }

    pub fn on_screen<'s>(&'s self, width: u32, height: u32, t: &Transform) -> OnScreen<'s> {
        return OnScreen::new(self, t, width, height);
    }

    pub fn index(&mut self, t: ScreenSlot, src: IndexSet) {
        for (x, y, todo) in IdxBoxIter::new(src.0, src.1, self.step) {
            match todo {
                IdxBoxAction::Remove => {
                    let mut clear_y = false;
                    match self.x.get_mut(&x) {
                        Some(idy) => match idy.get_mut(&y) {
                            Some(screen) => {
                                screen.remove(&t);
                                if screen.is_empty() {
                                    idy.remove(&y);
                                }
                                clear_y = idy.is_empty();
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                    if clear_y {
                        self.x.remove(&x);
                    }
                }
                IdxBoxAction::Add => match self.x.get_mut(&x) {
                    Some(idy) => match idy.get_mut(&y) {
                        Some(screen) => {
                            screen.add(&t);
                        }
                        None => {
                            let mut screen = ScreenBoundY::new();
                            screen.add(&t);
                            idy.insert(y, screen);
                        }
                    },
                    None => {
                        let mut idy = BTreeMap::new();
                        let mut screen = ScreenBoundY::new();
                        screen.add(&t);
                        idy.insert(y, screen);
                        self.x.insert(x, idy);
                    }
                },
            }
        }
    }
}
