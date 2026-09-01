use std::collections::HashSet;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::SpriteData;
use crate::TEXT_SIZE_FACTOR;

pub fn scenario_max_offset(scenario: &Scenario) -> f32 {
    let mut max_offset = 0.0;
    for sprite in &scenario.sprites {
        sprite.states.iter().for_each(|animation| if animation.offset > max_offset {
            max_offset = animation.offset;
        });
    }
    max_offset
}

pub fn scenario_text_size(scenario: &Scenario, canvas_size: f32) -> f32 {
    let mut sizes = Vec::new();
    for sprite in &scenario.sprites {
        if let SpriteData::Text(data) = &sprite.data {
            sizes.push(data.size as f32 * TEXT_SIZE_FACTOR * canvas_size);
        }
    }
    if sizes.is_empty() {
        // default size is 15
        15.0 * TEXT_SIZE_FACTOR * canvas_size
    } else {
        sizes.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sizes.len() / 2;
        if sizes.len() % 2 == 0 {
            (sizes[mid - 1] + sizes[mid]) / 2.0
        } else {
            sizes[mid]
        }
    }
}

pub fn scenario_letters(scenario: &Scenario, font_index: usize) -> String {
    let mut letters = HashSet::new();
    for sprite in &scenario.sprites {
        if let SpriteData::Text(data) = &sprite.data && data.font == font_index {
            data.text.chars().for_each(|letter| { letters.insert(letter); });
        }
    }
    String::from_iter(letters)
}