use wasm_bindgen::prelude::*;

use crate::{
    bsp::IndexXY,
    constants::{RAD2DEG, SCREEN_EPSILON, TRIANGLE_MARGINE_FOR_ERROR, ZERO_POINT},
};
pub mod bsp;
pub mod calc;
pub mod constants;
pub mod link;
pub mod node;

#[wasm_bindgen(inspectable)]
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub struct ScreenBox {
    pub width: u32,
    pub height: u32,
    pub x: i64,
    pub y: i64,
    pub step: i64,
}

impl CalculatorTrait for ScreenBox {}
#[wasm_bindgen]
impl ScreenBox {
    pub fn from_step(x: i64, y: i64, step: i64) -> Self {
        Self {
            width: step as u32,
            height: step as u32,
            x,
            y,
            step,
        }
    }
    pub fn transform(&self, t: &Transform) -> Self {
        let p = self.to_map_xy(
            &Point {
                x: self.x as f64,
                y: self.y as f64,
            },
            t,
        );
        let x = p.x as i64;
        let y = p.y as i64;
        let width = (self.width as f64 / t.k) as u32;
        let height = (self.height as f64 / t.k) as u32;

        Self {
            width,
            height,
            x,
            y,
            step: self.step,
        }
    }
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            step: 0,
        }
    }
    pub fn new(t: &Transform, mut width: u32, mut height: u32, step: i64) -> Self {
        width = (width as f64 / t.k) as u32;
        height = (height as f64 / t.k) as u32;
        let mut m = width % step as u32;
        if m == 0 {
            width -= step as u32;
        } else {
            width -= m;
        }
        m = height % step as u32;
        if m == 0 {
            height -= step as u32;
        } else {
            height -= m;
        }
        let p = ZERO_POINT.to_map_xy(&ZERO_POINT, t);
        let mut x = p.x as i64;
        let mut y = p.y as i64;
        for i in [&mut x, &mut y] {
            let m = *i % step;
            if m < 0 {
                *i -= step + m;
            } else {
                *i -= m;
            }
        }
        return Self {
            width: width + step as u32,
            height: height + step as u32,
            x,
            y,
            step,
        };
    }

    pub fn scale(&self) -> f64 {
        return (self.width as f64 / self.step as f64 + self.height as f64 / self.step as f64)
            as f64
            / 2.0;
    }

    pub fn get_center(&self) -> Point {
        let x = self.x as f64 + (self.width as f64 * 0.5);
        let y = self.y as f64 + (self.height as f64 * 0.5);
        return Point { x, y };
    }

    /// Returns a [Point] that represents where to place the screen [ScreenBox] so that it will be centered inside of self.
    pub fn center(&self, screen: &ScreenBox) -> Point {
        let inlay_point = screen.get_center();
        let host_point = self.get_center();
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        for (inlay, host, host_size, t) in [
            (inlay_point.x, host_point.x, self.width as f64 * 0.5, &mut x),
            (
                inlay_point.y,
                host_point.y,
                self.height as f64 * 0.5,
                &mut y,
            ),
        ] {
            if inlay.abs() < SCREEN_EPSILON {
                if host.abs() < SCREEN_EPSILON {
                    *t = host_size;
                } else {
                    *t = host + host_size;
                }
            } else if host.abs() < SCREEN_EPSILON {
                let distance = host - inlay;
                *t = distance + inlay + host_size;
            } else {
                let scale = host / inlay;
                *t = inlay * scale;
            }
        }

        return Point { x, y };
    }

    pub fn contains_x(&self, x: i64) -> bool {
        return !(self.x > x || self.bound_x() < x);
    }

    pub fn contains_y(&self, y: i64) -> bool {
        return !(self.y > y || self.bound_y() < y);
    }

    pub fn max_x(&self) -> i64 {
        return self.x + self.width as i64;
    }

    pub fn max_y(&self) -> i64 {
        return self.y + self.height as i64;
    }

    pub fn bound_x(&self) -> i64 {
        return self.x + self.width as i64 - self.step;
    }

    pub fn bound_y(&self) -> i64 {
        return self.y + self.height as i64 - self.step;
    }

    pub fn contains(&self, b: &ScreenBox) -> Option<ScreenBox> {
        if (self.contains_x(b.x) || self.contains_x(b.bound_y()))
            && (self.contains_y(b.y) || self.contains_y(b.bound_y()))
        {
            let width;
            let x;
            let y;
            let height;
            if b.x < self.x {
                x = self.x;
            } else {
                x = b.x;
            }
            let mut end = b.max_x();
            let mut max = self.max_x();
            if end > max {
                width = (max - x) as u32;
            } else {
                width = (end - x) as u32;
            }

            if b.y < self.x {
                y = self.y;
            } else {
                y = b.y;
            }
            end = b.max_y();
            max = self.max_y();
            if end > max {
                height = (max - y) as u32;
            } else {
                height = (end - y) as u32;
            }

            return Some(ScreenBox {
                width,
                height,
                x,
                y,
                step: self.step,
            });
        }
        return None;
    }
}

