use std::{cell::RefCell, collections::HashMap, rc::Rc};

use web_sys::HtmlImageElement;

use crate::img_loader::ImgLoader;

pub enum CacheState {
    Loading(Rc<RefCell<ImgLoader>>),
    Loaded(HtmlImageElement),
    NoImg(String),
}
pub struct ImgCache {
    images: HashMap<String, CacheState>,
    pub loading: u32,
    on_load: Rc<RefCell<dyn FnMut(&String, ImgLookupState, u32)>>,
}

pub enum ImgLookupState {
    Loading(HtmlImageElement),
    Loaded(HtmlImageElement),
    Failed(String),
    NoImg,
}

pub struct Render {}

pub struct CacheLoader {
    cache: Rc<RefCell<ImgCache>>,
}
impl CacheLoader {
    pub fn new(cb: Rc<RefCell<dyn FnMut(&String, ImgLookupState, u32)>>) -> Self {
        Self {
            cache: Rc::new(RefCell::new(ImgCache {
                images: HashMap::new(),
                loading: 0,
                on_load: cb,
            })),
        }
    }
    pub fn load_img(&mut self, src: &String) -> ImgLookupState {
        ImgCache::load_img(&mut self.cache, src)
    }
}

impl Drop for CacheLoader {
    fn drop(&mut self) {
        // prevent circular refs!
        let cache = &mut *self.cache.borrow_mut();
        cache.images.clear();
        cache.loading = 0;
    }
}

impl ImgCache {
    pub fn img_ready(&mut self, src: &String, res: Result<HtmlImageElement, String>) {
        // this number can go negative
        let cb = &mut *self.on_load.borrow_mut();
        match res {
            Err(msg) => {
                self.images
                    .insert(src.clone(), CacheState::NoImg(msg.clone()));
                cb(src, ImgLookupState::Failed(msg), self.loading);
            }
            Ok(img) => {
                self.images
                    .insert(src.clone(), CacheState::Loaded(img.clone()));
                cb(src, ImgLookupState::Loaded(img), self.loading);
            }
        };
    }
    pub fn load_img(cache: &Rc<RefCell<Self>>, src: &String) -> ImgLookupState {
        if src.is_empty() {
            return ImgLookupState::NoImg;
        }
        if let Some(state) = cache.borrow().images.get(src) {
            return Self::cache_state(state);
        }

        // if we get here.. the image has never been loaded.
        let res = ImgLoader::new(src.clone(), Rc::clone(&cache));
        match res {
            Ok(l) => {
                if let Some(state) = cache.borrow().images.get(src) {
                    // Loading all ready in progress..
                    // This is caused by the browser all
                    // all ready having the image in its cache!
                    return Self::cache_state(state);
                } else {
                    // Loader needs to run
                    let ret = l.borrow().img.clone();
                    cache
                        .borrow_mut()
                        .images
                        .insert(src.clone(), CacheState::Loading(l));
                    return ImgLookupState::Loading(ret);
                }
            }
            Err(msg) => {
                cache
                    .borrow_mut()
                    .images
                    .insert(src.clone(), CacheState::NoImg(String::from(msg)));
                return ImgLookupState::Failed(String::from(msg));
            }
        }
    }
    fn cache_state(state: &CacheState) -> ImgLookupState {
        match state {
            CacheState::Loading(i) => return ImgLookupState::Loading(i.borrow().img.clone()),
            CacheState::Loaded(i) => return ImgLookupState::Loaded(i.clone()),
            CacheState::NoImg(i) => ImgLookupState::Failed(i.clone()),
        }
    }
}
