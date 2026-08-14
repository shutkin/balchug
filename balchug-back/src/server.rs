use crate::CommonError;
use crate::atlas::{atlas_hash, create_atlas, create_empty_atlas, optimize_atlas_items};
use crate::codegen::{CARGO_TOML, INDEX_HTML, LIB_CODE, TRUNK_TOML, animations_to_code, atlas_to_code};
use crate::model::BalchugProject;
use balchug_common::api::ProjectSpriteProperties;
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, RwLock};
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;
use zip_extensions::zip_writer::zip_create_from_directory_with_options;

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
            sprite_properties: HashMap::new(),
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
            &HashMap::new(),
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
        let mut lock = self.projects.write().map_err(|_| "Failed to update project")?;
        let project = lock.get_mut(&project.id).ok_or("Failed to update project")?;
        project.scenario = scenario.clone();
        Ok(())
    }

    pub fn update_sprite_props(&self, project: BalchugProject, props: &HashMap<usize, ProjectSpriteProperties>) -> Result<(), CommonError> {
        let mut lock = self.projects.write().map_err(|_| "Failed to update project")?;
        let project = lock.get_mut(&project.id).ok_or("Failed to update project")?;
        project.sprite_properties = props.clone();
        Ok(())
    }

    pub fn compile(&self, project: BalchugProject) -> Result<Vec<u8>, CommonError> {
        std::fs::create_dir_all(format!("/tmp/balchug/{}/src", project.id))?;
        std::fs::create_dir_all(format!("/tmp/balchug/{}/assets", project.id))?;

        let original_atlas_hash = atlas_hash(&project.images_atlas);
        let scales = optimize_atlas_items(&project.images_atlas, &project.scenario, 1080);
        let atlas_optimized = create_atlas(
            &format!("./store/{}/image", project.id),
            &format!("/tmp/balchug/{}/assets/atlas-{}.webp", project.id, original_atlas_hash),
            &scales,
        )?;

        let atlas_code = atlas_to_code(&atlas_optimized)?;
        let scenario_code = animations_to_code(&project.scenario.sprites)?;
        std::fs::write(format!("/tmp/balchug/{}/Cargo.toml", project.id), CARGO_TOML)?;
        std::fs::write(format!("/tmp/balchug/{}/Trunk.toml", project.id), TRUNK_TOML)?;
        std::fs::write(format!("/tmp/balchug/{}/src/lib.rs", project.id),
                       LIB_CODE.replace("{atlas_hash}", &original_atlas_hash))?;
        std::fs::write(format!("/tmp/balchug/{}/src/create_atlas.rs", project.id), atlas_code)?;
        std::fs::write(format!("/tmp/balchug/{}/src/create_scenario.rs", project.id), scenario_code)?;
        std::fs::write(format!("/tmp/balchug/{}/index.html", project.id), INDEX_HTML)?;
        std::fs::copy("./font.otf", format!("/tmp/balchug/{}/assets/font.otf", project.id))?;

        let output = Command::new("trunk")
            .arg("build")
            .arg("--release")
            .current_dir(format!("/tmp/balchug/{}", project.id))
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("Output:\n{}", stdout);

            let dist_dir = PathBuf::from(format!("/tmp/balchug/{}/dist", project.id));
            let zip_path = PathBuf::from(format!("./store/{}/dist.zip", project.id));
            let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zip_create_from_directory_with_options(&zip_path, &dist_dir, |_| options)?;
            let result = std::fs::read(format!("./store/{}/dist.zip", project.id))?;
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output.stdout);
            eprintln!("Error:\n{}", stderr);
            Err("Failed to build project".into())
        }
    }
}
