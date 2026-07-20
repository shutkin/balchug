use std::collections::HashMap;
use balchug_common::atlas::{AtlasItem, FontData};
use balchug_common::sprite::{AnimationStates, Sprite, SpriteState, TextLine};

fn spline(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    y0 * (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2))
        + y1 * (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2))
        + y2 * (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1))
}

fn interpolate_sprite_state(s0: &SpriteState, s1: &SpriteState, s2: &SpriteState, offset: f32) -> SpriteState {
    SpriteState {
        offset,
        x: spline(s0.offset, s0.x, s1.offset, s1.x, s2.offset, s2.x, offset),
        y: spline(s0.offset, s0.y, s1.offset, s1.y, s2.offset, s2.y, offset),
        width: spline(s0.offset, s0.width, s1.offset, s1.width, s2.offset, s2.width, offset),
        color: [
            spline(s0.offset, s0.color[0], s1.offset, s1.color[0], s2.offset, s2.color[0], offset),
            spline(s0.offset, s0.color[1], s1.offset, s1.color[1], s2.offset, s2.color[1], offset),
            spline(s0.offset, s0.color[2], s1.offset, s1.color[2], s2.offset, s2.color[2], offset),
            spline(s0.offset, s0.color[3], s1.offset, s1.color[3], s2.offset, s2.color[3], offset),
        ],
    }
}

pub fn scale_sprite_state(state: &SpriteState, scale: f32) -> SpriteState {
    SpriteState {
        offset: state.offset,
        x: scale * state.x,
        y: scale * state.y,
        width: scale * state.width,
        color: state.color,
    }
}

pub fn interpolate_state(states: &AnimationStates, offset: f32) -> Option<SpriteState> {
    for index in 0 .. states.states.len() - 1 {
        if offset >= states.states[index].offset && offset <= states.states[index + 1].offset {
            return Some(interpolate_states(states, index, offset));
        }
    }
    None
}

fn interpolate_states(states: &AnimationStates, state_index: usize, offset: f32) -> SpriteState {
    let i0 = if state_index > 0 { state_index - 1 } else { 0 };
    let i1 = (i0 + 1).min(states.states.len() - 1);
    let i2 = (i0 + 2).min(states.states.len() - 1);
    interpolate_sprite_state(
        &states.states[i0],
        &states.states[i1],
        &states.states[i2],
        offset,
    )
}

pub fn arrange_text_line(line: &TextLine, cur_state: &SpriteState, font: &FontData, atlas_items: &HashMap<usize, AtlasItem>) -> Vec<Sprite> {
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
