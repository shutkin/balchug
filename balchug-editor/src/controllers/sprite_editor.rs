use std::cell::{Cell, RefCell};
use crate::states::sprite_editor::SpriteEditorState;
use balchug_engine::{start_engine, BalchugEngine, OffsetListener};
use dioxus::prelude::*;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, Window};
use balchug_common::atlas::AtlasItem;
use balchug_common::F32Rect;
use balchug_common::sprite::{SpriteAnimation, SpriteState};
use crate::components::timeline::{TimeLinePoint, TimeLinePoints};
use crate::states::project_state::ProjectState;

#[derive(Clone)]
pub struct SpriteEditController {
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    state: Signal<Option<SpriteEditorState>>,
    state_memo: Memo<Option<SpriteEditorState>>,
    preview_offset_listener: PreviewOffsetListener,
    canvas_rect: Rc<Cell<F32Rect>>,
    project_state: Rc<RefCell<ProjectState>>,
}

impl PartialEq for SpriteEditController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl SpriteEditController {
    pub fn new(engine: Rc<RefCell<Option<BalchugEngine>>>, project_state: Rc<RefCell<ProjectState>>) -> Self {
        let state = Signal::new(Option::<SpriteEditorState>::None);
        let state_memo = use_memo(move || state.read().cloned());
        Self {
            state,
            state_memo,
            engine,
            project_state,
            preview_offset_listener: PreviewOffsetListener::default(),
            canvas_rect: Rc::new(Cell::new(F32Rect::default())),
        }
    }

    pub fn start(&self, window: Window, canvas: HtmlCanvasElement) {
        let balchug_engine = start_engine(window, canvas);
        balchug_engine.set_offset_listener(Box::new(self.preview_offset_listener.clone()));
        self.engine.replace(Some(balchug_engine));
    }

