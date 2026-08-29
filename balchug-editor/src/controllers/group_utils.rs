use crate::states::project_state::SpriteGroup;
use balchug_common::sprite::{SpriteAnimation, SpriteData, SpriteState, SpriteTextData};
use balchug_engine::{BalchugEngine, TEXT_SIZE_FACTOR};

pub struct GroupUtils;

impl GroupUtils {
    pub fn create_init_and_final_states(
        cur_state: &SpriteState,
        parallax_factor: f32,
        screen_aspect_ratio: f32,
        item_proportion: f32,
        first_is_from_bottom: bool,
    ) -> (SpriteState, SpriteState) {
        let cur_y = if cur_state.from_bottom {1.0 / screen_aspect_ratio - cur_state.y} else {cur_state.y};
        let end_y = -cur_state.width / item_proportion;
        let end_offset = cur_state.offset + (cur_y - end_y) * parallax_factor;
        let start_y = 1.0 / screen_aspect_ratio;
        let start_offset = cur_state.offset - (start_y - cur_y) * parallax_factor;
        let correction = if start_offset < 0.0 {-start_offset} else {0.0};
        let start_y = start_y - correction / parallax_factor;
        let start_offset = start_offset + correction;

        let first_state = SpriteState {
            offset: start_offset,
            x: cur_state.x,
            y: if first_is_from_bottom {1.0 / screen_aspect_ratio - start_y} else {start_y},
            from_bottom: first_is_from_bottom,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        let last_state = SpriteState {
            offset: end_offset,
            x: cur_state.x,
            y: end_y,
            from_bottom: false,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        (first_state, last_state)
    }

    pub fn groups_to_sprites(groups: &[SpriteGroup], engine: &BalchugEngine) -> Vec<SpriteAnimation> {
        let mut sprites = Vec::new();
        for group in groups {
            let groups_sprites = match group.data.clone() {
                SpriteData::Image(data) => vec![SpriteAnimation {
                    sprite_id: sprites.len(),
                    data: SpriteData::Image(data),
                    states: group.states.clone(),
                    smooth_factor: group.smooth_factor,
                }],
                SpriteData::Text(data) => {
                    let lines = Self::split_text(&data, group, engine);
                    Self::create_text_sprites(lines, &data, group, sprites.len())
                }
            };
            sprites.extend(groups_sprites);
        }
        sprites
    }

    pub fn group_proportion(engine: &BalchugEngine, group: &SpriteGroup) -> (f32, f32) {
        match &group.data {
            SpriteData::Image(image_data) => {
                engine.get_atlas_item(image_data.atlas_item_id).map(|item| {
                    (item.origin_width as f32, item.origin_height as f32)
                }).unwrap_or((1.0, 1.0))
            }
            SpriteData::Text(text_data) => {
                let lines = Self::split_text(text_data, group, engine);
                let max_width = lines.iter().map(|(_, width)| *width).reduce(f32::max).unwrap_or(1.0);
                (max_width, lines.len() as f32 * text_data.size as f32 * TEXT_SIZE_FACTOR)
            }
        }
    }

    fn split_text(data: &SpriteTextData, group: &SpriteGroup, engine: &BalchugEngine) -> Vec<(String, f32)> {
        let space_width = engine.measure_text(&SpriteTextData { text: " ".to_string(), size: data.size }, 1.0).0;
        let words = data.text.split(' ')
            .filter(|word| !word.is_empty())
            .map(|word| {
                let word_data = SpriteTextData { text: word.to_string(), size: data.size };
                let width = engine.measure_text(&word_data, 1.0).0;
                (word.to_string(), width)
            })
            .collect::<Vec<_>>();
        let mut lines = Vec::new();
        let mut line = String::new();
        let mut line_width = 0.0;
        for (word, word_width) in words {
            if line_width + space_width + word_width > group.max_width {
                lines.push((line.clone(), line_width));
                line = String::new();
                line_width = 0.0;
            }
            if !line.is_empty() {
                line.push(' ');
                line_width += space_width;
            }
            line.push_str(&word);
            line_width += word_width;
        }
        if !line.is_empty() {
            lines.push((line, line_width));
        }
        lines
    }
    
    fn create_text_sprites(lines: Vec<(String, f32)>, data: &SpriteTextData, group: &SpriteGroup, start_id: usize) -> Vec<SpriteAnimation> {
        lines.into_iter().enumerate()
            .map(|(i, (line, _))| {
                let dy = i as f32 * data.size as f32 * TEXT_SIZE_FACTOR;
                SpriteAnimation {
                    sprite_id: start_id + i,
                    data: SpriteData::Text(SpriteTextData {text: line, size: data.size}),
                    states: Self::translate_states(&group.states, dy),
                    smooth_factor: group.smooth_factor,
                }
            })
            .collect()
    }

    fn translate_states(states: &[SpriteState], dy: f32) -> Vec<SpriteState> {
        states.iter().map(|state| {
            let mut state = *state;
            if state.from_bottom {
                state.y -= state.width * dy;
            } else {
                state.y += state.width * dy;
            }
            state
        }).collect()
    }
}