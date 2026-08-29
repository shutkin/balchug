use crate::controllers::api::Api;
use crate::controllers::group_utils::GroupUtils;
use crate::states::project_state::{ProjectState, SpriteGroup};
use balchug_common::atlas::Atlas;
use balchug_common::sprite::{Easing, SpriteState};
use balchug_engine::BalchugEngine;
use dioxus::html::FileData;
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct ResourcesController {
    api: Api,
    thumbs: Signal<Vec<String>>,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_state: ProjectState,
    adding_image_id: Signal<Option<usize>>,
    text_edit_open: Signal<bool>,
    edit_group_signal: Signal<Option<usize>>,
    groups_update_signal: Rc<Cell<bool>>,
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
    ) -> Self {
        Self {
            api,
            engine,
            project_state,
            thumbs: Default::default(),
            adding_image_id: Signal::new(None),
            text_edit_open: Signal::new(false),
            edit_group_signal: Signal::new(None),
            groups_update_signal,
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

    pub fn get_edit_group_signal(&self) -> Signal<Option<usize>> {
        self.edit_group_signal
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

    pub fn get_group(&self, group_id: usize) -> SpriteGroup {
        self.project_state.get_group(group_id)
    }

    pub fn update_group(&mut self, group_id: usize, group: &SpriteGroup) {
        self.project_state.update_group(group_id, group);
        if let Some(engine) = self.engine.borrow().as_ref() {
            engine.set_scenario(GroupUtils::groups_to_sprites(&self.project_state.get_groups(), engine));
        }
        self.groups_update_signal.set(true);
    }

    pub fn add_new_group_animation(&mut self, template: &SpriteGroup) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let offset = engine.get_offset();

            let aspect_ratio = *self.project_state.aspect_ratio.read();

            let color = self.project_state.properties.read().default_text_color;
            let cur_state = SpriteState {
                offset,
                x: 0.0,
                y: 0.5 / aspect_ratio,
                from_bottom: true,
                width: 1.0,
                color: [color[0], color[1], color[2], 255],
                easing: Easing::default(),
            };

            let (proportion_x, proportion_y) = GroupUtils::group_proportion(engine, &template);
            let (first, last) = GroupUtils::create_init_and_final_states(
                &cur_state,
                template.parallax_factor,
                aspect_ratio,
                proportion_x / proportion_y,
                true,
            );
            let mut group = template.clone();
            group.states = vec![first, last];

            self.project_state.add_group(group);
            let groups = self.project_state.get_groups();
            engine.set_scenario(GroupUtils::groups_to_sprites(&groups, engine));

            let new_group_id = groups.len() - 1;
            self.project_state.select_group(new_group_id);
            self.groups_update_signal.set(true);
        }
    }
    
    pub fn get_cur_tab(&self) -> usize {
        *self.project_state.cur_tab.read()
    }
    
    pub fn set_cur_tab(&mut self, tab: usize) {
        self.project_state.cur_tab.set(tab);
    }
}
