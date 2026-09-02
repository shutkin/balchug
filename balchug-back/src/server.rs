use crate::CommonError;
use crate::atlas::{atlas_hash, create_atlas, create_empty_atlas, optimize_atlas_items};
use crate::codegen::{CARGO_TOML, INDEX_HTML, LIB_CODE, TRUNK_TOML, animations_to_code, atlas_to_code};
use crate::font::subset_font;
use crate::model::{BalchugProject, ProjectGuard};
use balchug_common::api::{ProjectProperties, ProjectSpriteGroup};
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use log::{error, info};
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::sync::Semaphore;
use zip::CompressionMethod;
use zip::write::SimpleFileOptions;
use zip_extensions::zip_writer::zip_create_from_directory_with_options;

const STORE_DIR: &str = "./store";

#[derive(Clone, Default)]
pub struct Server {
    projects: Arc<Mutex<HashMap<String, BalchugProject>>>,
    semaphore: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
}

impl Server {
    fn read_project(project_id: &str) -> Result<BalchugProject, CommonError> {
        let json = std::fs::read(format!("{STORE_DIR}/{project_id}/project.json"))?;
        let project = serde_json::from_slice::<BalchugProject>(&json)?;
        Ok(project)
    }

    fn save_project_json(project: &BalchugProject) {
        let project_id = project.id.clone();
        match serde_json::to_string(project) {
            Ok(json) => {
                tokio::spawn(async move {
                    if let Err(err) = tokio::fs::write(format!("{STORE_DIR}/{}/project.json", project_id), json).await {
                        error!("Failed to save project json: {err}");
                    }
                });
            },
            Err(err) => {
                error!("Failed to serialize project json: {err}");
            }
        }
    }

    pub fn load(&self) -> Result<(), CommonError> {
        let dir = std::fs::read_dir(STORE_DIR)?;
        for entry_result in dir {
            if let Ok(entry) = entry_result
                && let Ok(metadata) = entry.metadata()
                && metadata.is_dir() {
                let project_id = entry.file_name().to_string_lossy().to_string();
                match Server::read_project(&project_id) {
                    Ok(project) => {
                        self.put_project(project)?;
                        info!("Loaded project {project_id}");
                    }
                    Err(err) => {
                        error!("Failed to load project {project_id}: {err}");
                    }
                }
            }
        }
        
        Ok(())
    }
    
    pub fn create_project(&self) -> Result<BalchugProject, CommonError> {
        let id = Alphanumeric.sample_string(&mut rand::rng(), 16);
        let project = BalchugProject {
            id,
            props: ProjectProperties::default(),
            images_atlas: Atlas::default(),
            thumbs: Vec::new(),
            groups: Vec::new(),
        };
        std::fs::create_dir(format!("{STORE_DIR}/{}", project.id))?;
        std::fs::create_dir(format!("{STORE_DIR}/{}/image", project.id))?;
        std::fs::create_dir(format!("{STORE_DIR}/{}/thumb", project.id))?;
        std::fs::copy("./regular.otf", format!("{STORE_DIR}/{}/regular.otf", project.id))?;
        std::fs::copy("./bold.otf", format!("{STORE_DIR}/{}/bold.otf", project.id))?;
        std::fs::copy("./italic.otf", format!("{STORE_DIR}/{}/italic.otf", project.id))?;
        create_empty_atlas(&format!("{STORE_DIR}/{}/atlas.webp", project.id))?;
        self.put_project(project.clone())?;
        Ok(project)
    }

    pub async fn get_project(&self, id: &str) -> Option<ProjectGuard> {
        let semaphore = self.semaphore.lock().ok()?
            .entry(id.to_string())
            .or_insert(Arc::new(Semaphore::new(1)))
            .clone();
        let permit = semaphore.acquire_owned().await.ok()?;
        info!("Lock project {id}");

        if let Ok(projects) = self.projects.lock()
            && let Some(project) = projects.get(id) {
            Some(ProjectGuard {
                project: project.clone(),
                _permit: permit,
            })
        } else {
            if let Ok(mut semaphores) = self.semaphore.lock() {
                info!("Drop unused semaphore {id}");
                semaphores.remove(id);
            }
            None
        }
    }

    fn put_project(&self, project: BalchugProject) -> Result<(), CommonError> {
        Self::save_project_json(&project.clone());
        let mut projects = self.projects.lock().map_err(|_| "Failed to lock projects map")?;
        projects.insert(project.id.clone(), project);
        //lock.insert(project.id.clone(), project);
        Ok(())
    }

    pub fn update_project_props(&self, mut project: BalchugProject, props: ProjectProperties) -> Result<(), CommonError> {
        project.props = props.clone();
        self.put_project(project)?;
        Ok(())
    }

