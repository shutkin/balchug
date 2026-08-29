use crate::font::font_builder::build_font;
use crate::fps::FpsCounter;
use crate::gl::GlRenderer;
use crate::inertia::Inertia;
use crate::scenario::{scenario_letters, scenario_max_offset, scenario_text_size};
use crate::settings::Settings;
use crate::sprite_util::{SpriteUtil, scale_sprite_state};
use crate::text_util::{TextUtil, measure_text_line};
use balchug_common::F32Rect;
use balchug_common::atlas::{Atlas, AtlasItem, FontData};
use balchug_common::scenario::Scenario;
use balchug_common::sprite::{Sprite, SpriteAnimation, SpriteData, SpriteState, SpriteTextData};
use log::{error, info};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AddEventListenerOptions, HtmlCanvasElement, HtmlImageElement, MouseEvent, Request, Response, TouchEvent, WebGl2RenderingContext, WheelEvent, Window, window};
use crate::font::glyphs_render::prepare_glyphs;

mod gl;
mod inertia;
mod scenario;
mod font;
mod sprite_util;
mod fps;
pub mod settings;
mod text_util;

pub const STATE_OFFSET_LAG: f32 = 0.0333;
pub const TEXT_SIZE_FACTOR: f32 = 0.002;

pub trait OffsetListener {
    fn offset_change(&mut self, offset: f32);
}

type FontListener = Box<dyn FnMut()>;

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
    font_listener: Rc<Cell<Option<FontListener>>>,
    scenario: Rc<RefCell<Scenario>>,
    canvas_width: Rc<Cell<f32>>,
    sprite_util: Rc<Cell<SpriteUtil>>,
    text_util: Rc<Cell<TextUtil>>,
    last_frame: Rc<Cell<f64>>,
    touch_start_screen: Rc<Cell<f32>>,
    touch_start_scroll: Rc<Cell<f32>>,
    offset_listener: Rc<RefCell<Option<Box<dyn OffsetListener>>>>,
    fps: Rc<RefCell<FpsCounter>>,
}

