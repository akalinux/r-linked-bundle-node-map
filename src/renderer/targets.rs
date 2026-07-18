use crate::Point;
use crate::ScreenBox;
use crate::renderer::Render;
use gloo::{events::EventListener, events::EventListenerOptions};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;
use web_sys::{Document, Element};
use web_sys::{
    Event, HtmlCanvasElement, HtmlDivElement, MouseEvent, ResizeObserver, ResizeObserverEntry,
    WheelEvent,
};

pub struct Targets {
    pub div: HtmlDivElement,
    pub boxnodes: HtmlCanvasElement,
    pub links: HtmlCanvasElement,
    pub animations: HtmlCanvasElement,
    pub nodes: HtmlCanvasElement,
    pub highlight: HtmlCanvasElement,
    pub root: Element,
    render: *mut Render,
    on_move: Option<EventListener>,
    on_down: Option<EventListener>,
    on_up: Option<EventListener>,
    on_leave: Option<EventListener>,
    on_wheel: Option<EventListener>,
    on_size: Option<SizeWatcher>,
    on_enter: Option<EventListener>,
    screen_box: ScreenBox,
}

impl Drop for Targets {
    fn drop(&mut self) {
        self.clear_watchers();
        let div = &self.div;
        for c in self.get_child_targets() {
            match div.remove_child(c) {
                _ => (),
            }
        }
        match self.root.remove_child(div) {
            _ => (),
        }
    }
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
        $self.$field = Some(EventListener::new_with_options(
            &$self.div,
            $target,
            EventListenerOptions::enable_prevent_default(),
            move |e: &Event| {
                e.prevent_default();
                e.stop_propagation();
                if let Some(event) = e.dyn_ref::<MouseEvent>() {
                    let p = Self::get_div_xy(&event, &div);
                    unsafe { (*ptr).$method(&p) };
                }
            },
        ))
    }};
}

macro_rules! get_canvas2d {
    ($field:expr) => {
        $field
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap()
    };
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
            get_canvas2d!(self.boxnodes),
            get_canvas2d!(self.links),
            get_canvas2d!(self.animations),
            get_canvas2d!(self.nodes),
        )
    }
    pub fn get_animation_target(&self) -> CanvasRenderingContext2d {
        get_canvas2d!(self.animations)
    }

    pub fn get_highlight_target(&self) -> CanvasRenderingContext2d {
        get_canvas2d!(self.highlight)
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
        let top = format!("{:.2}px", dst.y);
        let left = format!("{:.2}px", dst.x);
        for c in self.get_child_targets() {
            let style = c.style();
            match style.set_property("top", &top) {
                _ => (),
            };
            match style.set_property("left", &left) {
                _ => (),
            }
        }
    }

    pub fn new(
        render: *mut Render,
        id: String,
        div_style: String,
        canvas_style: String,
    ) -> Result<Self, JsValue> {
        let sb = unsafe { (*render).canvas_box() };

        let (dom, div, root) = Self::create_div(&id, div_style)?;
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

        let top = format!("{:.2}px;", dst.y);
        let left = format!("{:.2}px;", dst.x);
        let boxnodes = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let links = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let animations = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let nodes = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let highlight = Self::create_canvas(&dom, &div, &canvas_style, w, h, &top, &left)?;
        let mut res = Self {
            boxnodes,
            root,
            div,
            links,
            nodes,
            animations,
            highlight,
            render,
            on_down: None,
            on_enter: None,
            on_leave: None,
            on_move: None,
            on_size: None,
            on_up: None,
            on_wheel: None,
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
        style.set_property("width", &format!("{}px", w))?;
        style.set_property("height", &format!("{}px", h))?;
        style.set_property("top", top)?;
        style.set_property("left", left)?;
        div.append_child(&c)?;

        Ok(c)
    }

    fn create_div(
        id: &String,
        style: String,
    ) -> Result<(Document, HtmlDivElement, Element), JsValue> {
        let dom = web_sys::window()
            .expect("no global `window` exists")
            .document()
            .expect("Failed to get document from window!");
        let el = dom
            .get_element_by_id(&id)
            .expect(&format!("Failed to fetch element by ID: {}", &id));

        let div = dom
            .create_element("div")
            .expect("Could not create required div element")
            .dyn_into::<HtmlDivElement>()
            .expect("Browser Failed to set div up correctly!");

        div.set_attribute("style", &style)?;

        el.append_child(&div)?;
        Ok((dom, div, el))
    }
    pub fn clear_watchers(&mut self) {
        self.on_down = None;
        self.on_move = None;
        self.on_up = None;
        self.on_leave = None;
        self.on_wheel = None;
        self.on_size = None;
    }
    fn get_div_xy(e: &MouseEvent, div: &HtmlDivElement) -> Point {
        let rect = div.get_bounding_client_rect();
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();
        Point { x, y }
    }
    pub fn init_watchers(&mut self) -> Result<(), JsValue> {
        add_listen_callback!(self, "mousemove", on_move, mouse_move);
        add_listen_callback!(self, "mouseup", on_up, mouse_up);
        add_listen_callback!(self, "mousedown", on_down, mouse_down);
        add_listen_callback!(self, "mouseleave", on_leave, mouse_leave);
        add_listen_callback!(self, "mouseenter", on_enter, mouse_enter);
        let ptr = self.render;

        self.on_wheel = Some(EventListener::new_with_options(
            &self.div,
            "wheel",
            EventListenerOptions::enable_prevent_default(),
            move |e| {
                e.prevent_default();
                e.stop_propagation();
                if let Some(w) = e.dyn_ref::<WheelEvent>() {
                    let delta = w.delta_y() as f64;
                    if delta < 0.0 {
                        unsafe { (*ptr).zoom_in() }
                    } else {
                        unsafe { (*ptr).zoom_out() }
                    }
                }
            },
        ));

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
