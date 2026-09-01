use serde::{Deserialize, Serialize};
use crate::atlas::AtlasItem;


#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum Easing {
    #[default]
    Linear,
    InSine, OutSine, InOutSine, InCubic, OutCubic, InOutCubic,
}

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
    pub from_bottom: bool,
    pub scale: f32,
    pub color: [u8; 4],
    pub easing: Easing,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SpriteAnimation {
    pub sprite_id: usize,
    pub data: SpriteData,
    pub states: Vec<SpriteState>,
    pub smooth_factor: f32,
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
    pub font: usize,
    pub text: String,
    pub size: u8,
}
