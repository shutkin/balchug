use std::collections::HashMap;
use log::error;
use balchug_common::atlas::{Atlas, FontData};
use balchug_common::atlas_builder::build_atlas;
use crate::font::glyphs_render::{prepare_glyphs, GlyphImage};

pub struct BuildFontTask<'a> {
    pub font_index: usize,
    pub font_bytes: &'a [u8],
    pub letters: String,
    pub size: f32,
}

pub struct FontResult {
    pub data: Vec<u8>,
    pub fonts_data: HashMap<usize, FontData>,
    pub atlas: Atlas,
}

pub fn build_fonts(tasks: &[BuildFontTask]) -> Option<FontResult> {
    let mut prepared_fonts = Vec::new();
    let mut start_id = 0;
    for task in tasks {
        match prepare_glyphs(&task.letters, task.font_bytes, task.size, start_id) {
            Ok((font_data, glyph_images)) => {
                let dimensions = glyph_images.iter()
                    .map(|img| img.to_dimensions())
                    .collect::<Vec<_>>();
                start_id += glyph_images.len();
                prepared_fonts.push((task.font_index, font_data, glyph_images, dimensions));
            },
            Err(e) => {
                error!("Failed to prepare glyphs: {}", e);
                return None;
            }
        }
    }
    if prepared_fonts.is_empty() {
        None
    } else {
        let all_dimensions = prepared_fonts.iter()
            .flat_map(|(_, _, _, dimensions)| dimensions)
            .copied()
            .collect::<Vec<_>>();
        let mut fonts_data = HashMap::with_capacity(prepared_fonts.len());
        for (index, font, _, _) in &prepared_fonts {
            fonts_data.insert(*index, font.clone());
        }
        let all_glyphs = prepared_fonts.into_iter()
            .flat_map(|(_, _, glyphs, _)| glyphs)
            .collect::<Vec<_>>();
        let atlas = build_atlas(&all_dimensions, 16, false);
        let data = place_glyph_images(&atlas, &all_glyphs);
        Some(FontResult {
            data, fonts_data, atlas,
        })
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