use crate::components::timeline::{TimeLinePoint, TimeLinePoints};
use crate::controllers::group_utils::GroupUtils;
use crate::states::project_state::{ProjectState, SpriteGroup};
use crate::states::sprite_editor::SpriteEditorState;
use balchug_common::F32Rect;
use balchug_common::sprite::{Easing, SpriteData, SpriteState};
use balchug_engine::{BalchugEngine, OffsetListener, STATE_OFFSET_LAG, start_engine};
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::rc::Rc;
use gloo_timers::future::TimeoutFuture;
use web_sys::{HtmlCanvasElement, Window};

const EASING_LINEAR: &str = "Linear";
const EASING_IN_CUBIC: &str = "In Cubic";
const EASING_OUT_CUBIC: &str = "Out Cubic";
const EASING_IN_OUT_CUBIC: &str = "In-Out Cubic";
const EASING_IN_SINE: &str = "In Sine";
const EASING_OUT_SINE: &str = "Out Sine";
const EASING_IN_OUT_SINE: &str = "In-Out Sine";

pub const ALL_EASING_VARIANTS: [Easing; 7] = [
    Easing::Linear,
    Easing::InCubic, Easing::OutCubic, Easing::InOutCubic,
    Easing::InSine, Easing::OutSine, Easing::InOutSine,
];

pub fn map_str_to_easing(str: &str) -> Easing {
    match str {
        EASING_IN_CUBIC => Easing::InCubic,
        EASING_OUT_CUBIC => Easing::OutCubic,
        EASING_IN_OUT_CUBIC => Easing::InOutCubic,
        EASING_IN_SINE => Easing::InSine,
        EASING_OUT_SINE => Easing::OutSine,
        EASING_IN_OUT_SINE => Easing::InOutSine,
        _ => Easing::Linear,
    }
}

pub fn map_easing_to_str(easing: Easing) -> &'static str {
    match easing {
        Easing::InCubic => EASING_IN_CUBIC,
        Easing::OutCubic => EASING_OUT_CUBIC,
        Easing::InOutCubic => EASING_IN_OUT_CUBIC,
        Easing::InSine => EASING_IN_SINE,
        Easing::OutSine => EASING_OUT_SINE,
        Easing::InOutSine => EASING_IN_OUT_SINE,
        _ => EASING_LINEAR,
    }
}

#[derive(Clone)]
pub struct SpriteEditController {
    engine: Rc<RefCell<Option<BalchugEngine>>>,
    state: Signal<Option<SpriteEditorState>>,
    state_memo: Memo<Option<SpriteEditorState>>,
    preview_offset_listener: PreviewOffsetListener,
    canvas_rect: Rc<Cell<F32Rect>>,
    project_state: ProjectState,
    groups_update: Rc<Cell<bool>>,
    input_value: Store<(String, String)>,
    input_value_future: Store<Option<UseFuture>>,
    font_is_ready: Signal<bool>,
}

impl PartialEq for SpriteEditController {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl SpriteEditController {
    pub fn new(
        engine: Rc<RefCell<Option<BalchugEngine>>>,
        project_state: ProjectState,
        groups_update: Rc<Cell<bool>>,
    ) -> Self {
        let state = Signal::new(Option::<SpriteEditorState>::None);
        let state_memo = use_memo(move || state.read().cloned());
        let ps0 = project_state.clone();
        let mut ps1 = project_state.clone();
        let selected_group = use_memo(move || *ps0.selected_group.read());

        let controller = Self {
            state,
            state_memo,
            engine,
            project_state,
            preview_offset_listener: PreviewOffsetListener::default(),
            canvas_rect: Rc::new(Cell::new(F32Rect::default())),
            groups_update,
            input_value: Store::new((String::default(), String::default())),
            input_value_future: Store::new(None),
            font_is_ready: Signal::new(false),
        };
        let mut c0 = controller.clone();
        use_effect(move || {
            if let Some(group_id) = *selected_group.read() {
                c0.set_timeline_group(group_id);
                ps1.unselect_group();
            }
        });

        controller
    }

    pub fn start(&self, window: Window, canvas: HtmlCanvasElement) {
        let mut ready_signal = self.font_is_ready;
        let listener = move || {
            info!("Font data is ready");
            ready_signal.set(true);
        };
        let balchug_engine = start_engine(window, canvas, Default::default(), Some(Box::new(listener)));
        balchug_engine.set_offset_listener(Box::new(self.preview_offset_listener.clone()));
        self.engine.replace(Some(balchug_engine));
    }

