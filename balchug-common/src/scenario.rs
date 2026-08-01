use serde::{Deserialize, Serialize};
use crate::sprite::SpriteAnimation;

#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
pub struct Scenario {
    pub sprites: Vec<SpriteAnimation>,
}
