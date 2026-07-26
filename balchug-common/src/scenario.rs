use serde::{Deserialize, Serialize};
use crate::sprite::{SpriteAnimation, TextLine};

#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub images: Vec<SpriteAnimation>,
    pub text_lines: Vec<TextLine>,
}
