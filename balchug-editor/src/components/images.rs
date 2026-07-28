use dioxus::prelude::*;
use crate::controllers::images_controller::ImagesController;

#[component]
pub fn ImagesBank(controller: ImagesController) -> Element {
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
fn ImageUploader(controller: ImagesController) -> Element {
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
fn ImageAsset(controller: ImagesController, id: usize, url: String) -> Element {
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
                    controller.put_image(id, 1.0);
                },
                "Put"
            }
        }
    }
}