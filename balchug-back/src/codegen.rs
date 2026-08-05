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
        write!(code, "SpriteAnimation{{sprite_id:{},data:", a.sprite_id)?;

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
                    "SpriteData::Text(SpriteTextData{{text:{:?}.to_string(),relative_height:{:?}}})",
                    txt.text,
                    txt.relative_height
                )?;
            }
        }

        code.push_str(",states:vec![");
        for st in &a.states {
            write!(
                code,
                "SpriteState{{offset:{:?},x:{:?},y:{:?},width:{:?},color:{:?},easing:{}}},",
                st.offset,
                st.x,
                st.y,
                st.width,
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