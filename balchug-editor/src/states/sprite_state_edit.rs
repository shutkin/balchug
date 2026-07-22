use balchug_common::F32Rect;
use balchug_common::sprite::SpriteState;

#[derive(Copy, Clone, PartialEq)]
pub struct SpriteStateEdit {
    pub sprite_index: usize,
    pub state_index: usize,
    pub rect: F32Rect,
    pub state: SpriteState,
    pub original_state: SpriteState,
}

impl SpriteStateEdit {
    pub fn change_rect(&self, rect: F32Rect) -> Self {
        Self {
            sprite_index: self.sprite_index,
            state_index: self.state_index,
            rect,
            state: self.state,
            original_state: self.original_state,
        }
    }
    
    pub fn change_state(&self, state: SpriteState) -> Self {
        Self {
            sprite_index: self.sprite_index,
            state_index: self.state_index,
            rect: self.rect,
            state,
            original_state: self.original_state,
        }
    }
}