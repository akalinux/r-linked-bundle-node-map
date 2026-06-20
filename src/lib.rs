use std::{f64::consts::PI, ops::RangeInclusive};
use wasm_bindgen::prelude::*;
pub mod bsp;
pub mod calc;
pub mod link;
pub mod node;

const TRIANGLE_MARGINE_FOR_ERROR: f64 = 1.00004;
const RAD2DEG: f64 = 180.0 / PI;
//const FULL_CIRCLE: f64 = 2.0 * PI;

pub trait ContainsPoint {
    /// Returns true if the element contains the given point.
    fn contains_point(&self, p: &Point) -> bool;
}

#[wasm_bindgen(inspectable)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub k: f64,
}

#[wasm_bindgen(inspectable)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn transform(&mut self, p: &Point) {
        self.x += p.x;
        self.y += p.y;
    }
}

impl GetCenter for Point {
    fn get_center(&self) -> Point {
        return *self;
    }
}

pub trait GetCenter {
    fn get_center(&self) -> Point;
}

pub trait PointBox {
    fn get_min_x(&self) -> f64;
    fn get_max_x(&self) -> f64;
    fn get_max_y(&self) -> f64;
    fn get_min_y(&self) -> f64;
    fn ne(&self) -> Point {
        return Point {
            x: self.get_max_x(),
            y: self.get_min_y(),
        };
    }
    fn nw(&self) -> Point {
        return Point {
            x: self.get_min_x(),
            y: self.get_min_y(),
        };
    }
    fn se(&self) -> Point {
        return Point {
            x: self.get_max_x(),
            y: self.get_max_y(),
        };
    }
    fn sw(&self) -> Point {
        return Point {
            x: self.get_min_x(),
            y: self.get_max_y(),
        };
    }
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

    fn transform_full_box(&self, p: &Point) -> (Point, Point, Point, Point) {
        let mut res = self.full_box();
        res.0.transform(p);
        res.1.transform(p);
        res.2.transform(p);
        res.3.transform(p);
        return res;
    }

    fn index_bound(&self, boundry: i32) -> (RangeInclusive<i32>, RangeInclusive<i32>) {
        let min_x = self.get_min_x() as i32;
        let max_x = self.get_max_x() as i32;
        let min_y = self.get_min_y() as i32;
        let max_y = self.get_max_y() as i32;
        let sx = min_x - (min_x % boundry);
        let ex = max_x + (max_x % boundry);
        let sy = min_y - (min_y % boundry);
        let ey = max_y + (max_y % boundry);
        return (sx..=ex, sy..=ey);
    }

    fn getx_index_bound(&self, boundry: i32) -> i32 {
        let min_x = self.get_min_x() as i32;
        return min_x - (min_x % boundry);
    }
    fn gety_index_bound(&self, boundry: i32) -> i32 {
        let min_y = self.get_min_y() as i32;
        return min_y - (min_y % boundry);
    }
}

pub trait CalculatorTrait {
    /// Converts degree to radians
    fn rad(&self, degree: f64) -> f64 {
        return degree * PI / 180.0;
    }

    fn get_angle(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        let dx = x1 - x2;
        let dy = y1 - y2;

        let base = dy.atan2(dx) * RAD2DEG; //- 180;
        if base < 0.0 {
            return base + 360.0;
        }

        return base;
    }

    fn inside_square(&self, n: &Point, p: &Point, w: f64, h: f64) -> bool {
        let dx = (p.x - n.x).abs();
        let dy = (p.y - n.y).abs();
        return !(dx > w * 0.5 || dy > h * 0.5);
    }

    fn get_xy(&self, cx: f64, cy: f64, r: f64, degree: f64) -> Point {
        let rad = self.rad(degree);

        let x = cx + r * rad.cos();
        let y = cy + r * rad.sin();
        return Point { x, y };
    }

    fn compute_r_for_even_space_on_circle(&self, r: f64, points: f64) -> f64 {
        let degree = 360.0 / points;
        let a = self.get_xy(0.0, 0.0, r, 0.0);
        let b = self.get_xy(0.0, 0.0, r, degree);
        let cmp = self.get_distance(a.x, a.y, b.x, b.y);
        let scale = r / cmp;
        return r * scale;
    }

    fn inside_circle(&self, p: &Point, c: Point, r: f64) -> bool {
        return (p.x - c.x).powi(2) + (p.y - c.y).powi(2) <= r.powi(2);
    }

    fn triangle_area(&self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> f64 {
        return (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)).abs() * 0.5;
    }

    fn inside_box(&self, pbox: &impl PointBox, p: &Point) -> bool {
        let (nw, ne, sw, se) = pbox.full_box();
        let box_area = (self.triangle_area(ne.x, ne.y, nw.x, nw.y, se.x, se.y)
            + self.triangle_area(ne.x, ne.y, nw.x, nw.y, sw.x, sw.y))
            * TRIANGLE_MARGINE_FOR_ERROR;

        let mut triangle_sum = 0.0;
        let order = [&ne, &nw, &sw, &se, &ne];
        for id in 0..order.len() - 1 {
            let left = order[id];
            let right = order[id + 1];
            triangle_sum += self.triangle_area(p.x, p.y, left.x, left.y, right.x, right.y);
            if triangle_sum > box_area {
                return false;
            };
        }
        return true;
    }

    fn get_distance(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        return self.get_distance_square(x1, y1, x2, y2).sqrt();
    }

    fn get_distance_square(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        return (x1 - x2).powi(2) + (y1 - y2).powi(2);
    }

    /// Takes a given point on the screen and coverts it to point on the map.
    fn to_map_xy(&self, p: &Point, t: &Transform) -> Point {
        let px = p.x - t.x;
        let py = p.y - t.y;
        let x = px / t.k;
        let y = py / t.k;
        return Point { x, y };
    }

    /// Takes a point from the map and converts it to a point for the screen.
    fn to_screen_xy(&self, p: &Point, t: &Transform) -> Point {
        let px = p.x + t.x;
        let py = p.y + t.y;
        let x = px * t.k;
        let y = py * t.k;
        return Point { x, y };
    }

    fn center_transform(obj: &impl GetCenter, width: f64, height: f64, t: &Transform) -> Transform {
        let n = obj.get_center();

        let k = t.k;
        let x = width * 0.5 - n.x * k;
        let y = height * 0.5 - n.y * k;
        return Transform { x, y, k };
    }
}
