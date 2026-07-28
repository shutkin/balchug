use std::collections::HashMap;
use dioxus::prelude::*;

#[derive(Clone)]
pub struct ProjectState {
    pub aspect_ratio: Store<f32>,
    pub sprite_parallax_factors: Store<HashMap<usize, f32>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            aspect_ratio: Store::new(9.0 / 16.0),
            sprite_parallax_factors: Store::new(HashMap::new()),
        }
    }

    pub fn add_parallax_factor(&mut self, sprite_id: usize, factor: f32) {
        self.sprite_parallax_factors.with_mut(move |map| {
            map.insert(sprite_id, factor);
        });
    }
}