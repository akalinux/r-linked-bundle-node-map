use crate::{PointBox, link::LinkContainer, node::Node};
use std::{collections::BTreeMap, collections::HashMap, hash::Hash, ops::RangeInclusive};

pub type IndexPart = Option<(RangeInclusive<i32>, RangeInclusive<i32>)>;
pub type IndexSet = (IndexPart, IndexPart);

pub struct MouseIndex<T: Eq + PartialEq + Hash> {
    pub step: i32,
    pub idx_x: HashMap<i32, HashMap<i32, BTreeMap<T, ()>>>,
}

pub struct ScreenBoundY {
    pub nodes: BTreeMap<u32, ()>,
    pub links: BTreeMap<u64, ()>,
}

pub struct ScreenIndex {
    pub x: HashMap<i32, HashMap<i32, ScreenBoundY>>,
    pub step: i32,
}

impl ScreenIndex {
    pub fn new(step: i32, size: usize) -> Self {
        return Self {
            x: HashMap::with_capacity(size),
            step,
        };
    }
}
macro_rules! screen_idx {
    ($method:ident,$clear:ident,$update:ident,$field:ident,$id:ty) => {
        impl ScreenIndex {
            pub fn $clear(&mut self, id: $id, src: IndexPart) {
                if src.is_none() {
                    return;
                }
                let old = src.unwrap();
                let step = self.step as usize;
                for x in old.0.step_by(step) {
                    if !self.x.contains_key(&x) {
                        continue;
                    }
                    let idx_x = self.x.get_mut(&x).unwrap();
                    for y in old.1.clone().step_by(step) {
                        if !idx_x.contains_key(&y) {
                            continue;
                        }
                        let idx_y = idx_x.get_mut(&y).unwrap();
                        idx_y.$field.remove(&id);

                        if idx_y.nodes.is_empty() && idx_y.links.is_empty() {
                            idx_x.remove(&y);
                        }
                    }
                }
            }
            pub fn $update(&mut self, id: $id, src: IndexPart) {
                if src.is_none() {
                    return;
                }
                let new = src.unwrap();
                let step = self.step as usize;
                for x in new.0.step_by(step) {
                    let idx_x;
                    if let Some(t) = self.x.get_mut(&x) {
                        idx_x = t;
                    } else {
                        // None of this exists.. need to make it
                        self.x.insert(x, HashMap::new());
                        idx_x = self.x.get_mut(&x).unwrap();
                    }
                    for y in new.1.clone().step_by(step) {
                        let idx_y;
                        if let Some(t) = idx_x.get_mut(&y) {
                            idx_y = t;
                        } else {
                            let sb = ScreenBoundY {
                                nodes: BTreeMap::new(),
                                links: BTreeMap::new(),
                            };
                            idx_x.insert(y, sb);
                            idx_y = idx_x.get_mut(&y).unwrap();
                        }
                        idx_y.$field.insert(id, ());
                    }
                }
            }
            pub fn $method(&mut self, id: $id, data: IndexSet) {
                let (old, new) = data;
                self.$clear(id, old);
                self.$update(id, new);
            }
        }
    };
}
screen_idx!(index_node, clear_node, update_node, nodes, u32);
screen_idx!(index_link, clear_link, update_link, links, u64);

pub struct Indexers {
    pub node_mouse_idx: MouseIndex<u32>,
    pub link_mouse_idx: MouseIndex<u64>,
    pub screen_index: ScreenIndex,
    pub node_check: IsIndexed<u32>,
    pub link_check: IsIndexed<u64>,
}

pub struct IsIndexed<T: Eq + PartialEq + Hash + Copy + Clone> {
    screen: HashMap<T, ()>,
    mouse: HashMap<T, ()>,
}
impl<T: Eq + PartialEq + Hash + Copy + Clone> IsIndexed<T> {
    pub fn new(size: usize) -> Self {
        return Self {
            screen: HashMap::with_capacity(size),
            mouse: HashMap::with_capacity(size),
        };
    }
    pub fn clear(&mut self, t: T) {
        self.screen.remove(&t);
        self.mouse.remove(&t);
    }
    pub fn clear_screen(&mut self, t: T) {
        self.screen.remove(&t);
    }
    pub fn clear_mouse(&mut self, t: T) {
        self.mouse.remove(&t);
    }
    pub fn is_screen(&self, t: T) -> bool {
        return self.screen.contains_key(&t);
    }
    pub fn is_mouse(&self, t: T) -> bool {
        return self.mouse.contains_key(&t);
    }
    pub fn add(&mut self, t: T) {
        self.mouse.insert(t, ());
        self.screen.insert(t, ());
    }
    pub fn add_screen(&mut self, t: T) {
        self.screen.insert(t, ());
    }
    pub fn add_mouse(&mut self, t: T) {
        self.mouse.insert(t, ());
    }
}

