use crate::controllers::api::API;
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use balchug_common::sprite::{SpriteAnimation, SpriteImageData, SpriteData, SpriteState, SpriteTextData};
use crate::states::project_state::{ProjectState, SpriteProperties};

#[derive(Clone)]
pub struct ResourcesController {
    api: API,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_state: Rc<RefCell<ProjectState>>,
    sprite_id: Signal<Option<usize>>,
}

impl PartialEq for ResourcesController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ResourcesController {
    pub fn new(api: API, engine: Rc<RefCell<Option<BalchugEngine>>>, project_state: Rc<RefCell<ProjectState>>) -> Self {
        Self {
            api,
            engine,
            project_state,
            thumbs: Default::default(),
            sprite_id: Signal::new(None),
        }
    }
    
    pub fn get_thumbs(&self) -> Vec<String> {
        self.thumbs.read().clone()
    }
    
    pub fn get_sprite_id_signal(&self) -> Signal<Option<usize>> {
        self.sprite_id
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

    pub fn put_image(&self, image_id: usize, props: SpriteProperties) {
        let data = SpriteData::Image(SpriteImageData{ atlas_item_id: image_id });
        let proportion = self.engine.borrow().as_ref()
            .and_then(|engine| engine.get_atlas_item(image_id))
            .map(|item| item.origin_width as f32 / item.origin_height as f32)
            .unwrap_or(1.0);
        self.add_sprite_animation(data, proportion, props);
    }

    pub fn put_text(&self, text: String, size: i32, props: SpriteProperties) {
        let relative_height = size as f32 * 0.002;
        let data = SpriteData::Text(SpriteTextData{ text, relative_height });
        self.add_sprite_animation(data, 1.0 / relative_height, props);
    }

    fn add_sprite_animation(&self, data: SpriteData, proportion: f32, props: SpriteProperties) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut sprites = engine.get_sprites_animations(None);
            let sprite_id = sprites.len();
            let cur_offset = engine.get_offset();
            let aspect_ratio = *self.project_state.borrow().aspect_ratio.read();
            let animation = SpriteAnimation {
                sprite_id,
                data,
                states: Self::create_default_animation(cur_offset, aspect_ratio, proportion, props.parallax_factor),
            };
            sprites.push(animation);
            engine.set_scenario(sprites);
            self.project_state.borrow_mut().add_sprite_properties(sprite_id, props);
            self.project_state.borrow_mut().select_sprite(sprite_id);
        }
    }

    fn create_default_animation(cur_offset: f32, aspect_ratio: f32, item_proportion: f32, parallax_factor: f32) -> Vec<SpriteState> {
        let start_y = 1.0 / aspect_ratio;
        let end_y = -1.0 / item_proportion;
        let start_offset = cur_offset - (start_y - end_y) * 0.5 * parallax_factor;
        let correction = if start_offset < 0.0 {-start_offset} else {0.0};
        let start_y = start_y - correction / parallax_factor;
        let start_offset = start_offset + correction;
        let end_offset = start_offset + (start_y - end_y) * parallax_factor;

        let state_zero = SpriteState {
            offset: start_offset,
            x: 0.0,
            y: start_y,
            width: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        let state_one = SpriteState {
            offset: end_offset,
            x: 0.0,
            y: end_y,
            width: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
        };
        vec![state_zero, state_one]
    }
    
    pub fn get_cur_tab(&self) -> usize {
        *self.project_state.borrow().cur_tab.read()
    }
    
    pub fn set_cur_tab(&self, tab: usize) {
        self.project_state.borrow_mut().cur_tab.set(tab);
    }
}
