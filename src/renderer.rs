use std::{cell::RefCell, collections::HashMap, rc::Rc};
mod img_loader;
use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::{HtmlCanvasElement, HtmlDivElement, HtmlImageElement};

use crate::{ScreenBox, Transform, calc::Calculator, renderer::img_loader::ImgLoader};

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
    t: Transform,
}

pub struct RenderLayer {
    div: HtmlDivElement,
    links: HtmlCanvasElement,
    animations: HtmlCanvasElement,
    nodes: HtmlCanvasElement,
    highlight: HtmlCanvasElement,
}

impl Drop for RenderLayer {
    fn drop(&mut self) {
        let div = &self.div;
        for el in [&self.links, &self.nodes, &self.animations, &self.highlight] {
            match div.remove_child(el) {
                Ok(_) => (),
                Err(_) => (),
            }
        }
    }
}

#[wasm_bindgen]
impl Render {
    #[wasm_bindgen(constructor)]
    pub fn new(calc: &mut Calculator, id: String, width: u32, height: u32) -> Self {
        let screen_res = ScreenBox {
            width,
            height,
            x: 0,
            y: 0,
            step: calc.indexer().step,
        };
        let res = Self {
            calc: calc as *mut Calculator,
            screen_res,
            t: Transform {
                x: 0.0,
                y: 0.0,
                k: 1.0,
            },
            cache: None,
        };

        return res;
    }
    pub fn set_transform(&mut self, t: Transform) {
        self.screen_res = self.screen_res.transform(&t);
        self.t = t;
    }
}
impl Render {
    pub fn cache<'c>(&'c mut self) -> &'c mut ImgCache {
        if self.cache.is_none() {
            return self.build_cache();
        }
        return self.cache.as_mut().unwrap();
    }

    fn build_cache<'c>(&mut self) -> &'c mut ImgCache {
        unsafe {
            let s = self as *mut Self;
            let cache = ImgCache::new(s);
            (*s).cache = Some(cache);
            return (*s).cache.as_mut().unwrap();
        }
    }

    pub fn img_resolved(&mut self, _src: &String, _state: ImgLookupState, _loading: u32) {
        todo!("Need to implement this!")
    }
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
            CacheState::Loading(i) => return ImgLookupState::Loading,
            CacheState::Loaded(i) => return ImgLookupState::Loaded(i.clone()),
            CacheState::NoImg(i) => ImgLookupState::Failed(i.clone()),
        }
    }
}
