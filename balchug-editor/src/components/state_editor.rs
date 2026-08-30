use dioxus::prelude::*;
use crate::controllers::sprite_editor::{SpriteEditController, ALL_EASING_VARIANTS, map_easing_to_str};
use crate::states::sprite_editor::SpriteEditorState;

#[component]
pub fn StateEditor(controller: SpriteEditController) -> Element {
    if let Some(se) = controller.get_cur_state() {
        let mut c0 = controller.clone();
        let mut c1 = controller.clone();
        let mut c2 = controller.clone();

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
                div {
                    id: "state_props_body",
                    class: "panel-body",
                    StateStatsInputs {controller: controller.clone(), se: se.clone()},
                    div {
                        id: "state_props_btn_row",
                        class: "form-row",
                        if controller.is_group_fixing_possible() {
                            button {
                                id: "state_delete",
                                class: "btn btn-secondary",
                                onclick: move |_| {
                                    c0.fix_group(true);
                                },
                                "Fix States"
                            }
                        } else {
                            button {
                                id: "state_delete",
                                class: "btn btn-secondary",
                                onclick: move |_| {
                                    c0.fix_group(false);
                                },
                                "Unfix States"
                            }
                        }
                        if controller.is_modify_states_possible(false, true) {
                            button {
                                id: "state_delete",
                                class: "btn btn-danger",
                                onclick: move |_| {
                                    c1.remove_sprite_state();
                                },
                                "Delete"
                            }
                        }
                        if controller.is_modify_states_possible(true, false) {
                            button {
                                id: "state_add",
                                class: "btn btn-secondary",
                                onclick: move |_| {
                                    c2.add_new_sprite_state();
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
fn StateStatsInputs(controller: SpriteEditController, se: SpriteEditorState) -> Element {
    let mut c0 = controller.clone();
    let mut c1 = controller.clone();
    let mut c2 = controller.clone();
    let mut c3 = controller.clone();
    let mut c4 = controller.clone();
    let mut c5 = controller.clone();
    let mut c6 = controller.clone();

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
                        oninput: move |e| {
                            c0.input_change("offset", &e.value());
                        },
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
                        oninput: move |e| {
                            c1.input_change("x", &e.value());
                        },
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
                        oninput: move |e| {
                            c2.input_change("y", &e.value());
                        },
                    }
                }
            }
        }
        div {
            id: "state_props_row2",
            class: "form-row",
            div {
                id: "state_from_bottom",
                class: "form-group",
                label {
                    "Y-Axis Origin",
                    select {
                        id: "y_axis_select",
                        name: "y_axis",
                        oninput: move |e| {
                            c3.input_change("y_axis", &e.value());
                        },
                        option {
                            selected: !se.sprite_state.from_bottom,
                            "Top"
                        }
                        option {
                            selected: se.sprite_state.from_bottom,
                            "Bottom"
                        }
                    }
                }
            }
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
                        oninput: move |e| {
                            c4.input_change("scale", &e.value());
                        },
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
                        value: "{se.sprite_state.color[3]}",
                        step: "1",
                        oninput: move |e| {
                            c5.input_change("alpha", &e.value());
                        },
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
                        oninput: move |e| {
                            c6.input_change("easing", &e.value());
                        },
                        for &easing in ALL_EASING_VARIANTS.iter() {
                            option {
                                selected: se.sprite_state.easing == easing,
                                "{map_easing_to_str(easing)}"
                            }
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
