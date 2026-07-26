use crate::controllers::api::API;
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub struct ImagesController {
    api: API,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
}

impl PartialEq for ImagesController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ImagesController {
    pub fn new(api: API, engine: Rc<RefCell<Option<BalchugEngine>>>) -> Self {
        Self {
            api,
            engine,
            thumbs: Default::default(),
        }
    }
    
    pub fn get_thumbs(&self) -> Vec<String> {
        self.thumbs.read().clone()
    }
    
    pub async fn handle_upload(&mut self, files: Vec<FileData>) {
        if let Some(file_data) = files.first()
            && let Ok(bytes) = file_data.read_bytes().await {
            let mime_type = match infer::get(&bytes) {
                Some(kind) => kind.mime_type(),
                None => "application/octet-stream", // Safe fallback
            };
            if let Some((new_thumbs, atlas)) = self.api.upload_image(bytes, mime_type).await {
                self.thumbs.set(new_thumbs);
                if let Some(engine) = self.engine.borrow().as_ref() {
                    let now = instant::now().round() as u64;

                    let img_url = self.api.asset_url("atlas.webp");
                    engine.set_atlas(&format!("{img_url}?{now}"), atlas);
                }
            }
        }
    }
}
