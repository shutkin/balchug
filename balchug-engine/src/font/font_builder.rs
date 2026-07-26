use log::error;
use balchug_common::atlas::{Atlas, FontData};
use balchug_common::atlas_builder::build_atlas;
use crate::font::glyphs_render::{prepare_glyphs, GlyphImage};

pub struct FontResult {
    pub data: Vec<u8>,
    pub font_data: FontData,
    pub atlas: Atlas,
}

pub fn build_font(letters: &str, data: &[u8], size: f32) -> Option<FontResult> {
    match prepare_glyphs(letters, data, size) {
        Ok((font_data, glyph_images)) => {
            let dimensions = glyph_images.iter()
                .map(|img| img.to_dimensions())
                .collect::<Vec<_>>();
            let atlas = build_atlas(&dimensions, 16, false);
            let data = place_glyph_images(&atlas, &glyph_images);
            Some(FontResult {
                data, font_data, atlas,
            })
        },
        Err(e) => {
            error!("Failed to prepare glyphs: {}", e);
            None
        }
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