    pub fn get_ready_signal(&self) -> Signal<bool> {
        self.font_is_ready
    }

    pub fn resize(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let canvas_rect = engine.resize();
            self.canvas_rect.replace(canvas_rect);
            self.project_state.aspect_ratio.set(canvas_rect.width / canvas_rect.height);
        }
    }

    pub fn is_edit_mode(&self) -> bool {
        self.state_memo.read().is_some()
    }

    pub fn get_cur_state(&self) -> Option<SpriteEditorState> {
        self.state_memo.read().cloned()
    }

    /*
    Timeline
     */
    pub fn get_groups_titles(&self) -> Vec<String> {
        (*self.project_state.groups.read()).iter().map(|group| group.title.clone()).collect()
    }
    
    pub fn get_groups_states(&self) -> Vec<Vec<SpriteState>> {
        (*self.project_state.groups.read()).iter().map(|group| group.states.clone()).collect()
    }

    pub fn set_timeline_group(&mut self, group_id: usize) {
        if let Some(engine) = self.engine.borrow().as_ref() {
            let group = self.project_state.get_group(group_id);
            let indices = (0..group.states.len()).collect::<Vec<_>>();
            let cur_offset = engine.get_offset();
            let fixed_offset = if cur_offset >= group.states[0].offset
                && cur_offset <= group.states[group.states.len() - 1].offset {
                Some(cur_offset)
            } else {
                None
            };
            if let Some((offset, state)) = self.interpolate_selected_states(
                engine, group_id, &group, indices, fixed_offset) {
                engine.scroll_to_offset(offset);
                self.state.set(Some(state));
            }
        }
    }
    
