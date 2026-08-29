use crate::components::group_edit::GroupPropsEdit;
use crate::controllers::resources::ResourcesController;
use crate::states::project_state::SpriteGroup;
use balchug_common::sprite::{SpriteAnimation, SpriteData, SpriteImageData, SpriteTextData};
use dioxus::prelude::*;
use web_sys::console::group;

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
                ThumbsList {controller: controller.clone()}
                ImageUploader {controller: controller.clone()}
            }
        }
    }
}

#[component]
fn ThumbsList(controller: ResourcesController) -> Element {
    let thumbs = controller.get_thumbs();
    rsx! {
        div {
            id: "images_bank_list",
            class: "panel-resources",
            for row_index in 0..thumbs.len().div_ceil(3) {
                div {
                    id: "images_bank_row_{row_index}",
                    class: "asset-items-row",
                    for i in row_index * 3..(row_index + 1) * 3 {
                        if i < thumbs.len() {
                            ImageAsset {
                                controller: controller.clone(),
                                id: i,
                                url: thumbs[i].clone(),
                            }
                        }
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
    rsx! {
        div {
            id: format!("image_asset_{id}"),
            class: "asset-item",
            img {
                class: "asset-thumb",
                src: url,
            }
            button {
                id: format!("image_{id}_put"),
                class: "btn btn-secondary",
                onclick: move |_| {
                    controller.get_image_adding_signal().set(Some(id));
                },
                "Put"
            }
        }
    }
}

#[component]
pub fn AddImageDialog(mut controller: ResourcesController) -> Element {
    rsx! {
        if let Some(image_id) = *controller.get_image_adding_signal().read() {
            GroupPropsEdit {
                group: SpriteGroup::new_image(image_id),
                group_id: None,
                controller,
            }
        }
    }
}

#[component]
pub fn AddTextDialog(mut controller: ResourcesController) -> Element {
    rsx! {
        if *controller.get_text_adding_open().read() {
            GroupPropsEdit {
                group: SpriteGroup::new_text(),
                group_id: None,
                controller,
            }
        }
    }
}

fn parse_group_props(values: Vec<(String, FormValue)>, image_id: Option<usize>) -> SpriteGroup {
    let mut group = SpriteGroup {
        title: String::new(),
        data: if let Some(image_id) = image_id {
            SpriteData::Image(SpriteImageData{
                atlas_item_id: image_id
            })
        } else {
            SpriteData::Text(SpriteTextData {
                text: String::new(),
                size: 15,
            })
        },
        parallax_factor: 1.0,
        smooth_factor: 0.5,
        max_width: 1.0,
        states: Vec::new(),
    };
    for (name, value) in values {
        let v = match value {
            FormValue::Text(txt) => txt,
            FormValue::File(_) => String::new(),
        };
        match name.as_str() {
            "title" => group.title = v,
            "parallax" => group.parallax_factor = v.parse::<f32>().unwrap_or(1.0),
            "smoothness" => group.smooth_factor = v.parse::<f32>().unwrap_or(0.5),
            "size" => if let SpriteData::Text(data) = &mut group.data {data.size = v.parse::<u8>().unwrap_or(15)},
            "text" => if let SpriteData::Text(data) = &mut group.data {data.text = v},
            _ => {}
        }
    }
    group
}

#[component]
pub fn TextLine(controller: ResourcesController) -> Element {
    let c0 = controller.clone();

    rsx! {
        section {
            id: "text_section",
            class: "panel-card",
            button {
                id: "add_text_line",
                class: "btn btn-secondary",
                onclick: move |_| {
                    c0.get_text_adding_open().set(true);
                },
                "Add Text Line"
            }
        }
    }
}