    pub fn resize(&self) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let canvas_rect = engine.resize();
            self.canvas_rect.replace(canvas_rect);
            self.project_state.borrow_mut().aspect_ratio.set(canvas_rect.width / canvas_rect.height);
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        self.state_memo.read().is_some()
    }

    pub fn edit_mode_off(&mut self) {
        self.state.set(None);
    }

    pub fn get_cur_state(&self) -> Option<SpriteEditorState> {
        self.state_memo.read().cloned()
    }

    /*
    Timeline
     */
    pub fn set_timeline_point(&mut self, timeline_point: Option<TimeLinePoint>) {
        if let Some(point) = timeline_point {
            if let Some(engine) = self.engine.borrow().as_ref()
                && let Some(sprite_animation) = engine.get_scenario_images_states(Some(point.sprite_index)).first().cloned() {
                let atlas_item = engine.get_atlas_item(sprite_animation.atlas_item_id).unwrap();
                if let Some((offset, new_state)) = self.handle_new_point(point, &sprite_animation.states, atlas_item) {
                    engine.scroll_to_offset(offset);
                    self.state.set(Some(new_state));
                } else {
                    self.state.set(None);
                }
            }
        } else {
            self.state.set(None);
        }
    }

    fn handle_new_point(&self, point: TimeLinePoint, sprite_states: &[SpriteState], atlas_item: AtlasItem) -> Option<(f32, SpriteEditorState)> {
        if let Some(state) = self.state_memo.read().as_ref() {
            if state.timeline_points.sprite_index != point.sprite_index {
                let state = self.new_editor_state(
                    sprite_states[point.state_index], atlas_item,
                    point.sprite_index, vec![point.state_index]);
                return Some((sprite_states[point.state_index].offset, state));
            }
            let mut new_states = state.timeline_points.states_indices.clone();
            if new_states.contains(&point.state_index) {
                new_states.retain(|i| *i != point.state_index);
            } else {
                new_states.push(point.state_index);
            }
            if new_states.is_empty() {
                return None;
            }
            new_states.sort();
            let mut sum_offset = 0_f32;
            for &i in &new_states {
                sum_offset += sprite_states[i].offset;
            };
            let offset = sum_offset / new_states.len() as f32;
            if let Some(sprite_state) = BalchugEngine::interpolate_state(sprite_states, offset) {
                let state = self.new_editor_state(sprite_state, atlas_item, point.sprite_index, new_states);
                Some((offset, state))
            } else {
                None
            }
        } else {
            let state = self.new_editor_state(
                sprite_states[point.state_index], atlas_item,
                point.sprite_index, vec![point.state_index]);
            Some((sprite_states[point.state_index].offset, state))
        }
    }

    fn new_editor_state(&self, sprite_state: SpriteState, atlas_item: AtlasItem,
                        sprite_index: usize, states_indices: Vec<usize>) -> SpriteEditorState {
        let parallax_factor = self.project_state.borrow().sprite_parallax_factors.read()
            .get(&sprite_index).copied().unwrap_or(1.0);
        let rect = Self::scale_rect(sprite_state, self.canvas_rect.get(), atlas_item);
        SpriteEditorState {
            timeline_points: TimeLinePoints {sprite_index, states_indices},
            parallax_factor,
            sprite_state,
            original_sprite_state: sprite_state,
            rect,
        }
    }

    fn scale_rect(sprite_state: SpriteState, canvas_rect: F32Rect, atlas_item: AtlasItem) -> F32Rect {
        let proportion = atlas_item.origin_height as f32 / atlas_item.origin_width as f32;
        F32Rect {
            x: sprite_state.x * canvas_rect.width + canvas_rect.x,
            y: sprite_state.y * canvas_rect.width + canvas_rect.y,
            width: sprite_state.width * canvas_rect.width,
            height: sprite_state.width * canvas_rect.width * proportion,
        }
    }

    pub fn get_selected_points(&self) -> Option<TimeLinePoints> {
        self.state.read().as_ref().map(|s| s.timeline_points.clone())
    }

    pub fn get_preview_offset(&self) -> f32 {
        *self.preview_offset_listener.signal.read()
    }

    pub fn get_sprites_states(&self) -> Vec<SpriteAnimation> {
        if let Some(engine) = self.engine.borrow().as_ref() {
            engine.get_scenario_images_states(None)
        } else {
            Vec::new()
        }
    }

    /*
    Overlay
     */
    pub fn drag_offset(&mut self, start_offset: f32, dy: f32) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref()
            && let Some(sprite_animation) = engine.get_scenario_images_states(Some(state.timeline_points.sprite_index)).first() {
            let states = &sprite_animation.states;
            let points = state.timeline_points.states_indices.len();
            let offset_diapason = states[state.timeline_points.states_indices[0]].offset..=states[state.timeline_points.states_indices[points - 1]].offset;
            let new_offset = start_offset - dy / self.canvas_rect.get().height;
            let bound_offset = new_offset.max(*offset_diapason.start()).min(*offset_diapason.end());
            if let Some(new_sprite_state) = BalchugEngine::interpolate_state(states, bound_offset) {
                let atlas_item = engine.get_atlas_item(sprite_animation.atlas_item_id).unwrap();
                let canvas_rect = self.canvas_rect.get();
                let rect = Self::scale_rect(new_sprite_state, canvas_rect, atlas_item);
                self.state.set(Some(state.change_sprite_rect(rect, new_sprite_state)));
                engine.scroll_to_offset(new_offset);
            }
        }
    }

    pub fn set_sprite_rect(&mut self, new_rect: F32Rect) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().cloned()
            && let Some(mut sprite) = engine.get_scenario_images_states(Some(state.timeline_points.sprite_index)).first().cloned()
            && let Some(cur_state) = BalchugEngine::interpolate_state(&sprite.states, state.sprite_state.offset) {
            let canvas_rect = self.canvas_rect.get();
            let new_sprite_state = SpriteState {
                offset: cur_state.offset,
                x: (new_rect.x - canvas_rect.x) / canvas_rect.width,
                y: (new_rect.y - canvas_rect.y) / canvas_rect.width,
                width: new_rect.width / canvas_rect.width,
                color: cur_state.color,
            };
            Self::apply_states_change(&state.timeline_points, &mut sprite.states,
                                      new_sprite_state, state.parallax_factor);
            engine.set_image_sprite_states(state.timeline_points.sprite_index, sprite.states);
            self.state.set(Some(state.change_sprite_rect(new_rect, new_sprite_state)));
        }
    }

    fn apply_states_change(
        points: &TimeLinePoints,
        states: &mut [SpriteState],
        new_state: SpriteState,
        parallax_factor: f32,
    ) {
        for &index in &points.states_indices {
            let state = states[index];
            let dy = (new_state.offset - state.offset) * parallax_factor;
            let modified_state = SpriteState {
                offset: state.offset,
                x: new_state.x,
                y: new_state.y + dy,
                width: new_state.width,
                color: new_state.color,
            };
            states[index] = modified_state;
        }
    }

    /*
    State Editor
     */
    pub fn update_sprite_state(&self, new_state: SpriteState) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref()
            && let Some(mut sprite) = engine.get_scenario_images_states(Some(state.timeline_points.sprite_index)).first().cloned() {
            if state.timeline_points.states_indices.len() == 1 {
                let index = state.timeline_points.states_indices[0];
                sprite.states[index] = new_state;
            } else {
                Self::apply_states_change(&state.timeline_points, &mut sprite.states,
                                          new_state, state.parallax_factor);
            }
            engine.set_image_sprite_states(state.timeline_points.sprite_index, sprite.states);
        }
    }

    pub fn add_new_sprite_state(&mut self, is_before: bool) {
        /*if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read() {
            let states = &self.get_sprites_states()[state.sprite_index].states;
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
            self.timeline_points_signal.set(None);
        }*/
    }

    pub fn remove_sprite_state(&mut self) {
        /*if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = *self.state_memo.read() {
            engine.delete_image_sprite_state(state.sprite_index, state.state_index);
            self.state.set(None);
            self.timeline_points_signal.set(None);
        }*/
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
