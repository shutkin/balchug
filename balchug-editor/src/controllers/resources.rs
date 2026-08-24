use crate::controllers::api::Api;
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use balchug_common::atlas::Atlas;
use balchug_common::sprite::{SpriteAnimation, SpriteState, Easing};
use crate::controllers::group_utils::GroupUtils;
use crate::states::project_state::{ProjectState, SpriteGroupProperties};

#[derive(Clone)]
pub struct ResourcesController {
    api: Api,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_state: ProjectState,
    adding_image_id: Signal<Option<usize>>,
    text_edit_open: Signal<bool>,
    edit_sprite_signal: Signal<Option<usize>>,
    groups_update_signal: Rc<Cell<bool>>,
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
        groups_update_signal: Rc<Cell<bool>>,
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
            groups_update_signal,
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

    pub fn get_sprite_props(&self, sprite_id: usize) -> Option<(SpriteGroupProperties, SpriteAnimation)> {
        let props = self.project_state.sprite_group_properties.read().get(&sprite_id).cloned()?;
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(animation) = engine.get_sprites_animations(Some(sprite_id)).first().cloned() {
            Some((props, animation))
        } else {
            None
        }
    }

    pub fn set_sprite_props(&mut self, sprite_id: usize, props: &SpriteGroupProperties, animation: &SpriteAnimation) {
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
        self.project_state.sprite_group_properties.insert(sprite_id, props.clone());
        self.groups_update_signal.set(true);
    }

    pub fn add_new_sprite_animation(&mut self, templates: Vec<SpriteAnimation>, mut props: SpriteGroupProperties) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut sprites = engine.get_sprites_animations(None);
            props.main_sprite_id = sprites.len();
            props.sprites = vec![];
            let offset = engine.get_offset();

            let aspect_ratio = *self.project_state.aspect_ratio.read();
            let mut sprite_id = sprites.len();
            let mut new_sprites = Vec::with_capacity(templates.len());
            for template in templates {
                if sprite_id > props.main_sprite_id {
                    props.sprites.push(sprite_id);
                }

                let color = self.project_state.properties.read().default_text_color;
                let cur_state = SpriteState {
                    offset,
                    x: 0.0,
                    y: 0.5 / aspect_ratio,
                    from_bottom: false,
                    width: 1.0,
                    color: [color[0], color[1], color[2], 255],
                    easing: Easing::default(),
                };

                let animation = SpriteAnimation {
                    sprite_id,
                    data: template.data,
                    smooth_factor: template.smooth_factor,
                    states: vec![cur_state, cur_state],
                };
                 new_sprites.push(animation);

                sprite_id += 1;
            }

            let relations = GroupUtils::calculate_text_relation(engine, &new_sprites);
            for mut new_sprite in new_sprites {
                let proportion = GroupUtils::sprite_proportion(engine, &new_sprite);
                let cur_state = &new_sprite.states[0];
                let (first, last) = GroupUtils::create_init_and_final_states(
                    cur_state,
                    props.parallax_factor,
                    aspect_ratio,
                    proportion,
                    false,
                );
                new_sprite.states = vec![first, last];
                sprites.push(new_sprite);
            }
            props.relations = relations;

            engine.set_scenario(sprites);
            let group_id = self.project_state.sprite_group_properties.len();
            self.project_state.add_group_properties(group_id, props);
            self.project_state.select_sprite_group(group_id);
            self.groups_update_signal.set(true);
            self.scenario_update_signal.set(true);
        }
    }
    
    pub fn get_cur_tab(&self) -> usize {
        *self.project_state.cur_tab.read()
    }
    
    pub fn set_cur_tab(&mut self, tab: usize) {
        self.project_state.cur_tab.set(tab);
    }
}
