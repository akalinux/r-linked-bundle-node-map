use std::{collections::HashMap, hash::Hash, ops::RangeInclusive};
pub type IndexSet = (
    Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
    Option<(RangeInclusive<i32>, RangeInclusive<i32>)>,
);
pub struct Indexer<T: Eq + PartialEq + Hash> {
    pub step: i32,
    pub idx_x: HashMap<i32, HashMap<i32, HashMap<T, ()>>>,
}

impl Indexer<u32> {
    pub fn new(step: i32) -> Self {
        return Self {
            step,
            idx_x: HashMap::new(),
        };
    }
}

impl Indexer<u64> {
    pub fn new(step: i32) -> Self {
        return Self {
            step,
            idx_x: HashMap::new(),
        };
    }
}

impl<T: Eq + PartialEq + Hash + Copy + Clone> Indexer<T> {
    pub fn clear(&mut self) {
        self.idx_x.clear();
    }
    pub fn update(&mut self, id: T, src: IndexSet) {
        let x_idx = &mut self.idx_x;
        let step = self.step as usize;
        // first we clear
        if let Some((x, r)) = src.0 {
            for i in x.step_by(step) {
                if let Some(y_idx) = x_idx.get_mut(&i) {
                    let y = r.clone();
                    for i in y.step_by(step as usize) {
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
                for y in r.clone().step_by(step as usize) {
                    if let Some(y_idx) = x_idx.get_mut(&x) {
                        if let Some(n_idx) = y_idx.get_mut(&y) {
                            n_idx.insert(id, ());
                        } else {
                            let mut dst = HashMap::new();
                            dst.insert(id, ());
                            y_idx.insert(y, dst);
                        }
                    } else {
                        let mut dst = HashMap::new();
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
