use std::collections::HashMap;
use dioxus::prelude::*;
use balchug_common::api::ProjectProperties;

#[derive(Clone, PartialEq)]
pub struct SpriteGroupProperties {
    pub main_sprite_id: usize,
    pub sprites: Vec<usize>,
    pub title: String,
    pub parallax_factor: f32,
    pub relations: HashMap<usize, (f32, f32)>,
}

impl Default for SpriteGroupProperties {
    fn default() -> Self {
        Self {
            main_sprite_id: 0,
            sprites: Vec::new(),
            title: String::new(),
            parallax_factor: 1.0,
            relations: HashMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct ProjectState {
    pub properties: Store<ProjectProperties>,
    pub aspect_ratio: Store<f32>,
    pub sprite_group_properties: Store<HashMap<usize, SpriteGroupProperties>>,
    pub cur_tab: Signal<usize>,
    pub selected_sprite_group: Signal<Option<usize>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            properties: Store::new(ProjectProperties::default()),
            aspect_ratio: Store::new(9.0 / 16.0),
            sprite_group_properties: Store::new(HashMap::new()),
            cur_tab: Signal::new(1),
            selected_sprite_group: Signal::new(None),
        }
    }
    
    pub fn get_group_properties(&self, group_id: usize) -> SpriteGroupProperties {
        self.sprite_group_properties.read()
            .get(&group_id).cloned().unwrap_or_default()
    }

    pub fn add_group_properties(&mut self, group_id: usize, properties: SpriteGroupProperties) {
        self.sprite_group_properties.with_mut(move |map| {
            map.insert(group_id, properties);
        })
    }
    
    pub fn select_sprite_group(&mut self, sprite_group_id: usize) {
        self.selected_sprite_group.set(Some(sprite_group_id));
        self.cur_tab.set(2);
    }
    
    pub fn unselect_group(&mut self) {
        self.selected_sprite_group.set(None);
    }
}