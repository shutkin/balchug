use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use dioxus::prelude::*;
use balchug_common::api::OpenProjectResponse;
use balchug_common::scenario::Scenario;
use balchug_engine::BalchugEngine;
use balchug_engine::settings::Settings;
use crate::components::workspace::Workspace;
use crate::controllers::api::Api;
use crate::controllers::project_controller::ProjectController;
use crate::controllers::resources::ResourcesController;
use crate::controllers::sprite_editor::SpriteEditController;
use crate::controllers::storage::LocalStorage;
use crate::controllers::updates_sender::{PinnedFuture, UpdatesHandler, UpdatesSender};
use crate::states::project_state::{ProjectState, SpriteProperties};

mod components;
mod states;
mod controllers;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/style.css");

fn main() {
    launch(App);
}

struct ScenarioUpdateHandler {
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    api: Api,
}

impl UpdatesHandler<Scenario> for ScenarioUpdateHandler {
    fn collect(&self) -> Option<Scenario> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let animations = engine.get_sprites_animations(None);
            Some(Scenario { sprites: animations })
        } else {
            None
        }
    }
    
    fn send(&self, value: Scenario) -> PinnedFuture<'_> {
        Box::pin(self.api.update_scenario(value))
    }
}

struct SpritesUpdateHandler {
    project_state: ProjectState,
    api: Api,
}

impl UpdatesHandler<HashMap<usize, SpriteProperties>> for SpritesUpdateHandler {
    fn collect(&self) -> Option<HashMap<usize, SpriteProperties>> {
        Some(self.project_state.sprite_properties.read().clone())
    }

    fn send(&self, value: HashMap<usize, SpriteProperties>) -> PinnedFuture<'_> {
        Box::pin(self.api.update_sprites_props(value))
    }
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
    let project_state = ProjectState::new();

    let scenario_update_sender = UpdatesSender::new(
        ScenarioUpdateHandler {
            engine: engine.clone(),
            api: api.clone(),
        }
    );
    let scenario_update_signal = scenario_update_sender.start();
    
    let sprites_update_sender = UpdatesSender::new(
        SpritesUpdateHandler {
            project_state: project_state.clone(),
            api: api.clone(),
        }
    );
    let sprites_update_signal = sprites_update_sender.start();

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

    let edit_controller = SpriteEditController::new(
        engine.clone(),
        project_state.clone(),
        scenario_update_signal.clone(),
    );
    let resources_controller = ResourcesController::new(
        api.clone(),
        engine.clone(),
        project_state.clone(),
        sprites_update_signal.clone(),
        scenario_update_signal.clone(),
    );

    // open existing project
    let mut resources_clone = resources_controller.clone();
    let mut project_state_clone = project_state.clone();
    let engine_clone = engine.clone();
    use_effect(move || {
        if let Some(resp) = open_project_response.read().as_ref() {
            info!("{resp:?}");
            resources_clone.update_image_resources(resp.images_thumbs.clone(), resp.atlas.clone());

            project_state_clone.properties.set(resp.project_properties.clone());

            let mut sprite_props_map = HashMap::new();
            for (&sprite_id, props) in &resp.sprites_properties {
                sprite_props_map.insert(sprite_id, SpriteProperties {
                    title: props.title.clone(),
                    parallax_factor: props.parallax_factor,
                });
            }
            project_state_clone.sprite_properties.replace(sprite_props_map);

            if let Some(engine) = engine_clone.borrow().as_ref() {
                let sprites = resp.scenario.sprites.clone();
                engine.set_scenario(sprites);
                engine.update_settings(Settings {
                    background_color: resp.project_properties.background_color,
                });
            }
        }
    });
    
    let project_controller = ProjectController::new(
        api.clone(),
        project_state.clone(),
        engine.clone(),
    );

    rsx! {
        Title { "Balchug Editor" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {
            project_controller,
            resources_controller,
            edit_controller,
        }
    }
}
