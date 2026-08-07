use std::collections::HashMap;
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct SpriteProperties {
    pub title: String,
    pub parallax_factor: f32,
}

impl Default for SpriteProperties {
    fn default() -> Self {
        Self {
            title: String::new(),
            parallax_factor: 1.0,
        }
    }
}

#[derive(Clone)]
pub struct ProjectState {
    pub aspect_ratio: Store<f32>,
    pub sprite_properties: Store<HashMap<usize, SpriteProperties>>,
    pub cur_tab: Signal<usize>,
    pub selected_sprite: Signal<Option<usize>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            aspect_ratio: Store::new(9.0 / 16.0),
            sprite_properties: Store::new(HashMap::new()),
            cur_tab: Signal::new(0),
            selected_sprite: Signal::new(None),
        }
    }
    
    pub fn get_sprite_properties(&self, sprite_id: usize) -> SpriteProperties {
        self.sprite_properties.read()
            .get(&sprite_id).cloned().unwrap_or_default()
    }

    pub fn add_sprite_properties(&mut self, sprite_id: usize, properties: SpriteProperties) {
        self.sprite_properties.with_mut(move |map| {
            map.insert(sprite_id, properties);
        })
    }
    
    pub fn select_sprite(&mut self, sprite_id: usize) {
        self.selected_sprite.set(Some(sprite_id));
        self.cur_tab.set(1);
    }
    
    pub fn unselect_sprite(&mut self) {
        self.selected_sprite.set(None);
    }
}