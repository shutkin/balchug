use dioxus::prelude::*;

use crate::components::workspace::Workspace;
use crate::constants::{build_atlas, build_scenario};

mod components;
mod constants;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/style.css");

fn main() {
    launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    let atlas = build_atlas();
    let scenario = build_scenario(&atlas.items, &[2, 1, 5, 7, 10, 6, 3, 8, 4, 9]);

    let atlas_signal = Signal::new(atlas);
    let scenario_signal = Signal::new(scenario);
    let preview_offset_signal = Signal::new(0_f32);

    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        // In addition to element and text (which we will see later), rsx can contain other components. In this case,
        // we are using the `document::Link` component to add a link to our favicon and main CSS file into the head of our app.
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {
            atlas: atlas_signal,
            scenario: scenario_signal,
            preview_offset: preview_offset_signal,
        }

    }
}
