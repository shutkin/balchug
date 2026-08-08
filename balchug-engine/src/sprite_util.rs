use std::collections::HashMap;
use std::f32::consts::PI;
use balchug_common::atlas::{AtlasItem, FontData};
use balchug_common::sprite::{Easing, Sprite, SpriteAnimation, SpriteState, SpriteTextData};

#[inline]
fn linear(x0: f32, x1: f32, y: f32) -> f32 {
    x0 + (x1 - x0) * y
}

// linear(s0.offset, s1.offset, factor)
fn interpolate_sprite_2_states(s0: &SpriteState, s1: &SpriteState, offset: f32, factor: f32) -> SpriteState {
    SpriteState {
        offset,
        x: linear(s0.x, s1.x, factor),
        y: linear(s0.y, s1.y, factor),
        width: linear(s0.width, s1.width, factor),
        color: [
            linear(s0.color[0], s1.color[0], factor),
            linear(s0.color[1], s1.color[1], factor),
            linear(s0.color[2], s1.color[2], factor),
            linear(s0.color[3], s1.color[3], factor),
        ],
        easing: s1.easing,
    }
}

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
    interpolate_sprite_2_states(s0, s1, offset, ease)
    /*SpriteState {
        offset,
        x: linear(s0.x, s1.x, ease),
        y: linear(s0.y, s1.y, ease),
        width: linear(s0.width, s1.width, ease),
        color: [
            linear(s0.color[0], s1.color[0], ease),
            linear(s0.color[1], s1.color[1], ease),
            linear(s0.color[2], s1.color[2], ease),
            linear(s0.color[3], s1.color[3], ease),
        ],
        easing: s1.easing,
    }*/
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

pub fn interpolate_state(animation: &SpriteAnimation, offset: f32) -> Option<SpriteState> {
    for index in 0 .. animation.states.len() - 1 {
        if offset >= animation.states[index].offset && offset <= animation.states[index + 1].offset {
            return Some(interpolate_states(&animation.states, index, offset, animation.smooth_factor));
        }
    }
    None
}

fn interpolate_states(states: &[SpriteState], state_index: usize, offset: f32, smooth_factor: f32) -> SpriteState {
    let next_index = (state_index + 1).min(states.len() - 1);
    let mut state = interpolate_with_easing(&states[state_index], &states[next_index], offset, states[next_index].easing);
    let (offset0, offset1) = (states[state_index].offset, states[next_index].offset);
    let (delta, center) = ((offset1 - offset0) * 0.5, (offset0 + offset1) * 0.5);
    if offset < center {
        if state_index > 0 {
            let state1 = interpolate_with_easing(&states[state_index - 1], &states[state_index], offset, Easing::Linear);
            let factor = smooth_factor * (center - offset) / delta; // (1.0 - (offset - offset0) / ((offset1 - offset0) * 0.5)) * smooth_factor;
            let factor = factor * factor;
            state = interpolate_sprite_2_states(&state, &state1, offset, factor);
        }
    } else {
        if next_index < states.len() - 1 {
            let state1 = interpolate_with_easing(&states[next_index], &states[next_index + 1], offset, Easing::Linear);
            let factor = smooth_factor * (offset - center) / delta; // (1.0 - (offset1 - offset) / ((offset1 - offset0) * 0.5)) * smooth_factor;
            let factor = factor * factor;
            state = interpolate_sprite_2_states(&state, &state1, offset, factor);
        }
    }
    state
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

pub fn measure_text_line(text: &str, height: f32, scale: f32, font: &FontData) -> f32 {
    let scale = scale * height / font.height;
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
    cx
}
