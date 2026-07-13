use std::collections::HashMap;
use balchug_common::atlas::AtlasItem;
use balchug_common::sprite::{AnimationStates, SpriteAnimation, SpriteState};
use crate::text::TextLine;

#[derive(Clone, Default)]
pub struct Scenario {
    pub images: Vec<SpriteAnimation>,
    pub text_lines: Vec<TextLine>,
}

pub fn build_scenario(items: &HashMap<usize, AtlasItem>, pictures: &[usize]) -> Scenario {
    let mut scenario = Scenario::default();
    let mut offset = 0.0;
    let mut sprite_id = 0;
    for atlas_item_id in pictures {
        if let Some(item) = items.get(atlas_item_id) {
            let item_height = item.origin_height as f32 / item.origin_width as f32;
            let mut states = Vec::new();
            states.push(SpriteState {
                offset: -item_height,
                x: 0.0,
                y: offset + 0.25,
                width: 1.0,
                height: item_height,
                color: [0.0, 0.0, 0.0, 1.0],
            });
            if sprite_id > 0 {
                states.push(SpriteState {
                    offset: offset - item_height + 0.01,
                    x: 0.0,
                    y: item_height + 0.26,
                    width: 1.0,
                    height: item_height,
                    color: [0.0, 0.0, 0.0, 1.0],
                });
            }
            states.push(SpriteState {
                offset: if item_height < 1.0 {offset} else {offset + item_height * 0.5},
                x: 0.0,
                y: if item_height < 1.0 {0.0} else {-item_height * 0.5} + 0.25,
                width: 1.0,
                height: item_height,
                color: [0.0, 0.0, 0.0, 1.0],
            });
            states.push(SpriteState {
                offset: offset + item_height,
                x: -0.25,
                y: -item_height * 1.5,
                width: 1.5,
                height: item_height * 1.5,
                color: [0.0, 0.0, 0.0, 0.25],
            });
            scenario.images.push(SpriteAnimation {
                sprite_id,
                atlas_item_id: item.id,
                animation: AnimationStates { states },
            });
            sprite_id += 1;
            offset += item_height + 0.25;
        }
    }

    let mut states = Vec::new();
    states.push(SpriteState {
        offset: -1.0,
        x: 0.25,
        y: 0.015,
        width: 1.0,
        height: 1.0,
        color: [0.0, 0.0, 0.0, 1.0],
    });
    states.push(SpriteState {
        offset: 0.0,
        x: 0.25,
        y: 0.015,
        width: 1.0,
        height: 1.0,
        color: [0.0, 0.0, 0.0, 1.0],
    });
    states.push(SpriteState {
        offset: 0.125,
        x: 0.0,
        y: 0.01,
        width: 1.0,
        height: 1.5,
        color: [0.3, 0.1, 0.7, 0.0],
    });
    scenario.text_lines.push(TextLine {
        text: "Мотай вниз. Там самое интересное.".to_string(),
        relative_height: 0.018,
        animation: AnimationStates { states },
    });

    scenario
}

impl Scenario {
    pub fn max_offset(&self) -> f32 {
        let mut max_offset = 0.0;
        for sprite in &self.images {
            sprite.animation.states.iter().for_each(|animation| if animation.offset > max_offset {
                max_offset = animation.offset;
            });
        }
        max_offset
    }

    pub fn text_size(&self, canvas_size: f32) -> f32 {
        if self.text_lines.is_empty() {
            0.0
        } else {
            let sum = self.text_lines.iter().fold(0.0, |acc, line| acc + line.relative_height * canvas_size);
            sum / self.text_lines.len() as f32
        }
    }
}