impl ScreenBox {
    pub fn getxy_bounds(&self) -> IndexXY {
        return (self.x..=self.bound_x(), self.y..=self.bound_y());
    }
}

/// Map Movement transformation struct.
#[wasm_bindgen(inspectable)]
pub struct Move {
    /// The current map transformed point.
    pub start: Point,
    /// The current map transformation.
    pub transform: Transform,
}

#[wasm_bindgen]
impl Move {
    #[wasm_bindgen(constructor)]
    /// Creates a new [create::Move] instance, the [Point] argument converted to the map cordinates with the [Transform].
    pub fn new(p: &Point, t: Transform) -> Self {
        let res = Self {
            start: p.to_map_xy(p, &t),
            transform: t,
        };

        return res;
    }
    /// Returns a new [Point] representing the difference to the x and y value since the last call to [Move::stop] or object instantiation.
    pub fn stop(&mut self, p: &Point) -> Point {
        let n = p.to_map_xy(p, &self.transform);
        let diff = Point {
            x: n.x - self.start.x,
            y: n.y - self.start.y,
        };
        self.start = n;

        return diff;
    }
}

impl CalculatorTrait for Move {}
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
    pub fn move_to(&mut self, p: &Point) {
        self.x += p.x;
        self.y += p.y;
    }

    pub fn compute_center(&self, p: &Point) -> Point {
        return Point {
            x: (self.x + p.x) * 0.5,
            y: (self.y + p.y) * 0.5,
        };
    }
    pub fn to_index_point(&self, step: i64) -> (i64, i64) {
        let mut x = self.x.floor() as i64;
        let mut y = self.y.floor() as i64;
        for i in [&mut x, &mut y] {
            let m = *i % step;
            if m < 0 {
                *i -= step + m;
            } else {
                *i -= m;
            }
        }
        return (x, y);
    }
}
impl CalculatorTrait for Point {}

impl GetCenter for Point {
    fn get_center(&self) -> Point {
        return *self;
    }
}

pub trait GetCenter {
    fn get_center(&self) -> Point;
}

pub trait FullBox {
    fn full_box(&self) -> (Point, Point, Point, Point);
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

    fn index_bound(&self, boundry: i64) -> IndexXY {
        let mut min_x = self.get_min_x().floor() as i64;
        let mut min_y = self.get_min_y().floor() as i64;
        let mut max_x = self.get_max_x().ceil() as i64;
        let mut max_y = self.get_max_y().ceil() as i64;
        for i in [&mut min_x, &mut min_y, &mut max_x, &mut max_y] {
            let m = *i % boundry;
            if m < 0 {
                *i -= boundry + m;
            } else {
                *i -= m;
            }
        }
        return (min_x..=max_x, min_y..=max_y);
    }

    fn getx_index_bound(&self, boundry: i64) -> i64 {
        let min_x = self.get_min_x() as i64;
        return min_x - (min_x % boundry);
    }
    fn gety_index_bound(&self, boundry: i64) -> i64 {
        let min_y = self.get_min_y() as i64;
        return min_y - (min_y % boundry);
    }
}

pub trait CalculatorTrait {
    fn compute_line_box(&self, ne: &Point, points: [&Point; 3]) -> (f64, f64, f64, f64) {
        let mut min_x = ne.x;
        let mut max_x = ne.x;
        let mut min_y = ne.y;
        let mut max_y = ne.y;
        for p in points {
            if max_x < p.x {
                max_x = p.x;
            }
            if max_y < p.y {
                max_y = p.y;
            }
            if min_x > p.x {
                min_x = p.x;
            }
            if min_y > p.y {
                min_y = p.y;
            }
        }
        return (min_x, max_x, min_y, max_y);
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
        let rad = degree.to_radians();

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

    fn inside_circle(&self, p: &Point, c: &Point, r: f64) -> bool {
        return (p.x - c.x).powi(2) + (p.y - c.y).powi(2) <= r.powi(2);
    }

    fn triangle_area(&self, x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64) -> f64 {
        return (x1 * (y2 - y3) + x2 * (y3 - y1) + x3 * (y1 - y2)).abs() * 0.5;
    }

    fn inside_box(&self, pbox: &impl FullBox, p: &Point) -> bool {
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

macro_rules! id_compare {
    ($($t:ident),*) => {
        $(
            impl PartialEq for $t {
                fn eq(&self, other: &Self) -> bool {
                    self.id == other.id
                }
            }
        )*
    };
}

pub(crate) use id_compare;
