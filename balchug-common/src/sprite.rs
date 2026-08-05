use serde::{Deserialize, Serialize};
use crate::atlas::AtlasItem;

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Easing {
    Linear, InCubic, OutCubic, InOutCubic,
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Linear
    }
}

#[derive(Copy, Clone)]
pub struct Sprite {
    pub state: SpriteState,
    pub atlas_item: AtlasItem,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Easing {
    Linear, InSine, OutSine, InOutSine, InCubic, OutCubic, InOutCubic,
}

impl Default for Easing {
    fn default() -> Self {
        Easing::Linear
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SpriteState {
    pub offset: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub color: [f32; 4],
    pub easing: Easing,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpriteAnimation {
    pub sprite_id: usize,
    pub data: SpriteData,
    pub states: Vec<SpriteState>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum SpriteData {
    Image(SpriteImageData),
    Text(SpriteTextData),
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpriteImageData {
    pub atlas_item_id: usize,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpriteTextData {
    pub text: String,
    pub relative_height: f32,
}