impl Indexers {
    pub fn new(node_mouse_b: i32, link_mouse_b: i32, screen_mouse_b: i32, size: usize) -> Self {
        return Self {
            link_mouse_idx: MouseIndex::new(link_mouse_b, size * 8),
            node_mouse_idx: MouseIndex::new(node_mouse_b, size * 32),
            screen_index: ScreenIndex::new(screen_mouse_b, size),
            node_check: IsIndexed::new(size * 64),
            link_check: IsIndexed::new(size * 64),
        };
    }

    pub fn clear_mouse_node(&mut self, node: &Node) {
        if !self.node_check.is_mouse(node.id) {
            return;
        }
        self.node_mouse_idx.update(
            node.id,
            (Some(node.index_bound(self.node_mouse_idx.step)), None),
        );
        self.node_check.clear_mouse(node.id);
    }

    pub fn clear_screen_node(&mut self, node: &Node) {
        if !self.node_check.is_screen(node.id) {
            return;
        }
        self.screen_index
            .clear_node(node.id, Some(node.index_bound(self.screen_index.step)));
        self.node_check.clear_screen(node.id);
    }

    pub fn index_mouse_node(&mut self, node: &Node) {
        self.node_check.add_mouse(node.id);
        self.node_mouse_idx.update(
            node.id,
            (None, Some(node.index_bound(self.node_mouse_idx.step))),
        );
    }

    pub fn index_screen_node(&mut self, node: &Node) {
        self.node_check.add_screen(node.id);
        self.screen_index.index_node(
            node.id,
            (None, Some(node.index_bound(self.screen_index.step))),
        );
    }

    pub fn clear_node(&mut self, node: &Node) {
        self.clear_mouse_node(node);
        self.clear_screen_node(node);
    }

    pub fn add_node(&mut self, node: &Node) {
        self.index_mouse_node(node);
        self.index_screen_node(node);
    }

    pub fn index_mouse_link(&mut self, link: &mut LinkContainer) {
        self.link_check.add_mouse(link.id);
        self.link_mouse_idx
            .update(link.id, link.mouse_index(self.link_mouse_idx.step, true));
    }

    pub fn clear_mouse_link(&mut self, link: &mut LinkContainer) {
        if !self.link_check.is_mouse(link.id) {
            return;
        }
        self.link_check.clear_mouse(link.id);
        self.link_mouse_idx
            .update(link.id, link.mouse_index(self.link_mouse_idx.step, false));
    }

    pub fn index_screen_link(&mut self, link: &mut LinkContainer) {
        self.link_check.add_screen(link.id);
        self.link_mouse_idx
            .update(link.id, link.mouse_index(self.link_mouse_idx.step, true));
    }

    pub fn clear_screen_link(&mut self, link: &mut LinkContainer) {
        if !self.link_check.is_screen(link.id) {
            return;
        }
        self.link_check.clear_screen(link.id);
        self.link_mouse_idx
            .update(link.id, link.mouse_index(self.link_mouse_idx.step, false));
    }

    pub fn add_link(&mut self, link: &mut LinkContainer) {
        self.index_mouse_link(link);
        self.index_screen_link(link);
    }

    pub fn clear_link(&mut self, link: &mut LinkContainer) {
        self.clear_mouse_link(link);
        self.clear_screen_link(link);
    }
}

impl<T: Eq + PartialEq + Hash + Copy + Clone + Ord> MouseIndex<T> {
    pub fn new(step: i32, size: usize) -> Self {
        return Self {
            step,
            idx_x: HashMap::with_capacity(size),
        };
    }
    pub fn clear(&mut self) {
        self.idx_x.clear();
    }
    pub fn update(&mut self, id: T, src: IndexSet) {
        let x_idx = &mut self.idx_x;
        // first we clear
        let step = self.step as usize;
        if let Some((x, r)) = src.0 {
            for i in x.step_by(step) {
                if let Some(y_idx) = x_idx.get_mut(&i) {
                    let y = r.clone();
                    for i in y.step_by(step) {
                        if let Some(n_idx) = y_idx.get_mut(&i) {
                            n_idx.remove(&id);
                            if n_idx.is_empty() {
                                y_idx.remove(&i);
                            }
                        }
                    }
                    if y_idx.is_empty() {
                        x_idx.remove(&i);
                    }
                }
            }
        }

        // Now we create!
        if let Some((x, r)) = src.1 {
            for x in x.step_by(step) {
                for y in r.clone().step_by(step) {
                    if let Some(y_idx) = x_idx.get_mut(&x) {
                        if let Some(n_idx) = y_idx.get_mut(&y) {
                            n_idx.insert(id, ());
                        } else {
                            let mut dst = BTreeMap::new();
                            dst.insert(id, ());
                            y_idx.insert(y, dst);
                        }
                    } else {
                        let mut dst = BTreeMap::new();
                        dst.insert(id, ());
                        let mut src = HashMap::new();
                        src.insert(y, dst);
                        x_idx.insert(x, src);
                    }
                }
            }
        }
    }
}
