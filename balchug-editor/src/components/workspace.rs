use crate::components::group_edit::GroupEditDialog;
use crate::components::overlay::PreviewOverlay;
use crate::components::project::ProjectControl;
use crate::components::resources::ImagesBank;
use crate::components::state_editor::StateEditor;
use crate::components::timeline::{AddImageButton, AddImageDialog, AddTextButton, AddTextDialog, TimeLine};
use crate::controllers::project_controller::ProjectController;
use crate::controllers::resources::ResourcesController;
use crate::controllers::sprite_editor::SpriteEditController;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::window;

#[component]
pub fn Workspace(project_controller: ProjectController, resources_controller: ResourcesController, edit_controller: SpriteEditController) -> Element {
    if let Some(doc) = window().and_then(|window| window.document()) {
        let on_key = {
            let mut edit_controller = edit_controller.clone();
            let mut resources_controller = resources_controller.clone();
            Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
                if e.code() == "Escape" {
                    edit_controller.set_timeline_point(None);
                    resources_controller.close_popups();
                    e.prevent_default();
                }
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>)
        };
        if let Err(err) = doc.add_event_listener_with_callback("keydown", on_key.as_ref().unchecked_ref()) {
            error!("Failed to add key event handler: {err:?}");
        }
        on_key.forget();
    }

    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {controller: edit_controller.clone()},
            Sidebar {
                project_controller: project_controller.clone(),
                edit_controller: edit_controller.clone(),
                resources_controller: resources_controller.clone(),
            },
        }
    }
}

#[component]
pub fn BalchugPreview(controller: SpriteEditController) -> Element {
    let mut c0 = controller.clone();
    let c1 = controller.clone();
    let c2 = controller.clone();

    let status_class = use_memo(move || {
        if c2.is_edit_mode() {"status-dot-inactive"} else {"status-dot"}
    });

    rsx! {
        section {
            id: "balchug_preview_section",
            class: "canvas-container",
            div {
                id: "balchug_preview_status_div",
                class: "canvas-status",
                span {
                    class: "{status_class}",
                }
                span { "Live Viewport: 16:9 Vertical" }
            }
            div {
                id: "balchug_preview_frame",
                class: "mobile-frame",
                onresize: move |_event| {
                    c0.resize();
                },
                canvas {
                    id: "balchug_preview_canvas",
                    style: "display: block; width: 100%; height: 100%",
                    onmounted: move |mounted_data| {
                        let window = window().unwrap();
                        let raw_element = mounted_data.data.as_web_event();
                        if let Ok(canvas) = raw_element.dyn_into::<web_sys::HtmlCanvasElement>() {
                            c1.start(window, canvas);
                        }
                    },
                }
            }
            PreviewOverlay {controller}
        }
    }
}

#[component]
pub fn Sidebar(project_controller: ProjectController, resources_controller: ResourcesController, edit_controller: SpriteEditController) -> Element {
    let mut rc0 = resources_controller.clone();
    let mut rc1 = resources_controller.clone();
    let mut rc2 = resources_controller.clone();

    rsx! {
        aside {
            id: "sidebar",
            class: "sidebar",
            nav {
                id: "sidebar_tabs",
                class: "tab-navigation",
                button {
                    id: "sidebar_tab_project",
                    class: format!("tab-btn{}", if rc0.get_cur_tab() == 0 {" active"} else {""}),
                    onclick: move |_| {rc0.set_cur_tab(0);},
                    "Project"
                }
                button {
                    id: "sidebar_tab_props",
                    class: format!("tab-btn{}", if rc1.get_cur_tab() == 1 {" active"} else {""}),
                    onclick: move |_| {rc1.set_cur_tab(1);},
                    "Resources"
                }
                button {
                    id: "sidebar_tab_timeline",
                    class: format!("tab-btn{}", if rc2.get_cur_tab() == 2 {" active"} else {""}),
                    onclick: move |_| {rc2.set_cur_tab(2);},
                    "Timeline"
                }
            }
            match rc0.get_cur_tab() {
                0 => rsx! {ProjectPanel {controller: project_controller.clone()}},
                1 => rsx! {ResourcesPanel {controller: resources_controller.clone()}},
                _ => rsx! {TimelinePanel {controller: edit_controller.clone(), resources_controller: resources_controller.clone()}},
            }
        }
    }
}

#[component]
fn ProjectPanel(controller: ProjectController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_project",
            class: "panel-box",
            ProjectControl {controller}
        }
    }
}

#[component]
fn ResourcesPanel(controller: ResourcesController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_timeline",
            class: "panel-box",
            ImagesBank {controller: controller.clone()}
        }
    }
}

#[component]
fn TimelinePanel(controller: SpriteEditController, resources_controller: ResourcesController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_timeline",
            class: "panel-box",
            div {
                id: "timeline-controls",
                class: "form-row",
                AddTextButton {controller: resources_controller.clone()}
                AddImageButton {controller: resources_controller.clone()}
            }
            div {
                id: "sidebar_timeline_container",
                class: "timeline-container",
                TimeLine {controller: controller.clone(), resources_controller: resources_controller.clone()}
            }
            StateEditor {controller: controller.clone()}
            AddImageDialog {controller: resources_controller.clone()}
            AddTextDialog {controller: resources_controller.clone()}
            GroupEditDialog {controller: resources_controller.clone()}
        }
    }
}