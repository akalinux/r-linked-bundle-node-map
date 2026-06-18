use wasm_bindgen::prelude::*;

use crate::{CORE_R, CalculatorTrait, ContainsPoint, Point, PointBox};
#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
pub struct Node {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub w: f64,
    pub id: String,
    pub label: String,
    pub opt: String,
}

#[wasm_bindgen]
impl Node {
    pub fn new_defaults(x: f64, y: f64, id: String) -> Self {
        let label = String::from(&id);
        return Self {
            x,
            y,
            id,
            w: CORE_R,
            h: CORE_R,
            label,
            opt: String::from("node_defaults"),
        };
    }
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, id: String, label: String, w: f64, h: f64, opt: String) -> Self {
        return Self {
            x,
            y,
            id,
            w,
            h,
            label,
            opt,
        };
    }

    pub fn transformed(&self, x: f64, y: f64, scale: f64) -> Self {
        return Self::new(
            self.x + x,
            self.y + y,
            String::from(&self.id),
            String::from(&self.label),
            self.w * scale,
            self.h * scale,
            String::from(&self.opt),
        );
    }

    pub fn get_center(&self) -> Point {
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
