use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{console, Window, HtmlCanvasElement, HtmlImageElement, Request, WebGl2RenderingContext, Response, AddEventListenerOptions, WheelEvent, TouchEvent};
use balchug_common::atlas::{Atlas, AtlasItem, FontData};
use balchug_common::sprite::Sprite;
use crate::font::font_builder::build_font;
use crate::gl::GlRenderer;
use crate::inertia::Inertia;
use crate::r#const::{create_atlas, create_font, create_font_atlas, get_letters};
use crate::scenario::{build_scenario, Scenario};

pub mod gl;
mod r#const;
mod inertia;
mod scenario;
mod text;
mod font;

#[derive(Clone)]
struct AppContext {
    scroll: Rc<RefCell<Inertia>>,
    images_texture_ready: Rc<Cell<bool>>,
    txt_texture_ready: Rc<Cell<bool>>,
    atlas_items: Rc<RefCell<HashMap<usize, AtlasItem>>>,
    font_atlas_items: Rc<RefCell<HashMap<usize, AtlasItem>>>,
    font: Rc<RefCell<FontData>>,
    font_bytes: Rc<RefCell<Vec<u8>>>,
    scenario: Rc<RefCell<Scenario>>,
    canvas_width: Rc<Cell<f32>>,
    last_frame: Rc<Cell<instant::Instant>>,
    touch_start_screen: Rc<Cell<f32>>,
    touch_start_scroll: Rc<Cell<f32>>,
}

impl AppContext {
    fn new(canvas_width: f32, scenario: Scenario, atlas: Atlas, font_atlas: Atlas, font: FontData) -> Self {
        AppContext {
            scroll: Rc::new(RefCell::new(Inertia::new(0.0))),
            images_texture_ready: Rc::new(Cell::new(false)),
            txt_texture_ready: Rc::new(Cell::new(false)),
            atlas_items: Rc::new(RefCell::new(atlas.items)),
            font_atlas_items: Rc::new(RefCell::new(font_atlas.items)),
            scenario: Rc::new(RefCell::new(scenario)),
            font: Rc::new(RefCell::new(font)),
            font_bytes: Rc::new(RefCell::new(Vec::new())),
            canvas_width: Rc::new(Cell::new(canvas_width)),
            last_frame: Rc::new(Cell::new(instant::Instant::now())),
            touch_start_screen: Rc::new(Cell::new(0.0)),
            touch_start_scroll: Rc::new(Cell::new(0.0)),
        }
    }
}

#[derive(Clone)]
pub struct BalchugEngine {
    context: Rc<AppContext>,
    renderer: Rc<GlRenderer>,
    canvas: HtmlCanvasElement,
    pixel_ratio: f32,
}

impl BalchugEngine {
    pub fn resize(&self) {
        if let Some(parent) = self.canvas.parent_element() {
            let rect = parent.get_bounding_client_rect();
            let (width, height) = (rect.width() as f32 * self.pixel_ratio,
                                   rect.height() as f32 * self.pixel_ratio);
            console::log_1(&format!("Resizing canvas to {width}x{height}").into());
            let (width, height) = (width.round() as u32, height.round() as u32);
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.context.canvas_width.set(width as f32);
            self.renderer.set_sizes(width as f32, height as f32);

            let max_scroll = self.context.scenario.borrow().max_offset() * width as f32;
            self.context.scroll.borrow_mut().set_limit_up(max_scroll);

            rebuild_font(&self.context, &self.renderer);
        }
    }
}

fn rebuild_font(ctx: &AppContext, renderer: &GlRenderer) {
    let bytes = ctx.font_bytes.borrow();
    if !bytes.is_empty() {
        let font_size = ctx.scenario.borrow().text_size(ctx.canvas_width.get());
        console::log_1(&format!("Font size: {font_size}").into());
        if let Some(res) = build_font(&get_letters(), &bytes, font_size) {
            renderer.set_font_texture(res.atlas.width, res.atlas.height, &res.data);
            ctx.font.replace(res.font_data);
            ctx.font_atlas_items.replace(res.atlas.items);
            ctx.txt_texture_ready.replace(true);
        }
    }
}

fn animate_scene(ctx: &AppContext) -> (Vec<Sprite>, Vec<Sprite>) {
    let now = instant::Instant::now();
    let elapsed = now.duration_since(ctx.last_frame.get()).as_secs_f32();
    ctx.last_frame.set(now);

    if !ctx.images_texture_ready.get() || !ctx.txt_texture_ready.get() {
        return (vec![], vec![]);
    }

    let width = ctx.canvas_width.get();
    let scaled_offset = ctx.scroll.borrow_mut().live(elapsed) / width;
    let scenario = ctx.scenario.borrow();
    let (mut sprites, mut text_sprites) = (Vec::new(), Vec::new());

    for image_animation in &scenario.images {
        if let Some(cur_state) = image_animation.animation.interpolate_state(scaled_offset) && cur_state.color[3] > 0.01 {
            sprites.push(Sprite {
                state: cur_state.scale(width),
                atlas_item: *ctx.atlas_items.borrow().get(&image_animation.atlas_item_id).unwrap(),
            });
        }
    }
    for text_animation in &scenario.text_lines {
        if let Some(cur_state) = text_animation.animation.interpolate_state(scaled_offset) && cur_state.color[3] > 0.01 {
            for glyph_sprite in text_animation.arrange(&cur_state, &ctx.font.borrow(), &ctx.font_atlas_items.borrow()) {
                let glyph_sprite = Sprite {
                    state: glyph_sprite.state.scale(width),
                    atlas_item: glyph_sprite.atlas_item,
                };
                text_sprites.push(glyph_sprite);
            }
        }
    }
    /*console::log_1(&format!(
        "text sprites: {}",
        text_sprites.iter().map(|s| format!("{}x{} -> {}x{}", s.state.width, s.state.height, s.atlas_item.origin_width, s.atlas_item.origin_height)).collect::<Vec<_>>().join(", ")
    ).into());*/
    (sprites, text_sprites)
}