impl AppContext {
    fn new(canvas_width: f32, canvas_height: f32, font_listener: Option<FontListener>) -> Self {
        AppContext {
            force_rerender: Rc::new(Cell::new(false)),
            scroll: Rc::new(RefCell::new(Inertia::new(0.0))),
            images_texture_ready: Rc::new(Cell::new(false)),
            txt_texture_ready: Rc::new(Cell::new(false)),
            atlas_items: Rc::new(RefCell::new(HashMap::default())),
            font_atlas_items: Rc::new(RefCell::new(HashMap::default())),
            scenario: Rc::new(RefCell::new(Scenario::default())),
            font: Rc::new(RefCell::new(FontData::default())),
            font_listener: Rc::new(Cell::new(font_listener)),
            font_bytes: Rc::new(RefCell::new(Vec::new())),
            canvas_width: Rc::new(Cell::new(canvas_width)),
            sprite_util: Rc::new(Cell::new(SpriteUtil::new(canvas_width, canvas_height))),
            text_util: Rc::new(Cell::new(TextUtil::new(canvas_width, canvas_height))),
            last_frame: Rc::new(Cell::new(0.0)),
            touch_start_screen: Rc::new(Cell::new(0.0)),
            touch_start_scroll: Rc::new(Cell::new(0.0)),
            offset_listener: Rc::new(RefCell::new(None)),
            fps: Rc::new(RefCell::new(FpsCounter::default())),
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
            info!("Resizing canvas to {width}x{height}");
            let (width, height) = (width.round() as u32, height.round() as u32);
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.context.canvas_width.set(width as f32);
            self.context.sprite_util.set(SpriteUtil::new(width as f32, height as f32));
            self.context.text_util.set(TextUtil::new(width as f32, height as f32));
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

    pub fn get_offset(&self) -> f32 {
        self.context.scroll.borrow().get_value() / self.context.canvas_width.get()
    }

    pub fn set_atlas(&self, img_url: &str, atlas: Atlas) {
        load_images_texture(&self.context, &self.renderer, img_url);
        self.context.atlas_items.replace(atlas.items);
    }

    pub fn set_font(&self, img_url: &str) {
        load_font(&self.context, &self.renderer, img_url)
    }

    pub fn set_scenario(&self, sprites: Vec<SpriteAnimation>) {
        self.context.scenario.borrow_mut().sprites = sprites;
        self.update();
    }

    pub fn interpolate_state(&self, states: &[SpriteState], offset: f32, smooth_factor: f32) -> Option<SpriteState> {
        if let Some(mut result) = self.context.sprite_util.get().interpolate_state(states, offset, smooth_factor) {
            if let Some(fixed_state) = states.iter()
                .find(|state| (offset - state.offset).abs() < STATE_OFFSET_LAG) {
                result.easing = fixed_state.easing;
                result.from_bottom = fixed_state.from_bottom;
                result.y = fixed_state.y;
            }
            Some(result)
        } else {
            None
        }
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

    pub fn get_fps(&self) -> usize {
        self.context.fps.borrow().get_fps()
    }

    pub fn measure_text(&self, data: &SpriteTextData, scale: f32) -> (f32, f32) {
        if data.text == " " {
            return measure_text_line(&data.text, data.size, scale, &self.context.font.borrow());
        }

        let mut letters = scenario_letters(&self.context.scenario.borrow());
        let new_letters = data.text.chars()
            .filter(|c| !letters.contains(&c.to_string()))
            .collect::<String>();
        letters.push_str(&new_letters);

        let new_font_data = if !new_letters.is_empty() {
            let bytes = self.context.font_bytes.borrow();
            let font_size = data.size as f32 * TEXT_SIZE_FACTOR;
            if let Ok((font_data, _)) = prepare_glyphs(&letters, &bytes, font_size) {
                Some(font_data)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(font_data) = new_font_data {
            measure_text_line(&data.text, data.size, scale, &font_data)
        } else {
            measure_text_line(&data.text, data.size, scale, &self.context.font.borrow())
        }
    }

    pub fn update_settings(&self, settings: Settings) {
        let color = convert_color(settings.background_color);
        self.renderer.set_background_color(color);
        self.context.force_rerender.set(true);
    }
}

fn convert_color(color_u8: [u8; 3]) -> [f32; 3] {
    [
        color_u8[0] as f32 / 255.0,
        color_u8[1] as f32 / 255.0,
        color_u8[2] as f32 / 255.0,
    ]
}

fn animate_scene(ctx: &AppContext, renderer: &GlRenderer, current_time_ms: f64) {
    let last_time_ms = ctx.last_frame.get();
    let elapsed_ms = current_time_ms - last_time_ms;

    // Target 60 FPS (~16.67ms per frame)
    const FRAME_DURATION_MS: f64 = 1000.0 / 60.0;

    // If the mobile screen is running at 120Hz, skip every other frame
    // to maintain a perfectly uniform rendering cadence.
    if elapsed_ms < FRAME_DURATION_MS {
        return;
    }

    // Lock the delta to prevent timestamp rounding jitter from Chrome
    // and adjust for slight fractional drift.
    ctx.last_frame.set(current_time_ms - (elapsed_ms % FRAME_DURATION_MS));

    // Convert fixed frame duration to seconds for your physics engine (0.016666)
    let fixed_elapsed_secs = (FRAME_DURATION_MS / 1000.0) as f32;

    let now = instant::Instant::now();

    let (updated, offset) = ctx.scroll.borrow_mut().live(fixed_elapsed_secs);
    if !updated && !ctx.force_rerender.get() {
        ctx.fps.borrow_mut().new_frame(now, false);
        return;
    }

    let width = ctx.canvas_width.get();
    let sprite_util = ctx.sprite_util.get();
    let text_util = ctx.text_util.get();
    let scaled_offset = offset / width;
    if let Some(listener) = ctx.offset_listener.borrow_mut().as_mut() {
        listener.offset_change(scaled_offset)
    }
    let scenario = ctx.scenario.borrow();
    let (mut sprites, mut text_sprites) = (Vec::new(), Vec::new());

    for sprite_animation in &scenario.sprites {
        if let Some(cur_state) = sprite_util.interpolate_state(&sprite_animation.states, scaled_offset, sprite_animation.smooth_factor)
            && cur_state.color[3] > 0 {
            match &sprite_animation.data {
                SpriteData::Image(image_data) => {
                    sprites.push(Sprite {
                        state: scale_sprite_state(&cur_state, width),
                        atlas_item: *ctx.atlas_items.borrow().get(&image_data.atlas_item_id).unwrap(),
                    });
                }
                SpriteData::Text(text_data) => {
                    for glyph_sprite in text_util.arrange_text_line(text_data, &cur_state, &ctx.font.borrow(), &ctx.font_atlas_items.borrow()) {
                        let glyph_sprite = Sprite {
                            state: scale_sprite_state(&glyph_sprite.state, width),
                            atlas_item: glyph_sprite.atlas_item,
                        };
                        text_sprites.push(glyph_sprite);
                    }
                }
            }
        }
    }

    if !sprites.is_empty() && !ctx.images_texture_ready.get() || !text_sprites.is_empty() && !ctx.txt_texture_ready.get() {
        // some texture is not ready yet
        ctx.fps.borrow_mut().new_frame(now, false);
    } else {
        ctx.fps.borrow_mut().new_frame(now, true);
        renderer.render(&sprites, &text_sprites);
        ctx.force_rerender.set(false);
    }
}

pub fn start_engine(window: Window, canvas: HtmlCanvasElement, settings: Settings, font_listener: Option<FontListener>) -> BalchugEngine {
    wasm_logger::init(wasm_logger::Config::default());

    let pixel_ratio = window.device_pixel_ratio().max(2.0);

    let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap();
    let renderer = GlRenderer::init(gl, convert_color(settings.background_color)).unwrap();
    let (width, height) = (canvas.width(), canvas.height());
    renderer.set_sizes(width as f32, height as f32);

    let ctx = AppContext::new(width as f32, height as f32, font_listener);

    let ctx_clone = ctx.clone();
    let renderer_clone = renderer.clone();
    let render = move |timestamp: f64| {
        animate_scene(&ctx_clone, &renderer_clone, timestamp);
    };

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

    let on_frame = Rc::new(RefCell::new(
        Closure::wrap(Box::new(move |_| {}) as Box<dyn FnMut(f64)>)
    ));
    let on_frame_clone = on_frame.clone();
    let render_clone = render.clone();
    let window_clone = window.clone();
    *on_frame.borrow_mut() = Closure::wrap(Box::new(move |timestamp| {
        render_clone(timestamp);
        if let Err(err) = window_clone.request_animation_frame(on_frame_clone.borrow().as_ref().unchecked_ref()) {
            error!("Request animation frame failed: {err:?}");
        }
    }));
    if let Err(err) = window.request_animation_frame(on_frame.borrow().as_ref().unchecked_ref()) {
        error!("Request first animation frame failed: {err:?}");
    }

    BalchugEngine {
        pixel_ratio: pixel_ratio as f32,
        context: Rc::new(ctx),
        renderer: Rc::new(renderer),
        canvas
    }
}

fn load_images_texture(ctx: &AppContext, renderer: &GlRenderer, img_url: &str) {
    let atlas_img = HtmlImageElement::new().unwrap();
    atlas_img.set_cross_origin(Some("anonymous"));
    atlas_img.set_src(img_url);
    ctx.images_texture_ready.set(false);
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
}

fn load_font(ctx: &AppContext, renderer: &GlRenderer, font_url: &str) {
    match Request::new_with_str(font_url) {
        Ok(req) => {
            ctx.txt_texture_ready.set(false);
            let on_response = {
                let ctx = ctx.clone();
                let renderer = renderer.clone();
                let on_body = {
                    Closure::wrap(Box::new(move |buf_value: JsValue| {
                        let bytes = js_sys::Uint8Array::new(&buf_value).to_vec();
                        info!("Load font {} bytes", bytes.len());
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
            if let Some(window) = window() {
                let _ = window.fetch_with_request(&req).then(&on_response);
            }
            on_response.forget();
        }
        Err(err) => {
            error!("Error make font request: {err:?}");
        }
    }
}

fn rebuild_font(ctx: &AppContext, renderer: &GlRenderer) {
    let bytes = ctx.font_bytes.borrow();
    if bytes.is_empty() {
        return;
    }

    let letters = scenario_letters(&ctx.scenario.borrow());
    let known_letters = ctx.font.borrow().glyphs.keys().cloned().collect::<HashSet<_>>();
    let listener = ctx.font_listener.take();
    if listener.is_some() || letters.chars().any(|c| c != ' ' && !known_letters.contains(&c)) {
        let font_size = scenario_text_size(&ctx.scenario.borrow(), ctx.canvas_width.get());
        info!("Font size: {font_size}");
        if let Some(res) = build_font(&letters, &bytes, font_size) {
            ctx.font.replace(res.font_data);
            if !letters.is_empty() {
                renderer.set_font_texture(res.atlas.width, res.atlas.height, &res.data);
                ctx.font_atlas_items.replace(res.atlas.items);
                ctx.txt_texture_ready.replace(true);
                ctx.force_rerender.set(true);
            }
        }
    }
    if let Some(mut listener) = listener {
        listener();
    }
}
