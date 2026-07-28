use std::collections::HashSet;
use balchug_common::scenario::Scenario;

pub fn scenario_max_offset(scenario: &Scenario) -> f32 {
    let mut max_offset = 0.0;
    for sprite in &scenario.images {
        sprite.states.iter().for_each(|animation| if animation.offset > max_offset {
            max_offset = animation.offset;
        });
    }
    max_offset
}

pub fn scenario_text_size(scenario: &Scenario, canvas_size: f32) -> f32 {
    if scenario.text_lines.is_empty() {
        0.0
    } else {
        let sum = scenario.text_lines.iter().fold(0.0, |acc, line| acc + line.relative_height * canvas_size);
        sum / scenario.text_lines.len() as f32
    }
}

pub fn scenario_letters(scenario: &Scenario) -> String {
    let mut letters = HashSet::new();
    for line in &scenario.text_lines {
        line.text.chars().for_each(|letter| { letters.insert(letter); });
    }
    String::from_iter(letters.into_iter())
}