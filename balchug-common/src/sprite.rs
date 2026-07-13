use crate::atlas::AtlasItem;

#[derive(Copy, Clone)]
pub struct Sprite {
    pub state: SpriteState,
    pub atlas_item: AtlasItem,
}

#[derive(Debug, Copy, Clone)]
pub struct SpriteState {
    pub offset: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

fn spline(x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    y0 * (x - x1) * (x - x2) / ((x0 - x1) * (x0 - x2))
        + y1 * (x - x0) * (x - x2) / ((x1 - x0) * (x1 - x2))
        + y2 * (x - x0) * (x - x1) / ((x2 - x0) * (x2 - x1))
}

impl SpriteState {
    pub fn interpolate(s0: &SpriteState, s1: &SpriteState, s2: &SpriteState, offset: f32) -> SpriteState {
        SpriteState {
            offset,
            x: spline(s0.offset, s0.x, s1.offset, s1.x, s2.offset, s2.x, offset),
            y: spline(s0.offset, s0.y, s1.offset, s1.y, s2.offset, s2.y, offset),
            width: spline(s0.offset, s0.width, s1.offset, s1.width, s2.offset, s2.width, offset),
            height: spline(s0.offset, s0.height, s1.offset, s1.height, s2.offset, s2.height, offset),
            color: [
                spline(s0.offset, s0.color[0], s1.offset, s1.color[0], s2.offset, s2.color[0], offset),
                spline(s0.offset, s0.color[1], s1.offset, s1.color[1], s2.offset, s2.color[1], offset),
                spline(s0.offset, s0.color[2], s1.offset, s1.color[2], s2.offset, s2.color[2], offset),
                spline(s0.offset, s0.color[3], s1.offset, s1.color[3], s2.offset, s2.color[3], offset),
            ],
        }
    }

    pub fn scale(&self, scale: f32) -> SpriteState {
        SpriteState {
            offset: self.offset,
            x: scale * self.x,
            y: scale * self.y,
            width: scale * self.width,
            height: scale * self.height,
            color: self.color,
        }
    }
}

#[derive(Clone)]
pub struct SpriteAnimation {
    pub sprite_id: usize,
    pub atlas_item_id: usize,
    pub animation: AnimationStates,
}

#[derive(Clone)]
pub struct AnimationStates {
    pub states: Vec<SpriteState>,
}

impl AnimationStates {
    pub fn interpolate_state(&self, offset: f32) -> Option<SpriteState> {
        for index in 0 .. self.states.len() - 1 {
            if offset >= self.states[index].offset && offset < self.states[index + 1].offset {
                return Some(self.interpolate(index, offset));
            }
        }
        None
    }

    fn interpolate(&self, state_index: usize, offset: f32) -> SpriteState {
        let i0 = if state_index > 0 { state_index - 1 } else { 0 };
        let i1 = (i0 + 1).min(self.states.len() - 1);
        let i2 = (i0 + 2).min(self.states.len() - 1);
        SpriteState::interpolate(
            &self.states[i0],
            &self.states[i1],
            &self.states[i2],
            offset,
        )
    }
}
