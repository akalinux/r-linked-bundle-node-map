use std::collections::{HashMap, HashSet};
pub mod img_loader;
pub mod targets;
use crate::{
    Move, Point, ScreenBox, Transform,
    bsp::PointLookupResult,
    calc::{Calculator, Options},
    constants::{
        DEFAULT_CANVAS_STYLE, DEFAULT_DIV_STYLE, DEFAULT_FONT_FAMILY, DEFAULT_HIGHLIGHT_SCALE,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_SCREEN_ZOOM, DEFAULT_TEXT_ALIGN, ZERO_TRANSFORM,
    },
    link::{Bundle, Link, LinkContainer, LinkOpt},
    node::{Node, NodeOpt},
    renderer::{
        img_loader::{ImgCache, ImgLookupState},
        targets::Targets,
    },
};
use gloo::timers::callback::Timeout;
use pastey::paste;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::CanvasRenderingContext2d;

pub enum CurrentTarget {
    Node((Move, Node)),
    Box((Move, Node)),
    Screen(Move),
    Link((Move, Link)),
    Bundle((Move, Bundle)),
    NoTarget,
}

#[wasm_bindgen]
pub struct Render {
    calc: *mut Calculator,
    t: Transform,
    cache: Option<ImgCache>,
    targets: Option<Targets>,
    id: String,
    img_pending: HashMap<String, HashSet<u32>>,
    ct: CurrentTarget,
    current_timeout: Option<Timeout>,
    ops: RenderOpt,
    width: u32,
    height: u32,
    frame_tick: f64,
    animation_order: Vec<u64>,
}

#[wasm_bindgen(inspectable)]
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone, Debug)]
pub struct RenderOpt {
    pub wheel_move: f64,
    pub highlight_scale: f64,
    pub timeout: u32,
    pub font_family: String,
    pub text_align: String,
}

#[wasm_bindgen]
impl RenderOpt {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            wheel_move: DEFAULT_SCREEN_ZOOM,
            highlight_scale: DEFAULT_HIGHLIGHT_SCALE,
            timeout: DEFAULT_HOVER_TIMEOUT,
            font_family: String::from(DEFAULT_FONT_FAMILY),
            text_align: String::from(DEFAULT_TEXT_ALIGN),
        }
    }
}
macro_rules! render_opt {
    ($field:ident,$ty:ty) => {
        paste! {

            #[wasm_bindgen]
            impl RenderOpt {
                pub fn [<set_ $field>](&mut self,v: $ty) {
                    self.$field=v;
                }

                pub fn [<get_ $field>](&mut self) ->$ty{
                    self.$field.clone()
                }
            }
        }
    };
}
render_opt!(wheel_move, f64);
render_opt!(highlight_scale, f64);
render_opt!(font_family, String);
render_opt!(text_align, String);
render_opt!(timeout, u32);

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(constructor)]
    pub fn new(calc: &mut Calculator, ops: RenderOpt, width: u32, height: u32, id: String) -> Self {
        Self {
            width,
            height,
            id,
            img_pending: HashMap::new(),
            calc: calc as *mut Calculator,
            cache: None,
            targets: None,
            t: ZERO_TRANSFORM,
            ct: CurrentTarget::NoTarget,
            current_timeout: None,
            ops,
            frame_tick: 0.0,
            animation_order: Vec::new(),
        }
    }
    pub fn set_render_options(&mut self, ops: RenderOpt) {
        self.ops = ops
    }
    pub fn get_render_options(&mut self) -> RenderOpt {
        self.ops.clone()
    }

    pub fn render(&mut self) -> Result<bool, JsValue> {
        let targets;
        match &self.targets {
            Some(t) => targets = t,
            None => return Err(JsValue::from_str("Not currently mounted!")),
        };
        self.animation_order.clear();
        let t = &self.t;
        let iter = unsafe { (*self.calc).indexer().on_screen(self.width, self.height, t) };
        let (boxes, links, animations, nodes) = targets.get_render_targets();
        let x = self.t.x;
        let y = self.t.y;
        let k = self.t.k;
        for c in [&boxes, &links, &animations, &nodes] {
            c.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
            c.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
            c.set_transform(k, 0.0, 0.0, k, x, y)?;
        }
        let ns = unsafe { (*self.calc).nodes() };
        let co = unsafe { (*self.calc).options_mut() };
        let ls = unsafe { (*self.calc).links() };
        let will_animate = unsafe { (*self.calc).animations() };

        animations.set_line_dash_offset(self.frame_tick);

        for (node_list, link_list, box_list, _) in iter {
            for id in node_list {
                let node = ns.get(id).unwrap();
                let opt = co.get_node(&node.opt);
                self.draw_node(&nodes, node, opt, false);
            }
            for id in box_list {
                let node = ns.get_box(id).unwrap();
                let opt = co.get_node(&node.opt);
                self.draw_node(&boxes, node, opt, false);
            }
            for id in link_list {
                let lc = ls.get(&id).unwrap();

                self.draw_link(&links, lc, co, false);
                if will_animate.contains(&lc.id) {
                    self.draw_animation(&animations, lc, co);
                    self.animation_order.push(lc.id);
                }
            }
        }
        Ok(true)
    }

    fn draw_node(
        &self,
        ctx: &CanvasRenderingContext2d,
        node: &Node,
        opt: &NodeOpt,
        highlight: bool,
    ) {
    }
    fn draw_link(
        &self,
        ctx: &CanvasRenderingContext2d,
        link: &LinkContainer,
        opts: &mut Options,
        highlight: bool,
    ) {
    }
    fn draw_animation(
        &self,
        ctx: &CanvasRenderingContext2d,
        link: &LinkContainer,
        opts: &mut Options,
    ) {
    }
    pub fn mount(&mut self, element_id: String) -> Result<(), JsValue> {
        self.mount_with_options(
            element_id,
            String::from(DEFAULT_DIV_STYLE),
            String::from(DEFAULT_CANVAS_STYLE),
        )
    }
    pub fn mount_with_options(
        &mut self,
        element_id: String,
        div_style: String,
        canvas_style: String,
    ) -> Result<(), JsValue> {
        self.targets = None;
        let targets = Targets::new(self as *mut Self, element_id, div_style, canvas_style)?;
        self.targets = Some(targets);
        Ok(())
    }
    pub fn unmount(&mut self) {
        self.targets = None;
        self.ct = CurrentTarget::NoTarget;
        self.current_timeout = None;
    }
}
impl Drop for Render {
    fn drop(&mut self) {
        self.unmount();
    }
}
impl Render {
    pub fn get_id(&self) -> &String {
        &self.id
    }

