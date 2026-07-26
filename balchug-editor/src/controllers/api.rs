use dioxus::html::bytes::Bytes;
use dioxus::prelude::*;
use reqwest::Client;
use reqwest::header::CONTENT_TYPE;
use balchug_common::api::{AddImageResponse, StartProjectResponse};
use balchug_common::atlas::Atlas;

const SERVER_URL: &str = "http://localhost:3000";

#[derive(Clone)]
pub struct API {
    http_client: Client,
    project_id: Store<String>,
}

impl PartialEq for API {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Default for API {
    fn default() -> Self {
        Self {
            http_client: Client::new(),
            project_id: Store::new("".to_string()),
        }
    }
}

impl API {
    pub async fn start(&mut self) -> Option<String> {
        match self.http_client.post(format!("{SERVER_URL}/start")).send().await {
            Ok(response) => {
                if let Ok(resp) = response.json::<StartProjectResponse>().await {
                    info!("{resp:?}");
                    self.project_id.set(resp.project_id.clone());
                    return Some(format!("{SERVER_URL}/{}", resp.project_id));
                }
            }
            Err(err) => {
                error!("Failed to start: {err}");
            }
        }
        None
    }

    pub async fn upload_image(&self, data: Bytes, mime: &str) -> Option<(Vec<String>, Atlas)> {
        let url = format!("{SERVER_URL}/{}/image", self.project_id);
        match self.http_client.post(url).header(CONTENT_TYPE, mime).body(data).send().await {
            Ok(response) => {
                if let Ok(resp) = response.json::<AddImageResponse>().await {
                    info!("{resp:?}");
                    let thumbs = resp.thumbs.into_iter()
                        .map(|path| self.asset_url(&path))
                        .collect::<Vec<_>>();
                    return Some((thumbs, resp.atlas));
                }
            }
            Err(err) => {
                error!("Failed to upload image: {err}");
            }
        }
        None
    }

    pub fn asset_url(&self, path: &str) -> String {
        format!("{SERVER_URL}/{}/assets/{path}", self.project_id)
    }
}