pub mod img_loader;
pub mod targets;
use std::mem;

use crate::{
    ImgSrc, Move, Point, PointBox, RenderBox, ScreenBox, Transform,
    bsp::PointLookupResult,
    calc::{Calculator, MouseEvent, MouseImpacted, Options},
    constants::{
        DEFAULT_ANIMATION_DASHES, DEFAULT_CANVAS_STYLE, DEFAULT_DIV_STYLE, DEFAULT_FONT_FAMILY,
        DEFAULT_HIGHLIGHT_ALPHA, DEFAULT_HIGHLIGHT_COLOR, DEFAULT_HIGHLIGHT_SCALE,
        DEFAULT_HOVER_TIMEOUT, DEFAULT_SCREEN_ZOOM, DEFAULT_TEXT_ALIGN, ZERO_TRANSFORM,
    },
    link::{Bundle, Link, LinkContainer},
    node::{LabelPosition, Node, NodeOpt},
    renderer::{
        img_loader::{ImgCache, ImgLookupState},
        targets::{JsTimer, Targets},
    },
};
//use gloo::timers::callback::Timeout;
use js_sys::Array;
use js_sys::Function;
//use pastey::paste;
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
    ct: CurrentTarget,
    current_timeout: Option<JsTimer>,
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
    pub timeout: i32,
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

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(constructor)]
    pub fn new(calc: &mut Calculator, ops: RenderOpt, width: u32, height: u32) -> Self {
        Self {
            width,
            height,
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
        let iter = unsafe { (*self.calc).indexer.on_screen(self.width, self.height, t) };
        let (boxes, links, animations, nodes) = targets.get_render_targets();
        let x = self.t.x;
        let y = self.t.y;
        let k = self.t.k;
        for c in [&boxes, &links, &animations, &nodes] {
            c.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)?;
            c.clear_rect(0.0, 0.0, self.width as f64, self.height as f64);
            c.set_font(&self.ops.font_family);
            c.set_transform(k, 0.0, 0.0, k, x, y)?;
        }
        animations.set_global_alpha(self.ops.highlight_alpha);
        let ns = unsafe { &(*self.calc).nodes };
        let co = unsafe { &mut (*self.calc).options };
        let ls = unsafe { &(*self.calc).links };
        let will_animate = unsafe { &(*self.calc).animations };

        for (node_list, link_list, box_list, _) in iter {
            for id in node_list {
                if let Some(node) = ns.get(id) {
                    let opt = co.get_node(&node.opt);
                    self.draw_node(&nodes, node, opt, false)?;
                }
            }

            for id in box_list {
                if let Some(node) = ns.get_box(id) {
                    let opt = co.get_node(&node.opt);
                    self.draw_node(&boxes, node, opt, false)?;
                }
            }

            let dashes = self.ops.animation_dashes();
            for id in link_list {
                if let Some(lc) = ls.get(&id) {
                    self.draw_lc(&links, lc, co, false);
                    if will_animate.contains(&lc.id) {
                        let _ = self.draw_animation(&animations, lc, co, &dashes)?;
                        self.animation_order.push(lc.id);
                    }
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
            step: unsafe { (*self.calc).indexer.step },
        }
    }

    fn point_state(&self, p: &Point) -> CurrentTarget {
        let mut m = Move::new(p, self.t);
        let calc = self.calc;
        match unsafe { (*calc).in_point(p, &self.t) } {
            PointLookupResult::NoMatch => CurrentTarget::Screen(m),
            PointLookupResult::Bundle(b) => {
                m.center(&unsafe { (*calc).get_src_dst_center(b.src, b.dst) });
                CurrentTarget::Bundle((m, b.clone()))
            }
            PointLookupResult::Link(l) => {
                m.center(&unsafe { (*calc).get_src_dst_center(l.src, l.dst) });
                CurrentTarget::Link((m, l.clone()))
            }
            PointLookupResult::Node(n) => {
                m.center(&n.get_center());
                CurrentTarget::Node((m, n.clone()))
            }
            PointLookupResult::Box(n) => {
                m.center(&n.get_center());
                CurrentTarget::Node((m, n.clone()))
            }
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

    fn build_timeout(&mut self, p: &Point) {
        let window;
        match &self.targets {
            Some(t) => window = t.window.clone(),
            None => return,
        }
        let cb;
        {
            let this = self as *mut Self;
            let p = *p;
            cb = move || unsafe { (*this).render_highlight(&p) };
        }
        let mut jst = JsTimer::new(window);
        let _ = jst.set_timeout(cb, self.ops.timeout);
        self.current_timeout = Some(jst)
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
            let _ = cb.call2(&this, &v, &jsp);
        }
    }

    fn get_node<'n>(&self, id: u32) -> &'n Node {
        unsafe { (*self.calc).nodes.get(id).unwrap_unchecked() }
    }

    fn get_link_container<'lc>(&self, id: u64) -> &'lc LinkContainer {
        unsafe { (*self.calc).links.get(&id).unwrap_unchecked() }
    }

    pub fn render_highlight(&mut self, p: &Point) {
        let lookup = unsafe { (*self.calc).in_point(p, &self.t) };
        let calc = self.calc;
        let highlight;
        if let Some(t) = &self.targets {
            highlight = t.get_highlight_target();
        } else {
            return;
        }
        let mut nodes = Vec::new();
        let mut boxes = Vec::new();
        let mut bundles = Vec::new();
        let mut links = Vec::new();
        let ops = unsafe { &mut (*calc).options };
        match lookup {
            PointLookupResult::NoMatch => return,
            PointLookupResult::Box(node) => {
                let _ = self.draw_node(
                    &highlight,
                    &node,
                    unsafe { (*calc).get_node(&node.opt) },
                    true,
                );
                boxes.push(node.id);
            }
            PointLookupResult::Node(node) => {
                let _ = self.draw_node(
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

                let details = unsafe { lc.get_link_render(l.id).unwrap_unchecked() };
                let width = self.ops.highlight_scale * details.2;
                self.draw_line(
                    &highlight,
                    &details.0,
                    &details.1,
                    width,
                    &self.ops.highlight_color,
                );

                let _ = self.draw_highlight_nodes(l.src, l.dst, &highlight);
            }
            PointLookupResult::Bundle(b) => {
                nodes = Vec::from([b.src, b.dst]);
                bundles.push(b.id);
                let lc = self.get_link_container(b.link_id());
                for lid in &b.links {
                    if let Some(l) = lc.get_link_render(*lid) {
                        let width = self.ops.highlight_scale * l.2;
                        links.push(*lid);

                        self.draw_line(&highlight, &l.0, &l.1, width, &self.ops.highlight_color);
                    }
                }
                let rb = unsafe { lc.get_bundle_box(b.id).unwrap_unchecked() };
                let _ = self.draw_box(&highlight, &rb, ops.get_bundle(&b.opt), true);
                let _ = self.draw_highlight_nodes(b.src, b.dst, &highlight);
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
    fn draw_highlight_nodes(
        &mut self,
        a: u32,
        b: u32,
        ctx: &CanvasRenderingContext2d,
    ) -> Result<(), JsValue> {
        let src = self.get_node(a);
        let dst = self.get_node(b);
        let calc = self.calc;
        self.draw_box(ctx, src, unsafe { (*calc).get_node(&src.opt) }, true)?;
        self.draw_box(ctx, dst, unsafe { (*calc).get_node(&dst.opt) }, true)?;
        Ok(())
    }
    pub fn zoom(&mut self, delta: f64) {
        if delta < 0.0 {
            self.t.k += self.ops.wheel_move;
        } else {
            self.t.k -= self.ops.wheel_move;
            if self.t.k <= 0.0 {
                self.t.k = self.ops.wheel_move
            }
        }
        self.rndr();
    }

    fn get_step(&self) -> i64 {
        unsafe { (*self.calc).indexer.step }
    }
    fn build_cache<'c>(&mut self) -> &'c mut ImgCache {
        unsafe {
            let s = self as *mut Self;
            let cache = ImgCache::new(s);
            (*s).cache = Some(cache);
            (*s).cache.as_mut().unwrap_unchecked()
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
    ) -> Result<(), JsValue> {
        if highlight {
            ctx.set_fill_style_str(&self.ops.highlight_color);
            let scale = self.ops.highlight_scale;
            let x = target.y() - (target.x() * scale - target.x()) * 0.5;
            let y = target.y() - (target.y() * scale - target.y()) * 0.5;
            let w = target.width() * scale;
            let h = target.height() * scale;
            ctx.clear_rect(x, y, w, h);
            ctx.fill_rect(x, y, target.width() * scale, target.height() * scale);
        } else {
            let src = opt.img_src();
            match self.cache().load_img(&src) {
                ImgLookupState::Loaded(img) => {
                    ctx.draw_image_with_html_image_element(&img, target.x(), target.y())?;
                }
                _ => {
                    ctx.set_fill_style_str(&opt.box_color());
                    ctx.fill_rect(target.x(), target.y(), target.width(), target.height())
                }
            }
        }
        Ok(())
    }

    fn draw_text(
        &self,
        ctx: &CanvasRenderingContext2d,
        node: &Node,
        opt: &NodeOpt,
        highlight: bool,
    ) {
        if node.label.is_empty() {
            return;
        }
        let meta = unsafe { ctx.measure_text(&node.label).unwrap_unchecked() };
        let height = meta.font_bounding_box_ascent() + meta.font_bounding_box_descent();
        let x = node.x - (meta.width() * 0.5);
        let y;
        match opt.label_position {
            LabelPosition::Center => y = node.y - height * 0.5,
            LabelPosition::Top => y = node.get_max_y() + height,
            LabelPosition::Bottom => y = node.get_max_y(),
        }

        if highlight {
            ctx.set_fill_style_str(&self.ops.highlight_color);
            // zero out our x and y
            let x = x - meta.width() * 0.5;
            let y = y - height * 0.5;
            ctx.fill_rect(x, y, meta.width(), height);
        } else {
            ctx.set_fill_style_str(&opt.color);
            let _ = ctx.fill_text(&node.label, x, y);
        }
    }
    fn draw_node(
        &mut self,
        ctx: &CanvasRenderingContext2d,
        node: &Node,
        opt: &NodeOpt,
        highlight: bool,
    ) -> Result<(), JsValue> {
        self.draw_box(ctx, node, opt, highlight)?;
        self.draw_text(ctx, node, opt, highlight);
        Ok(())
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
        match &link.link_src {
            Some(l) => ls = l,
            None => return,
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
    ) -> Result<(), JsValue> {
        let ls;
        match &link.link_src {
            Some(l) => ls = l,
            None => return Err(JsValue::from("Link Has not been computed?")),
        }
        for a in &ls.cl.animations {
            let o = opts.get_link(&a.opt);

            ctx.begin_path();
            ctx.set_line_dash_offset(self.frame_tick);
            ctx.set_line_width(a.width);
            ctx.set_stroke_style_str(&o.animation_color);
            let _ = ctx.set_line_dash(dashes)?;
            ctx.move_to(a.src.x, a.src.y);
            ctx.line_to(a.dst.x, a.dst.y);
            ctx.close_path();
            ctx.stroke();
        }
        Ok(())
    }
}
