use crate::{Point, renderer::stater::Stater};
use gloo::{events::EventListener, events::EventListenerOptions};
use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlCanvasElement, HtmlDivElement, MouseEvent, WheelEvent};

pub struct Targets {
    pub div: HtmlDivElement,
    pub links: HtmlCanvasElement,
    pub animations: HtmlCanvasElement,
    pub nodes: HtmlCanvasElement,
    pub highlight: HtmlCanvasElement,
    stater: *mut Stater,
    on_move: Option<EventListener>,
    on_down: Option<EventListener>,
    on_up: Option<EventListener>,
    on_leave: Option<EventListener>,
    on_wheel: Option<EventListener>,
}

impl Drop for Targets {
    fn drop(&mut self) {
        self.clear_watchers();
    }
}

macro_rules! add_listen_callback {
    ($self:ident,$target:literal,$field:ident,$method:ident) => {{
        let ptr = $self.stater;
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
impl Targets {
    pub fn clear_watchers(&mut self) {
        self.on_down = None;
        self.on_move = None;
        self.on_up = None;
        self.on_leave = None;
        self.on_wheel = None;
    }
    fn get_div_xy(e: &MouseEvent, div: &HtmlDivElement) -> Point {
        let rect = div.get_bounding_client_rect();
        let x = e.client_x() as f64 - rect.left();
        let y = e.client_y() as f64 - rect.top();
        Point { x, y }
    }
    pub fn init_watchers(&mut self) {
        add_listen_callback!(self, "mousemove", on_move, mouse_move);
        add_listen_callback!(self, "mouseup", on_up, mouse_up);
        add_listen_callback!(self, "mousedown", on_down, mouse_down);
        add_listen_callback!(self, "mouseleave", on_leave, mouse_leave);
        let ptr = self.stater;

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
    }
}
