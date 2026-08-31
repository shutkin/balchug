use crate::controllers::resources::ResourcesController;
use crate::states::project_state::SpriteGroup;
use balchug_common::sprite::SpriteData;
use dioxus::prelude::*;

#[component]
pub fn GroupEditDialog(controller: ResourcesController) -> Element {
    if let Some(group_id) = *controller.get_edit_group_signal().read() {
        let group = controller.get_group(group_id);
        rsx! {
            GroupPropsEdit {
                group,
                group_id: Some(group_id),
                controller,
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
pub fn GroupPropsEdit(group: SpriteGroup, group_id: Option<usize>, controller: ResourcesController) -> Element {
    let mut c0 = controller.clone();
    let c1 = controller.clone();

    let title = group.title.clone();
    let mut title = use_signal(move || title);
    let (text, size) = if let SpriteData::Text(data) = &group.data {
        (data.text.clone(), data.size)
    } else {
        (String::new(), 15)
    };
    let mut text = use_signal(move || text);
    let mut size = use_signal(move || size);
    let mut parallax = use_signal(move || group.parallax_factor);
    let mut smoothness = use_signal(move || group.smooth_factor);

    rsx! {
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
                        let mut group = group.clone();
                        move |e| {
                            parse_values(e.values(), &mut group);
                            if let Some(group_id) = group_id {
                                c0.update_group(group_id, &group);
                            } else {
                                c0.add_new_group_animation(&group);
                            }
                            c0.close_popups();
                            e.prevent_default();
                        }
                    },
                    h4 {
                        if group_id.is_some() {
                            "Edit object '{group.title}'"
                        } else {
                            "Add object"
                        }
                    }
                    label {
                        "Title",
                        input {
                            id: "sprite_dialog_title",
                            name: "title",
                            r#type: "text",
                            value: "{title.read()}",
                            oninput: move |e| {title.set(e.value());}
                        }
                    }
                    if let SpriteData::Text(_) = group.data.clone() {
                        label {
                            "Text",
                            textarea {
                                id: "sprite_dialog_title",
                                name: "text",
                                rows: 4,
                                value: "{text.read()}",
                                oninput: move |e| {text.set(e.value());}
                            }
                        }
                        label {
                            "Size"
                            select {
                                id: "text_size_select",
                                name: "size",
                                onchange: move |e: Event<FormData>| {
                                    let v = e.value().parse::<u8>().unwrap_or(15);
                                    size.set(v);
                                },
                                for i in 15_u8..=30_u8 {
                                    option {
                                        selected: i == *size.read(),
                                        "{i}"
                                    }
                                }
                            }
                        }
                    }
                    label {
                        "Parallax Factor: {parallax.read()}",
                        input {
                            id: "sprite_dialog_parallax",
                            name: "parallax",
                            r#type: "range",
                            min: "0.5",
                            max: "2",
                            step: "0.1",
                            value: "{parallax.read()}",
                            oninput: move |event| {
                                if let Ok(v) = event.value().parse::<f32>() {
                                    parallax.set(v);
                                }
                            }
                        }
                    }
                    label {
                        "States Transition Smoothness: {smoothness.read()}",
                        input {
                            id: "sprite_dialog_smoothness",
                            name: "smoothness",
                            r#type: "range",
                            min: "0.0",
                            max: "0.75",
                            step: "0.05",
                            value: "{smoothness.read()}",
                            oninput: move |event| {
                                if let Ok(v) = event.value().parse::<f32>() {
                                    smoothness.set(v);
                                }
                            }
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
                                c1.get_edit_group_signal().set(None);
                            },
                            "Cancel"
                        }
                        if group_id.is_some() {
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

fn parse_values(values: Vec<(String, FormValue)>, group: &mut SpriteGroup) {
    for (name, value) in values {
        let v = match value {
            FormValue::Text(txt) => txt,
            FormValue::File(_) => String::new(),
        };
        info!("Parse group parameter {name} = '{v}'");
        match name.as_str() {
            "title" => group.title = v,
            "parallax" => group.parallax_factor = v.parse::<f32>().unwrap_or(1.0),
            "smoothness" => group.smooth_factor = v.parse::<f32>().unwrap_or(0.5),
            "size" => {
                if let SpriteData::Text(data) = &mut group.data {
                    data.size = v.parse::<u8>().unwrap_or(15)
                }
            }
            "text" => {
                if let SpriteData::Text(data) = &mut group.data {
                    data.text = v
                }
            }
            _ => {}
        }
    }
}
