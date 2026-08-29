use crate::states::project_state::SpriteGroup;
use balchug_common::api::{AddImageResponse, OpenProjectResponse, ProjectProperties, ProjectSpriteGroup, StartProjectResponse, UpdateGroupsRq, UpdateProjectPropertiesRq};
use balchug_common::atlas::Atlas;
use dioxus::html::bytes::Bytes;
use dioxus::prelude::*;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, HtmlAnchorElement, Url};
use balchug_common::scenario::Scenario;

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

    fn handle_reqwest_error(msg: &str, err: reqwest::Error) {
        error!("{msg}: {err}");
        if let Some(window) = web_sys::window()
            && let Err(js_err) = window.alert_with_message(&format!("{err}")) {
            error!("{js_err:?}");
        }
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
                Self::handle_reqwest_error("Failed to start", err);
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
                Self::handle_reqwest_error("Failed to upload image", err);
            }
        }
        None
    }
    
    pub async fn update_project_properties(&self, properties: ProjectProperties) {
        let url = format!("{SERVER_URL}/{}/props", self.project_id.borrow());
        let data = UpdateProjectPropertiesRq {
            properties,
        };
        match self.http_client.post(url).json(&data).send().await {
            Ok(_) => {
                info!("Project properties updated");
            }
            Err(err) => {
                Self::handle_reqwest_error("Failed to update project properties", err);
            }
        }
    }

    pub async fn update_groups(&self, groups: Vec<SpriteGroup>) {
        let groups = groups.into_iter()
            .map(|group| ProjectSpriteGroup {
                title: group.title,
                data: group.data,
                parallax_factor: group.parallax_factor,
                smooth_factor: group.smooth_factor,
                states: group.states,
                max_width: group.max_width,
            }).collect();

        let url = format!("{SERVER_URL}/{}/groups", self.project_id.borrow());
        let data = UpdateGroupsRq {
            groups,
        };
        match self.http_client.post(url).json(&data).send().await {
            Ok(_) => {},
            Err(err) => {
                Self::handle_reqwest_error("Failed to update sprites properties", err);
            },
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
                Self::handle_reqwest_error("Failed to get project", err);
            }
        }
        None
    }

    pub async fn download_dist(&self, scenario: Scenario) {
        let url = format!("{SERVER_URL}/{}/export", self.project_id.borrow());
        match self.http_client.post(url).json(&scenario).send().await {
            Ok(response) => {
                if let Err(err) = Self::save_bytes(response.bytes().await, "dist.zip") {
                    error!("Failed to save bytes: {err:?}");
                }
            }
            Err(err) => {
                Self::handle_reqwest_error("Failed to download dist", err);
            }
        }
    }

    fn save_bytes(bytes: std::result::Result<Bytes, reqwest::Error>, filename: &str) -> Result<(), JsValue> {
        let bytes = bytes.map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Create an array containing our bytes data
        let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
        let array = js_sys::Array::new();
        array.push(&uint8_array);

        // Build the Blob object
        let blob = Blob::new_with_buffer_source_sequence(&array)?;

        // 4. Create a temporary local URL for the blob
        let download_url = Url::create_object_url_with_blob(&blob)?;

        // 5. Create a hidden <a> tag and trigger download
        let window = web_sys::window().ok_or(JsValue::from_str("Failed to get window"))?;
        let document = window.document().ok_or(JsValue::from_str("Failed to get document"))?;
        let body = document.body().ok_or(JsValue::from_str("Failed to get document body"))?;

        let link = document.create_element("a")?.dyn_into::<HtmlAnchorElement>()?;
        link.set_href(&download_url);
        link.set_download(filename);

        body.append_child(&link)?;
        link.click();

        // 6. Clean up memory and remove element
        body.remove_child(&link)?;
        Url::revoke_object_url(&download_url)?;

        Ok(())
    }

    pub fn assets_url(&self, path: &str) -> String {
        format!("{SERVER_URL}/{}/assets/{path}", self.project_id.borrow())
    }
}