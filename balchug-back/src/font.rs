use std::hash::{DefaultHasher, Hash, Hasher};
use allsorts::binary::read::ReadScope;
use allsorts::font::read_cmap_subtable;
use allsorts::font_data::FontData;
use allsorts::gsub::{GlyphOrigin, RawGlyph, RawGlyphFlags};
use allsorts::subset;
use allsorts::subset::{CmapTarget, SubsetProfile};
use allsorts::tables::cmap::{Cmap, CmapSubtable};
use allsorts::tables::FontTableProvider;
use allsorts::tinyvec::tiny_vec;
use allsorts::unicode::VariationSelector;
use log::info;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::SpriteData;
use crate::CommonError;

pub fn subset_font(font_path: &str, scenario: &Scenario) -> Result<(Vec<u8>, String), CommonError> {
    let scenario_chars = get_scenario_chars(scenario);
    let mut hasher = DefaultHasher::new();
    scenario_chars.hash(&mut hasher);
    let hash = format!("{:x}", hasher.finish());

    let buffer = std::fs::read(font_path)?;
    let font_file = ReadScope::new(&buffer).read::<FontData>()?;
    let provider = font_file.table_provider(0)?;

    let mut glyphs = chars_to_glyphs(&provider, &scenario_chars)?;
    let notdef = RawGlyph {
        unicodes: tiny_vec![],
        glyph_index: 0,
        liga_component_pos: 0,
        glyph_origin: GlyphOrigin::Direct,
        flags: RawGlyphFlags::empty(),
        variation: None,
        extra_data: (),
    };
    glyphs.insert(0, Some(notdef));

    let mut glyphs: Vec<RawGlyph<()>> = glyphs.into_iter().flatten().collect();
    glyphs.sort_by_key(|a| a.glyph_index);
    let mut glyph_ids = glyphs
        .iter()
        .map(|glyph| glyph.glyph_index)
        .collect::<Vec<_>>();
    glyph_ids.dedup();
    if glyph_ids.is_empty() {
        return Err("no glyphs left in font".into());
    }

    info!("Number of glyphs in new font: {}", glyph_ids.len());

    // Subset
    let profile = SubsetProfile::Minimal;
    let cmap_target = CmapTarget::Unicode;
    let new_font = subset::subset(&provider, &glyph_ids, &profile, cmap_target)?;
    info!("Subset font {} -> {}", buffer.len(), new_font.len());
    Ok((new_font, hash))
}

fn chars_to_glyphs<F: FontTableProvider>(
    font_provider: &F,
    text: &str,
) -> Result<Vec<Option<RawGlyph<()>>>, CommonError> {
    let cmap_data = font_provider.read_table_data(allsorts::tag::CMAP)?;
    let cmap = ReadScope::new(&cmap_data).read::<Cmap>()?;
    let (_, cmap_subtable) =
        read_cmap_subtable(&cmap)?.ok_or("no suitable cmap sub-table found")?;

    let glyphs = text
        .chars()
        .map(|ch| map_glyph(&cmap_subtable, ch, None))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(glyphs)
}

fn map_glyph(
    cmap_subtable: &CmapSubtable,
    ch: char,
    variation: Option<VariationSelector>,
) -> Result<Option<RawGlyph<()>>, CommonError> {
    if let Some(glyph_index) = cmap_subtable.map_glyph(ch as u32)? {
        let glyph = make_glyph(ch, glyph_index, variation);
        Ok(Some(glyph))
    } else {
        Ok(None)
    }
}

fn make_glyph(
    ch: char,
    glyph_index: u16,
    variation: Option<VariationSelector>,
) -> RawGlyph<()> {
    RawGlyph {
        unicodes: tiny_vec![[char; 1] => ch],
        glyph_index,
        liga_component_pos: 0,
        glyph_origin: GlyphOrigin::Char(ch),
        flags: RawGlyphFlags::empty(),
        variation,
        extra_data: (),
    }
}

fn get_scenario_chars(scenario: &Scenario) -> String {
    let mut chars = Vec::new();
    for animation in &scenario.sprites {
        if let SpriteData::Text(data) = &animation.data {
            data.text.chars().for_each(|ch| {
                if !chars.contains(&ch) {
                    chars.push(ch);
                }
            })
        }
    }
    chars.sort();
    String::from_iter(chars)
}