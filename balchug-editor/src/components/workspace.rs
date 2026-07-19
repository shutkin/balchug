use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use balchug_engine::{start_engine, BalchugEngine, OffsetListener};
use crate::components::state_props::StateProps;
use crate::components::timeline::TimeLine;

static ASSETS_DIR: Asset = asset!("/assets");

#[component]
pub fn Workspace(atlas: Signal<Atlas>, scenario: Signal<Scenario>, preview_offset: Signal<f32>) -> Element {
    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {
                atlas,
                scenario,
                preview_offset,
            }
            Sidebar {
                scenario,
                preview_offset,
            }
        }
    }
}

#[derive(Clone)]
struct PreviewOffsetListener {
    signal: Signal<f32>,
}

impl OffsetListener for PreviewOffsetListener {
    fn offset_change(&mut self, offset: f32) {
        self.signal.set(offset);
    }
}

#[component]
pub fn BalchugPreview(atlas: Signal<Atlas>, scenario: Signal<Scenario>, preview_offset: Signal<f32>) -> Element {
    let listener = PreviewOffsetListener { signal: preview_offset };
    let mut engine: Signal<Option<BalchugEngine>> = use_signal(|| None);
    use_effect(move || {
        let atlas = atlas.read().clone();
        if let Some(engine) = engine.read().as_ref() {
            engine.set_atlas(atlas);
        }
    });
    use_effect(move || {
        let scenario = scenario.read().clone();
        if let Some(engine) = engine.read().as_ref() {
            engine.set_scenario(scenario);
        }
    });

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
                            let balchug_engine = start_engine(window, canvas, &ASSETS_DIR.to_string());
                            balchug_engine.set_offset_listener(Box::new(listener.clone()));
                            engine.set(Some(balchug_engine));
                        }
                    },
                }
            }
        }
    }
}

#[component]
pub fn Sidebar(scenario: Signal<Scenario>, preview_offset: Signal<f32>) -> Element {
    let cur_point = use_signal(|| None);
    
    rsx! {
        aside {
            id: "sidebar",
            class: "sidebar",
            nav {
                id: "sidebar_tabs",
                class: "tab-navigation",
                button {
                    id: "sidebar_btn_timeline",
                    class: "tab-btn active",
                    "Time Line"
                }
                button {
                    id: "sidebar_btn_props",
                    class: "tab-btn",
                    "Properties"
                }
            }
            div {
                id: "sidebar_container",
                class: "panel-box",
                StateProps {
                    scenario,
                    cur_point,
                }
                TimeLine {
                    scenario,
                    preview_offset,
                    cur_point,
                }
            }
        }
    }
}