    pub fn cache<'c>(&'c mut self) -> &'c mut ImgCache {
        if self.cache.is_none() {
            return self.build_cache();
        }
        return self.cache.as_mut().unwrap();
    }

    pub fn canvas_box(&self) -> ScreenBox {
        ScreenBox {
            width: self.width,
            height: self.height,
            x: 0,
            y: 0,
            step: unsafe { (*self.calc).indexer().step },
        }
    }

    fn point_state(&self, p: &Point) -> CurrentTarget {
        let mut m = Move::new(p, self.t);
        let res;
        let calc = self.calc;
        unsafe {
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

    pub fn mouse_up(&mut self, p: &Point) {
        self.mouse_move(p);
        self.ct = CurrentTarget::NoTarget;
    }
    pub fn mouse_down(&mut self, p: &Point) {
        self.current_timeout = None;
        self.ct = self.point_state(p);
    }

    pub fn build_timeout(&mut self, p: &Point) {
        let this = self as *mut Self;
        let p = *p;
        let cb = move || unsafe { (*this).render_highlight(&p) };
        self.current_timeout = Some(Timeout::new(self.ops.timeout, cb));
    }

    pub fn mouse_leave(&mut self, _p: &Point) {
        self.current_timeout = None;
        self.ct = CurrentTarget::NoTarget;
    }

    pub fn mouse_enter(&mut self, p: &Point) {
        self.build_timeout(p);
    }

    pub fn mouse_move(&mut self, p: &Point) {
        let calc = self.calc;
        self.current_timeout = None;
        unsafe {
            match &mut self.ct {
                CurrentTarget::NoTarget => {
                    self.build_timeout(p);
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

        self.rndr();
    }
    fn rndr(&mut self) {
        match self.render() {
            _ => (),
        }
    }
    pub fn render_highlight(&mut self, p: &Point) {}
    pub fn zoom_in(&mut self) {
        self.t.k += self.ops.wheel_move;
        self.rndr();
    }
    pub fn zoom_out(&mut self) {
        self.t.k -= self.ops.wheel_move;
        if self.t.k <= 0.0 {
            self.t.k = self.ops.wheel_move
        }
        self.rndr();
    }

    fn get_step(&self) -> i64 {
        unsafe { (*self.calc).indexer().step }
    }
    fn build_cache<'c>(&mut self) -> &'c mut ImgCache {
        unsafe {
            let s = self as *mut Self;
            let cache = ImgCache::new(s);
            (*s).cache = Some(cache);
            return (*s).cache.as_mut().unwrap();
        }
    }

    pub fn img_resolved(&mut self, src: &String, state: ImgLookupState, loading: u32) {}
}
