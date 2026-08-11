use dioxus::prelude::*;
use crate::controllers::api::Api;

#[derive(Clone, PartialEq)]
pub struct ProjectController {
    api: Api,
}

impl ProjectController {
    pub fn new(api: Api) -> Self {
        Self { api }
    }

    pub fn download_distributive(&self) {
        let api_clone = self.api.clone();
        use_future(move || {
            let api_clone = api_clone.clone();
            async move {
                api_clone.download_dist().await
            }
        });
    }
}