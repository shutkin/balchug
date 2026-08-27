use balchug_common::sprite::{Easing, SpriteState};
use std::f32::consts::PI;

#[inline]
fn linear(x0: f32, x1: f32, y: f32) -> f32 {
    x0 + (x1 - x0) * y
}

#[inline]
pub fn state_y(state: &SpriteState, height: f32) -> f32 {
    if state.from_bottom {
        height - state.y
    } else {
        state.y
    }
}

pub fn scale_sprite_state(state: &SpriteState, scale: f32) -> SpriteState {
    SpriteState {
        offset: state.offset,
        x: scale * state.x,
        y: scale * state.y,
        from_bottom: state.from_bottom,
        width: scale * state.width,
        color: state.color,
        easing: state.easing,
    }
}

#[derive(Copy, Clone)]
pub struct SpriteUtil {
    height: f32,
}

impl SpriteUtil {
    pub fn new(width: f32, height: f32) -> Self {
        Self { height: height / width }
    }
    
    fn interpolate_sprite_2_states(&self, s0: &SpriteState, s1: &SpriteState, offset: f32, factor: f32) -> SpriteState {
        SpriteState {
            offset,
            x: linear(s0.x, s1.x, factor),
            y: linear(state_y(s0, self.height), state_y(s1, self.height), factor),
            from_bottom: false,
            width: linear(s0.width, s1.width, factor),
            color: [
                linear(s0.color[0] as f32, s1.color[0] as f32, factor).round().min(255.0) as u8,
                linear(s0.color[1] as f32, s1.color[1] as f32, factor).round().min(255.0) as u8,
                linear(s0.color[2] as f32, s1.color[2] as f32, factor).round().min(255.0) as u8,
                linear(s0.color[3] as f32, s1.color[3] as f32, factor).round().min(255.0) as u8,
            ],
            easing: s1.easing,
        }
    }

    fn interpolate_with_easing(&self, s0: &SpriteState, s1: &SpriteState, offset: f32, easing: Easing) -> SpriteState {
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
        self.interpolate_sprite_2_states(s0, s1, offset, ease)
    }

    pub fn interpolate_state(&self, states: &[SpriteState], offset: f32, smooth_factor: f32) -> Option<SpriteState> {
        for index in 0 .. states.len() - 1 {
            if offset >= states[index].offset && offset <= states[index + 1].offset {
                return Some(self.interpolate_states(states, index, offset, smooth_factor));
            }
        }
        None
    }

    fn interpolate_states(&self, states: &[SpriteState], state_index: usize, offset: f32, smooth_factor: f32) -> SpriteState {
        let next_index = (state_index + 1).min(states.len() - 1);
        let mut state = self.interpolate_with_easing(&states[state_index], &states[next_index], offset, states[next_index].easing);
        let (offset0, offset1) = (states[state_index].offset, states[next_index].offset);
        let (delta, center) = ((offset1 - offset0) * 0.5, (offset0 + offset1) * 0.5);
        if offset < center {
            if state_index > 0 {
                let state1 = self.interpolate_with_easing(&states[state_index - 1], &states[state_index], offset, Easing::Linear);
                let factor = smooth_factor * (center - offset) / delta;
                let factor = factor * factor;
                state = self.interpolate_sprite_2_states(&state, &state1, offset, factor);
            }
        } else {
            if next_index < states.len() - 1 {
                let state1 = self.interpolate_with_easing(&states[next_index], &states[next_index + 1], offset, Easing::Linear);
                let factor = smooth_factor * (offset - center) / delta;
                let factor = factor * factor;
                state = self.interpolate_sprite_2_states(&state, &state1, offset, factor);
            }
        }
        state
    }
}