    pub fn set_timeline_point(&mut self, timeline_point: Option<TimeLinePoint>) {
        if let Some(point) = timeline_point {
            let group = self.project_state.get_group(point.sprite_group_index);
            if let Some(engine) = self.engine.borrow().as_ref() {
                if let Some((offset, new_state)) = self.handle_new_point(engine, &group, point) {
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
        group: &SpriteGroup,
        point: TimeLinePoint,
    ) -> Option<(f32, SpriteEditorState)> {
        let states = &group.states;
        if let Some(state) = self.state_memo.read().as_ref() {
            if state.timeline_points.sprite_group_index != point.sprite_group_index {
                let state = self.new_editor_state(
                    engine, group, states[point.state_index],
                    point.sprite_group_index, vec![point.state_index]);
                return Some((states[point.state_index].offset, state));
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
            self.interpolate_selected_states(engine, point.sprite_group_index, group, new_indices, None)
        } else {
            let state = self.new_editor_state(
                engine, group, states[point.state_index],
                point.sprite_group_index, vec![point.state_index]);
            Some((states[point.state_index].offset, state))
        }
    }
    
    fn interpolate_selected_states(
        &self,
        engine: &BalchugEngine,
        group_id: usize,
        group: &SpriteGroup,
        indices: Vec<usize>,
        fixed_offset: Option<f32>
    ) -> Option<(f32, SpriteEditorState)> {
        let offset = fixed_offset.unwrap_or_else(|| {
            let mut sum_offset = 0_f32;
            for &i in &indices {
                sum_offset += group.states[i].offset;
            };
            sum_offset / indices.len() as f32
        });
        if let Some(sprite_state) = engine.interpolate_state(&group.states, offset, group.smooth_factor) {
            let state = self.new_editor_state(
                engine, group, sprite_state, group_id, indices);
            Some((offset, state))
        } else {
            None
        }
    }

    fn new_editor_state(
        &self,
        engine: &BalchugEngine,
        group: &SpriteGroup,
        sprite_state: SpriteState,
        group_id: usize,
        states_indices: Vec<usize>
    ) -> SpriteEditorState {
        let rect = Self::scale_rect(engine, group, sprite_state, self.canvas_rect.get());
        SpriteEditorState {
            timeline_points: TimeLinePoints { sprite_group_index: group_id, states_indices},
            parallax_factor: group.parallax_factor,
            sprite_state,
            rect,
        }
    }

    fn scale_rect(
        engine: &BalchugEngine,
        group: &SpriteGroup,
        sprite_state: SpriteState,
        canvas_rect: F32Rect
    ) -> F32Rect {
        let (proportion_x, proportion_y) = GroupUtils::group_proportion(engine, group);
        let width = if let SpriteData::Text(_) = &group.data {
            proportion_x
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
            height: sprite_state.width * canvas_rect.width * proportion_y / proportion_x,
        }
    }

    pub fn get_selected_points(&self) -> Option<TimeLinePoints> {
        self.state.read().as_ref().map(|s| s.timeline_points.clone())
    }

    pub fn get_preview_offset(&self) -> f32 {
        *self.preview_offset_listener.signal.read()
    }

    /*
    Overlay
     */
    pub fn drag_offset(&mut self, start_offset: f32, dy: f32, move_offset: bool) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().cloned() {
            let mut group = self.project_state.get_group(state.timeline_points.sprite_group_index);
            let new_offset = start_offset - dy / self.canvas_rect.get().height;
            if move_offset {
                let offset_delta = new_offset - state.sprite_state.offset;
                for state in &mut group.states {
                    state.offset += offset_delta;
                }
                let rect = state.rect;
                let mut sprite_state = state.sprite_state;
                sprite_state.offset += offset_delta;
                let (new_state, groups) = self.update_editor_state(state, sprite_state, group, engine, rect);
                self.project_state.groups.replace(groups);
                self.state.set(Some(new_state));
            } else {
                let states = &group.states;
                let points = state.timeline_points.states_indices.len();
                let offset_diapason = states[state.timeline_points.states_indices[0]].offset..=states[state.timeline_points.states_indices[points - 1]].offset;
                let bound_offset = new_offset.max(*offset_diapason.start()).min(*offset_diapason.end());
                if let Some(new_sprite_state) = engine.interpolate_state(states, bound_offset, group.smooth_factor) {
                    let canvas_rect = self.canvas_rect.get();
                    let rect = Self::scale_rect(engine, &group, new_sprite_state, canvas_rect);
                    self.state.set(Some(state.change_sprite_rect(rect, new_sprite_state)));
                }
            }
            engine.scroll_to_offset(new_offset);
        }
    }

    pub fn set_sprite_rect(&mut self, new_rect: F32Rect) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().cloned() {
            let mut group = self.project_state.get_group(state.timeline_points.sprite_group_index);

            if let Some(sprite_state) = engine.interpolate_state(&group.states, state.sprite_state.offset, group.smooth_factor) {
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
                if let SpriteData::Text(_) = &group.data {
                    let (proportion_x, _) = GroupUtils::group_proportion(engine, &group);
                    new_sprite_state.width /= proportion_x;
                }
                Self::apply_states_change(&state.timeline_points, &mut group.states,
                                          new_sprite_state, state.parallax_factor);

                let (state, groups) = self.update_editor_state(state, new_sprite_state, group, engine, new_rect);
                self.project_state.groups.replace(groups);
                self.state.set(Some(state));
            }
        }
    }

    fn update_editor_state(
        &self,
        editor_state: SpriteEditorState,
        new_sprite_state: SpriteState,
        mut group: SpriteGroup,
        engine: &BalchugEngine,
        new_rect: F32Rect,
    ) -> (SpriteEditorState, Vec<SpriteGroup>) {
        if editor_state.timeline_points.states_indices.len() == 2 && group.states.len() == 2 {
            let canvas_rect = self.canvas_rect.get();
            let first_index = editor_state.timeline_points.states_indices[0];
            let last_index = editor_state.timeline_points.states_indices[editor_state.timeline_points.states_indices.len() - 1];
            let (proportion_x, proportion_y) = GroupUtils::group_proportion(engine, &group);
            let (first_state, last_state) = GroupUtils::create_init_and_final_states(
                &new_sprite_state,
                editor_state.parallax_factor,
                canvas_rect.width / canvas_rect.height,
                proportion_x / proportion_y,
                group.states[first_index].from_bottom,
            );
            group.states[first_index] = first_state;
            group.states[last_index] = last_state;
        }
        let mut groups = self.project_state.get_groups();
        groups[editor_state.timeline_points.sprite_group_index] = group;
        engine.set_scenario(GroupUtils::groups_to_sprites(&groups, engine));
        self.groups_update.set(true);
        (editor_state.change_sprite_rect(new_rect, new_sprite_state), groups)
    }

    fn apply_states_change(
        points: &TimeLinePoints,
        states: &mut [SpriteState],
        new_state: SpriteState,
        parallax_factor: f32,
    ) {
        for &index in &points.states_indices {
            let state = states[index];
            if (new_state.offset - state.offset).abs() < STATE_OFFSET_LAG || points.states_indices.len() < 2 {
                states[index] = new_state;
            } else {
                let dy = (new_state.offset - state.offset) / parallax_factor;
                let modified_state = SpriteState {
                    offset: state.offset,
                    x: new_state.x,
                    y: if state.from_bottom { new_state.y - dy } else { new_state.y + dy },
                    from_bottom: state.from_bottom,
                    width: new_state.width,
                    color: new_state.color,
                    easing: state.easing,
                };
                states[index] = modified_state;
            }
        }
    }

    /*
    State Editor
     */
    pub fn handle_input_change(&mut self, name: &str, value: &str) {
        if let Some(mut future) = self.input_value_future.as_mut() {
            future.cancel();
        }
        self.input_value.set((String::from(name), String::from(value)));
        let c = self.clone();
        let future = use_future(move || {
            let mut c = c.clone();
            async move {
                TimeoutFuture::new(1000).await;
                c.input_change();
                c.input_value_future.set(None);
            }
        });
        self.input_value_future.set(Some(future));
    }

    fn input_change(&mut self) {
        if let Some(cur_state) = self.state_memo.read().cloned() {
            let (name, value) = self.input_value.read().cloned();
            let mut new_sprite_state = cur_state.sprite_state;
            let num = value.parse::<f32>().unwrap_or_default();
            match name.as_str() {
                "offset" => new_sprite_state.offset = num,
                "x" => new_sprite_state.x = num,
                "y" => new_sprite_state.y = num,
                "scale" => new_sprite_state.width = num,
                "easing" => new_sprite_state.easing = map_str_to_easing(&value),
                "y_axis" => {
                    let from_bottom = value == "Bottom";
                    if new_sprite_state.from_bottom != from_bottom {
                        let canvas_rect = self.canvas_rect.get();
                        new_sprite_state.y = canvas_rect.height / canvas_rect.width - new_sprite_state.y;
                        new_sprite_state.from_bottom = from_bottom;
                    }
                },
                "alpha" => new_sprite_state.color[3] = value.parse::<u8>().unwrap_or_default(),
                _ => {}
            }
            if let Some(engine) = self.engine.borrow().as_ref() {
                let mut group = self.project_state.get_group(cur_state.timeline_points.sprite_group_index);
                Self::apply_states_change(&cur_state.timeline_points, &mut group.states, new_sprite_state, cur_state.parallax_factor);
                let new_rect = Self::scale_rect(engine, &group, new_sprite_state, self.canvas_rect.get());
                let (state, groups) = self.update_editor_state(cur_state, new_sprite_state, group, engine, new_rect);
                self.project_state.groups.replace(groups);
                self.state.set(Some(state));
            }
        }
    }

    fn find_cur_state_index(states: &[SpriteState], points: &TimeLinePoints, state: &SpriteState) -> Option<usize> {
        if points.states_indices.is_empty() {
            None
        } else if points.states_indices.len() == 1 {
            Some(points.states_indices[0])
        } else {
            for &i in &points.states_indices {
                if (states[i].offset - state.offset).abs() < STATE_OFFSET_LAG {
                    return Some(i);
                }
            }
            None
        }
    }

    pub fn is_modify_states_possible(&self, adding: bool, removing: bool) -> bool {
        if let Some(state) = self.state_memo.read().as_ref() {
            let group = self.project_state.get_group(state.timeline_points.sprite_group_index);
            let cur_state_index = Self::find_cur_state_index(&group.states, &state.timeline_points, &state.sprite_state);
            if adding && cur_state_index.is_none() {
                return true;
            }
            if removing && cur_state_index.is_some() && group.states.len() > 2 {
                return true;
            }
        }
        false
    }

    pub fn add_new_sprite_state(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref() {
            let mut groups = self.project_state.get_groups();
            groups[state.timeline_points.sprite_group_index].states.push(state.sprite_state);
            groups[state.timeline_points.sprite_group_index].states.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(Ordering::Equal));
            engine.set_scenario(GroupUtils::groups_to_sprites(&groups, engine));
            self.project_state.groups.replace(groups);
            self.groups_update.set(true);
            self.state.set(None);
        }
    }

    pub fn remove_sprite_state(&mut self) {
        if let Some(engine) = self.engine.borrow().as_ref()
            && let Some(state) = self.state_memo.read().as_ref() {
            let mut groups = self.project_state.get_groups();
            let group = &mut groups[state.timeline_points.sprite_group_index];
            if let Some(i) = Self::find_cur_state_index(&group.states, &state.timeline_points, &state.sprite_state) {
                group.states.remove(i);
            }
            engine.set_scenario(GroupUtils::groups_to_sprites(&groups, engine));
            self.project_state.groups.replace(groups);
            self.groups_update.set(true);
            self.state.set(None);
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
