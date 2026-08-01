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