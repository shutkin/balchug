use crate::components::overlay::PreviewOverlay;
use crate::components::state_editor::StateEditor;
use crate::components::timeline::TimeLine;
use crate::controllers::sprite_editor::SpriteEditController;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use crate::components::images::ImagesBank;
use crate::controllers::images_controller::ImagesController;

#[component]
pub fn Workspace(images_controller: ImagesController, edit_controller: SpriteEditController) -> Element {
    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {controller: edit_controller.clone()},
            Sidebar {
                edit_controller: edit_controller.clone(),
                images_controller: images_controller.clone(),
            },
        }
    }
}

#[component]
pub fn BalchugPreview(controller: SpriteEditController) -> Element {
    let c0 = controller.clone();
    let c1 = controller.clone();

    rsx! {
        section {
            id: "balchug_preview_section",
            class: "canvas-container",
            div {
                id: "balchug_preview_status_div",
                class: "canvas-status",
                span {
                    class: "status-dot",
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
pub fn Sidebar(images_controller: ImagesController, edit_controller: SpriteEditController) -> Element {
    let mut cur_tab = use_signal(move || 0_u8);

    rsx! {
        aside {
            id: "sidebar",
            class: "sidebar",
            nav {
                id: "sidebar_tabs",
                class: "tab-navigation",
                button {
                    id: "sidebar_btn_props",
                    class: format!("tab-btn{}", if *cur_tab.read() == 0 {" active"} else {""}),
                    onclick: move |_| {cur_tab.set(0);},
                    "Resources"
                }
                button {
                    id: "sidebar_btn_timeline",
                    class: format!("tab-btn{}", if *cur_tab.read() == 1 {" active"} else {""}),
                    onclick: move |_| {cur_tab.set(1);},
                    "Timeline"
                }
            }
            match *cur_tab.read() {
                0 => rsx! {ResourcesPanel {controller: images_controller.clone()}},
                _ => rsx! {TimelinePanel {controller: edit_controller.clone()}},
            }
        }
    }
}

#[component]
pub fn ResourcesPanel(controller: ImagesController) -> Element {
    rsx! {
        div {
            id: "sidebar_container_timeline",
            class: "panel-box",
            ImagesBank {controller: controller.clone()}
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