    pub fn add_image(&self, mut project: BalchugProject, image: &[u8], img_type: &str) -> Result<(Vec<String>, Atlas), CommonError> {
        let dyn_image = image::load_from_memory(image)?;
        let thumb = dyn_image.resize(200, 200, image::imageops::FilterType::Lanczos3);
        let image_index = project.thumbs.len();
        thumb.save(format!("{STORE_DIR}/{}/thumb/{image_index:05}.jpg", project.id))?;
        std::fs::write(format!("{STORE_DIR}/{}/image/{image_index:05}.{img_type}", project.id), image)?;
        let (atlas, webp) = create_atlas(
            &format!("{STORE_DIR}/{}/image", project.id),
            &HashMap::new(),
        )?;
        std::fs::write(format!("{STORE_DIR}/{}/atlas.webp", project.id), webp)?;

        project.images_atlas = atlas.clone();
        project.thumbs.push(format!("thumb_{image_index:05}.jpg"));
        let thumbs = project.thumbs.clone();
        self.put_project(project)?;
        
        Ok((thumbs, atlas))
    }

    pub fn update_groups_props(&self, mut project: BalchugProject, groups: Vec<ProjectSpriteGroup>) -> Result<(), CommonError> {
        project.groups = groups;
        self.put_project(project)?;
        Ok(())
    }

    pub fn compile(&self, project: BalchugProject, scenario: Scenario) -> Result<Vec<u8>, CommonError> {
        std::fs::create_dir_all(format!("/tmp/balchug/{}/src", project.id))?;
        std::fs::create_dir_all(format!("/tmp/balchug/{}/assets", project.id))?;

        let scales = optimize_atlas_items(&project.images_atlas, &project.groups, 1440);
        let (atlas_optimized, webp) = create_atlas(
            &format!("{STORE_DIR}/{}/image", project.id),
            &scales,
        )?;
        let atlas_hash = atlas_hash(&atlas_optimized);
        std::fs::write(format!("/tmp/balchug/{}/assets/atlas-{}.webp", project.id, atlas_hash), webp)?;

        let mut fonts = Vec::new();
        if let Some((font, font_hash)) = subset_font(&format!("{STORE_DIR}/{}/regular.otf", project.id), &scenario.sprites, 0)? {
            std::fs::write(format!("/tmp/balchug/{}/assets/regular-{font_hash}.otf", project.id), font)?;
            fonts.push(format!("\"assets/regular-{font_hash}.otf\""))
        }
        if let Some((font, font_hash)) = subset_font(&format!("{STORE_DIR}/{}/bold.otf", project.id), &scenario.sprites, 1)? {
            std::fs::write(format!("/tmp/balchug/{}/assets/bold-{font_hash}.otf", project.id), font)?;
            fonts.push(format!("\"assets/bold-{font_hash}.otf\""))
        }
        if let Some((font, font_hash)) = subset_font(&format!("{STORE_DIR}/{}/italic.otf", project.id), &scenario.sprites, 2)? {
            std::fs::write(format!("/tmp/balchug/{}/assets/italic-{font_hash}.otf", project.id), font)?;
            fonts.push(format!("\"assets/italic-{font_hash}.otf\""))
        }

        let atlas_code = atlas_to_code(&atlas_optimized)?;
        let scenario_code = animations_to_code(&scenario.sprites)?;
        let color = format!("{}, {}, {}", project.props.background_color[0], project.props.background_color[1], project.props.background_color[2]);
        std::fs::write(format!("/tmp/balchug/{}/Cargo.toml", project.id), CARGO_TOML)?;
        std::fs::write(format!("/tmp/balchug/{}/Trunk.toml", project.id), TRUNK_TOML)?;
        std::fs::write(
            format!("/tmp/balchug/{}/src/lib.rs", project.id),
            LIB_CODE
                .replace("{atlas_hash}", &atlas_hash)
                .replace("{settings.background_color}", &color)
                .replace("{fonts}", &fonts.join(","))
                .replace("{viscosity}", &format!("{}", project.props.viscosity))
                .replace("{inertion}", &format!("{}", project.props.inertion)),
        )?;
        std::fs::write(format!("/tmp/balchug/{}/src/create_atlas.rs", project.id), atlas_code)?;
        std::fs::write(format!("/tmp/balchug/{}/src/create_scenario.rs", project.id), scenario_code)?;
        std::fs::write(
            format!("/tmp/balchug/{}/index.html", project.id),
            INDEX_HTML
                .replace("{settings.name}", &project.props.name)
                .replace("{settings.background_color}", &color),
        )?;

        let output = Command::new("trunk")
            .arg("build")
            .arg("--release")
            .current_dir(format!("/tmp/balchug/{}", project.id))
            .output()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Build output:\n{}", stdout);

            let dist_dir = format!("/tmp/balchug/{}/dist", project.id);
            let zip_path = format!("/tmp/balchug/{}/dist.zip", project.id);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Zstd)
                .compression_level(Some(22));
            zip_create_from_directory_with_options(&zip_path.clone().into(), &dist_dir.into(), |_| options)?;
            let result = std::fs::read(zip_path)?;
            Ok(result)
        } else {
            let stderr = String::from_utf8_lossy(&output.stdout);
            Err(stderr.into())
        }
    }

    pub fn compile_clean(project_id: &str) -> Result<(), CommonError> {
        std::fs::remove_dir_all(format!("/tmp/balchug/{}", project_id))?;
        Ok(())
    }
}
