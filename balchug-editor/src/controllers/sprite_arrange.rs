use balchug_common::sprite::SpriteState;

pub struct SpriteArrange;

impl SpriteArrange {
    pub fn create_init_and_final_states(
        cur_state: &SpriteState,
        parallax_factor: f32,
        screen_aspect_ratio: f32,
        item_proportion: f32,
        first_is_from_bottom: bool,
        relation: (f32, f32)
    ) -> (SpriteState, SpriteState) {
        let cur_y = if cur_state.from_bottom {1.0 / screen_aspect_ratio - cur_state.y} else {cur_state.y};
        let cur_y = cur_y + relation.1;
        let end_y = -cur_state.width / item_proportion;
        let end_offset = cur_state.offset + (cur_y - end_y) * parallax_factor;
        let start_y = 1.0 / screen_aspect_ratio;
        let start_offset = cur_state.offset - (start_y - cur_y) * parallax_factor;
        let correction = if start_offset < 0.0 {-start_offset} else {0.0};
        let start_y = start_y - correction / parallax_factor;
        let start_offset = start_offset + correction;

        let first_state = SpriteState {
            offset: start_offset,
            x: cur_state.x + relation.0,
            y: if first_is_from_bottom {1.0 / screen_aspect_ratio - start_y} else {start_y},
            from_bottom: first_is_from_bottom,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        let last_state = SpriteState {
            offset: end_offset,
            x: cur_state.x + relation.0,
            y: end_y,
            from_bottom: false,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        (first_state, last_state)
    }
}