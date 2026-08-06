use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dioxus::prelude::*;
use balchug_common::api::OpenProjectResponse;
use balchug_engine::BalchugEngine;
use crate::components::workspace::Workspace;
use crate::controllers::api::Api;
use crate::controllers::resources::ResourcesController;
use crate::controllers::sprite_editor::SpriteEditController;
use crate::controllers::storage::LocalStorage;
use crate::states::project_state::{ProjectState, SpriteProperties};

mod components;
mod states;
mod controllers;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/style.css");

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let api = Api::new(LocalStorage::get("project_id").unwrap_or_default());
    let mut open_project_response = use_signal(|| Option::<OpenProjectResponse>::None);

    let api_clone = api.clone();
    let project_is_ready = use_resource(move || {
        let mut api = api_clone.clone();
        async move {
            // try open last project
            if api.has_project() && let Some(resp) = api.open_project().await {
                open_project_response.set(Some(resp));
                return true;
            } else {
                // create new project
                if let Some(id) = api.start().await {
                    LocalStorage::set("project_id", &id);
                    return true;
                }
            }
            LocalStorage::remove("project_id");
            false
        }
    });

    let engine = Rc::new(RefCell::new(Option::<BalchugEngine>::None));
    let project_state = Rc::new(RefCell::new(ProjectState::new()));

    // load font when assets url is known
    let api_clone = api.clone();
    let engine_clone = engine.clone();
    use_effect(move || {
        if project_is_ready.read().unwrap_or(false) {
            let font_url = api_clone.assets_url("font.otf");
            if let Some(engine) = engine_clone.borrow().as_ref() {
                engine.set_font(&font_url);
            }
        }
    });

    let edit_controller = SpriteEditController::new(engine.clone(), project_state.clone(), api.clone());
    let resources_controller = ResourcesController::new(api.clone(), engine.clone(), project_state.clone());

    // open existing project
    let mut resources_clone = resources_controller.clone();
    let project_state_clone = project_state.clone();
    let engine_clone = engine.clone();
    use_effect(move || {
        if let Some(resp) = open_project_response.read().as_ref() {
            info!("{resp:?}");
            resources_clone.update_image_resources(resp.images_thumbs.clone(), resp.atlas.clone());

            let mut sprite_props_map = HashMap::new();
            for (&sprite_id, props) in &resp.sprites_properties {
                sprite_props_map.insert(sprite_id, SpriteProperties {
                    title: props.title.clone(),
                    parallax_factor: props.parallax_factor,
                });
            }

            project_state_clone.borrow_mut().sprite_properties.replace(sprite_props_map);

            if let Some(engine) = engine_clone.borrow().as_ref() {
                engine.set_scenario(resp.scenario.sprites.clone());
            }
        }
    });

    rsx! {
        Title { "Balchug Editor" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {
            resources_controller,
            edit_controller,
        }
    }
}
