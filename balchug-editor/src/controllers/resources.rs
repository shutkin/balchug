use crate::controllers::api::Api;
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use balchug_common::atlas::Atlas;
use balchug_common::sprite::{SpriteAnimation, SpriteState, Easing};
use crate::controllers::sprite_editor::SpriteEditController;
use crate::states::project_state::{ProjectState, SpriteProperties};

#[derive(Clone)]
pub struct ResourcesController {
    api: Api,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_state: ProjectState,
    adding_image_id: Signal<Option<usize>>,
    text_edit_open: Signal<bool>,
    edit_sprite_signal: Signal<Option<usize>>,
    sprites_update_signal: Rc<Cell<bool>>,
    scenario_update_signal: Rc<Cell<bool>>,
}

impl PartialEq for ResourcesController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ResourcesController {
    pub fn new(
        api: Api,
        engine: Rc<RefCell<Option<BalchugEngine>>>,
        project_state: ProjectState,
        sprites_update_signal: Rc<Cell<bool>>,
        scenario_update_signal: Rc<Cell<bool>>,
    ) -> Self {
        Self {
            api,
            engine,
            project_state,
            thumbs: Default::default(),
            adding_image_id: Signal::new(None),
            text_edit_open: Signal::new(false),
            edit_sprite_signal: Signal::new(None),
            sprites_update_signal,
            scenario_update_signal,
        }
    }
    
    pub fn get_thumbs(&self) -> Vec<String> {
        self.thumbs.read().clone()
    }
    
    pub fn get_image_adding_signal(&self) -> Signal<Option<usize>> {
        self.adding_image_id
    }

    pub fn get_text_adding_open(&self) -> Signal<bool> {
        self.text_edit_open
    }

    pub fn get_edit_sprite_signal(&self) -> Signal<Option<usize>> {
        self.edit_sprite_signal
    }
    
    pub async fn handle_upload(&mut self, files: Vec<FileData>) {
        if let Some(file_data) = files.first()
            && let Ok(bytes) = file_data.read_bytes().await {
            let mime_type = match infer::get(&bytes) {
                Some(kind) => kind.mime_type(),
                None => "application/octet-stream", // Safe fallback
            };
            if let Some((new_thumbs, atlas)) = self.api.upload_image(bytes, mime_type).await {
                self.update_image_resources(new_thumbs, atlas);
            }
        }
    }

    pub fn update_image_resources(&mut self, new_thumbs: Vec<String>, atlas: Atlas) {
        self.thumbs.set(new_thumbs.iter().map(|thumb| self.api.assets_url(thumb)).collect());
        if let Some(engine) = self.engine.borrow().as_ref() {
            let now = instant::now().round() as u64;
            let img_url = self.api.assets_url("atlas.webp");
            engine.set_atlas(&format!("{img_url}?{now}"), atlas);
        }
    }

    pub fn get_sprite_props(&self, sprite_id: usize) -> Option<(SpriteProperties, SpriteAnimation)> {
        let props = self.project_state.sprite_properties.read().get(&sprite_id).cloned()?;
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(animation) = engine.get_sprites_animations(Some(sprite_id)).first().cloned() {
            Some((props, animation))
        } else {
            None
        }
    }

    pub fn set_sprite_props(&mut self, sprite_id: usize, props: &SpriteProperties, animation: &SpriteAnimation) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut animations = engine.get_sprites_animations(None);
            for sprite_animation in &mut animations {
                if sprite_animation.sprite_id == sprite_id {
                    *sprite_animation = animation.clone();
                }
            }
            engine.set_scenario(animations);
            self.scenario_update_signal.set(true);
        }
        self.project_state.sprite_properties.insert(sprite_id, props.clone());
        self.sprites_update_signal.set(true);
    }

    pub fn add_new_sprite_animation(&mut self, template: SpriteAnimation, props: SpriteProperties) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let proportion = SpriteEditController::sprite_proportion(engine, &template);

            let mut sprites = engine.get_sprites_animations(None);
            let sprite_id = sprites.len();
            let cur_offset = engine.get_offset();
            let aspect_ratio = *self.project_state.aspect_ratio.read();
            let animation = SpriteAnimation {
                sprite_id,
                data: template.data,
                smooth_factor: template.smooth_factor,
                states: Self::create_default_animation(cur_offset, aspect_ratio, proportion, props.parallax_factor),
            };
            sprites.push(animation);
            engine.set_scenario(sprites);
            self.project_state.add_sprite_properties(sprite_id, props);
            self.project_state.select_sprite(sprite_id);
            self.sprites_update_signal.set(true);
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
            y: 1.0 / aspect_ratio - start_y,
            from_bottom: true,
            width: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            easing: Easing::default(),
        };
        let state_one = SpriteState {
            offset: end_offset,
            x: 0.0,
            y: end_y,
            from_bottom: false,
            width: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            easing: Easing::default(),
        };
        vec![state_zero, state_one]
    }
    
    pub fn get_cur_tab(&self) -> usize {
        *self.project_state.cur_tab.read()
    }
    
    pub fn set_cur_tab(&mut self, tab: usize) {
        self.project_state.cur_tab.set(tab);
    }
}
