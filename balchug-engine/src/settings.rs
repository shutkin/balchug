#[derive(Copy, Clone)]
pub struct Settings {
    pub background_color: [u8;3],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            background_color: [0, 0, 0],
        }
    }
}