use dioxus::prelude::*;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::SpriteState;
use crate::components::timeline::TimeLinePoint;
use crate::states::sprite_state_edit::SpriteStateEdit;

#[component]
pub fn StateEditor(
    edit_state: Signal<Option<SpriteStateEdit>>,
    scenario: Signal<Scenario>,
    selected_point: Signal<Option<TimeLinePoint>>,
) -> Element {
    let memo = use_memo(move || *edit_state.read());
    let scenario_memo = use_memo(move || scenario.read().clone());

    if let Some(se) = *memo.read() {
        let mut apply_fn = move |values: Vec<(String, FormValue)>| {
            let mut state = SpriteState::default();
            for (name, value) in values {
                let v = match value {
                    FormValue::Text(txt) => txt.parse::<f32>().unwrap_or(f32::NAN),
                    _ => f32::NAN,
                };
                if !v.is_nan() {
                    match name.as_str() {
                        "offset" => state.offset = v,
                        "x" => state.x = v,
                        "y" => state.y = v,
                        "scale" => state.width = v,
                        "alpha" => state.color[3] = v,
                        _ => {}
                    }
                }
            }
            let mut ns = scenario_memo.read().clone();
            ns.images[se.sprite_index].animation.states[se.state_index] = state;
            scenario.set(ns);
            selected_point.set(None);
        };

        rsx! {
            section {
                id: "state_props_container",
                class: "panel-card",
                div {
                    id: "state_props_header",
                    class: "panel-header",
                    h4 {
                        "State Properties"
                    }
                }
                form {
                    id: "state_props_body",
                    class: "panel-body",
                    onsubmit: move |event| {
                        event.prevent_default();
                        apply_fn(event.values());
                    },
                    div {
                        id: "state_props_row1",
                        class: "form-row",
                        div {
                            id: "state_offset",
                            class: "form-group",
                            label {
                                "Offset",
                                input {
                                    r#type: "number",
                                    name: "offset",
                                    value: "{round(se.state.offset)}",
                                    step: "0.001",
                                }
                            }
                        }
                        div {
                            id: "state_x",
                            class: "form-group",
                            label {
                                "X",
                                input {
                                    r#type: "number",
                                    name: "x",
                                    value: "{round(se.state.x)}",
                                    step: "0.001",
                                }
                            }
                        }
                        div {
                            id: "state_y",
                            class: "form-group",
                            label {
                                "Y",
                                input {
                                    r#type: "number",
                                    name: "y",
                                    value: "{round(se.state.y)}",
                                    step: "0.001",
                                }
                            }
                        }
                    }
                    div {
                        id: "state_props_row2",
                        class: "form-row",
                        div {
                            id: "state_width",
                            class: "form-group",
                            label {
                                "Scale",
                                input {
                                    r#type: "number",
                                    name: "scale",
                                    value: "{round(se.state.width)}",
                                    step: "0.001",
                                }
                            }
                        }
                        div {
                            id: "state_alpha",
                            class: "form-group",
                            label {
                                "Alpha",
                                input {
                                    r#type: "number",
                                    name: "alpha",
                                    value: "{round(se.state.color[3])}",
                                    step: "0.001",
                                }
                            }
                        }
                    }
                    div {
                        id: "state_props_btn_row",
                        class: "form-row",
                        input {
                            id: "state_apply",
                            class: "btn btn-primary",
                            r#type: "submit",
                            "Apply"
                        }
                        button {
                            id: "state_cancel",
                            class: "btn btn-secondary",
                            onclick: move |_| {
                                let mut ns = scenario_memo.read().clone();
                                ns.images[se.sprite_index].animation.states[se.state_index] = se.original_state;
                                scenario.set(ns);
                                selected_point.set(None);
                            },
                            "Cancel"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn round(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}