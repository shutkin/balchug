use dioxus::prelude::*;
use balchug_engine::BalchugEngine;
use crate::components::workspace::Workspace;
use crate::constants::{build_atlas, build_scenario};

mod components;
mod constants;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/styling/style.css");

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    let atlas = build_atlas();
    let scenario = build_scenario(&atlas.items, &[2, 1, 5, 7, 10, 6, 3, 8, 4, 9]);

    let atlas_signal = Signal::new(atlas);
    let scenario_signal = Signal::new(scenario);
    let engine: Signal<Option<BalchugEngine>> = use_signal(|| None);
    use_effect(move || {
        let atlas = atlas_signal.read().clone();
        if let Some(engine) = engine.read().as_ref() {
            engine.set_atlas(atlas);
        }
    });
    use_effect(move || {
        let scenario = scenario_signal.read().clone();
        if let Some(engine) = engine.read().as_ref() {
            engine.set_scenario(scenario);
        }
    });

    rsx! {
        Title { "Balchug Editor" }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }

        Workspace {
            atlas: atlas_signal,
            scenario: scenario_signal,
            engine,
        }

    }
}
