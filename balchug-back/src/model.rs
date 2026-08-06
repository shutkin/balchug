use std::collections::HashMap;
use balchug_common::api::ProjectSpriteProperties;
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;

#[derive(Clone)]
pub struct BalchugProject {
    pub id: String,
    pub images_atlas: Atlas,
    pub scenario: Scenario,
    pub thumbs: Vec<String>,
    pub sprite_properties: HashMap<usize, ProjectSpriteProperties>,
}