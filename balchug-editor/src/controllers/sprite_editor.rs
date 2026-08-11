use crate::components::timeline::{TimeLinePoint, TimeLinePoints};
use crate::states::project_state::ProjectState;
use crate::states::sprite_editor::SpriteEditorState;
use balchug_common::F32Rect;
use balchug_common::sprite::{SpriteAnimation, SpriteData, SpriteState};
use balchug_engine::{BalchugEngine, OffsetListener, start_engine};
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use web_sys::{HtmlCanvasElement, Window};

#[derive(Clone)]
pub struct SpriteEditController {
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    state: Signal<Option<SpriteEditorState>>,
    state_memo: Memo<Option<SpriteEditorState>>,
    preview_offset_listener: PreviewOffsetListener,
    canvas_rect: Rc<Cell<F32Rect>>,
    project_state: Rc<RefCell<ProjectState>>,
    scenario_update: Rc<Cell<bool>>,
    sprites_update: Rc<Cell<bool>>,
}

impl PartialEq for SpriteEditController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl SpriteEditController {
    pub fn new(
        engine: Rc<RefCell<Option<BalchugEngine>>>,
        project_state: Rc<RefCell<ProjectState>>,
        scenario_update: Rc<Cell<bool>>,
        sprites_update: Rc<Cell<bool>>,
    ) -> Self {
        let state = Signal::new(Option::<SpriteEditorState>::None);
        let state_memo = use_memo(move || state.read().cloned());
        let ps0 = project_state.clone();
        let ps1 = project_state.clone();
        let selected_sprite = use_memo(move || *ps0.borrow().selected_sprite.read());

        let controller = Self {
            state,
            state_memo,
            engine,
            project_state,
            preview_offset_listener: PreviewOffsetListener::default(),
            canvas_rect: Rc::new(Cell::new(F32Rect::default())),
            scenario_update,
            sprites_update,
        };
        let mut c0 = controller.clone();
        use_effect(move || {
            if let Some(sprite_id) = *selected_sprite.read() {
                c0.set_timeline_sprite(sprite_id);
                ps1.borrow_mut().unselect_sprite();
            }
        });

        controller
    }

    pub fn get_canvas_rect(&self) -> F32Rect {
        self.canvas_rect.get()
    }

