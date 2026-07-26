use std::collections::HashMap;
use binpack2d::{bin_new, Dimension, BinType};
use log::{debug, info};
use crate::atlas::{Atlas, AtlasItem};

pub fn build_atlas(dimensions: &[(i32, i32)], step_divider: u32, scale: bool) -> Atlas {
    let dimensions = dimensions.iter()
        .map(|(w, h)| Dimension::new(*w, *h))
        .collect::<Vec<_>>();
    let total_area: u32 = dimensions.iter()
        .map(|dimension| dimension.width() as u32 * dimension.height() as u32)
        .sum();
    let default_size = ((total_area as f64).sqrt() * 0.9) as u32;
    info!("Total images area {total_area}, default atlas size {default_size}");
    let (mut atlas_width, mut atlas_height) = (default_size, default_size);
    let mut is_extend_horizontal = true;
    let rectangles;
    loop {
        let mut bin = bin_new(BinType::MaxRects, atlas_width as i32, atlas_height as i32);
        let (inserted, rejected) = bin.insert_list(&dimensions);
        if rejected.is_empty() {
            info!("Atlas {atlas_width}x{atlas_height} occupancy of the bin: {:.1}%", bin.occupancy() * 100.0);
            rectangles = inserted;
            break;
        }
        debug!("Atlas {atlas_width}x{atlas_height} rejected images: {}", rejected.len());
        if is_extend_horizontal {
            atlas_width += (atlas_width / step_divider).max(1);
        } else {
            atlas_height += (atlas_height / step_divider).max(1);
        }
        is_extend_horizontal = !is_extend_horizontal;
    }
    let min_id = rectangles.iter().map(|rectangle| rectangle.id())
        .min().unwrap_or_default() as usize;
    let atlas_width = rectangles.iter()
        .map(|rect| (rect.x() + rect.width()) as u32)
        .max().unwrap_or(atlas_width);
    let atlas_height = rectangles.iter()
        .map(|rect| (rect.y() + rect.height()) as u32)
        .max().unwrap_or(atlas_height);
    let mut items = HashMap::new();
    rectangles.into_iter().map(|rect| {
        let original = &dimensions[rect.id() as usize - min_id];
        AtlasItem {
            id: rect.id() as usize - min_id,
            x: rect.x() as u32,
            y: rect.y() as u32,
            width: rect.width() as u32,
            height: rect.height() as u32,
            origin_width: original.width() as u32,
            origin_height: original.height() as u32,
        }
    }).for_each(|item| {
        items.insert(item.id, item);
    });
    pot_atlas(Atlas {
        width: atlas_width,
        height: atlas_height,
        items,
    }, scale)
}

fn pot_atlas(atlas: Atlas, scale: bool) -> Atlas {
    let mut pot = 2_u32;
    let (mut pot_width, mut pot_height) = (None, None);
    while pot_width.is_none() || pot_height.is_none() {
        if atlas.width <= pot && pot_width.is_none() {
            pot_width = Some(pot);
        }
        if atlas.height <= pot && pot_height.is_none() {
            pot_height = Some(pot);
        }
        pot *= 2;
    }
    if !scale {
        Atlas {
            width: pot_width.unwrap_or(pot),
            height: pot_height.unwrap_or(pot),
            items: atlas.items.clone(),
        }
    } else {
        let pot_width = pot_width.unwrap_or(pot) / 2;
        let pot_height = pot_height.unwrap_or(pot) / 2;
        let scale_x = pot_width as f64 / atlas.width as f64;
        let scale_y = pot_height as f64 / atlas.height as f64;
        info!("Resize atlas {}x{} -> {pot_width}x{pot_height}", atlas.width, atlas.height);
        let mut items = HashMap::new();
        atlas.items.values().for_each(|item| {
            items.insert(item.id, scale_item(item, scale_x, scale_y));
        });
        Atlas {
            width: pot_width,
            height: pot_height,
            items,
        }
    }
}

fn scale_item(item: &AtlasItem, scale_x: f64, scale_y: f64) -> AtlasItem {
    AtlasItem {
        id: item.id,
        x: (item.x as f64 * scale_x) as u32,
        y: (item.y as f64 * scale_y) as u32,
        width: (item.width as f64 * scale_x) as u32,
        height: (item.height as f64 * scale_y) as u32,
        origin_width: item.origin_width,
        origin_height: item.origin_height,
    }
}


pub fn gaps_atlas(atlas: Atlas, gap: u32) -> Atlas {
    let mut items = HashMap::new();
    atlas.items.values().for_each(|item| {
        let item_gaps = AtlasItem {
            id: item.id,
            x: item.x + gap,
            y: item.y + gap,
            width: item.width - 2 * gap,
            height: item.height - 2 * gap,
            origin_width: item.origin_width,
            origin_height: item.origin_height,
        };
        items.insert(item.id, item_gaps);
    });
    Atlas {
        width: atlas.width,
        height: atlas.height,
        items,
    }
}