pub fn start_engine(window: Window, canvas: HtmlCanvasElement, assets_folder: &str) -> BalchugEngine {
    let pixel_ratio = window.device_pixel_ratio();

    let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap();
    let renderer = GlRenderer::init(gl).unwrap();
    let (width, height) = (canvas.width(), canvas.height());
    //renderer.set_sizes(width as f32, height as f32);

    let atlas_img = HtmlImageElement::new().unwrap();
    atlas_img.set_src(&format!("{assets_folder}/atlas.webp"));

    let atlas = create_atlas();
    let font_atlas = create_font_atlas();
    let font = create_font();
    let scenario = build_scenario(&atlas.items, &vec![2, 1, 5, 7, 10, 6, 3, 8, 4, 9]);
    let ctx = AppContext::new(width as f32, scenario, atlas, font_atlas, font);

    let render = {
        let ctx = ctx.clone();
        let renderer = renderer.clone();
        move || {
            let (sprites, text_sprites) = animate_scene(&ctx);
            renderer.render(&sprites, &text_sprites);
        }
    };

    let req = Request::new_with_str(&format!("{assets_folder}/georgia.otf")).unwrap();
    let on_response = {
        let ctx = ctx.clone();
        let renderer = renderer.clone();
        let on_body = {
            Closure::wrap(Box::new(move |buf_value: JsValue| {
                let bytes = js_sys::Uint8Array::new(&buf_value).to_vec();
                console::log_1(&format!("Load font {} bytes", bytes.len()).into());
                ctx.font_bytes.replace(bytes);
                rebuild_font(&ctx, &renderer);
            }) as Box<dyn FnMut(JsValue)>)
        };
        Closure::wrap(Box::new(move |result: JsValue| {
            let response: Response = result.dyn_into().unwrap();
            if let Ok(promise) = response.array_buffer() {
                let _ = promise.then(&on_body);
            }
        }) as Box<dyn FnMut(JsValue)>)
    };
    let _ = window.fetch_with_request(&req).then(&on_response);
    on_response.forget();

    let on_load = {
        let renderer = renderer.clone();
        let img = atlas_img.clone();
        let texture_ready = ctx.images_texture_ready.clone();
        Closure::wrap(Box::new(move || {
            renderer.set_texture(&img);
            texture_ready.replace(true);
        }) as Box<dyn FnMut()>)
    };
    atlas_img.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();

    let options = AddEventListenerOptions::new();
    options.set_passive(false); // Explicitly allow preventDefault()
    let on_wheel = {
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: WheelEvent| {
            let cur_scroll = scroll.borrow().get_value();
            scroll.borrow_mut().set_target(cur_scroll + (e.delta_y() * pixel_ratio) as f32, false);
            e.prevent_default();
        }) as Box<dyn FnMut(WheelEvent)>)
    };
    canvas.add_event_listener_with_callback_and_add_event_listener_options(
        "wheel",
        on_wheel.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_wheel.forget();

    let on_touch_start = {
        let touch_start_screen = ctx.touch_start_screen.clone();
        let touch_start_scroll = ctx.touch_start_scroll.clone();
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: TouchEvent| {
            if let Some(touch) = e.touches().item(0) {
                touch_start_screen.set(touch.page_y() as f32);
            }
            touch_start_scroll.set(scroll.borrow().get_value());
            e.prevent_default();
        }) as Box<dyn FnMut(TouchEvent)>)
    };
    canvas.add_event_listener_with_callback_and_add_event_listener_options(
        "touchstart",
        on_touch_start.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_touch_start.forget();

    let on_touch_move = {
        let touch_start_screen = ctx.touch_start_screen.clone();
        let touch_start_scroll = ctx.touch_start_scroll.clone();
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: TouchEvent| {
            if let Some(touch) = e.touches().item(0) {
                let delta = touch.page_y() as f32 - touch_start_screen.get();
                scroll.borrow_mut().set_target(touch_start_scroll.get() - delta, true);
            }
            e.prevent_default();
        }) as Box<dyn FnMut(TouchEvent)>)
    };
    canvas.add_event_listener_with_callback_and_add_event_listener_options(
        "touchmove",
        on_touch_move.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_touch_move.forget();

    let on_touch_end = {
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: TouchEvent| {
            scroll.borrow_mut().clear_target();
            e.prevent_default();
        }) as Box<dyn FnMut(TouchEvent)>)
    };
    window.add_event_listener_with_callback_and_add_event_listener_options(
        "touchend",
        on_touch_end.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_touch_end.forget();

    let on_frame: Rc<RefCell<Closure<dyn FnMut()>>> = Rc::new(RefCell::new(Closure::wrap(Box::new(move || {}))));
    let on_frame_clone = on_frame.clone();
    let render_clone = render.clone();
    let window_clone = window.clone();
    *on_frame.borrow_mut() = Closure::wrap(Box::new(move || {
        render_clone();
        if let Err(err) = window_clone.request_animation_frame(on_frame_clone.borrow().as_ref().unchecked_ref()) {
            console::error_1(&format!("Request animation frame failed: {err:?}").into());
        }
    }));
    if let Err(err) = window.request_animation_frame(on_frame.borrow().as_ref().unchecked_ref()) {
        console::error_1(&format!("Request first animation frame failed: {err:?}").into());
    }

    BalchugEngine {
        pixel_ratio: pixel_ratio as f32,
        context: Rc::new(ctx),
        renderer: Rc::new(renderer),
        canvas
    }
}
