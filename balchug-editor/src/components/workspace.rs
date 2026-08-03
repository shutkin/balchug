use crate::components::overlay::PreviewOverlay;
use crate::components::state_editor::StateEditor;
use crate::components::timeline::TimeLine;
use crate::controllers::sprite_editor::SpriteEditController;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use crate::components::resources::{ImagesBank, SpriteDialog, TextLine};
use crate::controllers::resources::ResourcesController;

#[component]
pub fn Workspace(resources_controller: ResourcesController, edit_controller: SpriteEditController) -> Element {
    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {controller: edit_controller.clone()},
            Sidebar {
                edit_controller: edit_controller.clone(),
                resources_controller: resources_controller.clone(),
            },
        }
    }
}

#[component]
pub fn BalchugPreview(controller: SpriteEditController) -> Element {
    let c0 = controller.clone();
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
                span { "Live Viewport: 16:9 Horizontal" }
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
pub fn Sidebar(resources_controller: ResourcesController, edit_controller: SpriteEditController) -> Element {
    let rc0 = resources_controller.clone();
    let rc1 = resources_controller.clone();

    rsx! {
        aside {
            id: "sidebar",
            class: "sidebar",
            nav {
                id: "sidebar_tabs",
                class: "tab-navigation",
                button {
                    id: "sidebar_btn_props",
                    class: format!("tab-btn{}", if rc0.get_cur_tab() == 0 {" active"} else {""}),
                    onclick: move |_| {rc0.set_cur_tab(0);},
                    "Resources"
                }
                button {
                    id: "sidebar_btn_timeline",
                    class: format!("tab-btn{}", if rc1.get_cur_tab() == 1 {" active"} else {""}),
                    onclick: move |_| {rc1.set_cur_tab(1);},
                    "Timeline"
                }
            }
            match rc0.get_cur_tab() {
                0 => rsx! {ResourcesPanel {controller: resources_controller.clone()}},
                _ => rsx! {TimelinePanel {controller: edit_controller.clone()}},
            }
        }
    }
}

#[component]
pub fn ResourcesPanel(controller: ResourcesController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_timeline",
            class: "panel-box",
            ImagesBank {controller: controller.clone()}
            TextLine {controller: controller.clone()}
            SpriteDialog {controller: controller.clone()}
        }
    }
}

#[component]
pub fn TimelinePanel(controller: SpriteEditController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_timeline",
            class: "panel-box",
            TimeLine {controller: controller.clone()}
            StateEditor {controller: controller.clone()}
        }
    }
}