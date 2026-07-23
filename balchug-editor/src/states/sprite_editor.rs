use balchug_common::F32Rect;
use balchug_common::sprite::SpriteState;

#[derive(Copy, Clone, PartialEq)]
pub struct SpriteEditorState {
    pub sprite_index: usize,
    pub state_index: usize,
    pub rect: F32Rect,
    pub sprite_state: SpriteState,
    pub original_sprite_state: SpriteState,
}

impl SpriteEditorState {
    pub fn change_sprite_rect(&self, rect: F32Rect, sprite_state: SpriteState) -> Self {
        Self {
            sprite_index: self.sprite_index,
            state_index: self.state_index,
            rect,
            sprite_state,
            original_sprite_state: self.original_sprite_state,
        }
    }
}