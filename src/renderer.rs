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
    render: *mut Render,
}

pub enum ImgLookupState {
    Loading(HtmlImageElement),
    Loaded(HtmlImageElement),
    Failed(String),
    NoImg,
}

pub struct Render {}
impl Render {
    pub fn img_resolved(&mut self, _src: &String, _state: ImgLookupState, _loading: u32) {
        todo!("Need to implement this!")
    }
}

impl Drop for ImgCache {
    fn drop(&mut self) {
        // prevent circular refs!
        self.images.clear();
        self.loading = 0;
    }
}

impl ImgCache {
    pub fn img_ready(&mut self, src: &String, res: Result<HtmlImageElement, String>) {
        // this number can go negative
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
                    let ret = l.borrow().img.clone();
                    self.images.insert(src.clone(), CacheState::Loading(l));
                    return ImgLookupState::Loading(ret);
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
            CacheState::Loading(i) => return ImgLookupState::Loading(i.borrow().img.clone()),
            CacheState::Loaded(i) => return ImgLookupState::Loaded(i.clone()),
            CacheState::NoImg(i) => ImgLookupState::Failed(i.clone()),
        }
    }
}
