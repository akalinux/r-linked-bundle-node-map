use crate::{CalculatorTrait, PointBox, ScreenBox, Transform, link::LinkContainer, node::Node};
use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
    ops::RangeInclusive,
};

pub type IndexXY = (RangeInclusive<i64>, RangeInclusive<i64>);
pub type IndexPart = Option<IndexXY>;
pub type IndexSet = (IndexPart, IndexPart);

pub struct MouseIndex<T: Eq + PartialEq + Hash> {
    pub step: i64,
    pub idx_x: HashMap<i64, HashMap<i64, BTreeMap<T, ()>>>,
}

pub struct ScreenBoundY {
    pub nodes: BTreeMap<u32, ()>,
    pub links: BTreeMap<u64, ()>,
}

pub struct ScreenIndex {
    pub x: BTreeMap<i64, BTreeMap<i64, ScreenBoundY>>,
    pub step: i64,
}

impl ScreenIndex {
    pub fn new(step: i64) -> Self {
        return Self {
            x: BTreeMap::new(),
            step,
        };
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
        let width = (end_x - start_x) as u64;
        let height = (end_y - start_y) as u64;
        return Some(ScreenBox {
            x: *start_x,
            y: *start_y,
            width,
            height,
            step: self.step,
        });
    }

    pub fn on_screen(
        &self,
        width: u32,
        height: u32,
        t: &Transform,
        node_count: u32,
    ) -> (Vec<u32>, Vec<u64>) {
        match self.max_screen() {
            Some(sb) => {
                let cmp = ScreenBox::new(t, width as u64, height as u64, self.step);
                match sb.contains(&cmp) {
                    Some(view) => {
                        // if we got here.. then we have a valid view point
                        let size = ((view.scale() / sb.scale()) * node_count as f64) as usize;
                        let mut nodes = Vec::with_capacity(size);
                        let mut links = Vec::with_capacity(size);
                        let mut known_nodes = HashMap::with_capacity(size);
                        let mut known_links = HashMap::with_capacity(size);
                        let (x, y) = view.getxy_bounds();
                        let idx = &self.x;
                        for x in x.step_by(self.step as usize) {
                            for y in y.clone().step_by(self.step as usize) {
                                let sets = idx.get(&x).unwrap().get(&y).unwrap();
                                for (node_id, _) in &sets.nodes {
                                    if known_nodes.contains_key(node_id) {
                                        continue;
                                    }
                                    nodes.push(*node_id);
                                    known_nodes.insert(*node_id, ());
                                }
                                for (link_id, _) in &sets.links {
                                    if known_links.contains_key(link_id) {
                                        continue;
                                    }
                                    links.push(*link_id);
                                    known_links.insert(*link_id, ());
                                }
                            }
                        }
                        // free any lost memory here!
                        nodes.shrink_to_fit();
                        links.shrink_to_fit();
                        return (nodes, links);
                    }
                    None => (),
                }
                return (Vec::new(), Vec::new());
            }
            None => return (Vec::new(), Vec::new()),
        }
    }
}

