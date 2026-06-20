use wasm_bindgen::prelude::*;

use crate::{CalculatorTrait, ContainsPoint, GetCenter, Point, PointBox};
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
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct NodeOpt {
    pub id: String,
    pub src: String,
    pub color: String,
}

#[wasm_bindgen]
impl Node {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, w: f64, h: f64, id: u32, label: String, opt: String) -> Self {
        return Self {
            x,
            y,
            w,
            h,
            id,
            label,
            opt,
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
