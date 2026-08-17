use std::collections::HashMap;
use balchug_common::api::{ProjectProperties, ProjectSpriteProperties};
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct BalchugProject {
    pub id: String,
    pub props: ProjectProperties,
    pub images_atlas: Atlas,
    pub scenario: Scenario,
    pub thumbs: Vec<String>,
    pub sprite_properties: HashMap<usize, ProjectSpriteProperties>,
}