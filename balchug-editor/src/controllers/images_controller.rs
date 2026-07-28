use crate::controllers::api::API;
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{SpriteAnimation, SpriteState};
use crate::states::project_state::ProjectState;

#[derive(Clone)]
pub struct ImagesController {
    api: API,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_state: Rc<RefCell<ProjectState>>,
}

impl PartialEq for ImagesController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ImagesController {
    pub fn new(api: API, engine: Rc<RefCell<Option<BalchugEngine>>>, project_state: Rc<RefCell<ProjectState>>) -> Self {
        Self {
            api,
            engine,
            project_state,
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

    pub fn put_image(&self, image_id: usize, parallax_factor: f32) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut scenario_images = engine.get_scenario_images_states(None);
            let sprite_id = scenario_images.len();
            let cur_offset = engine.get_offset();
            let aspect_ratio = *self.project_state.borrow().aspect_ratio.read();

            let start_offset = (cur_offset - 0.5 * parallax_factor / aspect_ratio).max(0.0);

            let state_zero = SpriteState {
                offset: start_offset,
                x: 0.0,
                y: 1.0 / aspect_ratio,
                width: 1.0,
                color: [0.0, 0.0, 0.0, 1.0],
            };
            let state_one = SpriteState {
                offset: start_offset + parallax_factor / aspect_ratio,
                x: 0.0,
                y: 0.0,
                width: 1.0,
                color: [0.0, 0.0, 0.0, 1.0],
            };
            let animation = SpriteAnimation {
                sprite_id,
                atlas_item_id: image_id,
                states: vec![state_zero, state_one],
            };
            scenario_images.push(animation);
            engine.set_scenario(Scenario {
                images: scenario_images,
                text_lines: Vec::new(),
            });
            self.project_state.borrow_mut().add_parallax_factor(sprite_id, parallax_factor);
        }
    }
}
