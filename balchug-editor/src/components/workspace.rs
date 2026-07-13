use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use balchug_engine::{start_engine, BalchugEngine};

static ASSETS_DIR: Asset = asset!("/assets");

#[component]
pub fn Workspace() -> Element {
    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {}
            Sidebar {}
        }
    }
}

#[component]
pub fn BalchugPreview() -> Element {
    let mut engine: Signal<Option<BalchugEngine>> = use_signal(|| None);
    
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
                    if let Some(engine) = &engine() {
                        engine.resize();
                    }
                },
                canvas {
                    id: "balchug_preview_canvas",
                    style: "display: block; width: 100%; height: 100%",
                    onmounted: move |mounted_data| {
                        let window = window().unwrap();
                        let raw_element = mounted_data.data.as_web_event();
                        if let Ok(canvas) = raw_element.dyn_into::<web_sys::HtmlCanvasElement>() {
                            engine.set(Some(start_engine(window, canvas, &ASSETS_DIR.to_string())));
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn Sidebar() -> Element {
    rsx! {
        aside {
            id: "sidebar",
            class: "sidebar",
        }
    }
}