use dioxus::prelude::*;
use balchug_common::scenario::Scenario;
use crate::components::timeline::TimeLinePoint;

#[component]
pub fn StateProps(scenario: Signal<Scenario>, cur_point: Signal<Option<TimeLinePoint>>) -> Element {
    let pnt = *cur_point.read();
    let state = pnt
        .and_then(|cur_point| {
            scenario.read().images.get(cur_point.object_index)
                .and_then(|o| o.animation.states.get(cur_point.state_index).copied())
        });
    if let Some(state) = state && let Some(pnt) = pnt {
        let mut apply_fn = move |values: Vec<(String, FormValue)>| {
            let mut new_scenario = scenario.read().clone();
            let new_state = new_scenario.images[pnt.object_index].animation.states.get_mut(pnt.state_index).unwrap();
            for (name, value) in values {
                let v = match value {
                    FormValue::Text(txt) => txt.parse::<f32>().unwrap_or(f32::NAN),
                    _ => f32::NAN,
                };
                if !v.is_nan() {
                    match name.as_str() {
                        "offset" => new_state.offset = v,
                        "x" => new_state.x = v,
                        "y" => new_state.y = v,
                        "scale" => new_state.width = v,
                        "alpha" => new_state.color[3] = v,
                        _ => {}
                    }
                }
            }
            scenario.set(new_scenario);
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
                        cur_point.set(None);
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
                                    value: "{state.offset}",
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
                                    value: "{state.x}",
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
                                    value: "{state.y}",
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
                                    value: "{state.width}",
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
                                    value: "{state.color[3]}",
                                    step: "0.001",
                                }
                            }
                        }
                    }
                    input {
                        id: "state_apply",
                        class: "btn btn-primary",
                        r#type: "submit",
                        "Apply"
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}