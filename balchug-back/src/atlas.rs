use std::fs::ReadDir;
use image::{DynamicImage, GenericImage, ImageReader, RgbaImage};
use image::imageops::FilterType;
use log::{error, info};
use balchug_common::atlas::Atlas;
use balchug_common::atlas_builder::{build_atlas, gaps_atlas};
use crate::CommonError;

pub fn create_empty_atlas(atlas_path: &str) -> Result<(), CommonError> {
    let image = RgbaImage::new(4, 4);
    let encoder = webp::Encoder::from_rgba(image.as_raw(), 4, 4);
    let webp = encoder.encode_simple(false, 90.0)
        .map_err(|err| format!("failed to encode atlas.webp: {err:?}"))?;
    std::fs::write(atlas_path, webp.to_vec())?;
    Ok(())
}

pub fn create_atlas(images_dir: &str, atlas_path: &str) -> Result<Atlas, CommonError> {
    let dir = std::fs::read_dir(images_dir)?;
    
    let images = read_images(dir);
    let dimensions = images.iter()
        .map(|image| (image.width() as i32, image.height() as i32))
        .collect::<Vec<_>>();
    let atlas = gaps_atlas(build_atlas(&dimensions, 512, true), 2);

    let atlas_image = place_images(&atlas, &images);
    let encoder = webp::Encoder::from_rgba(atlas_image.as_raw(), atlas.width, atlas.height);
    let webp = encoder.encode_simple(false, 90.0)
        .map_err(|err| format!("failed to encode atlas.webp: {err:?}"))?;
    std::fs::write(atlas_path, webp.to_vec())?;
    Ok(atlas)

    /*let mut code = Vec::new();
    code.push("use std::collections::HashMap;\nuse crate::atlas::{Atlas, AtlasItem, FontData, FontGlyph};\n".to_string());
    code.push(atlas.to_code("create_atlas"));
    if let Err(err) = std::fs::write("const.rs", &code.join("\n")) {
        error!("Failed to save creation code: {err:?}");
    }*/
}

fn read_images(dir: ReadDir) -> Vec<DynamicImage> {
    let mut images = Vec::new();
    for entry in dir {
        let entry = entry.expect("failed to read dir entry");
        if let Ok(reader) = ImageReader::open(entry.path())
            && let Ok(image) = reader.decode() {
            let width = image.width();
            let height = image.height();
            info!("Read image {:?}: {width}x{height}", entry.path());
            images.push(image);
        }
    }
    images
}

fn place_images(atlas: &Atlas, images: &[DynamicImage]) -> RgbaImage {
    let mut atlas_image = RgbaImage::new(atlas.width, atlas.height);
    for item in atlas.items.values() {
        let image = &images[item.id];
        let resized = image.resize_exact(item.width, item.height, FilterType::CatmullRom);
        if let Err(err) = atlas_image.copy_from(&resized, item.x, item.y) {
            error!("Failed to copy image: {err}");
        }
    }
    atlas_image
}
