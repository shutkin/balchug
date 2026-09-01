use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, Debug, Hash)]
pub struct AtlasItem {
    pub id: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub origin_width: u32,
    pub origin_height: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub items: HashMap<usize, AtlasItem>,
}

impl Default for Atlas {
    fn default() -> Self {
        Self {
            width: 4,
            height: 4,
            items: HashMap::new(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct FontGlyph {
    pub item_id: usize,
    pub h_advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

#[derive(Clone, Default)]
pub struct FontData {
    pub ascend: f32,
    pub height: f32,
    pub line_gap: f32,
    pub space_width: f32,
    pub glyphs: HashMap<char, FontGlyph>,
}
