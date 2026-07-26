use dioxus::prelude::*;
use crate::controllers::images_controller::ImagesController;

#[component]
pub fn ImageUploader(controller: ImagesController) -> Element {
    rsx! {
        div {
            id: "image_uploader_body",
            class: "panel-body",
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
}

#[component]
pub fn ImagesList(controller: ImagesController) -> Element {
    rsx! {
        div {
            id: "image_list_container",
            class: "asset-item",
            for (i, thumb_url) in controller.get_thumbs().iter().enumerate() {
                div {
                    id: format!("thumb_{i}"),
                    class: "asset-preview",
                    img {
                        src: thumb_url,
                    }
                }
            }
        }
    }
}