use crate::{
    Move, Point, Transform,
    bsp::PointLookupResult,
    link::{Bundle, Link},
    node::Node,
    renderer::Render,
};
use gloo::timers::callback::Timeout;

pub enum CurrentTarget {
    Node((Move, Node)),
    Box((Move, Node)),
    Screen(Move),
    Link((Move, Link)),
    Bundle((Move, Bundle)),
    NoTarget,
}

pub struct Stater {
    pub render: *mut Render,
    pub target: CurrentTarget,
    pub t: Transform,
    pub wheel_move: f64,
    timeout_value: u32,
    current_timeout: Option<Timeout>,
}

impl Stater {
    pub fn zoom_in(&mut self) {
        self.t.k -= self.wheel_move;
        self.render();
    }
    pub fn zoom_out(&mut self) {
        self.t.k += self.wheel_move;
        self.render();
    }

    fn point_state(&self, p: &Point) -> CurrentTarget {
        let mut m = Move::new(p, self.t);
        let res;
        let calc;
        unsafe {
            calc = (*self.render).calc();
            res = (*calc).in_point(p, &self.t);
        }

        match res {
            None => CurrentTarget::Screen(m),
            Some(f) => match f {
                PointLookupResult::Bundle(b) => {
                    m.center(&unsafe { (*calc).get_src_dst_center(b.src, b.dst) });
                    CurrentTarget::Bundle((m, b))
                }
                PointLookupResult::Link(l) => {
                    m.center(&unsafe { (*calc).get_src_dst_center(l.src, l.dst) });
                    CurrentTarget::Link((m, l))
                }
                PointLookupResult::Node(n) => {
                    m.center(&n.get_center());
                    CurrentTarget::Node((m, n))
                }
                PointLookupResult::Box(n) => {
                    m.center(&n.get_center());
                    CurrentTarget::Node((m, n))
                }
            },
        }
    }
    pub fn mouse_over(&self, p: &Point) {
        let target = self.point_state(p);
        match target {
            CurrentTarget::Screen(_) | CurrentTarget::NoTarget => return,
            _ => (),
        };
        unsafe { (*self.render).mouse_over(&target) };
    }

    pub fn mouse_up(&mut self, p: &Point) {
        self.mouse_move(p);
        unsafe { (*self.render).mouse_up(&self.target) };
        self.target = CurrentTarget::NoTarget;
    }

    pub fn mouse_down(&mut self, p: &Point) {
        self.target = self.point_state(p);
        unsafe { (*self.render).mouse_down(&self.target) };
    }

    pub fn mouse_leave(&mut self, _p: &Point) {
        self.current_timeout = None;
        self.target = CurrentTarget::NoTarget;
    }

    pub fn render_highlight(&mut self, p: &Point) {}
    pub fn mouse_move(&mut self, p: &Point) {
        unsafe {
            let calc = (*self.render).calc();

            self.current_timeout = None;
            match &mut self.target {
                CurrentTarget::NoTarget => {
                    let this = self as *mut Stater;
                    let p = *p;
                    self.current_timeout = Some(Timeout::new(self.timeout_value, move || {
                        #[allow(unused_unsafe)]
                        unsafe {
                            (*this).render_highlight(&p)
                        }
                    }));
                    return;
                }
                CurrentTarget::Bundle((m, b)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[b.src, b.dst], &m.stop(p), false)
                }
                CurrentTarget::Link((m, l)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[l.src, l.dst], &m.stop(p), false)
                }
                CurrentTarget::Node((m, n)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[n.id], &m.stop(p), true)
                }
                CurrentTarget::Box((m, n)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[n.id], &m.stop(p), true)
                }
                CurrentTarget::Screen(m) => {
                    m.transform = self.t;
                    m.stop(p);
                    self.t.x = m.start.x;
                    self.t.y = m.start.y;
                }
            };
        };

        self.render();
    }

    pub fn render(&self) {
        unsafe {
            (*self.render).render();
        }
    }
    pub fn new(render: *mut Render, timeout_value: u32) -> Self {
        Self {
            render,
            wheel_move: 0.05,
            target: CurrentTarget::NoTarget,
            current_timeout: None,
            timeout_value,
            t: Transform {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            },
        }
    }
}
