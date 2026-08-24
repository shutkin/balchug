use balchug_common::api::{ProjectProperties, ProjectSpriteGroupProperties};
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::OwnedSemaphorePermit;

#[derive(Serialize, Deserialize, Clone)]
pub struct BalchugProject {
    pub id: String,
    pub props: ProjectProperties,
    pub images_atlas: Atlas,
    pub scenario: Scenario,
    pub thumbs: Vec<String>,
    pub groups_properties: HashMap<usize, ProjectSpriteGroupProperties>,
}

pub struct ProjectGuard {
    pub project: BalchugProject,
    pub _permit: OwnedSemaphorePermit,
}