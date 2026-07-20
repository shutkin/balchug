pub mod sprite;
pub mod atlas;
pub mod scenario;

#[derive(Copy, Clone, Debug)]
pub struct F32Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
