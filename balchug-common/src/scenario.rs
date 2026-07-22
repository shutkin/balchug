use crate::sprite::{SpriteAnimation, TextLine};

#[derive(Clone, Default, PartialEq)]
pub struct Scenario {
    pub images: Vec<SpriteAnimation>,
    pub text_lines: Vec<TextLine>,
}
