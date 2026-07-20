use dioxus::html::geometry::ElementPoint;
use dioxus::prelude::*;
use balchug_common::F32Rect;

const GAP: f32 = 5.0;

enum RectArea {
    Outside,
    Inside,
    Left,
    Right,
    Top,
    Bottom,
}

#[component]
pub fn PreviewOverlay(rect: Signal<Option<F32Rect>>) -> Element {
    let mut cursor_type = use_signal(move || "default");

    rsx! {
        if let Some(rect) = *rect.read() {
            svg {
                id: "preview_overlay",
                style: "position: absolute; left: 0; top: 4px; width: 100%; height: 100%; cursor: {cursor_type};",

                onmousemove: move |event: Event<MouseData>| {
                    let coordinates = event.element_coordinates();
                    let cursor = match rect_area(rect, coordinates) {
                        RectArea::Inside => "move",
                        RectArea::Left | RectArea::Right => "ew-resize",
                        RectArea::Top | RectArea::Bottom => "ns-resize",
                        _ => "default",
                    };
                    cursor_type.set(cursor);
                },

                path {
                    fill: "none",
                    stroke: "var(--accent-blue)",
                    stroke_width: "5",
                    d: build_rect_d(rect),
                }
            }
        }
    }
}

fn rect_area(rect: F32Rect, point: ElementPoint) -> RectArea {
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