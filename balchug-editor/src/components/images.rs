use dioxus::prelude::*;
use crate::controllers::resources::ResourcesController;
use crate::states::project_state::SpriteProperties;

#[component]
pub fn ImagesBank(controller: ResourcesController) -> Element {
    rsx! {
        section {
            id: "images_bank_section",
            class: "panel-card",
            div {
                id: "images_bank_header",
                class: "panel-header",
                h4 {
                    "Image Bank"
                }
            }
            div {
                id: "images_bank_body",
                class: "panel-body",
                ImageUploader {controller: controller.clone()}
                for (i, thumb_url) in controller.get_thumbs().iter().enumerate() {
                    ImageAsset {
                        controller: controller.clone(),
                        id: i,
                        url: thumb_url,
                    }
                }
            }
        }
    }
}

#[component]
fn ImageUploader(controller: ResourcesController) -> Element {
    rsx! {
        label {
            id: "image_uploader_label",
            class: "file-dropzone",
            input {
                id: "image_upload_input",
                style: "display:none;",
                r#type: "file",
                accept: "image/*",
                onchange: move |event| {
                    let mut c = controller.clone();
                    async move {
                        c.handle_upload(event.files()).await;
                    }
                }
            }
            span {
                "Upload Image"
            }
        }
    }
}

#[component]
fn ImageAsset(controller: ResourcesController, id: usize, url: String) -> Element {
    let asset_name = format!("{id}");
    rsx! {
        div {
            id: format!("image_asset_{id}"),
            class: "asset-item",
            img {
                class: "asset-preview",
                src: url,
            }
            span {
                class: "asset-name",
                "{asset_name}"
            }
            button {
                id: format!("image_{id}_put"),
                class: "btn btn-secondary",
                onclick: move |_| {
                    controller.get_sprite_id_signal().set(Some(id));
                },
                "Put"
            }
        }
    }
}

#[component]
pub fn SpriteDialog(mut controller: ResourcesController) -> Element {
    rsx! {
        if let Some(sprite_id) = *controller.get_sprite_id_signal().read() {
            div {
                id: "sprite_dialog_overlay",
                class: "modal-overlay",
                div {
                    id: "sprite_dialog_box",
                    class: "modal-box",
                    form {
                        id: "sprite_dialog_body",
                        class: "panel-body",
                        onsubmit: move |e| {
                            controller.get_sprite_id_signal().set(None);
                            let props = parse_sprite_props(e.values());
                            controller.put_image(sprite_id, props);
                            e.prevent_default();
                        },
                        h4 {
                            "Add image {sprite_id}"
                        }
                        label {
                            "Title",
                            input {
                                id: "sprite_dialog_title",
                                name: "title",
                                r#type: "text",
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
                                value: "1.0",
                            }
                        }
                        input {
                            id: "sprite_dialog_submit",
                            class: "btn btn-primary",
                            r#type: "submit",
                            "Ok"
                        }
                    }
                }
            }
        }
    }
}

fn parse_sprite_props(values: Vec<(String, FormValue)>) -> SpriteProperties {
    let mut props = SpriteProperties::default();
    for (name, value) in values {
        let v = match value {
            FormValue::Text(txt) => txt,
            FormValue::File(_) => String::new(),
        };
        match name.as_str() {
            "title" => props.title = v,
            "parallax" => props.parallax_factor = v.parse::<f32>().unwrap_or(1.0),
            _ => {}
        }
    }
    props
}

#[component]
pub fn TextLine(controller: ResourcesController) -> Element {
    let c0 = controller.clone();
    let apply_fn = move |values: Vec<(String, FormValue)>| {
        let mut text = String::new();
        let mut size = 10;
        for (name, value) in values {
            let txt_value = match value {
                FormValue::Text(txt) => txt,
                _ => String::new(),
            };
            if name == "text" {
                text = txt_value;
            } else {
                size = txt_value.parse::<i32>().unwrap_or(10);
            }
        }
        let title = text.chars().take(12).collect::<String>();
        c0.put_text(text, size, SpriteProperties {title, parallax_factor: 1.0});
    };

    rsx! {
        section {
            id: "text_section",
            class: "panel-card",
            div {
                id: "text_header",
                class: "panel-header",
                h4 {
                    "Text Line"
                }
            }
            form {
                id: "text_body",
                class: "panel-body",
                onsubmit: move |event| {
                    event.prevent_default();
                    apply_fn(event.values());
                },
                div {
                    id: "text_body_row",
                    class: "form-row",
                    label {
                        "Text"
                        input {
                            id: "text_body_input",
                            name: "text",
                            r#type: "text",
                        }
                    }
                    label {
                        "Size"
                        select {
                            id: "text_size_select",
                            name: "size",
                            option {"10"}
                            option {"12"}
                            option {"14"}
                            option {"16"}
                        }
                    }
                }
                div {
                    id: "text_submit_row",
                    class: "form-row",
                    input {
                        id: "text_submit",
                        class: "btn btn-primary",
                        r#type: "submit",
                        "Put"
                    }
                }
            }
        }
    }
}