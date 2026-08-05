use std::collections::HashMap;
use std::f32::consts::PI;
use balchug_common::atlas::{AtlasItem, FontData};
use balchug_common::sprite::{Easing, Sprite, SpriteState, SpriteTextData};

fn interpolate_with_easing(s0: &SpriteState, s1: &SpriteState, offset: f32, easing: Easing) -> SpriteState {
    let x = (offset - s0.offset) / (s1.offset - s0.offset);
    let ease = match easing {
        Easing::Linear => x,
        Easing::InSine => 1.0 - (x * PI / 2.0).cos(),
        Easing::OutSine => (x * PI / 2.0).sin(),
        Easing::InOutSine => -((x * PI).cos() - 1.0) / 2.0,
        Easing::InCubic => x * x * x,
        Easing::OutCubic => 1.0 - (1.0 - x).powi(3),
        Easing::InOutCubic => if x < 0.5 {4.0 * x * x * x} else {1.0 - (-2.0 * x + 2.0).powi(3) / 2.0},
    };
    SpriteState {
        offset,
        x: s0.x + (s1.x - s0.x) * ease,
        y: s0.y + (s1.y - s0.y) * ease,
        width: s0.width + (s1.width - s0.width) * ease,
        color: [
            s0.color[0] + (s1.color[0] - s0.color[0]) * ease,
            s0.color[1] + (s1.color[1] - s0.color[1]) * ease,
            s0.color[2] + (s1.color[2] - s0.color[2]) * ease,
            s0.color[3] + (s1.color[3] - s0.color[3]) * ease,
        ],
        easing: s1.easing,
    }
}

pub fn scale_sprite_state(state: &SpriteState, scale: f32) -> SpriteState {
    SpriteState {
        offset: state.offset,
        x: scale * state.x,
        y: scale * state.y,
        width: scale * state.width,
        color: state.color,
        easing: state.easing,
    }
}

pub fn interpolate_state(states: &[SpriteState], offset: f32) -> Option<SpriteState> {
    for index in 0 .. states.len() - 1 {
        if offset >= states[index].offset && offset <= states[index + 1].offset {
            return Some(interpolate_states(states, index, offset));
        }
    }
    None
}

fn interpolate_states(states: &[SpriteState], state_index: usize, offset: f32) -> SpriteState {
    let next_index = (state_index + 1).min(states.len() - 1);
    interpolate_with_easing(&states[state_index], &states[next_index], offset, states[next_index].easing)
}

pub fn arrange_text_line(line: &SpriteTextData, cur_state: &SpriteState, font: &FontData, atlas_items: &HashMap<usize, AtlasItem>) -> Vec<Sprite> {
    let mut result = Vec::new();
    let scale = cur_state.width * line.relative_height / font.height;
    let (mut cx, cy) = (cur_state.x, cur_state.y + font.ascend * scale);
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
