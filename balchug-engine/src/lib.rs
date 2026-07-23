use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Window, HtmlCanvasElement, HtmlImageElement, Request, WebGl2RenderingContext, Response, AddEventListenerOptions, WheelEvent, TouchEvent, MouseEvent};
use balchug_common::atlas::{Atlas, AtlasItem, FontData};
use balchug_common::F32Rect;
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{Sprite, SpriteAnimation, SpriteState};
use crate::font::font_builder::build_font;
use crate::gl::GlRenderer;
use crate::inertia::Inertia;
use crate::scenario::{scenario_letters, scenario_max_offset, scenario_text_size};
use crate::sprite_util::{arrange_text_line, interpolate_state, scale_sprite_state};

pub mod gl;
mod inertia;
mod scenario;
mod font;
mod sprite_util;

pub trait OffsetListener {
    fn offset_change(&mut self, offset: f32);
}

#[derive(Clone)]
struct AppContext {
    force_rerender: Rc<Cell<bool>>,
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
    offset_listener: Rc<RefCell<Option<Box<dyn OffsetListener>>>>,
}

impl AppContext {
    fn new(canvas_width: f32) -> Self {
        AppContext {
            force_rerender: Rc::new(Cell::new(false)),
            scroll: Rc::new(RefCell::new(Inertia::new(0.0))),
            images_texture_ready: Rc::new(Cell::new(false)),
            txt_texture_ready: Rc::new(Cell::new(false)),
            atlas_items: Rc::new(RefCell::new(HashMap::default())),
            font_atlas_items: Rc::new(RefCell::new(HashMap::default())),
            scenario: Rc::new(RefCell::new(Scenario::default())),
            font: Rc::new(RefCell::new(FontData::default())),
            font_bytes: Rc::new(RefCell::new(Vec::new())),
            canvas_width: Rc::new(Cell::new(canvas_width)),
            last_frame: Rc::new(Cell::new(instant::Instant::now())),
            touch_start_screen: Rc::new(Cell::new(0.0)),
            touch_start_scroll: Rc::new(Cell::new(0.0)),
            offset_listener: Rc::new(RefCell::new(None)),
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
    pub fn resize(&self) -> F32Rect {
        if let Some(parent) = self.canvas.parent_element() {
            let rect = parent.get_bounding_client_rect();
            let (width, height) = (rect.width() as f32 * self.pixel_ratio,
                                   rect.height() as f32 * self.pixel_ratio);
            web_sys::console::log_1(&format!("Resizing canvas to {width}x{height}").into());
            let (width, height) = (width.round() as u32, height.round() as u32);
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.context.canvas_width.set(width as f32);
            self.renderer.set_sizes(width as f32, height as f32);
            self.update();
            let rect = self.canvas.get_bounding_client_rect();
            F32Rect {
                x: rect.x() as f32,
                y: rect.y() as f32,
                width: rect.width() as f32,
                height: rect.height() as f32,
            }
        } else {
            F32Rect::default()
        }
    }

    pub fn set_offset_listener(&self, listener: Box<dyn OffsetListener>) {
        self.context.offset_listener.borrow_mut().replace(listener);
    }

    pub fn set_atlas(&self, atlas: Atlas) {
        self.context.atlas_items.replace(atlas.items);
    }

    pub fn set_scenario(&self, scenario: Scenario) {
        web_sys::console::log_1(&"Update scenario".into());
        self.context.scenario.replace(scenario);
        self.update();
    }

    pub fn get_scenario_images(&self) -> Vec<SpriteAnimation> {
        self.context.scenario.borrow().images.clone()
    }
    
    pub fn get_atlas_item(&self, id: usize) -> Option<AtlasItem> {
        self.context.atlas_items.borrow().get(&id).copied()
    }

    fn update(&self) {
        let width = self.context.canvas_width.get();
        let max_scroll = scenario_max_offset(&self.context.scenario.borrow()) * width;
        self.context.scroll.borrow_mut().set_limit_up(max_scroll);
        rebuild_font(&self.context, &self.renderer);
        self.context.force_rerender.set(true);
    }

    pub fn scroll_to_offset(&self, offset: f32) {
        let offset = offset * self.context.canvas_width.get();
        self.context.scroll.borrow_mut().set_value(offset);
        if let Some(l) = self.context.offset_listener.borrow_mut().as_mut() {
            l.offset_change(offset);
        }
        self.context.force_rerender.set(true);
    }

    pub fn set_image_sprite_state(&self, new_state: SpriteState, object_index: usize, state_index: usize) {
        let scenario = &mut self.context.scenario.borrow_mut();
        scenario.images[object_index].animation.states[state_index] = new_state;
        self.context.force_rerender.set(true);
    }
}

fn rebuild_font(ctx: &AppContext, renderer: &GlRenderer) {
    let bytes = ctx.font_bytes.borrow();
    let letters = scenario_letters(&ctx.scenario.borrow());
    if !bytes.is_empty() && !letters.is_empty() {
        let font_size = scenario_text_size(&ctx.scenario.borrow(), ctx.canvas_width.get());
        web_sys::console::log_1(&format!("Font size: {font_size}").into());
        if let Some(res) = build_font(&letters, &bytes, font_size) {
            renderer.set_font_texture(res.atlas.width, res.atlas.height, &res.data);
            ctx.font.replace(res.font_data);
            ctx.font_atlas_items.replace(res.atlas.items);
            ctx.txt_texture_ready.replace(true);
            ctx.force_rerender.set(true);
        }
    }
}

fn animate_scene(ctx: &AppContext) -> (Vec<Sprite>, Vec<Sprite>) {
    let now = instant::Instant::now();
    let elapsed = now.duration_since(ctx.last_frame.get()).as_secs_f32();
    ctx.last_frame.set(now);

    let width = ctx.canvas_width.get();
    let (updated, offset) = ctx.scroll.borrow_mut().live(elapsed);
    let scaled_offset = offset / width;
    if !updated && !ctx.force_rerender.get() {
        return (Vec::new(), Vec::new());
    }
    ctx.force_rerender.set(false);
    if let Some(listener) = ctx.offset_listener.borrow_mut().as_mut() {
        listener.offset_change(scaled_offset)
    }
    let scenario = ctx.scenario.borrow();
    let (mut sprites, mut text_sprites) = (Vec::new(), Vec::new());

    for image_animation in &scenario.images {
        if let Some(cur_state) = interpolate_state(&image_animation.animation, scaled_offset) && cur_state.color[3] > 0.01 {
            sprites.push(Sprite {
                state: scale_sprite_state(&cur_state, width),
                atlas_item: *ctx.atlas_items.borrow().get(&image_animation.atlas_item_id).unwrap(),
            });
        }
    }
    for text_animation in &scenario.text_lines {
        if let Some(cur_state) = interpolate_state(&text_animation.animation, scaled_offset) && cur_state.color[3] > 0.01 {
            for glyph_sprite in arrange_text_line(text_animation, &cur_state, &ctx.font.borrow(), &ctx.font_atlas_items.borrow()) {
                let glyph_sprite = Sprite {
                    state: scale_sprite_state(&glyph_sprite.state, width),
                    atlas_item: glyph_sprite.atlas_item,
                };
                text_sprites.push(glyph_sprite);
            }
        }
    }

    if !sprites.is_empty() && !ctx.images_texture_ready.get() || !text_sprites.is_empty() && !ctx.txt_texture_ready.get() {
        (vec![], vec![])
    } else {
        (sprites, text_sprites)
    }
}

pub fn start_engine(window: Window, canvas: HtmlCanvasElement, assets_folder: &str) -> BalchugEngine {
    let pixel_ratio = window.device_pixel_ratio().max(2.0);

    let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap();
    let renderer = GlRenderer::init(gl).unwrap();
    let (width, height) = (canvas.width(), canvas.height());
    renderer.set_sizes(width as f32, height as f32);

    let ctx = AppContext::new(width as f32);

    let render = {
        let ctx = ctx.clone();
        let renderer = renderer.clone();
        move || {
            let (sprites, text_sprites) = animate_scene(&ctx);
            if !sprites.is_empty() || !text_sprites.is_empty() {
                renderer.render(&sprites, &text_sprites);
            }
        }
    };

    let req = Request::new_with_str(&format!("{assets_folder}/georgia.otf")).unwrap();
    let on_response = {
        let ctx = ctx.clone();
        let renderer = renderer.clone();
        let on_body = {
            Closure::wrap(Box::new(move |buf_value: JsValue| {
                let bytes = js_sys::Uint8Array::new(&buf_value).to_vec();
                web_sys::console::log_1(&format!("Load font {} bytes", bytes.len()).into());
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

    let atlas_img = HtmlImageElement::new().unwrap();
    atlas_img.set_src(&format!("{assets_folder}/atlas.webp"));
    let on_load = {
        let renderer = renderer.clone();
        let img = atlas_img.clone();
        let texture_ready = ctx.images_texture_ready.clone();
        let force_rerender = ctx.force_rerender.clone();
        Closure::wrap(Box::new(move || {
            renderer.set_texture(&img);
            texture_ready.replace(true);
            force_rerender.set(true);
        }) as Box<dyn FnMut()>)
    };
    atlas_img.set_onload(Some(on_load.as_ref().unchecked_ref()));
    on_load.forget();

    let options = AddEventListenerOptions::new();
    options.set_passive(false); // Explicitly allow preventDefault()
    let on_wheel = {
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: WheelEvent| {
            if !scroll.borrow().has_permanent_target() {
                let cur_scroll = scroll.borrow().get_value();
                scroll.borrow_mut().set_target(cur_scroll + (e.delta_y() * pixel_ratio) as f32, false);
            }
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

    let on_mouse_down = {
        let touch_start_screen = ctx.touch_start_screen.clone();
        let touch_start_scroll = ctx.touch_start_scroll.clone();
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: MouseEvent| {
            touch_start_screen.set(e.client_y() as f32);
            let cur_scroll_value = scroll.borrow().get_value();
            touch_start_scroll.set(cur_scroll_value);
            scroll.borrow_mut().set_target(cur_scroll_value, true);
            e.prevent_default();
        }) as Box<dyn FnMut(MouseEvent)>)
    };
    canvas.add_event_listener_with_callback_and_add_event_listener_options(
        "mousedown",
        on_mouse_down.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_mouse_down.forget();

    let on_touch_move = {
        let touch_start_screen = ctx.touch_start_screen.clone();
        let touch_start_scroll = ctx.touch_start_scroll.clone();
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: TouchEvent| {
            if let Some(touch) = e.touches().item(0) {
                let delta = (touch.page_y() as f32 - touch_start_screen.get()) * pixel_ratio as f32;
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

    let on_mouse_move = {
        let touch_start_screen = ctx.touch_start_screen.clone();
        let touch_start_scroll = ctx.touch_start_scroll.clone();
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: MouseEvent| {
            if scroll.borrow().has_permanent_target() {
                let delta = (e.client_y() as f32 - touch_start_screen.get()) * pixel_ratio as f32;
                scroll.borrow_mut().set_target(touch_start_scroll.get() - delta, true);
                e.prevent_default();
            }
        }) as Box<dyn FnMut(MouseEvent)>)
    };
    canvas.add_event_listener_with_callback_and_add_event_listener_options(
        "mousemove",
        on_mouse_move.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_mouse_move.forget();

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

    let on_mouse_up = {
        let scroll = ctx.scroll.clone();
        Closure::wrap(Box::new(move |e: MouseEvent| {
            scroll.borrow_mut().clear_target();
            e.prevent_default();
        }) as Box<dyn FnMut(MouseEvent)>)
    };
    window.add_event_listener_with_callback_and_add_event_listener_options(
        "mouseup",
        on_mouse_up.as_ref().unchecked_ref(),
        &options
    ).unwrap();
    on_mouse_up.forget();

    let on_frame: Rc<RefCell<Closure<dyn FnMut()>>> = Rc::new(RefCell::new(Closure::wrap(Box::new(move || {}))));
    let on_frame_clone = on_frame.clone();
    let render_clone = render.clone();
    let window_clone = window.clone();
    *on_frame.borrow_mut() = Closure::wrap(Box::new(move || {
        render_clone();
        if let Err(err) = window_clone.request_animation_frame(on_frame_clone.borrow().as_ref().unchecked_ref()) {
            web_sys::console::error_1(&format!("Request animation frame failed: {err:?}").into());
        }
    }));
    if let Err(err) = window.request_animation_frame(on_frame.borrow().as_ref().unchecked_ref()) {
        web_sys::console::error_1(&format!("Request first animation frame failed: {err:?}").into());
    }

    BalchugEngine {
        pixel_ratio: pixel_ratio as f32,
        context: Rc::new(ctx),
        renderer: Rc::new(renderer),
        canvas
    }
}
