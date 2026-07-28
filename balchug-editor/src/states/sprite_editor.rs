use balchug_common::F32Rect;
use balchug_common::sprite::SpriteState;
use crate::components::timeline::TimeLinePoints;

#[derive(Clone, PartialEq)]
pub struct SpriteEditorState {
    pub timeline_points: TimeLinePoints,
    pub parallax_factor: f32,
    pub rect: F32Rect,
    pub sprite_state: SpriteState,
    pub original_sprite_state: SpriteState,
}

impl SpriteEditorState {
    pub fn change_sprite_rect(&self, rect: F32Rect, sprite_state: SpriteState) -> Self {
        Self {
            timeline_points: self.timeline_points.clone(),
            parallax_factor: self.parallax_factor,
            rect,
            sprite_state,
            original_sprite_state: self.original_sprite_state,
        }
    }
}