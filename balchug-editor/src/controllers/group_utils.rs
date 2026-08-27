use crate::states::project_state::SpriteGroup;
use balchug_common::sprite::{SpriteAnimation, SpriteData, SpriteState};
use balchug_engine::{BalchugEngine, TEXT_SIZE_FACTOR};
use std::collections::HashMap;

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

    pub fn groups_to_sprites(groups: &[SpriteGroup]) -> Vec<SpriteAnimation> {
        let mut sprites = Vec::new();
        for group in groups {
            let sprite = SpriteAnimation {
                sprite_id: sprites.len(),
                data: group.data.clone(),
                states: group.states.clone(),
                smooth_factor: group.smooth_factor,
            };
            sprites.push(sprite);
        }
        sprites
    }
    
    pub fn apply_relation_to_states(animation: &mut SpriteAnimation, root_sprite: &SpriteAnimation, relations: &[HashMap<usize, (f32, f32)>]) {
        if animation.sprite_id == root_sprite.sprite_id {
            return;
        }
        for (i, root_state) in root_sprite.states.iter().enumerate() {
            if let Some(state) = animation.states.get_mut(i)
                && let Some(state_relations) = relations.get(i)
                && let Some(relation) = state_relations.get(&animation.sprite_id) {
                state.x = root_state.x + relation.0;
                state.y = root_state.y + relation.1;
            }
        }
    }

    pub fn group_proportion(engine: &BalchugEngine, group: &SpriteGroup) -> f32 {
        match &group.data {
            SpriteData::Image(image_data) => {
                engine.get_atlas_item(image_data.atlas_item_id).map(|item| {
                    item.origin_width as f32 / item.origin_height as f32
                }).unwrap_or(1.0)
            }
            SpriteData::Text(text_data) => {
                1.0 / (text_data.size as f32 * TEXT_SIZE_FACTOR)
            }
        }
    }

    /*pub fn calculate_text_relation(
        engine: &BalchugEngine,
        sprites: &[SpriteAnimation],
    ) -> Vec<HashMap<usize, (f32, f32)>> {
        let mut result = Vec::with_capacity(sprites[0].states.len());
        for state in &sprites[0].states {
            let mut relations = HashMap::with_capacity(sprites.len());
            let mut rel_y = 0.0;
            for sprite in sprites {
                let proportion = Self::group_proportion(engine, sprite);
                relations.insert(sprite.sprite_id, (0.0, rel_y));
                rel_y += state.width / proportion;
            }
            result.push(relations);
        }
        result
    }*/
}