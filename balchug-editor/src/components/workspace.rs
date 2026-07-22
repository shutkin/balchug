use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use wasm_bindgen::JsCast;
use web_sys::window;
use balchug_common::atlas::Atlas;
use balchug_common::scenario::Scenario;
use balchug_engine::{start_engine, BalchugEngine, OffsetListener};
use crate::components::overlay::PreviewOverlay;
use crate::components::state_editor::StateEditor;
use crate::components::timeline::{TimeLine, TimeLinePoint};
use crate::states::sprite_state_edit::SpriteStateEdit;

static ASSETS_DIR: Asset = asset!("/assets");

#[component]
pub fn Workspace(atlas: ReadSignal<Atlas>, scenario: Signal<Scenario>, engine: Signal<Option<BalchugEngine>>) -> Element {
    let selected_point = use_signal(move || Option::<TimeLinePoint>::None);
    let mut edit_state = use_signal(move || Option::<SpriteStateEdit>::None);
    let preview_offset = use_signal(move || 0_f32);

    use_effect(move || {
        if let Some(engine) = engine.read().as_ref() {
            if let Some(point) = *selected_point.read() {
                let state = scenario.read().images[point.sprite_index].animation.states[point.state_index];
                let rect = engine.scroll_to_image_state(point.sprite_index, point.state_index);
                let s = SpriteStateEdit {
                    sprite_index: point.sprite_index,
                    state_index: point.state_index,
                    state,
                    original_state: state,
                    rect,
                };
                web_sys::console::log_1(&format!("New edit state from {point:?}").into());
                edit_state.set(Some(s));
            } else {
                edit_state.set(None);
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
                edit_state,
            }
            Sidebar {
                scenario,
                preview_offset,
                selected_point,
                edit_state,
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
pub fn BalchugPreview(
    edit_state: Signal<Option<SpriteStateEdit>>,
    preview_offset: Signal<f32>,
    engine: Signal<Option<BalchugEngine>>,
) -> Element {
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
                            web_sys::console::log_1(&"Engine start".into());
                            engine.set(Some(balchug_engine));
                        }
                    },
                }
            }
            PreviewOverlay {
                edit_state,
                engine,
            }
        }
    }
}

#[component]
pub fn Sidebar(
    scenario: Signal<Scenario>,
    edit_state: Signal<Option<SpriteStateEdit>>,
    preview_offset: Signal<f32>,
    selected_point: Signal<Option<TimeLinePoint>>,
) -> Element {
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
                    selected_point,
                }
                StateEditor {
                    scenario,
                    edit_state,
                    selected_point,
                }
            }
        }
    }
}