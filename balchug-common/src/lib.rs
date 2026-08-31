pub mod sprite;
pub mod atlas;
pub mod scenario;
pub mod atlas_builder;
pub mod api;
pub mod settings;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct F32Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for F32Rect {
    fn default() -> Self {
        F32Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }
    }
}