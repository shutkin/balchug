use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::info;
use rand::distr::{Alphanumeric, SampleString};
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use crate::atlas::{create_atlas, create_empty_atlas};
use crate::codegen::animations_to_code;
use crate::CommonError;
use crate::model::BalchugProject;

#[derive(Clone, Default)]
pub struct Server {
    projects: Arc<RwLock<HashMap<String, BalchugProject>>>,
}

impl Server {
    pub fn create_project(&self) -> Result<BalchugProject, CommonError> {
        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        let project = BalchugProject {
            id,
            scenario: Scenario::default(),
            images_atlas: Atlas::default(),
            thumbs: Vec::new(),
        };
        std::fs::create_dir(format!("./store/{}", project.id))?;
        std::fs::create_dir(format!("./store/{}/image", project.id))?;
        std::fs::create_dir(format!("./store/{}/thumb", project.id))?;
        std::fs::copy("./font.otf", format!("./store/{}/font.otf", project.id))?;
        create_empty_atlas(&format!("./store/{}/atlas.webp", project.id))?;
        if let Ok(mut lock) = self.projects.write() {
            lock.insert(project.id.clone(), project.clone());
        }
        Ok(project)
    }

    pub fn get_project(&self, id: &str) -> Option<BalchugProject> {
        self.projects.read().unwrap().get(id).cloned()
    }

    pub fn add_image(&self, mut project: BalchugProject, image: &[u8], img_type: &str) -> Result<(Vec<String>, Atlas), CommonError> {
        let dyn_image = image::load_from_memory(image)?;
        let thumb = dyn_image.resize(200, 200, image::imageops::FilterType::Lanczos3);
        let image_index = project.thumbs.len();
        thumb.save(format!("./store/{}/thumb/{image_index:05}.jpg", project.id))?;
        std::fs::write(format!("./store/{}/image/{image_index:05}.{img_type}", project.id), image)?;
        let atlas = create_atlas(
            &format!("./store/{}/image", project.id),
            &format!("./store/{}/atlas.webp", project.id),
        )?;
        project.images_atlas = atlas.clone();
        project.thumbs.push(format!("thumb_{image_index:05}.jpg"));
        let thumbs = project.thumbs.clone();
        if let Ok(mut lock) = self.projects.write() {
            lock.insert(project.id.clone(), project);
        }
        Ok((thumbs, atlas))
    }
    
    pub fn update_scenario(&self, project: BalchugProject, scenario: &Scenario) -> Result<(), CommonError> {
        if let Ok(mut lock) = self.projects.write() {
            let project = lock.get_mut(&project.id).ok_or("Failed to update project")?;
            project.scenario = scenario.clone();
            
            let code = animations_to_code(&scenario.sprites);
            info!("Code:\n{code}");
        }
        Ok(())
    }
}