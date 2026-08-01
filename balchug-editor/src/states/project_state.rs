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
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            aspect_ratio: Store::new(9.0 / 16.0),
            sprite_properties: Store::new(HashMap::new()),
        }
    }
    
    pub fn get_sprite_properties(&self, sprite_id: usize) -> SpriteProperties {
        self.sprite_properties.read()
            .get(&sprite_id).cloned().unwrap_or_default()
    }

    pub fn add_sprite_properties(&mut self, sprite_id: usize, properties: SpriteProperties) {
        self.sprite_properties.with_mut(move |map| {
            map.insert(sprite_id, properties);
        });
    }
}