pub mod img_loader;
pub mod targets;
use std::{mem, process};

use crate::{
    ImgSrc, Move, Point, RenderBox, ScreenBox, Transform,
    bsp::PointLookupResult,
    calc::{Calculator, MouseEvent, MouseImpacted, Options},
    constants::{
        DEFAULT_ANIMATION_DASHES, DEFAULT_CANVAS_STYLE, DEFAULT_DIV_STYLE, DEFAULT_FONT_FAMILY,
        DEFAULT_HIGHLIGHT_ALPHA, DEFAULT_HIGHLIGHT_COLOR, DEFAULT_HIGHLIGHT_SCALE,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_SCREEN_ZOOM, DEFAULT_TEXT_ALIGN, ZERO_TRANSFORM,
    },
    link::{Bundle, Link, LinkContainer},
    node::{Node, NodeOpt},
    renderer::{
        img_loader::{ImgCache, ImgLookupState},
        targets::Targets,
    },
};
use gloo::timers::callback::Timeout;
use js_sys::Array;
use js_sys::Function;
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
    ct: CurrentTarget,
    current_timeout: Option<Timeout>,
    ops: RenderOpt,
    width: u32,
    height: u32,
    frame_tick: f64,
    animation_order: Vec<u64>,
}

#[wasm_bindgen(inspectable, getter_with_clone)]
#[derive(Clone, Debug)]
pub struct RenderOpt {
    pub wheel_move: f64,
    pub highlight_scale: f64,
    pub timeout: u32,
    pub font_family: String,
    pub text_align: String,
    pub animation_dashes: Vec<f64>,
    pub highlight_alpha: f64,
    pub highlight_color: String,
    pub bulk_img_update: bool,
    pub callback: Option<Function>,
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
            animation_dashes: Vec::from(DEFAULT_ANIMATION_DASHES),
            highlight_alpha: DEFAULT_HIGHLIGHT_ALPHA,
            highlight_color: String::from(DEFAULT_HIGHLIGHT_COLOR),
            bulk_img_update: true,
            callback: None,
        }
    }
}
impl RenderOpt {
    pub fn animation_dashes(&self) -> Array {
        let res = Array::new_with_length(self.animation_dashes.len() as u32);
        for p in &self.animation_dashes {
            res.push(&JsValue::from_f64(*p));
        }
        return res;
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
render_opt!(highlight_alpha, f64);
render_opt!(highlight_scale, f64);
render_opt!(highlight_color, String);
render_opt!(font_family, String);
render_opt!(text_align, String);
render_opt!(timeout, u32);
render_opt!(animation_dashes, Vec<f64>);
render_opt!(bulk_img_update, bool);
render_opt!(callback, Option<Function>);

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(constructor)]
    pub fn new(calc: &mut Calculator, ops: RenderOpt, width: u32, height: u32, id: String) -> Self {
        Self {
            width,
            height,
            id,
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
        animations.set_global_alpha(self.ops.highlight_alpha);
        let ns = unsafe { (*self.calc).nodes() };
        let co = unsafe { (*self.calc).options_mut() };
        let ls = unsafe { (*self.calc).links() };
        let will_animate = unsafe { (*self.calc).animations() };

        for (node_list, link_list, box_list, _) in iter {
            for id in node_list {
                match ns.get(id) {
                    Some(node) => {
                        let opt = co.get_node(&node.opt);
                        self.draw_node(&nodes, node, opt, false);
                    }
                    _ => (),
                }
            }

            for id in box_list {
                match ns.get_box(id) {
                    Some(node) => {
                        let opt = co.get_node(&node.opt);
                        self.draw_node(&boxes, node, opt, false);
                    }
                    _ => (),
                }
            }

            let dashes = self.ops.animation_dashes();
            for id in link_list {
                match ls.get(&id) {
                    Some(lc) => {
                        self.draw_lc(&links, lc, co, false);
                        if will_animate.contains(&lc.id) {
                            self.draw_animation(&animations, lc, co, &dashes);
                            self.animation_order.push(lc.id);
                        }
                    }
                    _ => (),
                }
            }
        }
        Ok(true)
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
        match self.cache.as_mut() {
            Some(c) => return unsafe { mem::transmute(c) },
            None => (),
        }
        self.build_cache()
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
        match &self.ct {
            CurrentTarget::NoTarget => return,
            _ => {
                if let Some(res) = self.mouse_move(p) {
                    self.hanlde_mouse_event(res, p);
                }
            }
        }
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

    pub fn mouse_leave(&mut self, p: &Point) {
        self.current_timeout = None;
        match &self.ct {
            CurrentTarget::NoTarget => return,
            _ => {
                if let Some(res) = self.mouse_move(p) {
                    self.hanlde_mouse_event(res, p);
                }
            }
        }
        self.ct = CurrentTarget::NoTarget;
    }

    pub fn mouse_enter(&mut self, p: &Point) {
        self.build_timeout(p);
    }

    pub fn mouse_move(&mut self, p: &Point) -> Option<MouseEvent> {
        let calc = self.calc;
        self.current_timeout = None;
        let res;
        unsafe {
            match &mut self.ct {
                CurrentTarget::NoTarget => {
                    self.build_timeout(p);
                    return None;
                }
                CurrentTarget::Bundle((m, b)) => {
                    m.transform = self.t;
                    res = Some(MouseEvent::Moved((*calc).move_nodes(
                        &[b.src, b.dst],
                        &m.stop(p),
                        false,
                    )));
                }
                CurrentTarget::Link((m, l)) => {
                    m.transform = self.t;
                    res = Some(MouseEvent::Moved((*calc).move_nodes(
                        &[l.src, l.dst],
                        &m.stop(p),
                        false,
                    )));
                }
                CurrentTarget::Node((m, n)) => {
                    m.transform = self.t;
                    res = Some(MouseEvent::Moved((*calc).move_nodes(
                        &[n.id],
                        &m.stop(p),
                        true,
                    )));
                }
                CurrentTarget::Box((m, n)) => {
                    m.transform = self.t;
                    res = Some(MouseEvent::Moved((*calc).move_nodes(
                        &[n.id],
                        &m.stop(p),
                        true,
                    )));
                }
                CurrentTarget::Screen(m) => {
                    m.transform = self.t;
                    m.stop(p);
                    self.t.x = m.start.x;
                    self.t.y = m.start.y;
                    res = Some(MouseEvent::CanvasMove(self.t));
                }
            };
        };

        self.rndr();
        res
    }
    fn rndr(&mut self) {
        let _ = self.render();
    }

    fn hanlde_mouse_event(&self, i: MouseEvent, p: &Point) {
        if let Some(cb) = &self.ops.callback {
            let this = JsValue::null();
            let v = JsValue::from(i);
            let jsp = JsValue::from(*p);
            match cb.call2(&this, &v, &jsp) {
                Err(_) => process::abort(),
                Ok(_) => {}
            }
        }
    }

    fn get_node<'n>(&self, id: u32) -> &'n Node {
        unsafe {
            match (*self.calc).nodes().get(id) {
                Some(node) => node,
                None => process::abort(),
            }
        }
    }

    fn get_link_container<'lc>(&self, id: u64) -> &'lc LinkContainer {
        match unsafe { (*self.calc).links().get(&id) } {
            Some(lc) => lc,
            None => process::abort(),
        }
    }

    pub fn render_highlight(&mut self, p: &Point) {
        let res = unsafe { (*self.calc).in_point(p, &self.t) };
        let calc = self.calc;
        let highlight;
        if let Some(t) = &self.targets {
            highlight = t.get_highlight_target();
        } else {
            return;
        }
        if let Some(lookup) = res {
            let mut nodes = Vec::new();
            let mut boxes = Vec::new();
            let mut bundles = Vec::new();
            let mut links = Vec::new();
            let ops = unsafe { (*calc).options_mut() };
            match lookup {
                PointLookupResult::Box(node) => {
                    self.draw_node(
                        &highlight,
                        &node,
                        unsafe { (*calc).get_node(&node.opt) },
                        true,
                    );
                    boxes.push(node.id);
                }
                PointLookupResult::Node(node) => {
                    self.draw_node(
                        &highlight,
                        &node,
                        unsafe { (*calc).get_node(&node.opt) },
                        true,
                    );
                    nodes.push(node.id);
                }
                PointLookupResult::Link(l) => {
                    let lc = self.get_link_container(l.link_id());
                    nodes = Vec::from([l.src, l.dst]);
                    links.push(l.id);

                    let details;
                    match lc.get_link_render(l.id) {
                        Some(d) => details = d,
                        None => process::abort(),
                    }
                    let width = self.ops.highlight_scale * details.2;
                    self.draw_line(
                        &highlight,
                        &details.0,
                        &details.1,
                        width,
                        &self.ops.highlight_color,
                    );

                    self.draw_highlight_nodes(l.src, l.dst, &highlight);
                }
                PointLookupResult::Bundle(b) => {
                    nodes = Vec::from([b.src, b.dst]);
                    bundles.push(b.id);
                    let lc = self.get_link_container(b.link_id());
                    for lid in &b.links {
                        if let Some(l) = lc.get_link_render(*lid) {
                            let width = self.ops.highlight_scale * l.2;
                            links.push(*lid);

                            self.draw_line(
                                &highlight,
                                &l.0,
                                &l.1,
                                width,
                                &self.ops.highlight_color,
                            );
                        }
                    }
                    let rb;
                    match lc.get_bundle_box(b.id) {
                        Some(bb) => rb = bb,
                        None => process::abort(),
                    }
                    self.draw_box(&highlight, &rb, ops.get_bundle(&b.opt), true);
                    self.draw_highlight_nodes(b.src, b.dst, &highlight);
                }
            }
            self.hanlde_mouse_event(
                MouseEvent::MouseOver(MouseImpacted {
                    nodes,
                    links,
                    boxes,
                    bundles,
                }),
                p,
            );
        }
    }
    fn draw_highlight_nodes(&mut self, a: u32, b: u32, ctx: &CanvasRenderingContext2d) {
        let src = self.get_node(a);
        let dst = self.get_node(b);
        let calc = self.calc;
        self.draw_box(ctx, src, unsafe { (*calc).get_node(&src.opt) }, true);
        self.draw_box(ctx, dst, unsafe { (*calc).get_node(&dst.opt) }, true);
    }
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
            match (*s).cache.as_mut() {
                Some(c) => c,
                None => process::abort(),
            }
        }
    }

