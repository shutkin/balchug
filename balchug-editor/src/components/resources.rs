use crate::controllers::resources::ResourcesController;
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
        }
    }
}
