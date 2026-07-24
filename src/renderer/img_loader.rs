use std::collections::HashMap;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, HtmlImageElement};

use crate::renderer::Render;

pub struct ImgLoader {
    onload: Option<Closure<dyn FnMut()>>,
    onerr: Option<Closure<dyn FnMut(ErrorEvent)>>,
    pub img: HtmlImageElement,
}

pub enum CacheState {
    Loading(ImgLoader),
    Loaded(HtmlImageElement),
    NoImg(String),
}
pub struct ImgCache {
    images: HashMap<String, CacheState>,
    loading: u32,
    render: *mut Render,
}

pub enum ImgLookupState {
    Loading,
    Loaded(HtmlImageElement),
    Failed(String),
    NoImg,
}
impl Drop for ImgCache {
    fn drop(&mut self) {
        // prevent circular refs!
        self.clear()
    }
}

impl ImgCache {
    pub fn new(render: *mut Render) -> Self {
        Self {
            render,
            images: HashMap::new(),
            loading: 0,
        }
    }
    pub fn clear(&mut self) {
        self.images.clear();
        self.loading = 0;
    }
    pub fn uptick(&mut self) {
        self.loading += 1;
    }

    pub fn img_ready(&mut self, src: &String, res: Result<HtmlImageElement, String>) {
        self.loading -= 1;
        match res {
            Err(msg) => {
                self.images
                    .insert(src.clone(), CacheState::NoImg(msg.clone()));
                unsafe {
                    (*self.render).img_resolved(src, ImgLookupState::Failed(msg), self.loading);
                }
            }
            Ok(img) => {
                self.images
                    .insert(src.clone(), CacheState::Loaded(img.clone()));
                unsafe {
                    (*self.render).img_resolved(src, ImgLookupState::Loaded(img), self.loading);
                }
            }
        };
    }
    pub fn load_img(&mut self, src: &String) -> ImgLookupState {
        if src.is_empty() {
            return ImgLookupState::NoImg;
        }
        if let Some(state) = self.images.get(src) {
            return Self::cache_state(state);
        }

        // if we get here.. the image has never been loaded.
        let res = ImgLoader::new(&src, self);
        match res {
            Ok(l) => {
                if let Some(state) = self.images.get(src) {
                    // Loading all ready in progress..
                    // This is caused by the browser all
                    // all ready having the image in its cache!
                    return Self::cache_state(state);
                } else {
                    // Loader needs to run
                    self.images.insert(src.clone(), CacheState::Loading(l));
                    return ImgLookupState::Loading;
                }
            }
            Err(e) => {
                let msg = match e.as_string() {
                    Some(m) => m,
                    None => String::from("Failed to fetch img"),
                };
                self.images
                    .insert(src.clone(), CacheState::NoImg(msg.clone()));
                return ImgLookupState::Failed(String::from(msg));
            }
        }
    }
    fn cache_state(state: &CacheState) -> ImgLookupState {
        match state {
            CacheState::Loading(_) => return ImgLookupState::Loading,
            CacheState::Loaded(i) => return ImgLookupState::Loaded(i.clone()),
            CacheState::NoImg(i) => ImgLookupState::Failed(i.clone()),
        }
    }
}

impl ImgLoader {
    pub fn new(url: &String, cache: *mut ImgCache) -> Result<Self, JsValue> {
        let img = HtmlImageElement::new()?;

        let wanted = cache as *mut ImgCache;

        unsafe { (*wanted).uptick() };
        let mut res = Self {
            onerr: None,
            onload: None,
            img,
        };

        let img_ok = res.img.clone();
        let src = url.clone();
        let on_load = Closure::once(move || {
            // This call causes self to drop
            unsafe { (*wanted).img_ready(&src, Ok(img_ok)) }
        });
        res.img.set_onload(Some(on_load.as_ref().unchecked_ref()));
        res.onload = Some(on_load);

        let src = url.clone();
        let on_err = Closure::once(move |e: ErrorEvent| {
            let msg;
            match e.as_string() {
                Some(err) => msg = err,
                None => msg = String::from("Unknown Error"),
            };
            // This call causes self to drop
            unsafe { (*wanted).img_ready(&src, Err(msg)) }
        });
        res.img.set_onerror(Some(on_err.as_ref().unchecked_ref()));
        res.onerr = Some(on_err);

        // this can run the callback before we return a value!
        res.img.set_src(url);

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
