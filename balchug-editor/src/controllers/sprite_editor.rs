use std::cell::{Cell, RefCell};
use crate::states::sprite_editor::SpriteEditorState;
use balchug_engine::{start_engine, BalchugEngine, OffsetListener};
use dioxus::prelude::*;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, Window};
use balchug_common::atlas::AtlasItem;
use balchug_common::F32Rect;
use balchug_common::sprite::{SpriteAnimation, SpriteState};
use crate::components::timeline::TimeLinePoint;
use crate::constants::{build_atlas, build_scenario};

static ASSETS_DIR: Asset = asset!("/assets");

#[derive(Clone)]
pub struct SpriteEditController {
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    state: Signal<Option<SpriteEditorState>>,
    state_memo: Memo<Option<SpriteEditorState>>,
    preview_offset_listener: PreviewOffsetListener,
    timeline_point_signal: Signal<Option<TimeLinePoint>>,
    canvas_rect: Rc<Cell<F32Rect>>,
}

impl PartialEq for SpriteEditController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Default for SpriteEditController {
    fn default() -> Self {
        let state = Signal::new(Option::<SpriteEditorState>::None);
        let state_memo = use_memo(move || *state.read());
        Self {
            state,
            state_memo,
            preview_offset_listener: PreviewOffsetListener::default(),
            timeline_point_signal: Signal::new(None),
            engine: Rc::new(RefCell::new(None)),
            canvas_rect: Rc::new(Cell::new(F32Rect::default())),
        }
    }
}

impl SpriteEditController {
    fn get_scenario_state(&self, sprite_index: usize, state_index: usize) -> Option<(AtlasItem, SpriteState)> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let sprite = engine.get_scenario_images().get(sprite_index)?.clone();
            let state = *sprite.animation.states.get(state_index)?;
            let atlas_item = engine.get_atlas_item(sprite.atlas_item_id)?;
            Some((atlas_item, state))
        } else {
            None
        }
    }

    pub fn start(&self, window: Window, canvas: HtmlCanvasElement) {
        let balchug_engine = start_engine(window, canvas, &ASSETS_DIR.to_string());
        web_sys::console::log_1(&"Engine start".into());
        balchug_engine.set_offset_listener(Box::new(self.preview_offset_listener.clone()));
        let atlas = build_atlas();
        balchug_engine.set_scenario(build_scenario(&atlas.items, &[2, 1, 5, 7, 10, 6, 3, 8, 4, 9]));
        balchug_engine.set_atlas(atlas);
        self.engine.replace(Some(balchug_engine));
    }

    pub fn resize(&self) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let canvas_rect = engine.resize();
            self.canvas_rect.replace(canvas_rect);
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        self.timeline_point_signal.read().is_some()
    }

    pub fn edit_mode_off(&mut self) {
        self.timeline_point_signal.set(None);
    }

    pub fn get_cur_state(&self) -> Option<SpriteEditorState> {
        *self.state_memo.read()
    }

    /*
    Timeline
     */
    pub fn timeline_point_listener(&mut self) -> Signal<Option<TimeLinePoint>> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            if let Some(point) = *self.timeline_point_signal.read()
                && let Some((atlas_item, sprite_state)) = self.get_scenario_state(point.sprite_index, point.state_index) {
                engine.scroll_to_offset(sprite_state.offset);

                let proportion = atlas_item.origin_height as f32 / atlas_item.origin_width as f32;
                let canvas_rect = self.canvas_rect.get();
                let rect = F32Rect {
                    x: sprite_state.x * canvas_rect.width + canvas_rect.x,
                    y: sprite_state.y * canvas_rect.width + canvas_rect.y,
                    width: sprite_state.width * canvas_rect.width,
                    height: sprite_state.width * canvas_rect.width * proportion,
                };

                let s = SpriteEditorState {
                    sprite_index: point.sprite_index,
                    state_index: point.state_index,
                    sprite_state,
                    original_sprite_state: sprite_state,
                    rect,
                };
                self.state.set(Some(s));
            } else {
                self.state.set(None);
            }
        }
        self.timeline_point_signal
    }

    pub fn set_timeline_point(&mut self, timeline_point: Option<TimeLinePoint>) {
        self.timeline_point_signal.set(timeline_point);
    }

    pub fn get_preview_offset(&self) -> f32 {
        *self.preview_offset_listener.signal.read()
    }

    pub fn get_sprites_states(&self) -> Vec<SpriteAnimation> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            engine.get_scenario_images()
        } else {
            Vec::new()
        }
    }

    /*
    Overlay
     */
    pub fn set_sprite_rect(&mut self, new_rect: F32Rect) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read()
            && let Some((_, cur_state)) = self.get_scenario_state(state.sprite_index, state.state_index) {
            let canvas_rect = self.canvas_rect.get();
            let new_sprite_state = SpriteState {
                offset: cur_state.offset,
                x: (new_rect.x - canvas_rect.x) / canvas_rect.width,
                y: (new_rect.y - canvas_rect.y) / canvas_rect.width,
                width: new_rect.width / canvas_rect.width,
                color: cur_state.color,
            };
            engine.set_image_sprite_state(new_sprite_state, state.sprite_index, state.state_index);
            self.state.set(Some(state.change_sprite_rect(new_rect, new_sprite_state)));
        }
    }

    /*
    State Editor
     */
    pub fn update_sprite_state(&self, new_state: SpriteState) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read() {
            engine.set_image_sprite_state(new_state, state.sprite_index, state.state_index);
        }
    }

    pub fn add_new_sprite_state(&mut self, is_before: bool) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read() {
            let states = &self.get_sprites_states()[state.sprite_index].animation.states;
            let new_state_index;
            let mut new_state = state.original_sprite_state;
            if is_before {
                new_state_index = state.state_index;
                if state.state_index == 0 {
                    new_state.offset -= 1.0;
                } else {
                    new_state = Self::half_states(&new_state, &states[state.state_index - 1]);
                }
            } else {
                new_state_index = state.state_index + 1;
                if state.state_index == states.len() - 1 {
                    new_state.offset += 1.0;
                } else {
                    new_state = Self::half_states(&new_state, &states[state.state_index + 1]);
                }
            }
            engine.insert_image_sprite_state(new_state, state.sprite_index, new_state_index);
            self.state.set(None);
            self.timeline_point_signal.set(None);
        }
    }

    pub fn remove_sprite_state(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read() {
            engine.delete_image_sprite_state(state.sprite_index, state.state_index);
            self.state.set(None);
            self.timeline_point_signal.set(None);
        }
    }

    fn half_states(state0: &SpriteState, state1: &SpriteState) -> SpriteState {
        SpriteState {
            offset: (state0.offset + state1.offset) * 0.5,
            x: (state0.x + state1.x) * 0.5,
            y: (state0.y + state1.y) * 0.5,
            width: (state0.width + state1.width) * 0.5,
            color: [
                (state0.color[0] + state1.color[0]) * 0.5,
                (state0.color[1] + state1.color[1]) * 0.5,
                (state0.color[2] + state1.color[2]) * 0.5,
                (state0.color[3] + state1.color[3]) * 0.5
            ],
        }
    }
}

#[derive(Clone, Default)]
struct PreviewOffsetListener {
    signal: Signal<f32>,
}

impl OffsetListener for PreviewOffsetListener {
    fn offset_change(&mut self, offset: f32) {
        self.signal.set(offset);
    }
}
