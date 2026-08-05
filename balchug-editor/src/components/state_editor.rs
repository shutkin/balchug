use dioxus::prelude::*;
use balchug_common::sprite::Easing;
use crate::controllers::sprite_editor::SpriteEditController;
use crate::states::sprite_editor::SpriteEditorState;

const EASING_LINEAR: &'static str = "Linear";
const EASING_INCUBIC: &'static str = "In Cubic";
const EASING_OUTCUBIC: &'static str = "Out Cubic";
const EASING_INOUTCUBIC: &'static str = "In-Out Cubic";

fn map_str_to_easing(str: &str) -> Easing {
    match str {
        EASING_LINEAR => Easing::Linear,
        EASING_INCUBIC => Easing::InCubic,
        EASING_OUTCUBIC => Easing::OutCubic,
        EASING_INOUTCUBIC => Easing::InOutCubic,
        _ => Easing::Linear,
    }
}

fn map_easing_to_str(easing: Easing) -> &'static str {
    match easing {
        Easing::Linear => EASING_LINEAR,
        Easing::InCubic => EASING_INCUBIC,
        Easing::OutCubic => EASING_OUTCUBIC,
        Easing::InOutCubic => EASING_INOUTCUBIC,
        _ => EASING_LINEAR,
    }
}

#[component]
pub fn StateEditor(controller: SpriteEditController) -> Element {
    if let Some(se) = controller.get_cur_state() {
        let mut c0 = controller.clone();
        let mut c1 = controller.clone();
        let mut c2 = controller.clone();
        let mut c3 = controller.clone();

        let mut apply_fn = move |values: Vec<(String, FormValue)>| {
            let mut state = c0.get_cur_state()
                .map(|s| s.original_sprite_state).unwrap_or_default();
            for (name, value) in values {
                let txt = match value {
                    FormValue::Text(txt) => txt,
                    FormValue::File(_) => String::default(),
                };
                let num = txt.parse::<f32>().unwrap_or(f32::NAN);
                if !num.is_nan() {
                    match name.as_str() {
                        "offset" => state.offset = num,
                        "x" => state.x = num,
                        "y" => state.y = num,
                        "scale" => state.width = num,
                        "alpha" => state.color[3] = num,
                        _ => {}
                    }
                } else {
                    match name.as_str() {
                        "easing" => state.easing = map_str_to_easing(&txt),
                        _ => {}
                    }
                }
            }
            c0.update_sprite_state(state);
            c0.edit_mode_off();
            let c = c0.clone();
            use_resource(move || {
                let c = c.clone();
                async move {
                    c.send_scenario().await
                }
            });
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
                    StateStatsInputs {se: se.clone()},
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
                                c1.update_sprite_state(se.original_sprite_state);
                                c1.edit_mode_off();
                            },
                            "Cancel"
                        }
                        div {
                            class: "vert-separator",
                        }
                        if controller.is_modify_states_possible(false, true) {
                            button {
                                id: "state_delete",
                                class: "btn btn-danger",
                                onclick: move |_| {
                                    c2.remove_sprite_state();
                                },
                                "Delete"
                            }
                        }
                        if controller.is_modify_states_possible(true, false) {
                            button {
                                id: "state_add",
                                class: "btn btn-secondary",
                                onclick: move |_| {
                                    c3.add_new_sprite_state();
                                },
                                "Add New State"
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn StateStatsInputs(se: SpriteEditorState) -> Element {
    rsx! {
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
                        value: "{round(se.sprite_state.offset)}",
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
                        value: "{round(se.sprite_state.x)}",
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
                        value: "{round(se.sprite_state.y)}",
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
                        value: "{round(se.sprite_state.width)}",
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
                        value: "{round(se.sprite_state.color[3])}",
                        step: "0.001",
                    }
                }
            }
            div {
                id: "state_easing",
                class: "form-group",
                label {
                    "Easing",
                    select {
                        id: "easing_select",
                        name: "easing",
                        value: "{map_easing_to_str(se.sprite_state.easing)}",
                        option {
                            selected: se.sprite_state.easing == Easing::Linear,
                            "{EASING_LINEAR}"
                        }
                        option {
                            selected: se.sprite_state.easing == Easing::InCubic,
                            "{EASING_INCUBIC}"
                        }
                        option {
                            selected: se.sprite_state.easing == Easing::OutCubic,
                            "{EASING_OUTCUBIC}"
                        }
                        option {
                            selected: se.sprite_state.easing == Easing::InOutCubic,
                            "{EASING_INOUTCUBIC}"
                        }
                    }
                }
            }
        }
    }
}

fn round(value: f32) -> f32 {
    (value * 1000.0).round() / 1000.0
}