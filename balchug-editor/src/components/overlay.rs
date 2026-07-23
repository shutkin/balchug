use dioxus::html::geometry::ElementPoint;
use dioxus::prelude::*;
use balchug_common::F32Rect;
use crate::controllers::sprite_editor::SpriteEditController;

const GAP: f32 = 5.0;

#[derive(Clone, Copy)]
enum RectArea {
    Outside,
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[component]
pub fn PreviewOverlay(controller: SpriteEditController) -> Element {
    if !controller.is_edit_mode() {
        return rsx! {};
    }
    
    let mut c0 = controller.clone();
    let c1 = controller.clone();
    let c2 = controller.clone();

    let mut cursor_type = use_signal(move || "default");
    let mut drag_rect_area = use_signal(move || RectArea::Outside);
    let mut start_drag_rect = use_signal(F32Rect::default);
    let mut start_drag_coordinates: Signal<Option<ElementPoint>> = use_signal(move || None);

    rsx! {
        div {
            id: "preview_overlay_container",
            style: "position: absolute; left: 0; top: 4px; width: 100%; height: 100%; cursor: {cursor_type};",
            onmousemove: move |event: Event<MouseData>| {
                event.prevent_default();
                if let Some(state) = c0.get_cur_state() {
                    if let Some(start_coordinates) = *start_drag_coordinates.read() {
                        let coordinates = event.element_coordinates();
                        let dx = (coordinates.x - start_coordinates.x) as f32;
                        let dy = (coordinates.y - start_coordinates.y) as f32;
                        let start_rect = *start_drag_rect.read();
                        let new_rect = modify_rect(start_rect, dx, dy, *drag_rect_area.read());
                        c0.set_sprite_rect(new_rect);
                    } else {
                        let coordinates = event.element_coordinates();
                        let cursor = match check_rect_area(state.rect, coordinates) {
                            RectArea::Inside => "move",
                            RectArea::Left | RectArea::Right => "ew-resize",
                            RectArea::Top | RectArea::Bottom => "ns-resize",
                            _ => "default",
                        };
                        cursor_type.set(cursor);
                    }
                }
            },
            onmousedown: move |event: Event<MouseData>| {
                if let Some(state) = c1.get_cur_state() {
                    let coordinates = event.element_coordinates();
                    let area = check_rect_area(state.rect, coordinates);
                    drag_rect_area.set(area);
                    if !matches!(area, RectArea::Outside) {
                        start_drag_rect.set(state.rect);
                        start_drag_coordinates.set(Some(coordinates));
                    }
                }
            },
            onmouseup: move |_: Event<MouseData>| {
                drag_rect_area.set(RectArea::Outside);
                start_drag_coordinates.set(None);
                cursor_type.set("default");
            },
            svg {
                id: "preview_overlay",
                style: "width: 100%; height: 100%;",
                path {
                    fill: "none",
                    stroke: "var(--accent-purple)",
                    stroke_width: "5",
                    d: build_rect_d(c2.get_cur_state().map(|s| s.rect).unwrap_or_default()),
                }
            }
        }
    }
}

fn modify_rect(start_rect: F32Rect, dx: f32, dy: f32, area: RectArea) -> F32Rect {
    match area {
        RectArea::Outside => start_rect,
        RectArea::Inside => F32Rect {
            x: start_rect.x + dx,
            y: start_rect.y + dy,
            width: start_rect.width,
            height: start_rect.height,
        },
        RectArea::Left => F32Rect {
            x: start_rect.x + dx,
            y: start_rect.y,
            width: start_rect.width - dx,
            height: start_rect.height - dx * start_rect.height / start_rect.width,
        },
        RectArea::Right => F32Rect {
            x: start_rect.x,
            y: start_rect.y,
            width: start_rect.width + dx,
            height: start_rect.height + dx * start_rect.height / start_rect.width,
        },
        RectArea::Top => F32Rect {
            x: start_rect.x,
            y: start_rect.y + dy,
            width: start_rect.width - dy * start_rect.width / start_rect.height,
            height: start_rect.height - dy,
        },
        RectArea::Bottom => F32Rect {
            x: start_rect.x,
            y: start_rect.y,
            width: start_rect.width + dy * start_rect.width / start_rect.height,
            height: start_rect.height + dy,
        },
    }
}

fn check_rect_area(rect: F32Rect, point: ElementPoint) -> RectArea {
    let x = point.x as f32;
    let y = point.y as f32;
    if x < rect.x - GAP || x > rect.x + rect.width + GAP || y < rect.y - GAP || y > rect.y + rect.height + GAP {
        RectArea::Outside
    } else if x < rect.x + GAP {
        RectArea::Left
    } else if x > rect.x + rect.width - GAP {
        RectArea::Right
    } else if y < rect.y + GAP {
        RectArea::Top
    } else if y > rect.y + rect.height - GAP {
        RectArea::Bottom
    } else {
        RectArea::Inside
    }
}

fn build_rect_d(rect: F32Rect) -> String {
    format!("M{},{} h{} v{} h{} v{} z", rect.x as i32, rect.y as i32,
            rect.width as i32, rect.height as i32, -rect.width as i32, -rect.height as i32)
}