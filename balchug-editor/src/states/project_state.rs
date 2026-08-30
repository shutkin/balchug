use balchug_common::sprite::{SpriteData, SpriteImageData, SpriteState, SpriteTextData};
use dioxus::prelude::*;
use balchug_common::api::ProjectProperties;

#[derive(Clone, PartialEq)]
pub struct SpriteGroup {
    pub title: String,
    pub data: SpriteData,
    pub parallax_factor: f32,
    pub smooth_factor: f32,
    pub max_width: f32,
    pub states: Vec<SpriteState>,
    pub is_fixed: bool,
}

impl SpriteGroup {
    pub fn new_text() -> Self {
        Self {
            title: String::new(),
            data: SpriteData::Text(SpriteTextData {text: String::new(), size: 15}),
            parallax_factor: 1.0,
            smooth_factor: 0.5,
            max_width: 1.0,
            states: Vec::new(),
            is_fixed: true,
        }
    }

    pub fn new_image(image_id: usize) -> Self {
        Self {
            title: String::new(),
            data: SpriteData::Image(SpriteImageData {atlas_item_id: image_id}),
            parallax_factor: 1.0,
            smooth_factor: 0.5,
            max_width: 1.0,
            states: Vec::new(),
            is_fixed: true,
        }
    }
}

#[derive(Clone)]
pub struct ProjectState {
    pub properties: Store<ProjectProperties>,
    pub aspect_ratio: Store<f32>,
    pub groups: Store<Vec<SpriteGroup>>,
    pub cur_tab: Signal<usize>,
    pub selected_group: Signal<Option<usize>>,
}

impl ProjectState {
    pub fn new() -> Self {
        Self {
            properties: Store::new(Default::default()),
            aspect_ratio: Store::new(9.0 / 16.0),
            groups: Store::new(Vec::new()),
            cur_tab: Signal::new(1),
            selected_group: Signal::new(None),
        }
    }
    
    pub fn get_groups(&self) -> Vec<SpriteGroup> {
        self.groups.read().clone()
    }
    
    pub fn get_group(&self, group_id: usize) -> SpriteGroup {
        self.groups.read()[group_id].clone()
    }

    pub fn add_group(&mut self, group: SpriteGroup) {
        self.groups.write().push(group);
    }

    pub fn update_group(&mut self, group_id: usize, group: &SpriteGroup) {
        (*self.groups.write())[group_id] = group.clone();
    }
    
    pub fn select_group(&mut self, group_id: usize) {
        self.selected_group.set(Some(group_id));
        self.cur_tab.set(2);
    }
    
    pub fn unselect_group(&mut self) {
        self.selected_group.set(None);
    }
}