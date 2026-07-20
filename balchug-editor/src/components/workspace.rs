use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use balchug_common::atlas::Atlas;
use balchug_common::F32Rect;
use balchug_common::scenario::Scenario;
use balchug_engine::{start_engine, BalchugEngine, OffsetListener};
use crate::components::overlay::PreviewOverlay;
use crate::components::state_props::StateProps;
use crate::components::timeline::{TimeLine, TimeLinePoint};

static ASSETS_DIR: Asset = asset!("/assets");

#[component]
pub fn Workspace(atlas: Signal<Atlas>, scenario: Signal<Scenario>, engine: Signal<Option<BalchugEngine>>) -> Element {
    let preview_offset = Signal::new(0_f32);
    let cur_point: Signal<Option<TimeLinePoint>> = use_signal(|| None);
    let mut edit_rect = use_signal(move || None);

    use_effect(move || {
        if let Some(engine) = engine.read().as_ref() {
            if let Some(cur_point) = cur_point.read().as_ref()  {
                engine.set_interactive(false);
                engine.set_offset_to_image_state(cur_point.object_index, cur_point.state_index);
            } else {
                engine.set_interactive(true);
            }
        }
    });

    use_effect(move || {
        if let Some(engine) = engine.read().as_ref() {
            if let Some(point) = cur_point.read().as_ref() {
                let rect = engine.get_image_rect(point.object_index, point.offset);
                edit_rect.set(rect);
            } else {
                edit_rect.set(None);
            }
        }
    });

    rsx! {
        main {
            id: "workspace_main",
            class: "workspace",
            BalchugPreview {
                preview_offset,
                engine,
                edit_rect,
            }
            Sidebar {
                scenario,
                preview_offset,
                cur_point,
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
pub fn BalchugPreview(preview_offset: Signal<f32>, engine: Signal<Option<BalchugEngine>>, edit_rect: Signal<Option<F32Rect>>) -> Element {
    let listener = PreviewOffsetListener { signal: preview_offset };

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
            PreviewOverlay {
                rect: edit_rect,
            }
        }
    }
}

#[component]
pub fn Sidebar(scenario: Signal<Scenario>, preview_offset: Signal<f32>, cur_point: Signal<Option<TimeLinePoint>>) -> Element {
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
                TimeLine {
                    scenario,
                    preview_offset,
                    cur_point,
                }
                StateProps {
                    scenario,
                    cur_point,
                }
            }
        }
    }
}