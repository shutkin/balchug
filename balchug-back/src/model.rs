use balchug_common::api::{ProjectProperties, ProjectSpriteGroup};
use balchug_common::atlas::Atlas;
use serde::{Deserialize, Serialize};
use tokio::sync::OwnedSemaphorePermit;

#[derive(Serialize, Deserialize, Clone)]
pub struct BalchugProject {
    pub id: String,
    pub props: ProjectProperties,
    pub images_atlas: Atlas,
    pub thumbs: Vec<String>,
    pub groups: Vec<ProjectSpriteGroup>,
}

pub struct ProjectGuard {
    pub project: BalchugProject,
    pub _permit: OwnedSemaphorePermit,
}