    pub fn img_resolved(&mut self, _: &String, _: ImgLookupState, loading: u32) {
        if self.ops.bulk_img_update {
            if loading == 0 {
                let _ = self.render();
            }
        } else {
            let _ = self.render();
        }
    }

    pub fn draw_box(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        target: &impl RenderBox,
        opt: &impl ImgSrc,
        highlight: bool,
    ) {
        if highlight {
            ctx.set_fill_style_str(&self.ops.highlight_color);
            let scale = self.ops.highlight_scale;
            let x = target.y() - (target.x() * scale - target.x()) * 0.5;
            let y = target.y() - (target.y() * scale - target.y()) * 0.5;
            let w = target.width() * scale;
            let h = target.height() * scale;
            ctx.clear_rect(x, y, w, h);
            ctx.fill_rect(x, y, target.width() * scale, target.height() * scale);
            return;
        }
        let src = opt.img_src();
        match self.cache().load_img(&src) {
            ImgLookupState::Loaded(img) => {
                let _ = ctx.draw_image_with_html_image_element(&img, target.x(), target.y());
            }
            _ => ctx.fill_rect(target.x(), target.y(), target.width(), target.height()),
        }
    }
    fn draw_node(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        node: &Node,
        opt: &NodeOpt,
        highlight: bool,
    ) {
        self.draw_box(ctx, node, opt, highlight);
    }
    fn draw_line(
        &self,
        ctx: &CanvasRenderingContext2d,
        src: &Point,
        dst: &Point,
        width: f64,
        color: &String,
    ) {
        ctx.begin_path();
        ctx.set_line_width(width);
        ctx.set_stroke_style_str(&color);
        ctx.move_to(src.x, src.y);
        ctx.line_to(dst.x, dst.y);
        ctx.close_path();
        ctx.stroke();
    }
    fn draw_lc(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        link: &LinkContainer,
        opts: &mut Options,
        highlight: bool,
    ) {
        let ls;
        if let Some(l) = &link.link_src {
            ls = l
        } else {
            return;
        }
        let w;
        if highlight {
            w = ls.cl.width * self.ops.highlight_scale
        } else {
            w = ls.cl.width
        }
        for cl in &ls.cl.links {
            let o = opts.get_link(&cl.opt);
            self.draw_line(ctx, &cl.src, &cl.dst, w, &o.color);
        }
    }
    fn draw_animation(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        link: &LinkContainer,
        opts: &mut Options,
        dashes: &JsValue,
    ) {
        let ls;
        if let Some(l) = &link.link_src {
            ls = l
        } else {
            return;
        }

        for a in &ls.cl.animations {
            let o = opts.get_link(&a.opt);

            ctx.begin_path();
            ctx.set_line_dash_offset(self.frame_tick);
            ctx.set_line_width(a.width);
            ctx.set_stroke_style_str(&o.animation_color);
            let _ = ctx.set_line_dash(dashes);
            ctx.move_to(a.src.x, a.src.y);
            ctx.line_to(a.dst.x, a.dst.y);
            ctx.close_path();
            ctx.stroke();
        }
    }
}
