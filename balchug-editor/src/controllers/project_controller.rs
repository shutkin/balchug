use crate::controllers::api::Api;
use crate::controllers::group_utils::GroupUtils;
use crate::states::project_state::ProjectState;
use balchug_common::scenario::Scenario;
use balchug_common::settings::{BalchugSettings, InertiaProperties};
use balchug_engine::BalchugEngine;
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Clone)]
pub struct ProjectController {
    api: Api,
    state: ProjectState,
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    project_props_update_signal: Rc<Cell<bool>>,
}

impl PartialEq for ProjectController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl ProjectController {
    pub fn new(
        api: Api,
        state: ProjectState,
        engine: Rc<RefCell<Option<BalchugEngine>>>,
        project_props_update_signal: Rc<Cell<bool>>,
    ) -> Self {
        Self { api, state, engine, project_props_update_signal }
    }
    
    pub fn get_project_name(&self) -> String {
        self.state.properties.read().name.clone()
    }
    
    pub fn set_project_name(&mut self, project_name: String) {
        let mut properties = self.state.properties.read().clone();
        properties.name = project_name;
        self.state.properties.set(properties.clone());
        self.project_props_update_signal.set(true);
    }

    pub fn get_background_color(&self) -> String {
        let c = &self.state.properties.read().background_color;
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    }
    
    pub fn get_text_color(&self) -> String {
        let c = &self.state.properties.read().default_text_color;
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    }

    pub fn get_viscosity(&self) -> u8 {
        self.state.properties.read().viscosity
    }

    pub fn get_inertion(&self) -> u8 {
        self.state.properties.read().inertion
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
            let mut properties = self.state.properties.read().clone();
            properties.background_color = color;

            let settings = BalchugSettings {
                background_color: color,
                inertia_properties: InertiaProperties {
                    inertion: properties.inertion,
                    viscosity: properties.viscosity,
                }
            };
            engine.update_settings(settings);

            self.state.properties.set(properties.clone());
            self.project_props_update_signal.set(true);
        }
    }

    pub fn set_text_color(&mut self, color: String) {
        if let Ok(color) = Self::parse_hex_color(&color) {
            let mut properties = self.state.properties.read().clone();
            properties.default_text_color = color;
            self.state.properties.set(properties.clone());
            self.project_props_update_signal.set(true);
        }
    }

    pub fn set_inertion(&mut self, inertion: u8) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut properties = self.state.properties.read().clone();
            properties.inertion = inertion;

            let settings = BalchugSettings {
                background_color: properties.background_color,
                inertia_properties: InertiaProperties {
                    inertion,
                    viscosity: properties.viscosity,
                }
            };
            engine.update_settings(settings);

            self.state.properties.set(properties.clone());
            self.project_props_update_signal.set(true);
        }
    }

    pub fn set_viscosity(&mut self, viscosity: u8) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut properties = self.state.properties.read().clone();
            properties.viscosity = viscosity;

            let settings = BalchugSettings {
                background_color: properties.background_color,
                inertia_properties: InertiaProperties {
                    inertion: properties.inertion,
                    viscosity,
                }
            };
            engine.update_settings(settings);

            self.state.properties.set(properties.clone());
            self.project_props_update_signal.set(true);
        }
    }

    pub fn download_distributive(&self) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let api_clone = self.api.clone();
            let sprites = GroupUtils::groups_to_sprites(&self.state.get_groups(), engine);
            use_future(move || {
                let api_clone = api_clone.clone();
                let sprites = sprites.clone();
                async move {
                    api_clone.download_dist(Scenario {sprites}).await
                }
            });
        }
    }
}