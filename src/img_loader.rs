use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::renderer::ImgCache;

pub struct ImgLoader {
    src: String,
    wanted: Rc<RefCell<ImgCache>>,
    onload: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
    onerr: Rc<RefCell<Option<Closure<dyn FnMut(ErrorEvent)>>>>,
    pub img: HtmlImageElement,
}

impl ImgLoader {
    pub fn new(
        src: String,
        wanted: Rc<RefCell<ImgCache>>,
    ) -> Result<Rc<RefCell<Self>>, &'static str> {
        let img;
        match HtmlImageElement::new() {
            Ok(i) => img = i,
            Err(_) => return Err(&"Failed to create HtmlImageElement"),
        }

        wanted.borrow_mut().loading += 1;
        let res = Rc::new(RefCell::new(Self {
            src: String::from(src),
            wanted,
            onerr: Rc::new(RefCell::new(None)),
            onload: Rc::new(RefCell::new(None)),
            img: img.clone(),
        }));

        let img_ok = img.clone();
        let res_ok = Rc::clone(&res);
        let on_load = Closure::once(move || {
            // This call causes our drop
            res_ok.borrow_mut().wanted.borrow_mut().loading -= 1;
            res_ok
                .borrow_mut()
                .wanted
                .borrow_mut()
                .img_ready(&res_ok.borrow().src, Ok(img_ok));

            // drop takes care of this!
            //res_ok.borrow_mut().clear();
        });
        img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.borrow_mut().onload.borrow_mut().replace(on_load);

        let res_err = Rc::clone(&res);
        let on_err = Closure::once(move |e: ErrorEvent| {
            res_err.borrow_mut().wanted.borrow_mut().loading -= 1;
            match e.as_string() {
                // This call causes our drop
                Some(err) => res_err
                    .borrow_mut()
                    .wanted
                    .borrow_mut()
                    .img_ready(&res_err.borrow().src, Err(err)),
                // This call causes our drop
                None => res_err
                    .borrow_mut()
                    .wanted
                    .borrow_mut()
                    // This call causes our drop
                    .img_ready(&res_err.borrow().src, Err(String::from("Unknown Error"))),
            }

            // drop takes care of this!
            //res_err.borrow_mut().clear();
        });
        img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.borrow_mut().onerr.borrow_mut().replace(on_err);

        // this can run the callback before we return a value!
        img.set_src(&res.borrow().src);

        return Ok(res);
    }
    pub fn clear(&mut self) {
        self.img.set_onload(None);
        self.img.set_onerror(None);
        self.onerr.replace(None);
        self.onload.replace(None);
    }
}

impl Drop for ImgLoader {
    fn drop(&mut self) {
        self.clear();
    }
}
