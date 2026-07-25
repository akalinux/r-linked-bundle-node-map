use crate::Point;
use crate::ScreenBox;
use crate::renderer::Render;
use js_sys::Number;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    AddEventListenerOptions, CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement,
    HtmlDivElement, PointerEvent, ResizeObserver, ResizeObserverEntry, WheelEvent, Window,
};

pub struct DivEventWatcher {
    target: String,
    cb: Closure<dyn FnMut(Event)>,
    div: HtmlDivElement,
}

impl DivEventWatcher {
    pub fn new<F>(s: &str, f: F, div: &HtmlDivElement) -> Result<Self, JsValue>
    where
        F: FnMut(Event) + 'static,
    {
        let target = String::from(s);
        let cb = Closure::wrap(Box::new(f));
        let options = AddEventListenerOptions::new();
        options.set_passive(false);
        div.add_event_listener_with_callback_and_add_event_listener_options(
            &target,
            cb.as_ref().unchecked_ref(),
            &options,
        )?;

        Ok(Self {
            div: div.clone(),
            target,
            cb,
        })
    }
    pub fn clear(&self) {
        let _ = self.div.remove_event_listener_with_callback_and_bool(
            &self.target,
            self.cb.as_ref().unchecked_ref(),
            false,
        );
    }
}
impl Drop for DivEventWatcher {
    fn drop(&mut self) {
        self.clear();
    }
}

pub struct JsTimer {
    cb: Option<(i32, Closure<dyn FnMut()>)>,
    window: Window,
}
impl JsTimer {
    pub fn new(w: Window) -> Self {
        Self {
            window: w.clone(),
            cb: None,
        }
    }
    pub fn set_timeout<F>(&mut self, f: F, timeout: i32) -> Result<i32, JsValue>
    where
        F: FnMut() + 'static,
    {
        let cb = Closure::wrap(Box::new(f));
        self.clear();
        let t = self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                timeout,
            )?;

        self.cb = Some((t, cb));
        Ok(t)
    }
    pub fn clear(&mut self) {
        if let Some((t, _)) = &self.cb {
            self.window.clear_timeout_with_handle(*t);
        }
    }
}

impl Drop for JsTimer {
    fn drop(&mut self) {
        self.clear();
    }
}
pub struct Targets {
    pub div: HtmlDivElement,
    pub boxnodes: HtmlCanvasElement,
    pub links: HtmlCanvasElement,
    pub animations: HtmlCanvasElement,
    pub nodes: HtmlCanvasElement,
    pub highlight: HtmlCanvasElement,
    pub root: Element,
    pub window: Window,
    render: *mut Render,
    on_enter: Option<DivEventWatcher>,
    on_move: Option<DivEventWatcher>,
    on_down: Option<DivEventWatcher>,
    on_up: Option<DivEventWatcher>,
    on_leave: Option<DivEventWatcher>,
    on_wheel: Option<DivEventWatcher>,
    on_size: Option<SizeWatcher>,
    screen_box: ScreenBox,
}

impl Drop for Targets {
    fn drop(&mut self) {
        self.clear_watchers();
        let div = &self.div;
        for c in self.get_child_targets() {
            let _ = div.remove_child(c);
        }
        let _ = self.root.remove_child(div);
    }
}

fn to_fixed_px(n: f64) -> String {
    let js_num: Number = n.into();
    let js_str = unsafe { js_num.to_fixed(2).unwrap_unchecked() };
    let mut str = String::from(js_str);
    str.push_str("px");
    str
}
pub struct SizeWatcher {
    _callback: Closure<dyn FnMut(Vec<ResizeObserverEntry>, ResizeObserver)>,
    watcher: ResizeObserver,
}

