use std::ops::{Add, AddAssign, MulAssign};
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{SpriteState};

#[component]
pub fn TimeLine(scenario: Signal<Scenario>, preview_offset: Signal<f32>) -> Element {
    let mut offset = use_signal(|| -1_f32);
    let mut scale = use_signal(|| 50_f32);
    let mut svg_size = use_signal(|| (0_f32, 0_f32));
    let build_view = move || {
        let size = svg_size.read();
        TimeLineView {
            offset: *offset.read(),
            scale: *scale.read(),
            width: size.0,
            height: size.1,
        }
    };
    rsx! {
        div {
            id: "timeline_body",
            class: "timeline-body",
            onwheel: move |event| {
                event.prevent_default();
                if event.as_web_event().shift_key() {
                    scale.mul_assign(wheel_zoom_factor(event.as_ref()));
                } else {
                    offset.add_assign(wheel_offset_factor(event.as_ref()) / *scale.read());
                }
            },
            onresize: move |event| {
                if let Ok(size) = event.data.get_content_box_size() {
                    svg_size.set((size.width as f32, size.height as f32));
                }
            },
            svg {
                id: "timeline_svg",
                style: "height:100%;width:100%;",
                for (index, a) in scenario.read().images.iter().enumerate() {
                    AnimationPath {
                        states: a.animation.states.clone(),
                        index,
                        view: build_view(),
                    }
                }
                CurOffsetPath {
                    cur_offset: *preview_offset.read(),
                    view: build_view(),
                }
            }
        }
    }
}

fn wheel_offset_factor(data: &WheelData) -> f32 {
    match data.delta() {
        WheelDelta::Pixels(pixels) => {
            0.1 * pixels.y as f32
        }
        WheelDelta::Lines(lines) => {
            1.0 * lines.y as f32
        }
        WheelDelta::Pages(pages) => {
            5.0 * pages.y as f32
        }
    }
}

fn wheel_zoom_factor(data: &WheelData) -> f32 {
    match data.delta() {
        WheelDelta::Pixels(pixels) => {
            1.0 + 0.001 * pixels.y as f32
        }
        WheelDelta::Lines(lines) => {
            1.0 + 0.0075 * lines.y as f32
        }
        WheelDelta::Pages(pages) => {
            1.0 + 0.05 * pages.y as f32
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
struct TimeLineView {
    offset: f32,
    scale: f32,
    width: f32,
    height: f32,
}

#[component]
fn AnimationPath(states: Vec<SpriteState>, index: usize, view: TimeLineView) -> Element {
    rsx! {
        path {
            fill: "none",
            stroke: "var(--text-main)",
            d: build_path_d(&states, index, view),
        }
    }
}

#[component]
fn CurOffsetPath(cur_offset: f32, view: TimeLineView) -> Element {
    rsx! {
        path {
            fill: "none",
            stroke: "var(--accent-blue)",
            d: build_offset_d(cur_offset, view),
        }
    }
}

fn build_offset_d(offset: f32, view: TimeLineView) -> String {
    let y = (offset - view.offset) * view.scale;
    format!("M0,{} h{}", y as i32, view.width as i32)
}

fn build_path_d(states: &[SpriteState], index: usize, view: TimeLineView) -> String {
    let points = states.iter().enumerate()
        .map(|(i, state)| (i, (state.offset - view.offset) * view.scale))
        .filter(|(_, y)| *y > 0.0 && *y < view.height)
        .collect::<Vec<_>>();
    if points.is_empty() {
        return if states.is_empty() {
            "".to_string()
        } else {
            format!("M{},{} v{}", index * 20 + 10, 0, view.height as i32)
        }
    }

    let (mut up, mut down) = (view.height, 0_f32);
    let (mut up_is_closed, mut down_is_closed) = (false, false);
    let mut marks = Vec::new();
    for (i, point) in points {
        if point < up {
            up = point;
            up_is_closed = i == 0;
        }
        if point > down {
            down = point;
            down_is_closed = i == states.len() - 1;
        }
        marks.push(format!("M{},{} h10", index * 20 + 5, point as i32));
    }
    let marks = marks.join(" ");
    up = if up_is_closed {up} else {0.0};
    down = if down_is_closed {down} else {view.height};
    format!("M{},{} v{} {marks} z", index * 20 + 10, up, down - up)
}
