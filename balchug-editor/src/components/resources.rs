use crate::components::group_edit::GroupPropsEdit;
use crate::controllers::resources::ResourcesController;
use crate::states::project_state::SpriteGroup;
use dioxus::prelude::*;

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
                "Add Text Paragraph"
            }
        }
    }
}