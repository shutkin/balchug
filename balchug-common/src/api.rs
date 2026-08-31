use serde::{Deserialize, Serialize};
use crate::atlas::Atlas;
use crate::scenario::Scenario;
use crate::settings::InertiaProperties;
use crate::sprite::{SpriteData, SpriteState};

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
    pub viscosity: u8,
    pub inertion: u8,
}

impl Default for ProjectProperties {
    fn default() -> Self {
        let inertia_props = InertiaProperties::default();
        Self {
            name: "Balchug Project".to_string(),
            background_color: [0, 0, 0],
            default_text_color: [255, 255, 255],
            inertion: inertia_props.inertion,
            viscosity: inertia_props.viscosity,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectSpriteGroup {
    pub title: String,
    pub data: SpriteData,
    pub parallax_factor: f32,
    pub smooth_factor: f32,
    pub max_width: f32,
    pub states: Vec<SpriteState>,
    pub is_fixed: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateGroupsRq {
    pub groups: Vec<ProjectSpriteGroup>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenProjectResponse {
    pub project_properties: ProjectProperties,
    pub images_thumbs: Vec<String>,
    pub atlas: Atlas,
    pub groups: Vec<ProjectSpriteGroup>,
}