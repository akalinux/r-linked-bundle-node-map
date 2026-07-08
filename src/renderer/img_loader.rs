use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::renderer::ImgCache;

pub struct ImgLoader {
    src: String,
    wanted: *mut ImgCache,
    onload: Option<Closure<dyn FnMut()>>,
    onerr: Option<Closure<dyn FnMut(ErrorEvent)>>,
    pub img: HtmlImageElement,
}

impl ImgLoader {
    pub fn new(src: String, cache: *mut ImgCache) -> Result<Rc<RefCell<Self>>, &'static str> {
        let img;
        match HtmlImageElement::new() {
            Ok(i) => img = i,
            Err(_) => return Err(&"Failed to create HtmlImageElement"),
        }
        let wanted = cache as *mut ImgCache;

        unsafe { (*wanted).uptick() };
        let res = Rc::new(RefCell::new(Self {
            src: src,
            wanted,
            onerr: None,
            onload: None,
            img: img.clone(),
        }));

        let img_ok = img.clone();
        let res_ok = Rc::clone(&res);
        let on_load = Closure::once(move || {
            unsafe {
                // This call causes self to drop
                (*res_ok.borrow_mut().wanted).img_ready(&res_ok.borrow().src, Ok(img_ok));
            };
        });
        img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.borrow_mut().onload = Some(on_load);

        let res_err = Rc::clone(&res);
        let on_err = Closure::once(move |e: ErrorEvent| {
            let msg;
            match e.as_string() {
                Some(err) => msg = err,
                None => msg = String::from("Unknown Error"),
            };
            unsafe {
                // This call causes self to drop
                (*res_err.borrow_mut().wanted).img_ready(&res_err.borrow().src, Err(msg));
            };
        });
        img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.borrow_mut().onerr = Some(on_err);

        // this can run the callback before we return a value!
        img.set_src(&res.borrow().src);

        return Ok(res);
    }
    pub fn clear(&mut self) {
        self.img.set_onload(None);
        self.img.set_onerror(None);
        self.onerr = None;
        self.onload = None;
    }
}

impl Drop for ImgLoader {
    fn drop(&mut self) {
        // Safly clean up our handlers when we get dropped!
        self.clear();
    }
}
