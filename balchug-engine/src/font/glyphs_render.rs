use ab_glyph::{Font, FontRef, Glyph, ScaleFont};
use balchug_common::atlas::{FontData, FontGlyph};
use std::collections::HashMap;

const PADDING: u16 = 4;

pub struct GlyphImage {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

impl GlyphImage {
    pub fn to_dimensions(&self) -> (i32, i32) {
        (self.width as i32, self.height as i32)
    }
}

pub struct GlyphData {
    pub offset_x: f32,
    pub offset_y: f32,
    pub h_advance: f32,
    pub img: GlyphImage,
}

pub fn prepare_glyphs(
    letters: &str,
    font_data: &[u8],
    text_size: f32,
    start_id: usize,
) -> Result<(FontData, Vec<GlyphImage>), Box<dyn std::error::Error>> {
    let font = FontRef::try_from_slice(font_data)?;
    let font_scaled = font.as_scaled(text_size);

    let mut glyphs = HashMap::new();
    let mut images = Vec::new();
    for c in letters.chars() {
        if c.is_control() || c.is_whitespace() || glyphs.contains_key(&c) {
            continue;
        }
        let glyph_id = font.glyph_id(c);
        let glyph = glyph_id.with_scale(text_size);
        let h_advance = font_scaled.h_advance(glyph_id);
        if let Some(sprite) = render_glyph(glyph, font.clone(), h_advance) {
            glyphs.insert(c, FontGlyph {
                h_advance: sprite.h_advance,
                offset_x: sprite.offset_x,
                offset_y: sprite.offset_y,
                item_id: images.len() + start_id,
            });
            images.push(sprite.img);
        }
    }

    let space_width = font.glyph_bounds(&font.glyph_id(' ').with_scale(text_size)).width();
    let font = FontData {
        space_width,
        ascend: font_scaled.ascent(),
        height: font_scaled.height(),
        line_gap: font_scaled.line_gap(),
        glyphs,
    };
    Ok((font, images))
}

fn render_glyph(glyph: Glyph, font: FontRef, h_advance: f32) -> Option<GlyphData> {
    let outlined = font.outline_glyph(glyph)?;
    let bounds = outlined.px_bounds();
    let width = bounds.width().ceil() as u16 + PADDING * 2;
    let height = bounds.height().ceil() as u16 + PADDING * 2;
    let mut buffer = vec![0_u8; width as usize * height as usize];
    outlined.draw(|x, y, a| {
        let (x, y) = (x as u16 + PADDING, y as u16 + PADDING);
        if x < width && y < height {
            let a = (a * 255.0) as u8;
            buffer[y as usize * width as usize + x as usize] = a;
        }
    });

    let img = GlyphImage {
        width,
        height,
        data: buffer,
    };
    Some(GlyphData {
        offset_x: bounds.min.x - PADDING as f32,
        offset_y: bounds.min.y - PADDING as f32,
        h_advance,
        img,
    })
}
