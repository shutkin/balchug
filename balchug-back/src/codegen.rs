use std::collections::HashMap;
use std::fmt::Write;
use balchug_common::atlas::{Atlas, AtlasItem};
use balchug_common::sprite::{Easing, SpriteAnimation, SpriteData, SpriteState};
use crate::CommonError;

pub fn create_atlas() ->Atlas{
    let mut items = HashMap::new();
    items.insert(0,AtlasItem{id:0,x:0,y:0,width:100,height:100,origin_width:100,origin_height:100});
    Atlas{width:1024,height:1024,items}
}

pub fn atlas_to_code(atlas: &Atlas) -> Result<String, CommonError> {
    let mut code = String::new();

    code.push_str("use std::collections::HashMap;\n");
    code.push_str("use balchug_common::atlas::{Atlas,AtlasItem};\n");
    code.push_str("pub fn create_atlas()->Atlas{let mut items = HashMap::new();");

    for i in atlas.items.values() {
        write!(
            code,
            "items.insert({},AtlasItem{{id:{},x:{},y:{},width:{},height:{},origin_width:{},origin_height:{}}});",
            i.id,
            i.id,
            i.x,
            i.y,
            i.width,
            i.height,
            i.origin_width,
            i.origin_height,
        )?;
    }

    writeln!(code, "Atlas{{width:{},height:{},items}}}}", atlas.width, atlas.height)?;

    Ok(code)
}

pub fn animations_to_code(animations: &[SpriteAnimation]) -> Result<String, CommonError> {
    let mut code = String::new();

    code.push_str("use balchug_common::sprite::{SpriteAnimation,SpriteState,SpriteData,SpriteImageData,SpriteTextData,Easing};\n");
    code.push_str("pub fn create_animations()->Vec<SpriteAnimation>{vec![");

    for a in animations {
        write!(code, "SpriteAnimation{{sprite_id:{},smooth_factor:{:?},data:", a.sprite_id, a.smooth_factor)?;

        match &a.data {
            SpriteData::Image(img) => {
                write!(
                    code,
                    "SpriteData::Image(SpriteImageData{{atlas_item_id:{}}})",
                    img.atlas_item_id
                )?;
            }
            SpriteData::Text(txt) => {
                write!(
                    code,
                    "SpriteData::Text(SpriteTextData{{text:{:?}.to_string(),size:{:?}}})",
                    txt.text,
                    txt.size
                )?;
            }
        }

        code.push_str(",states:vec![");
        for st in &a.states {
            write!(
                code,
                "SpriteState{{offset:{:?},x:{:?},y:{:?},from_bottom:{:?},width:{:?},color:{:?},easing:{}}},",
                st.offset,
                st.x,
                st.y,
                st.from_bottom,
                st.scale,
                st.color,
                easing(st)
            )?;
        }
        code.push_str("]},");
    }

    code.push_str("]}\n");
    Ok(code)
}

fn easing(st: &SpriteState) -> &str {
    match st.easing {
        Easing::Linear => "Easing::Linear",
        Easing::InCubic => "Easing::InCubic",
        Easing::OutCubic => "Easing::OutCubic",
        Easing::InOutCubic => "Easing::InOutCubic",
        Easing::InSine => "Easing::InSine",
        Easing::OutSine => "Easing::OutSine",
        Easing::InOutSine => "Easing::InOutSine",
    }
}

pub const CARGO_TOML: &str = r#"
[package]
name = "balchug-demo"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
balchug-common = {path = "../balchug-common"}
balchug-engine = {path = "../balchug-engine"}
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["HtmlCanvasElement", "Window", "Document"] }
"#;

pub const TRUNK_TOML: &str = r#"
[build]
target = "index.html"
release = true
minify-html = "0.15.0"
minify-js = "0.5.6"
"#;

pub const LIB_CODE: &str = r#"
mod create_atlas;
mod create_scenario;

use balchug_engine::settings::Settings;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{window, HtmlCanvasElement, Event};

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let window = window().ok_or(JsValue::from_str("No global window found"))?;
    let document = window.document().ok_or(JsValue::from_str("No document found"))?;
    let canvas = document.get_element_by_id("canvas")
        .ok_or(JsValue::from_str("Canvas element not found"))?
        .dyn_into::<HtmlCanvasElement>()?;

    let settings = Settings {background_color: [{settings.background_color}]};
    let engine = balchug_engine::start_engine(window.clone(), canvas, settings);
    let atlas = create_atlas::create_atlas();
    engine.set_atlas("assets/atlas-{atlas_hash}.webp", atlas);
    engine.set_font("assets/font-{font.hash}.otf");
    engine.set_scenario(create_scenario::create_animations());

    let on_resize = {
        let engine = engine.clone();
        wasm_bindgen::closure::Closure::wrap(Box::new(move |_| {
            let _rect = engine.resize();
        }) as Box<dyn FnMut(Event)>)
    };
    window.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())?;
    on_resize.forget();

    let _rect = engine.resize();
    Ok(())
}"#;

pub const INDEX_HTML: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, height=device-height, initial-scale=1, maximum-scale=1, user-scalable=0"/>
    <link data-trunk rel="copy-dir" href="./assets"/>
    <title>{settings.name}</title>
    <style>
        body { margin: 0; overflow: hidden; background-color: rgb({settings.background_color}); }
        canvas { display: block;  }
    </style>
</head>
<body>
<div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; overflow: hidden;">
    <canvas id="canvas" style="display: block; width: 100%; height: 100%"></canvas>
</div>
</body>
</html>
"#;