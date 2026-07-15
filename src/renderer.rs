use std::{cell::RefCell, collections::HashMap, rc::Rc};
pub mod img_loader;
pub mod stater;
pub mod targets;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::HtmlImageElement;

use crate::{
    ScreenBox, Transform,
    calc::Calculator,
    constants::{DEFAULT_CANVAS_STYLE, DEFAULT_DIV_STYLE, DEFAULT_HOVER_TIMEOUT},
    renderer::{
        img_loader::ImgLoader,
        stater::{CurrentTarget, Stater},
        targets::Targets,
    },
};

pub enum CacheState {
    Loading(Rc<RefCell<ImgLoader>>),
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

#[wasm_bindgen]
pub struct Render {
    calc: *mut Calculator,
    screen_res: ScreenBox,
    cache: Option<ImgCache>,
    stater: Option<Stater>,
    targets: Option<Targets>,
    id: String,
}

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(constructor)]
    pub fn new(calc: &mut Calculator, width: u32, height: u32, id: String) -> Self {
        let screen_res = ScreenBox {
            width,
            height,
            x: 0,
            y: 0,
            step: calc.indexer().step,
        };
        let res = Self {
            id,
            calc: calc as *mut Calculator,
            screen_res,
            cache: None,
            stater: None,
            targets: None,
        };

        return res;
    }
    pub fn move_screen(&mut self, t: &Transform) {
        self.screen_res = self.screen_res.transform(t);
        self.stater().t = t.clone();
    }
    pub fn render(&mut self) {}
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
        let stater;
        if let Some(s) = &mut self.stater {
            stater = s as *mut Stater
        } else {
            return Err(JsValue::from_str("No Stater ready!"));
        }
        // people do dumb things!
        // be nice and clean up the old instance!
        self.targets = None;
        let targets = Targets::new(stater, element_id, div_style, canvas_style)?;
        self.targets = Some(targets);
        Ok(())
    }
    pub fn unmount(&mut self) {
        self.targets = None;
    }
}
impl Render {
    pub fn get_id(&self) -> &String {
        &self.id
    }
    pub fn get_screenbox(&self) -> &ScreenBox {
        &self.screen_res
    }
    pub fn cache<'c>(&'c mut self) -> &'c mut ImgCache {
        if self.cache.is_none() {
            return self.build_cache();
        }
        return self.cache.as_mut().unwrap();
    }

    // todo need to implement these!
    pub fn mouse_up(&mut self, _ct: &CurrentTarget) {}
    pub fn mouse_down(&mut self, _ct: &CurrentTarget) {}
    pub fn mouse_over(&mut self, _ct: &CurrentTarget) {}

    pub fn calc(&self) -> *mut Calculator {
        return self.calc;
    }

    pub fn stater<'c>(&'c mut self) -> &'c mut Stater {
        if self.stater.is_none() {
            unsafe {
                let s = self as *mut Self;
                let stater = Stater::new(s, DEFAULT_HOVER_TIMEOUT);
                (*s).stater = Some(stater);
            }
        }
        return self.stater.as_mut().unwrap();
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
        let res = ImgLoader::new(src.clone(), self);
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
            Err(msg) => {
                self.images
                    .insert(src.clone(), CacheState::NoImg(String::from(msg)));
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