    pub fn start(&self, window: Window, canvas: HtmlCanvasElement) {
        let balchug_engine = start_engine(window, canvas, Default::default());
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
    
    fn sprite_proportion(engine: &BalchugEngine, sprite_animation: &SpriteAnimation) -> f32 {
        match &sprite_animation.data {
            SpriteData::Image(image_data) => {
                engine.get_atlas_item(image_data.atlas_item_id).map(|item| {
                    item.origin_width as f32 / item.origin_height as f32
                }).unwrap_or(1.0)
            }
            SpriteData::Text(text_data) => {
                1.0 / text_data.relative_height
            }
        }
    }

    /*
    Timeline
     */
    pub fn get_sprite_titles(&self) -> Vec<String> {
        let sprites = self.get_sprites_states();
        sprites.iter().map(|sprite| {
            self.project_state.borrow().get_sprite_properties(sprite.sprite_id).title
        }).collect()
    }
    
    pub fn remove_sprite(&mut self, sprite_id: usize) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let mut sprites = engine.get_sprites_animations(None);
            for sprite in sprites.iter_mut() {
                if sprite.sprite_id > sprite_id {
                    sprite.sprite_id -= 1;
                }
            }
            sprites.retain(|sprite| sprite.sprite_id != sprite_id);
            engine.set_scenario(sprites);
            self.project_state.borrow_mut().sprite_properties.remove(&sprite_id);
            self.sprites_update.set(true);
        }
    }
    
    pub fn set_timeline_sprite(&mut self, sprite_id: usize) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(sprite_animation) = engine.get_sprites_animations(Some(sprite_id)).first().cloned() {
            let indices = (0..sprite_animation.states.len()).collect::<Vec<_>>();
            let cur_offset = engine.get_offset();
            let fixed_offset = if cur_offset >= sprite_animation.states[0].offset
                && cur_offset <= sprite_animation.states[sprite_animation.states.len() - 1].offset {
                Some(cur_offset)
            } else {
                None
            };
            if let Some((offset, state)) = self.interpolate_selected_states(
                engine, &sprite_animation, indices, fixed_offset) {
                engine.scroll_to_offset(offset);
                self.state.set(Some(state));
            }
        }
    }
    
    pub fn set_timeline_point(&mut self, timeline_point: Option<TimeLinePoint>) {
        if let Some(point) = timeline_point {
            if let Some(engine) = self.engine.borrow().as_ref()
                && let Some(sprite_animation) = engine.get_sprites_animations(Some(point.sprite_index)).first().cloned() {
                if let Some((offset, new_state)) = self.handle_new_point(engine, &sprite_animation, point) {
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

    fn handle_new_point(
        &self,
        engine: &BalchugEngine,
        sprite_animation: &SpriteAnimation,
        point: TimeLinePoint,
    ) -> Option<(f32, SpriteEditorState)> {
        let sprite_states = &sprite_animation.states;
        if let Some(state) = self.state_memo.read().as_ref() {
            if state.timeline_points.sprite_index != point.sprite_index {
                let state = self.new_editor_state(
                    engine, sprite_animation, sprite_states[point.state_index],
                    point.sprite_index, vec![point.state_index]);
                return Some((sprite_states[point.state_index].offset, state));
            }
            let mut new_indices = state.timeline_points.states_indices.clone();
            if new_indices.contains(&point.state_index) {
                new_indices.retain(|i| *i != point.state_index);
            } else {
                new_indices.push(point.state_index);
            }
            if new_indices.is_empty() {
                return None;
            }
            new_indices.sort();
            self.interpolate_selected_states(engine, sprite_animation, new_indices, None)
        } else {
            let state = self.new_editor_state(
                engine, sprite_animation, sprite_states[point.state_index],
                point.sprite_index, vec![point.state_index]);
            Some((sprite_states[point.state_index].offset, state))
        }
    }
    
    fn interpolate_selected_states(
        &self,
        engine: &BalchugEngine,
        sprite_animation: &SpriteAnimation,
        indices: Vec<usize>,
        fixed_offset: Option<f32>
    ) -> Option<(f32, SpriteEditorState)> {
        let offset = fixed_offset.unwrap_or_else(|| {
            let mut sum_offset = 0_f32;
            for &i in &indices {
                sum_offset += sprite_animation.states[i].offset;
            };
            sum_offset / indices.len() as f32
        });
        if let Some(sprite_state) = engine.interpolate_state(sprite_animation, offset) {
            let state = self.new_editor_state(
                engine, sprite_animation, sprite_state, sprite_animation.sprite_id, indices);
            Some((offset, state))
        } else {
            None
        }
    }

    fn new_editor_state(
        &self,
        engine: &BalchugEngine,
        sprite_animation: &SpriteAnimation,
        sprite_state: SpriteState,
        sprite_id: usize,
        states_indices: Vec<usize>
    ) -> SpriteEditorState {
        let parallax_factor = self.project_state.borrow().get_sprite_properties(sprite_id).parallax_factor;
        let rect = Self::scale_rect(engine, sprite_animation, sprite_state, self.canvas_rect.get());
        SpriteEditorState {
            timeline_points: TimeLinePoints { sprite_index: sprite_id, states_indices},
            parallax_factor,
            sprite_state,
            original_sprite_state: sprite_state,
            rect,
        }
    }

    fn scale_rect(
        engine: &BalchugEngine,
        sprite_animation: &SpriteAnimation,
        sprite_state: SpriteState,
        canvas_rect: F32Rect
    ) -> F32Rect {
        let proportion = Self::sprite_proportion(engine, sprite_animation);
        let width = if let SpriteData::Text(data) = &sprite_animation.data {
            engine.measure_text(data, sprite_state.width)
        } else {
            sprite_state.width
        };
        let scaled_y = sprite_state.y * canvas_rect.width;
        let y = if sprite_state.from_bottom {
            canvas_rect.y + canvas_rect.height - scaled_y
        } else {
            canvas_rect.y + scaled_y
        };
        F32Rect {
            x: sprite_state.x * canvas_rect.width + canvas_rect.x,
            y,
            width: width * canvas_rect.width,
            height: sprite_state.width * canvas_rect.width / proportion,
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
            engine.get_sprites_animations(None)
        } else {
            Vec::new()
        }
    }

    /*
    Overlay
     */
    pub fn drag_offset(&mut self, start_offset: f32, dy: f32, move_offset: bool) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().cloned()
            && let Some(mut animation) = engine.get_sprites_animations(Some(state.timeline_points.sprite_index)).into_iter().next() {
            let new_offset = start_offset - dy / self.canvas_rect.get().height;
            if move_offset {
                let offset_delta = new_offset - state.sprite_state.offset;
                for state in &mut animation.states {
                    state.offset += offset_delta;
                }
                let rect = state.rect;
                let mut sprite_state = state.sprite_state;
                sprite_state.offset += offset_delta;
                self.state.set(Some(self.update_editor_state(state, sprite_state, animation, engine, rect)));
            } else {
                let states = &animation.states;
                let points = state.timeline_points.states_indices.len();
                let offset_diapason = states[state.timeline_points.states_indices[0]].offset..=states[state.timeline_points.states_indices[points - 1]].offset;
                let bound_offset = new_offset.max(*offset_diapason.start()).min(*offset_diapason.end());
                if let Some(new_sprite_state) = engine.interpolate_state(&animation, bound_offset) {
                    let canvas_rect = self.canvas_rect.get();
                    let rect = Self::scale_rect(engine, &animation, new_sprite_state, canvas_rect);
                    self.state.set(Some(state.change_sprite_rect(rect, new_sprite_state)));
                }
            }
            engine.scroll_to_offset(new_offset);
        }
    }

    pub fn set_sprite_rect(&mut self, new_rect: F32Rect) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().cloned()
            && let Some(mut animation) = engine.get_sprites_animations(Some(state.timeline_points.sprite_index)).into_iter().next()
            && let Some(sprite_state) = engine.interpolate_state(&animation, state.sprite_state.offset) {
            let canvas_rect = self.canvas_rect.get();
            let new_y = if sprite_state.from_bottom {
                canvas_rect.y + canvas_rect.height - new_rect.y
            } else {
                new_rect.y - canvas_rect.y
            };
            let mut new_sprite_state = SpriteState {
                offset: sprite_state.offset,
                x: (new_rect.x - canvas_rect.x) / canvas_rect.width,
                y: new_y / canvas_rect.width,
                from_bottom: sprite_state.from_bottom,
                width: new_rect.width / canvas_rect.width,
                color: sprite_state.color,
                easing: sprite_state.easing,
            };
            if let SpriteData::Text(data) = &animation.data {
                let measured = engine.measure_text(data, 1.0);
                new_sprite_state.width /= measured;
            }
            Self::apply_states_change(&state.timeline_points, &mut animation.states,
                                      new_sprite_state, state.parallax_factor);
            self.state.set(Some(self.update_editor_state(state, new_sprite_state, animation, engine, new_rect)));
        }
    }

    fn update_editor_state(
        &self,
        cur_state: SpriteEditorState,
        new_sprite_state: SpriteState,
        mut animation: SpriteAnimation,
        engine: &BalchugEngine,
        new_rect: F32Rect,
    ) -> SpriteEditorState {
        if cur_state.timeline_points.states_indices.len() == 2 && animation.states.len() == 2 {
            let canvas_rect = self.canvas_rect.get();
            let first_index = cur_state.timeline_points.states_indices[0];
            let last_index = cur_state.timeline_points.states_indices[cur_state.timeline_points.states_indices.len() - 1];
            let (first_state, last_state) = Self::scroll_adjust(
                new_sprite_state,
                cur_state.parallax_factor,
                canvas_rect.width / canvas_rect.height,
                Self::sprite_proportion(engine, &animation),
                animation.states[first_index].from_bottom,
            );
            animation.states[first_index] = first_state;
            animation.states[last_index] = last_state;
        }

        engine.set_sprite_animation_states(cur_state.timeline_points.sprite_index, animation.states);
        self.scenario_update.set(true);
        cur_state.change_sprite_rect(new_rect, new_sprite_state)
    }

    fn scroll_adjust(cur_state: SpriteState, parallax_factor: f32, aspect_ratio: f32,
                     item_proportion: f32, first_is_from_bottom: bool) -> (SpriteState, SpriteState) {
        let cur_y = if cur_state.from_bottom {1.0 / aspect_ratio - cur_state.y} else {cur_state.y};
        let end_y = -cur_state.width / item_proportion;
        let end_offset = cur_state.offset + (cur_y - end_y) * parallax_factor;
        let mut start_y = 1.0 / aspect_ratio;
        let mut start_offset = cur_state.offset - (start_y - cur_y) * parallax_factor;
        if start_offset < 0.0 {
            let f = cur_state.offset / (cur_state.offset - start_offset);
            start_y = cur_y + f * (start_y - cur_y) / parallax_factor;
            start_offset = 0.0;
        }
        let first_state = SpriteState {
            offset: start_offset,
            x: cur_state.x,
            y: if first_is_from_bottom {1.0 / aspect_ratio - start_y} else {start_y},
            from_bottom: first_is_from_bottom,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        let last_state = SpriteState {
            offset: end_offset,
            x: cur_state.x,
            y: end_y,
            from_bottom: false,
            width: cur_state.width,
            color: cur_state.color,
            easing: cur_state.easing,
        };
        (first_state, last_state)
    }

    fn apply_states_change(
        points: &TimeLinePoints,
        states: &mut [SpriteState],
        new_state: SpriteState,
        parallax_factor: f32,
    ) {
        for &index in &points.states_indices {
            let state = states[index];
            let dy = (new_state.offset - state.offset) / parallax_factor;
            let modified_state = SpriteState {
                offset: state.offset,
                x: new_state.x,
                y: if state.from_bottom {new_state.y - dy} else {new_state.y + dy},
                from_bottom: state.from_bottom,
                width: new_state.width,
                color: new_state.color,
                easing: new_state.easing,
            };
            states[index] = modified_state;
        }
    }

    /*
    State Editor
     */
    pub fn update_sprite_state(&mut self, new_state: SpriteState) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref()
            && let Some(mut sprite) = engine.get_sprites_animations(Some(state.timeline_points.sprite_index)).first().cloned() {
            if state.timeline_points.states_indices.len() == 1 {
                let index = state.timeline_points.states_indices[0];
                sprite.states[index] = new_state;
            } else {
                Self::apply_states_change(&state.timeline_points, &mut sprite.states,
                                          new_state, state.parallax_factor);
            }
            engine.set_sprite_animation_states(state.timeline_points.sprite_index, sprite.states);
            self.scenario_update.set(true);
        }
    }

    fn find_cur_state_index(states: &[SpriteState], points: &TimeLinePoints, state: &SpriteState) -> Option<usize> {
        if points.states_indices.is_empty() {
            None
        } else if points.states_indices.len() == 1 {
            Some(points.states_indices[0])
        } else {
            for &i in &points.states_indices {
                if (states[i].offset - state.offset).abs() < 0.01 {
                    return Some(i);
                }
            }
            None
        }
    }

    pub fn is_modify_states_possible(&self, adding: bool, removing: bool) -> bool {
        if let Some(state) = self.state_memo.read().as_ref()
            && let Some(engine) = self.engine.borrow().as_ref()
            && let Some(animation) = engine.get_sprites_animations(Some(state.timeline_points.sprite_index)).first() {
            let cur_state_index = Self::find_cur_state_index(&animation.states, &state.timeline_points, &state.sprite_state);
            if adding && cur_state_index.is_none() {
                return true;
            }
            if removing && cur_state_index.is_some() && animation.states.len() > 2 {
                return true;
            }
        }
        false
    }

    pub fn add_new_sprite_state(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref() {
            let mut animations = engine.get_sprites_animations(None);
            if let Some(animation) = animations.get_mut(state.timeline_points.sprite_index) {
                animation.states.push(state.sprite_state);
                animation.states.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(Ordering::Equal));
            }
            engine.set_scenario(animations);
            self.state.set(None);
        }
    }

    pub fn remove_sprite_state(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref() {
            let mut animations = engine.get_sprites_animations(None);
            if let Some(animation) = animations.get_mut(state.timeline_points.sprite_index)
                && let Some(i) = Self::find_cur_state_index(&animation.states, &state.timeline_points, &state.sprite_state) {
                animation.states.remove(i);
                engine.set_scenario(animations);
                self.state.set(None);
            }
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