impl CalculatorTrait for ScreenIndex {}
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

                    let mut clear = false;
                    {
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
                            if idx_x.is_empty() {
                                clear = true;
                            }
                        }
                    }
                    if clear {
                        self.x.remove(&x);
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
                        self.x.insert(x, BTreeMap::new());
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

enum IndexLoadState {
    Screen,
    Mouse,
    Both,
    None,
}

pub struct IsIndexed<T: Eq + PartialEq + Hash + Copy + Clone + Ord> {
    idx: HashMap<T, IndexLoadState>,
}
impl<T: Eq + PartialEq + Hash + Copy + Clone + Ord> IsIndexed<T> {
    pub fn new(size: usize) -> Self {
        return Self {
            idx: HashMap::with_capacity(size),
        };
    }
    pub fn reserve(&mut self, size: usize) {
        self.idx.reserve(size);
    }

    pub fn shrink_to_fit(&mut self) {
        self.idx.shrink_to_fit();
    }

    pub fn clear(&mut self, t: T) {
        self.idx.remove(&t);
    }

    fn manage(&mut self, t: T, add: bool, screen: bool) {
        let dst;
        let current = self.idx.get(&t);
        match current {
            None => match add {
                true => match screen {
                    true => dst = IndexLoadState::Screen, // add this as screen object
                    false => dst = IndexLoadState::Mouse, // add this as mouse object
                },
                false => return, // Nothing to add
            },
            Some(v) => match add {
                true => match screen {
                    true => match v {
                        IndexLoadState::Mouse => dst = IndexLoadState::Both,
                        _ => return,
                    },
                    false => match v {
                        IndexLoadState::Screen => dst = IndexLoadState::Both,
                        _ => return,
                    },
                },
                false => match screen {
                    true => match v {
                        IndexLoadState::Both => dst = IndexLoadState::Mouse,
                        IndexLoadState::Screen => dst = IndexLoadState::None,
                        _ => return,
                    },
                    false => match v {
                        IndexLoadState::Both => dst = IndexLoadState::Screen,
                        IndexLoadState::Mouse => dst = IndexLoadState::None,
                        _ => return,
                    },
                },
            },
        }

        match dst {
            IndexLoadState::None => self.idx.remove(&t),
            _ => self.idx.insert(t, dst),
        };
    }
    fn is(&self, t: T, screen: bool) -> bool {
        match self.idx.get(&t) {
            None => return false,
            Some(v) => match v {
                IndexLoadState::Both => return true,
                IndexLoadState::Mouse => match screen {
                    true => return false,
                    false => return true,
                },
                IndexLoadState::Screen => match screen {
                    true => return true,
                    false => return false,
                },
                IndexLoadState::None => return false,
            },
        }
    }
    pub fn is_both(&self, t: T) -> bool {
        match self.idx.get(&t) {
            None => return false,
            Some(v) => match v {
                IndexLoadState::Both => return true,
                _ => return false,
            },
        };
    }
    pub fn clear_screen(&mut self, t: T) {
        self.manage(t, false, true);
    }
    pub fn clear_mouse(&mut self, t: T) {
        self.manage(t, false, false);
    }
    pub fn is_screen(&self, t: T) -> bool {
        return self.is(t, true);
    }
    pub fn is_mouse(&self, t: T) -> bool {
        return self.is(t, false);
    }
    pub fn add(&mut self, t: T) {
        self.idx.insert(t, IndexLoadState::Both);
    }
    pub fn add_screen(&mut self, t: T) {
        self.manage(t, true, true);
    }
    pub fn add_mouse(&mut self, t: T) {
        self.manage(t, true, false);
    }
    pub fn is_empty(&self) -> bool {
        return self.idx.is_empty();
    }
    pub fn remove(&mut self, t: T) {
        self.idx.remove(&t);
    }
}

impl Indexers {
    pub fn new(node_mouse_b: i64, link_mouse_b: i64, screen_mouse_b: i64, size: usize) -> Self {
        return Self {
            link_mouse_idx: MouseIndex::new(link_mouse_b, size),
            node_mouse_idx: MouseIndex::new(node_mouse_b, size),
            screen_index: ScreenIndex::new(screen_mouse_b),
            node_check: IsIndexed::new(size),
            link_check: IsIndexed::new(size),
        };
    }

    pub fn reserve(&mut self, size: usize) {
        self.link_mouse_idx.reserve(size);
        self.node_mouse_idx.reserve(size);
        self.node_check.reserve(size);
        self.link_check.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.link_mouse_idx.shrink_to_fit();
        self.node_mouse_idx.shrink_to_fit();
        self.node_check.shrink_to_fit();
        self.link_check.shrink_to_fit();
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
    pub fn new(step: i64, size: usize) -> Self {
        return Self {
            step,
            idx_x: HashMap::with_capacity(size),
        };
    }

    pub fn reserve(&mut self, size: usize) {
        self.idx_x.reserve(size);
    }
    pub fn shrink_to_fit(&mut self) {
        self.idx_x.shrink_to_fit();
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
