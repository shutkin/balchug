use std::ops::{AddAssign, MulAssign};
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use dioxus::web::WebEventExt;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{SpriteState};

#[derive(Copy, Clone, PartialEq)]
struct TimeLineView {
    offset: f32,
    scale: f32,
    width: f32,
    height: f32,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TimeLinePoint {
    pub object_index: usize,
    pub state_index: usize,
    svg_x: f32,
    svg_y: f32,
}

#[component]
pub fn TimeLine(scenario: Signal<Scenario>, preview_offset: Signal<f32>, cur_point: Signal<Option<TimeLinePoint>>) -> Element {
    let mut offset = use_signal(|| -1_f32);
    let mut scale = use_signal(|| 50_f32);
    let mut svg_size = use_signal(|| (0_f32, 0_f32));
    let mut cursor_type = use_signal(|| "default".to_string());
    let points: Store<Vec<TimeLinePoint>> = use_store(Vec::new);

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
            style: "cursor: {cursor_type};",
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
            onmousemove: move |event| {
                if find_point(event.as_ref(), &points.read()).is_some() {
                    cursor_type.set("pointer".to_string());
                } else {
                    cursor_type.set("default".to_string());
                }
            },
            onmousedown: move |event| {
                cur_point.set(find_point(event.as_ref(), &points.read()));
            },
            svg {
                id: "timeline_svg",
                style: "height:100%;width:100%;",
                for (index, a) in scenario.read().images.iter().enumerate() {
                    AnimationPath {
                        states: a.animation.states.clone(),
                        index,
                        view: build_view(),
                        points,
                    }
                }
                CurPointMark {
                    cur_point,
                }
                CurOffsetPath {
                    cur_offset: *preview_offset.read(),
                    view: build_view(),
                }
            }
        }
    }
}

fn find_point(data: &MouseData, points: &[TimeLinePoint]) -> Option<TimeLinePoint> {
    let mouse_point = data.element_coordinates();
    let mx = mouse_point.x as f32;
    let my = mouse_point.y as f32;
    points.iter()
        .find(|p| (mx - p.svg_x).abs() <= 5.0 && (my - p.svg_y).abs() <= 5.0 )
        .copied()
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

#[component]
fn AnimationPath(states: Vec<SpriteState>, index: usize, view: TimeLineView, points: Store<Vec<TimeLinePoint>>) -> Element {
    rsx! {
        path {
            fill: "none",
            stroke: "var(--text-main)",
            d: build_path_d(&states, index, view, points),
        }
    }
}

#[component]
fn CurPointMark(cur_point: Signal<Option<TimeLinePoint>>) -> Element {
    if let Some(point) = cur_point.read().as_ref() {
        rsx! {
            path {
                fill: "none",
                stroke: "var(--accent-purple)",
                d: build_mark_d(point),
            }
        }
    } else {
        rsx! {}
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

fn build_mark_d(cur_point: &TimeLinePoint) -> String {
    format!("M{},{} l5,-5 l5,5 l-5,5 l-5,-5 z", cur_point.svg_x as i32 - 5, cur_point.svg_y as i32)
}

fn build_offset_d(offset: f32, view: TimeLineView) -> String {
    let y = (offset - view.offset) * view.scale;
    format!("M0,{} h{}", y as i32, view.width as i32)
}

fn build_path_d(states: &[SpriteState], index: usize, view: TimeLineView, mut points_store: Store<Vec<TimeLinePoint>>) -> String {
    let x = index * 20 + 10;
    let points = states.iter().enumerate()
        .map(|(i, state)| (i, (state.offset - view.offset) * view.scale))
        .filter(|(_, y)| *y > 0.0 && *y < view.height)
        .map(|(i, y)| TimeLinePoint {
            object_index: index,
            state_index: i,
            svg_x: x as f32,
            svg_y: y,
        })
        .collect::<Vec<_>>();
    let mut points_write = points_store.write();
    points_write.retain(|point| point.object_index != index);
    points_write.extend_from_slice(&points);
    if points.is_empty() {
        return if states.is_empty() {
            "".to_string()
        } else {
            let max_offset = states.iter().map(|s| s.offset).reduce(f32::max).unwrap_or_default();
            let min_offset = states.iter().map(|s| s.offset).reduce(f32::min).unwrap_or_default();
            if view.offset > max_offset || view.offset < min_offset {
                "".to_string()
            } else {
                format!("M{},{} v{}", x, 0, view.height as i32)
            }
        }
    }

    let (mut up, mut down) = (view.height, 0_f32);
    let (mut up_is_closed, mut down_is_closed) = (false, false);
    let mut marks = Vec::new();
    for point in points {
        if point.svg_y < up {
            up = point.svg_y;
            up_is_closed = point.state_index == 0;
        }
        if point.svg_y > down {
            down = point.svg_y;
            down_is_closed = point.state_index == states.len() - 1;
        }
        marks.push(format!("M{},{} h10", index * 20 + 5, point.svg_y as i32));
    }
    let marks = marks.join(" ");
    up = if up_is_closed {up} else {0.0};
    down = if down_is_closed {down} else {view.height};
    format!("M{},{} v{} {marks} z", index * 20 + 10, up, down - up)
}
