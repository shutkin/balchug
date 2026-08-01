use std::fmt::Write;
use balchug_common::sprite::{SpriteAnimation, SpriteData};

pub fn animations_to_code(animations: &[SpriteAnimation]) -> String {
    let mut code = String::new();

    code.push_str("use balchug_common::sprite::{SpriteAnimation,SpriteState,SpriteData,SpriteImageData,SpriteTextData};\n");
    code.push_str("pub fn create_animations()->Vec<SpriteAnimation>{vec![");

    for a in animations {
        write!(code, "SpriteAnimation{{sprite_id:{},data:", a.sprite_id).unwrap();

        match &a.data {
            SpriteData::Image(img) => {
                write!(
                    code,
                    "SpriteData::Image(SpriteImageData{{atlas_item_id:{}}})",
                    img.atlas_item_id
                ).unwrap();
            }
            SpriteData::Text(txt) => {
                write!(
                    code,
                    "SpriteData::Text(SpriteTextData{{text:{:?}.to_string(),relative_height:{:?}}})",
                    txt.text,
                    txt.relative_height
                ).unwrap();
            }
        }

        code.push_str(",states:vec![");
        for st in &a.states {
            write!(
                code,
                "SpriteState{{offset:{:?},x:{:?},y:{:?},width:{:?},color:{:?}}},",
                st.offset,
                st.x,
                st.y,
                st.width,
                st.color
            ).unwrap();
        }
        code.push_str("]},");
    }

    code.push_str("]}\n");
    code
}