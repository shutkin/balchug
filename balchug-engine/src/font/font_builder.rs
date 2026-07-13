use std::collections::HashMap;
use binpack2d::{bin_new, BinType, Dimension};
use balchug_common::atlas::{Atlas, AtlasItem, FontData};
use crate::font::glyphs_render::{prepare_glyphs, GlyphImage};

pub struct FontResult {
    pub data: Vec<u8>,
    pub font_data: FontData,
    pub atlas: Atlas,
}

pub fn build_font(letters: &str, data: &[u8], size: f32) -> Option<FontResult> {
    match prepare_glyphs(&letters, data, size) {
        Ok((font_data, glyph_images)) => {
            let dimensions = glyph_images.iter()
                .map(|img| img.to_dimensions())
                .collect::<Vec<_>>();
            let atlas = pot(build_atlas(&dimensions));
            let data = place_glyph_images(&atlas, &glyph_images);
            Some(FontResult {
                data, font_data, atlas,
            })
        },
        Err(e) => {
            web_sys::console::error_1(&format!("Failed to prepare glyphs: {}", e).into());
            None
        }
    }
}

pub fn pot(atlas: Atlas) -> Atlas {
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
    Atlas {
        width: pot_width.unwrap_or(pot),
        height: pot_height.unwrap_or(pot),
        items: atlas.items,
    }
}

fn build_atlas(dimensions: &[Dimension]) -> Atlas {
    let total_area: u32 = dimensions.iter()
        .map(|dimension| dimension.width() as u32 * dimension.height() as u32)
        .sum();
    let default_size = ((total_area as f64).sqrt() * 0.9) as u32;
    web_sys::console::log_1(&format!("Total images area {total_area}, default atlas size {default_size}").into());
    let (mut atlas_width, mut atlas_height) = (default_size, default_size);
    let mut is_extend_horizontal = true;
    let rectangles;
    loop {
        let mut bin = bin_new(BinType::MaxRects, atlas_width as i32, atlas_height as i32);
        let (inserted, rejected) = bin.insert_list(&dimensions);
        if rejected.is_empty() {
            web_sys::console::log_1(&format!("Occupancy of the bin: {:.1}%", bin.occupancy() * 100.0).into());
            rectangles = inserted;
            break;
        }
        web_sys::console::log_1(&format!("Atlas {atlas_width}x{atlas_height} rejected images: {}", rejected.len()).into());
        if is_extend_horizontal {
            atlas_width += (atlas_width / 16).max(1);
        } else {
            atlas_height += (atlas_height / 16).max(1);
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
    Atlas {
        width: atlas_width,
        height: atlas_height,
        items,
    }
}

fn place_glyph_images(atlas: &Atlas, images: &[GlyphImage]) -> Vec<u8> {
    let mut data = vec![0_u8; atlas.width as usize * atlas.height as usize];
    for item in atlas.items.values() {
        let image = &images[item.id];
        let mut atlas_offset = (item.y * atlas.width + item.x) as usize;
        let mut image_offset = 0;
        for _ in 0..image.height {
            data[atlas_offset..atlas_offset + image.width as usize].copy_from_slice(
                &image.data[image_offset..image_offset + image.width as usize]);
            atlas_offset += atlas.width as usize;
            image_offset += image.width as usize;
        }
    }
    data
}