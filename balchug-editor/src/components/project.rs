use dioxus::prelude::*;
use web_sys::window;
use crate::controllers::project_controller::ProjectController;
use crate::controllers::storage::{LocalStorage, KEY_PROJECT_ID};

#[component]
pub fn ProjectControl(controller: ProjectController) -> Element {
    let mut c0 = controller.clone();
    let mut c1 = controller.clone();
    let mut c2 = controller.clone();
    let mut c3 = controller.clone();
    let mut c4 = controller.clone();

    rsx! {
        section {
            id: "project_section",
            class: "panel-card",
            div {
                id: "project_control_header",
                class: "panel-header",
                h4 {
                    "Project Control"
                }
            }
            div {
                id: "project_control",
                class: "form-row",
                button {
                    id: "project_btn_new",
                    class: "btn btn-danger",
                    onclick: move |_| {
                        LocalStorage::remove(KEY_PROJECT_ID);
                        window().map(|window| window.location().reload());
                    },
                    "New Project"
                }
                button {
                    id: "project_btn_export",
                    class: "btn btn-secondary",
                    onclick: move |_| {
                        controller.download_distributive();
                    },
                    "Export Project"
                }
            }
        }
        section {
            id: "project_props",
            class: "panel-card",
            div {
                id: "project_props_header",
                class: "panel-header",
                h4 {
                    "Project Properties"
                }
            }
            div {
                id: "project_props_body",
                class: "panel-body",
                div {
                    id: "project_props_colors",
                    class: "form-row",
                    label {
                        "Name",
                        input {
                            r#type: "text",
                            value: "{c0.get_project_name()}",
                            onchange: move |event| {
                                let name = event.value();
                                c0.set_project_name(name);
                            }
                        }
                    }
                    label {
                        "Background Color"
                        input {
                            r#type: "color",
                            value: "{c1.get_background_color()}",
                            oninput: move |event| {
                                let color = event.value();
                                c1.set_background_color(color);
                            }
                        }
                    }
                    label {
                        "Text Color"
                        input {
                            r#type: "color",
                            value: "{c2.get_text_color()}",
                            oninput: move |event| {
                                let color = event.value();
                                c2.set_text_color(color);
                            }
                        }
                    }
                }
                div {
                    id: "project_props_viscosity",
                    class: "form-row",
                    label {
                        "Scroll Viscosity",
                        input {
                            id: "viscosity_range",
                            r#type: "range",
                            min: 0,
                            max: 100,
                            step: 5,
                            value: "{c3.get_viscosity()}",
                            oninput: move |event| {
                                if let Ok(v) = event.value().parse::<u8>() {
                                    c3.set_viscosity(v);
                                }
                            }
                        }
                    }
                    label {
                        "Scroll Inertion",
                        input {
                            id: "inertion_range",
                            r#type: "range",
                            min: 0,
                            max: 100,
                            step: 5,
                            value: "{c4.get_inertion()}",
                            oninput: move |event| {
                                if let Ok(v) = event.value().parse::<u8>() {
                                    c4.set_inertion(v);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}