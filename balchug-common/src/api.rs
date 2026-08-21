use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::atlas::Atlas;
use crate::scenario::Scenario;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartProjectResponse {
    pub project_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateProjectPropertiesRq {
    pub properties: ProjectProperties,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddImageResponse {
    pub thumbs: Vec<String>,
    pub atlas: Atlas,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateScenarioRq {
    pub scenario: Scenario,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectProperties {
    pub name: String,
    pub background_color: [u8; 3],
    pub default_text_color: [u8; 3],
}

impl Default for ProjectProperties {
    fn default() -> Self {
        Self {
            name: "Balchug Project".to_string(),
            background_color: [0, 0, 0],
            default_text_color: [255, 255, 255],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectSpriteGroupProperties {
    pub main_sprite: usize,
    pub sprites: Vec<usize>,
    pub title: String,
    pub parallax_factor: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSpritesPropsRq {
    pub sprites_properties: HashMap<usize, ProjectSpriteGroupProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenProjectResponse {
    pub project_properties: ProjectProperties,
    pub images_thumbs: Vec<String>,
    pub atlas: Atlas,
    pub scenario: Scenario,
    pub sprites_groups: HashMap<usize, ProjectSpriteGroupProperties>,
}