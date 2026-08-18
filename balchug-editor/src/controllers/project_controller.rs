use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use balchug_common::api::ProjectProperties;
use balchug_engine::BalchugEngine;
use balchug_engine::settings::Settings;
use crate::controllers::api::Api;
use crate::states::project_state::ProjectState;

#[derive(Clone)]
pub struct ProjectController {
    api: Api,
    state: ProjectState,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
}

impl PartialEq for ProjectController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ProjectController {
    pub fn new(api: Api, state: ProjectState, engine: Rc<RefCell<Option<BalchugEngine>>>) -> Self {
        Self { api, state, engine }
    }
    
    pub fn get_project_name(&self) -> String {
        self.state.properties.read().name.clone()
    }
    
    pub fn set_project_name(&mut self, project_name: String) {
        let mut properties = self.state.properties.read().clone();
        properties.name = project_name;
        self.state.properties.set(properties.clone());
        self.update_properties(properties);
    }

    pub fn get_background_color(&self) -> String {
        let c = &self.state.properties.read().background_color;
        format!("rgb({},{},{})", c[0], c[1], c[2])
    }
    
    pub fn get_text_color(&self) -> String {
        let c = &self.state.properties.read().default_text_color;
        format!("rgb({},{},{})", c[0], c[1], c[2])
    }

    fn parse_hex_color(hex: &str) -> Result<[u8; 3], std::num::ParseIntError> {
        let hex = hex.trim_start_matches('#');
        let r = u8::from_str_radix(&hex[0..2], 16)?;
        let g = u8::from_str_radix(&hex[2..4], 16)?;
        let b = u8::from_str_radix(&hex[4..6], 16)?;
        Ok([r, g, b])
    }

    pub fn set_background_color(&mut self, color: String) {
        if let Ok(color) = Self::parse_hex_color(&color)
            && let Some(engine) = self.engine.borrow().as_ref() {
            let settings = Settings {
                background_color: color,
            };
            engine.update_settings(settings);

            let mut properties = self.state.properties.read().clone();
            properties.background_color = color;
            self.state.properties.set(properties.clone());
            self.update_properties(properties);
        }
    }

    pub fn set_text_color(&mut self, color: String) {
        if let Ok(color) = Self::parse_hex_color(&color) {
            let mut properties = self.state.properties.read().clone();
            properties.default_text_color = color;
            self.state.properties.set(properties.clone());
            self.update_properties(properties);
        }
    }
    
    pub fn update_properties(&self, properties: ProjectProperties) {
        let api_clone = self.api.clone();
        use_future(move || {
            let api_clone = api_clone.clone();
            let properties = properties.clone();
            async move {
                api_clone.update_project_properties(properties).await
            }
        });
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