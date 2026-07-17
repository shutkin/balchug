use std::collections::HashMap;
use balchug_common::atlas::{Atlas, AtlasItem};
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{AnimationStates, SpriteAnimation, SpriteState, TextLine};

pub fn build_atlas() -> Atlas {
    let mut items = HashMap::with_capacity(10);
    items.insert(6, AtlasItem {id:6,x:1367,y:2,width:1361,height:1792,origin_width:1920,origin_height:2876});
    items.insert(3, AtlasItem {id:3,x:2,y:2,width:1361,height:1794,origin_width:1920,origin_height:2880});
    items.insert(2, AtlasItem {id:2,x:2732,y:3298,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(7, AtlasItem {id:7,x:2732,y:1600,width:1361,height:894,origin_width:1920,origin_height:1439});
    items.insert(9, AtlasItem {id:9,x:1367,y:3495,width:1361,height:517,origin_width:1920,origin_height:835});
    items.insert(8, AtlasItem {id:8,x:1367,y:1798,width:1361,height:894,origin_width:1920,origin_height:1438});
    items.insert(4, AtlasItem {id:4,x:2732,y:2499,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(5, AtlasItem {id:5,x:2,y:1800,width:1361,height:1794,origin_width:1920,origin_height:2880});
    items.insert(1, AtlasItem {id:1,x:1367,y:2696,width:1361,height:795,origin_width:1920,origin_height:1280});
    items.insert(0, AtlasItem {id:0,x:2732,y:2,width:1361,height:1594,origin_width:1920,origin_height:2560});
    Atlas {width:4096,height:4096,items}
}

pub fn build_scenario(items: &HashMap<usize, AtlasItem>, pictures: &[usize]) -> Scenario {
    let mut scenario = Scenario::default();
    let mut offset = 0.0;
    let mut sprite_id = 0;
    for atlas_item_id in pictures {
        if let Some(item) = items.get(atlas_item_id) {
            let item_height = item.origin_height as f32 / item.origin_width as f32;
            let mut states = Vec::new();
            states.push(SpriteState {
                offset: -item_height,
                x: 0.0,
                y: offset + 0.25,
                width: 1.0,
                height: item_height,
                color: [0.0, 0.0, 0.0, 1.0],
            });
            if sprite_id > 0 {
                states.push(SpriteState {
                    offset: offset - item_height + 0.01,
                    x: 0.0,
                    y: item_height + 0.26,
                    width: 1.0,
                    height: item_height,
                    color: [0.0, 0.0, 0.0, 1.0],
                });
            }
            states.push(SpriteState {
                offset: if item_height < 1.0 {offset} else {offset + item_height * 0.5},
                x: 0.0,
                y: if item_height < 1.0 {0.0} else {-item_height * 0.5} + 0.25,
                width: 1.0,
                height: item_height,
                color: [0.0, 0.0, 0.0, 1.0],
            });
            states.push(SpriteState {
                offset: offset + item_height,
                x: -0.25,
                y: -item_height * 1.5,
                width: 1.5,
                height: item_height * 1.5,
                color: [0.0, 0.0, 0.0, 0.25],
            });
            scenario.images.push(SpriteAnimation {
                sprite_id,
                atlas_item_id: item.id,
                animation: AnimationStates { states },
            });
            sprite_id += 1;
            offset += item_height + 0.25;
        }
    }

    let mut states = Vec::new();
    states.push(SpriteState {
        offset: -1.0,
        x: 0.25,
        y: 0.015,
        width: 1.0,
        height: 1.0,
        color: [0.0, 0.0, 0.0, 1.0],
    });
    states.push(SpriteState {
        offset: 0.0,
        x: 0.25,
        y: 0.015,
        width: 1.0,
        height: 1.0,
        color: [0.0, 0.0, 0.0, 1.0],
    });
    states.push(SpriteState {
        offset: 0.125,
        x: 0.0,
        y: 0.01,
        width: 1.0,
        height: 1.5,
        color: [0.3, 0.1, 0.7, 0.0],
    });
    scenario.text_lines.push(TextLine {
        text: "Мотай вниз. Там самое интересное.".to_string(),
        relative_height: 0.024,
        animation: AnimationStates { states },
    });

    scenario
}
