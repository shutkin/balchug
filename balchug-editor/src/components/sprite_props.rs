use dioxus::prelude::*;
use balchug_common::sprite::{SpriteAnimation, SpriteData};
use crate::controllers::resources::ResourcesController;
use crate::states::project_state::SpriteProperties;

#[component]
pub fn SpritePropsDialog(controller: ResourcesController) -> Element {
    let c0 = controller.clone();
    let c1 = controller.clone();

    rsx! {
        if let Some(sprite_id) = *controller.get_edit_sprite_signal().read()
            && let Some((props, animation)) = controller.get_sprite_props(sprite_id) {
            div {
                id: "sprite_dialog_overlay",
                class: "modal-overlay",
                div {
                    id: "sprite_dialog_box",
                    class: "modal-box",
                    form {
                        id: "sprite_dialog_body",
                        class: "panel-body",
                        onsubmit: {
                            let mut props = props.clone();
                            let mut animation = animation.clone();
                            move |e| {
                                parse_values(e.values(), &mut animation, &mut props);
                                c0.set_sprite_props(sprite_id, &props, &animation);
                                c0.get_edit_sprite_signal().set(None);
                                e.prevent_default();
                            }
                        },
                        h4 {
                            "Edit sprite {props.title}"
                        }
                        label {
                            "Title",
                            input {
                                id: "sprite_dialog_title",
                                name: "title",
                                r#type: "text",
                                value: "{props.title}",
                            }
                        }
                        if let SpriteData::Text(data) = animation.data.clone() {
                            label {
                                "Text",
                                input {
                                    id: "sprite_dialog_title",
                                    name: "text",
                                    r#type: "text",
                                    value: "{data.text}"
                                }
                            }
                            label {
                                "Size"
                                select {
                                    id: "text_size_select",
                                    name: "size",
                                    for i in 15..=30 {
                                        option {
                                            selected: i == data.size,
                                            "{i}"
                                        }
                                    }
                                }
                            }
                        }
                        label {
                            "Parallax Factor",
                            input {
                                id: "sprite_dialog_parallax",
                                name: "parallax",
                                r#type: "range",
                                min: "0.5",
                                max: "2",
                                step: "0.1",
                                value: "{props.parallax_factor}",
                            }
                        }
                        label {
                            "States Transition Smoothness",
                            input {
                                id: "sprite_dialog_smoothness",
                                name: "smoothness",
                                r#type: "range",
                                min: "0.0",
                                max: "0.75",
                                step: "0.05",
                                value: "{animation.smooth_factor}",
                            }
                        }
                        div {
                            id: "sprite_dialog_cntrl",
                            class: "form-row",
                            input {
                                id: "sprite_dialog_submit",
                                class: "btn btn-primary",
                                r#type: "submit",
                                "Ok"
                            }
                            button {
                                id: "sprite_dialog_cancel",
                                class: "btn btn-secondary",
                                formmethod: "dialog",
                                onclick: move |_| {
                                    c1.get_edit_sprite_signal().set(None);
                                },
                                "Cancel"
                            }
                            button {
                                id: "sprite_dialog_remove",
                                class: "btn btn-danger",
                                "Remove"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn parse_values(values: Vec<(String, FormValue)>, animation: &mut SpriteAnimation, props: &mut SpriteProperties) {
    for (name, value) in values {
        let v = match value {
            FormValue::Text(txt) => txt,
            FormValue::File(_) => String::new(),
        };
        match name.as_str() {
            "title" => props.title = v,
            "parallax" => props.parallax_factor = v.parse::<f32>().unwrap_or(1.0),
            "smoothness" => animation.smooth_factor = v.parse::<f32>().unwrap_or(0.5),
            "size" => if let SpriteData::Text(data) = &mut animation.data {
                data.size = v.parse::<u8>().unwrap_or(15)
            },
            "text" => if let SpriteData::Text(data) = &mut animation.data {
                data.text = v
            },
            _ => {}
        }
    }
}
