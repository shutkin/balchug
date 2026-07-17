use crate::sprite::{SpriteAnimation, TextLine};

#[derive(Clone, Default)]
pub struct Scenario {
    pub images: Vec<SpriteAnimation>,
    pub text_lines: Vec<TextLine>,
}
