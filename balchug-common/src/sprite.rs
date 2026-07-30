use serde::{Deserialize, Serialize};
use crate::atlas::AtlasItem;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub state: SpriteState,
    pub atlas_item: AtlasItem,
}

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SpriteState {
    pub offset: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub color: [f32; 4],
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpriteAnimation {
    pub sprite_id: usize,
    pub atlas_item_id: usize,
    pub states: Vec<SpriteState>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct TextLine {
    pub text: String,
    pub relative_height: f32,
    pub animation: Vec<SpriteState>,
}
