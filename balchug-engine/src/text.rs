use std::collections::HashMap;
use balchug_common::atlas::{AtlasItem, FontData};
use balchug_common::sprite::{AnimationStates, Sprite, SpriteState};

#[derive(Clone)]
pub struct TextLine {
    pub text: String,
    pub animation: AnimationStates,
    pub relative_height: f32,
}

impl TextLine {
    pub fn arrange(&self, cur_state: &SpriteState, font: &FontData, atlas_items: &HashMap<usize, AtlasItem>) -> Vec<Sprite> {
        let mut result = Vec::new();
        let scale = cur_state.height * self.relative_height / font.height;
        let (mut cx, cy) = (cur_state.x, cur_state.y + font.ascend * scale);
        for c in self.text.chars() {
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
                    width: item.origin_width as f32 * scale,
                    height: item.origin_height as f32 * scale,
                };
                result.push(Sprite {
                    atlas_item: *item,
                    state: glyph_state,
                });
                cx += glyph.h_advance * scale;
            }
        }

        /*web_sys::console::log_1(&format!(
            "Glyphs sprites: {:?}",
            result.iter().map(|g| format!("({} {} {}x{})", g.atlas_item.x, g.atlas_item.y, g.atlas_item.width, g.atlas_item.height)).collect::<String>()
        ).into());*/
        result
    }
}
