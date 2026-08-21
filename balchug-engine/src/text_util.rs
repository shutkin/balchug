use std::collections::HashMap;
use balchug_common::atlas::{AtlasItem, FontData};
use balchug_common::sprite::{Sprite, SpriteState, SpriteTextData};
use crate::sprite_util::state_y;
use crate::TEXT_SIZE_FACTOR;

#[derive(Copy, Clone)]
pub struct TextUtil {
    height: f32,
}

impl TextUtil {
    pub fn new(width: f32, height: f32) -> Self {
        Self { height: height / width }
    }

    pub fn arrange_text_line(&self, line: &SpriteTextData, cur_state: &SpriteState, font: &FontData, atlas_items: &HashMap<usize, AtlasItem>) -> Vec<Sprite> {
        let mut result = Vec::new();
        let scale = cur_state.width * line.size as f32 * TEXT_SIZE_FACTOR / font.height;
        let (mut cx, cy) = (cur_state.x, state_y(cur_state, self.height) + font.ascend * scale);
        for c in line.text.chars() {
            if c.is_control() {
                continue;
            }
            if c.is_whitespace() {
                cx += font.space_width * scale;
                continue;
            }
            if let Some(glyph) = font.glyphs.get(&c) && let Some(item) = atlas_items.get(&glyph.item_id) {
                let glyph_state = SpriteState {
                    offset: cur_state.offset,
                    color: cur_state.color,
                    x: cx + glyph.offset_x * scale,
                    y: cy + glyph.offset_y * scale,
                    from_bottom: false,
                    width: item.origin_width as f32 * scale,
                    easing: cur_state.easing,
                };
                result.push(Sprite {
                    atlas_item: *item,
                    state: glyph_state,
                });
                cx += glyph.h_advance * scale;
            }
        }
        result
    }
}

pub fn measure_text_line(text: &str, size: u8, scale: f32, font: &FontData) -> (f32, f32) {
    let scale = scale * size as f32 * TEXT_SIZE_FACTOR / font.height;
    let mut cx = 0.0;
    for c in text.chars() {
        if c.is_control() {
            continue;
        }
        if c.is_whitespace() {
            cx += font.space_width * scale;
            continue;
        }
        if let Some(glyph) = font.glyphs.get(&c) {
            cx += glyph.h_advance * scale;
        }
    }
    (cx, (font.height + font.line_gap) * scale)
}
