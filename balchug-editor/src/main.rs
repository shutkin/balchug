use std::cell::RefCell;
use std::rc::Rc;
use dioxus::prelude::*;
use balchug_engine::BalchugEngine;
use crate::components::workspace::Workspace;
use crate::controllers::api::API;
use crate::controllers::resources::ResourcesController;
use crate::controllers::sprite_editor::SpriteEditController;
use crate::states::project_state::ProjectState;

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
    let engine = Rc::new(RefCell::new(Option::<BalchugEngine>::None));

    let api = API::default();
    let api_clone = api.clone();
    let project_url = use_resource(move || {
        let mut api = api_clone.clone();
        async move {
            api.start().await
        }
    });
    let api_clone = api.clone();
    let engine_clone = engine.clone();
    use_effect(move || {
        if let Some(project_url) = project_url.read().as_ref().and_then(|f| f.clone()) {
            info!("Project URL: {project_url}");
            let font_url = api_clone.asset_url("font.otf");
            if let Some(engine) = engine_clone.borrow().as_ref() {
                engine.set_font(&font_url);
            }
        }
    });
    
    let project_state = Rc::new(RefCell::new(ProjectState::new()));

    let edit_controller = SpriteEditController::new(engine.clone(), project_state.clone(), api.clone());
    let images_controller = ResourcesController::new(api.clone(), engine.clone(), project_state.clone());

    rsx! {
        Title { "Balchug Editor" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {
            images_controller,
            edit_controller,
        }
    }
}