impl SizeWatcher {
    pub fn new<F>(div: &HtmlDivElement, f: F) -> Result<Self, JsValue>
    where
        F: FnMut(Vec<ResizeObserverEntry>, ResizeObserver) + 'static,
    {
        let cb = Closure::wrap(Box::new(f));
        match ResizeObserver::new(cb.as_ref().unchecked_ref()) {
            Ok(o) => {
                o.observe(div);
                Ok(Self {
                    watcher: o,
                    _callback: cb,
                })
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for SizeWatcher {
    fn drop(&mut self) {
        self.watcher.disconnect();
    }
}

macro_rules! add_listen_callback {
    ($self:ident,$target:literal,$field:ident,$method:ident) => {{
        let ptr = $self.render;
        let div = $self.div.clone();
        let cb = move |e: Event| unsafe {
            if let Some(p) = Targets::get_div_xy(&e, &div) {
                (*ptr).$method(&p);
            }
        };
        let res = DivEventWatcher::new(&$target, cb, &$self.div)?;
        $self.$field = Some(res);
    }};
}

impl Targets {
    pub fn get_render_targets(
        &self,
    ) -> (
        CanvasRenderingContext2d,
        CanvasRenderingContext2d,
        CanvasRenderingContext2d,
        CanvasRenderingContext2d,
    ) {
        (
            Self::unpack_canvas(&self.boxnodes),
            Self::unpack_canvas(&self.links),
            Self::unpack_canvas(&self.animations),
            Self::unpack_canvas(&self.nodes),
        )
    }
    fn unpack_canvas(c: &HtmlCanvasElement) -> CanvasRenderingContext2d {
        unsafe {
            c.get_context("2d")
                .unwrap_err_unchecked()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap_unchecked()
        }
    }
    pub fn get_animation_target(&self) -> CanvasRenderingContext2d {
        Self::unpack_canvas(&self.animations)
    }

    pub fn get_highlight_target(&self) -> CanvasRenderingContext2d {
        Self::unpack_canvas(&self.highlight)
    }

    fn get_child_targets(&self) -> [&HtmlCanvasElement; 5] {
        [
            &self.boxnodes,
            &self.animations,
            &self.highlight,
            &self.links,
            &self.nodes,
        ]
    }
    pub fn center_canvas(&mut self) {
        let src = self.get_screenbox();
        if src == self.screen_box {
            return;
        }
        self.screen_box = src;
        let dst = src.center(&unsafe { (*self.render).canvas_box() });
        let top = to_fixed_px(dst.y);
        let left = to_fixed_px(dst.x);
        for c in self.get_child_targets() {
            let style = c.style();
            let _ = style.set_property("top", &top);
            let _ = style.set_property("left", &left);
        }
    }

    pub fn new(
        render: *mut Render,
        id: String,
        div_style: String,
        canvas_style: String,
    ) -> Result<Self, JsValue> {
        let sb = unsafe { (*render).canvas_box() };

        let (dom, div, root, window) = Self::create_div(&id, div_style)?;
        let rect = div.get_bounding_client_rect();
        let screen_box = ScreenBox {
            x: 0,
            y: 0,
            width: rect.width() as u32,
            height: rect.height() as u32,
            step: sb.step,
        };
        let dst = screen_box.center(&sb);
        let w = sb.width;
        let h = sb.height;

        let top = to_fixed_px(dst.y);
        let left = to_fixed_px(dst.x);
        let boxnodes = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let links = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let animations = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let nodes = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let highlight = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let mut res = Self {
            window,
            boxnodes,
            root,
            div,
            links,
            nodes,
            animations,
            highlight,
            render,
            on_enter: None,
            on_leave: None,
            on_down: None,
            on_up: None,
            on_move: None,
            on_wheel: None,
            on_size: None,
            screen_box: screen_box,
        };
        res.init_watchers()?;

        //let links=Self::create_canvas(&dom, &div, &canvas_style, width, height, top, left);
        Ok(res)
    }
    fn create_canvas(
        dom: &Document,
        div: &HtmlDivElement,
        canvas_style: &String,
        w: u32,
        h: u32,
        top: &String,
        left: &String,
    ) -> Result<HtmlCanvasElement, JsValue> {
        let c = dom
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        c.set_width(w);
        c.set_height(h);
        let style = c.style();
        c.set_attribute("style", canvas_style)?;
        let mut width = w.to_string();
        width.push_str("px");
        style.set_property("width", &width)?;
        let mut height = h.to_string();
        height.push_str("px");
        style.set_property("height", &height)?;
        style.set_property("top", top)?;
        style.set_property("left", left)?;
        div.append_child(&c)?;

        Ok(c)
    }

    fn create_div(
        id: &String,
        style: String,
    ) -> Result<(Document, HtmlDivElement, Element, Window), JsValue> {
        let dom;
        let window;
        match web_sys::window() {
            Some(w) => match w.document() {
                Some(d) => {
                    dom = d;
                    window = w
                }
                None => return Err(JsValue::from_str("no `document` exists")),
            },
            None => return Err(JsValue::from_str("no global `window` exist")),
        }
        let root;
        match dom.get_element_by_id(&id) {
            Some(e) => root = e,
            None => return Err(JsValue::from_str("Failed to fetch ID")),
        }

        let el = dom.create_element("div")?;
        let div;
        match el.dyn_ref::<HtmlDivElement>() {
            Some(d) => div = d,
            None => return Err(JsValue::from_str("Div creation failed")),
        }

        div.set_attribute("style", &style)?;

        root.append_child(&div)?;
        Ok((dom, div.clone(), el, window))
    }
    pub fn clear_watchers(&mut self) {
        self.on_down = None;
        self.on_move = None;
        self.on_up = None;
        self.on_leave = None;
        self.on_wheel = None;
        self.on_size = None;
    }
    fn get_div_xy(e: &Event, div: &HtmlDivElement) -> Option<Point> {
        e.prevent_default();
        e.stop_propagation();
        let rect = div.get_bounding_client_rect();
        if let Some(e) = e.dyn_ref::<PointerEvent>() {
            let x = e.client_x() as f64 - rect.left();
            let y = e.client_y() as f64 - rect.top();
            return Some(Point { x, y });
        }
        None
    }
    pub fn init_watchers(&mut self) -> Result<(), JsValue> {
        // mouse down
        add_listen_callback!(self, "pointerdown", on_down, mouse_down);

        // mouse up
        add_listen_callback!(self, "pointerup", on_up, mouse_up);

        // mouse move
        add_listen_callback!(self, "pointermove", on_move, mouse_move);

        // desktop only
        add_listen_callback!(self, "pointerenter", on_leave, mouse_leave);
        add_listen_callback!(self, "pointerleave", on_enter, mouse_enter);
        let ptr = self.render;

        self.on_wheel = Some(DivEventWatcher::new(
            "wheel",
            move |e| {
                e.prevent_default();
                e.stop_propagation();
                if let Some(w) = e.dyn_ref::<WheelEvent>() {
                    unsafe { (*ptr).zoom(w.delta_y()) }
                }
            },
            &self.div,
        )?);

        let ptr = self as *mut Self;
        let cb = move |entries: Vec<ResizeObserverEntry>, _observer: ResizeObserver| {
            for _ in entries {
                unsafe { (*ptr).center_canvas() };
                return;
            }
        };
        match SizeWatcher::new(&self.div, cb) {
            Err(err) => return Err(err),
            Ok(w) => self.on_size = Some(w),
        };

        Ok(())
    }

    pub fn get_screenbox(&self) -> ScreenBox {
        let rect = self.div.get_bounding_client_rect();
        ScreenBox {
            x: 0,
            y: 0,
            step: unsafe { (*self.render).get_step() },
            width: rect.width() as u32,
            height: rect.height() as u32,
        }
    }
}
