use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::atlas::Atlas;
use crate::scenario::Scenario;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartProjectResponse {
    pub project_id: String,
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
pub struct ProjectSpriteProperties {
    pub title: String,
    pub parallax_factor: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSpritesPropsRq {
    pub sprites_properties: HashMap<usize, ProjectSpriteProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenProjectResponse {
    pub images_thumbs: Vec<String>,
    pub atlas: Atlas,
    pub scenario: Scenario,
    pub sprites_properties: HashMap<usize, ProjectSpriteProperties>,
}