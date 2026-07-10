use crate::{
    Move, Point, Transform,
    bsp::PointLookupResult,
    link::{Bundle, Link},
    node::Node,
    renderer::Render,
};

pub enum CurrentTarget {
    Node((Move, Node)),
    Screen(Move),
    Link((Move, Link)),
    Bundle((Move, Bundle)),
    NoTarget,
}

pub struct Stater {
    render: *mut Render,
    pub target: CurrentTarget,
    pub t: Transform,
    pub wheel_move: f64,
}

impl Stater {
    pub fn wheel_up(&mut self) {
        self.t.k -= self.wheel_move;
        self.render();
    }
    pub fn wheel_down(&mut self) {
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
        self.target = CurrentTarget::NoTarget;
        unsafe { (*self.render).mouse_up() };
    }

    pub fn mouse_down(&mut self, p: &Point) {
        self.target = self.point_state(p);

        unsafe { (*self.render).mouse_down(&self.target) };
    }
    pub fn mouse_move(&mut self, p: &Point) {
        unsafe {
            let calc = (*self.render).calc();

            match &mut self.target {
                CurrentTarget::NoTarget => return,
                CurrentTarget::Bundle((m, b)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[b.src, b.dst], &m.stop(p))
                }
                CurrentTarget::Link((m, l)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[l.src, l.dst], &m.stop(p))
                }
                CurrentTarget::Node((m, n)) => {
                    m.transform = self.t;
                    (*calc).move_nodes(&[n.id], &m.stop(p))
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
    pub fn new(render: *mut Render) -> Self {
        Self {
            render,
            wheel_move: 0.05,
            target: CurrentTarget::NoTarget,
            t: Transform {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            },
        }
    }
}
