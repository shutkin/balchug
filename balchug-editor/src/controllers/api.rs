use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dioxus::html::bytes::Bytes;
use dioxus::prelude::*;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use balchug_common::api::{AddImageResponse, OpenProjectResponse, ProjectSpriteProperties, StartProjectResponse, UpdateScenarioRq, UpdateSpritesPropsRq};
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use crate::states::project_state::SpriteProperties;

const SERVER_URL: &str = "http://localhost:3000";

#[derive(Clone)]
pub struct Api {
    http_client: Client,
    project_id: Rc<RefCell<String>>,
}

impl PartialEq for Api {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Api {
    pub fn new(project_id: String) -> Self {
        Self {
            http_client: Client::new(),
            project_id: Rc::new(RefCell::new(project_id)),
        }
    }

    pub fn has_project(&self) -> bool {
        !self.project_id.borrow().is_empty()
    }

    pub async fn start(&mut self) -> Option<String> {
        match self.http_client.post(format!("{SERVER_URL}/start")).send().await {
            Ok(response) => {
                if let Ok(resp) = response.json::<StartProjectResponse>().await {
                    info!("{resp:?}");
                    self.project_id.replace(resp.project_id.clone());
                    //return Some(format!("{SERVER_URL}/{}", resp.project_id));
                    return Some(resp.project_id);
                }
            }
            Err(err) => {
                error!("Failed to start: {err}");
            }
        }
        None
    }

    pub async fn upload_image(&self, data: Bytes, mime: &str) -> Option<(Vec<String>, Atlas)> {
        let url = format!("{SERVER_URL}/{}/image", self.project_id.borrow());
        match self.http_client.post(url).header(CONTENT_TYPE, mime).body(data).send().await {
            Ok(response) => {
                if let Ok(resp) = response.json::<AddImageResponse>().await {
                    info!("{resp:?}");
                    return Some((resp.thumbs, resp.atlas));
                }
            }
            Err(err) => {
                error!("Failed to upload image: {err}");
            }
        }
        None
    }

    pub async fn update_sprites_props(&self, props: HashMap<usize, SpriteProperties>) {
        let mut sprites_properties = HashMap::new();
        for (sprite_id, properties) in props {
            sprites_properties.insert(sprite_id, ProjectSpriteProperties {
                title: properties.title,
                parallax_factor: properties.parallax_factor,
            });
        }

        let url = format!("{SERVER_URL}/{}/sprites", self.project_id.borrow());
        let data = UpdateSpritesPropsRq {
            sprites_properties,
        };
        match self.http_client.post(url).json(&data).send().await {
            Ok(_) => {
                info!("Sprites properties updated");
            }
            Err(err) => {
                error!("Failed to update sprites properties: {err}");
            }
        }
    }

    pub async fn update_scenario(&self, scenario: Scenario) {
        let url = format!("{SERVER_URL}/{}/scenario", self.project_id.borrow());
        let data = UpdateScenarioRq {
            scenario,
        };
        match self.http_client.post(url).json(&data).send().await {
            Ok(_) => {
                info!("Scenario updated");
            }
            Err(err) => {
                error!("Failed to update scenario: {err}");
            }
        }
    }
    
    pub async fn open_project(&self) -> Option<OpenProjectResponse> {
        let url = format!("{SERVER_URL}/{}/project", self.project_id.borrow());
        match self.http_client.get(url).send().await {
            Ok(response) => {
                if let Ok(resp) = response.json::<OpenProjectResponse>().await {
                    return Some(resp);
                }
            }
            Err(err) => {
                error!("Failed to get project: {err}");
            }
        }
        None
    }

    pub fn assets_url(&self, path: &str) -> String {
        format!("{SERVER_URL}/{}/assets/{path}", self.project_id.borrow())
    }
}