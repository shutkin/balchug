use crate::components::workspace::Workspace;
use crate::controllers::api::Api;
use crate::controllers::group_utils::GroupUtils;
use crate::controllers::project_controller::ProjectController;
use crate::controllers::resources::ResourcesController;
use crate::controllers::sprite_editor::SpriteEditController;
use crate::controllers::storage::{KEY_PROJECT_ID, LocalStorage};
use crate::controllers::updates_sender::{PinnedFuture, UpdatesHandler, UpdatesSender};
use crate::states::project_state::{ProjectState, SpriteGroup};
use balchug_common::api::{OpenProjectResponse, ProjectProperties};
use balchug_engine::BalchugEngine;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use balchug_common::settings::{BalchugSettings, InertiaProperties};

mod components;
mod states;
mod controllers;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/style.css");

fn main() {
    launch(App);
}

struct GroupsUpdateHandler {
    project_state: ProjectState,
    api: Api,
}

impl UpdatesHandler<Vec<SpriteGroup>> for GroupsUpdateHandler {
    fn collect(&self) -> Option<Vec<SpriteGroup>> {
        Some(self.project_state.groups.read().clone())
    }

    fn send(&self, value: Vec<SpriteGroup>) -> PinnedFuture<'_> {
        Box::pin(self.api.update_groups(value))
    }
}

struct ProjectPropsUpdateHandler {
    project_state: ProjectState,
    api: Api,
}

impl UpdatesHandler<ProjectProperties> for ProjectPropsUpdateHandler {
    fn collect(&self) -> Option<ProjectProperties> {
        Some(self.project_state.properties.read().clone())
    }
    
    fn send(&self, value: ProjectProperties) -> PinnedFuture<'_> {
        Box::pin(self.api.update_project_properties(value))
    }
}

#[component]
fn App() -> Element {
    let api = Api::new(LocalStorage::get(KEY_PROJECT_ID).unwrap_or_default());
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
                    LocalStorage::set(KEY_PROJECT_ID, &id);
                    return true;
                }
            }
            LocalStorage::remove(KEY_PROJECT_ID);
            false
        }
    });

    let engine = Rc::new(RefCell::new(Option::<BalchugEngine>::None));
    let project_state = ProjectState::new();
    
    let groups_update_sender = UpdatesSender::new(
        GroupsUpdateHandler {
            project_state: project_state.clone(),
            api: api.clone(),
        }
    );
    let groups_update_signal = groups_update_sender.start();
    
    let project_props_update_sender = UpdatesSender::new(
        ProjectPropsUpdateHandler {
            project_state: project_state.clone(),
            api: api.clone(),
        }
    );
    let project_props_update_signal = project_props_update_sender.start();

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
        groups_update_signal.clone(),
    );
    let resources_controller = ResourcesController::new(
        api.clone(),
        engine.clone(),
        project_state.clone(),
        groups_update_signal.clone(),
    );

    // open existing project
    let mut resources_clone = resources_controller.clone();
    let mut project_state_clone = project_state.clone();
    let engine_clone = engine.clone();
    let c0 = edit_controller.clone();
    use_effect(move || {
        if *c0.get_ready_signal().read() &&
            let Some(resp) = open_project_response.read().as_ref() {
            info!("Open project");
            // Back: 243 216 240, Text: 42 5 61
            resources_clone.update_image_resources(resp.images_thumbs.clone(), resp.atlas.clone());

            project_state_clone.properties.set(resp.project_properties.clone());

            if let Some(engine) = engine_clone.borrow().as_ref() {
                engine.update_settings(BalchugSettings {
                    background_color: resp.project_properties.background_color,
                    inertia_properties: InertiaProperties {
                        inertion: resp.project_properties.inertion,
                        viscosity: resp.project_properties.viscosity,
                    },
                });

                let groups = resp.groups.iter().map(|group| SpriteGroup {
                    title: group.title.clone(),
                    data: group.data.clone(),
                    parallax_factor: group.parallax_factor,
                    smooth_factor: group.smooth_factor,
                    max_width: group.max_width,
                    states: group.states.clone(),
                    is_fixed: group.is_fixed,
                }).collect::<Vec<_>>();
                let sprites = GroupUtils::groups_to_sprites(&groups, engine);
                engine.set_scenario(sprites);
                project_state_clone.groups.replace(groups);
            }
        }
    });
    
    let project_controller = ProjectController::new(
        api.clone(),
        project_state.clone(),
        engine.clone(),
        project_props_update_signal,
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
