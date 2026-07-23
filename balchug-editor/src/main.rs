use dioxus::prelude::*;
use crate::components::workspace::Workspace;
use crate::controllers::sprite_editor::SpriteEditController;

mod components;
mod constants;
mod states;
mod controllers;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/style.css");

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let sprite_edit_controller = SpriteEditController::default();
    
    rsx! {
        Title { "Balchug Editor" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {controller: sprite_edit_controller}

    }
}
