use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use web_sys::{HtmlImageElement, WebGl2RenderingContext as GL, WebGlProgram, WebGlShader, WebGlTexture};
use balchug_common::sprite::Sprite;

#[derive(Clone)]
pub struct GlRenderer {
    gl: GL,
    program: WebGlProgram,
    txt_program: WebGlProgram,
    texture: WebGlTexture,
    font_texture: WebGlTexture,
    texture_size: Rc<Cell<(u32, u32)>>,
    font_size: Rc<Cell<(u32, u32)>>,
}

impl GlRenderer {
    pub fn init(gl: GL) -> Result<GlRenderer, JsValue> {
        // --- Shaders ---
        let vert_code = r#"#version 300 es
        precision highp float;

        in vec2 a_position;

        // Uniforms for sprite placement (in screen pixels)
        uniform vec2 u_spritePos;    // sprite_x, sprite_y
        uniform vec2 u_spriteSize;   // sprite_width, sprite_height
        uniform vec2 u_canvasSize;   // width, height of your canvas

        // Uniforms for texture area (in atlas pixels)
        uniform vec2 u_texPos;       // tex_x, tex_y
        uniform vec2 u_texSize;      // tex_width, tex_height
        uniform vec2 u_atlasSize;

        out vec2 v_texCoord;

        void main() {
            vec2 pixelPos = u_spritePos + (a_position * u_spriteSize);
            vec2 clipSpace = (pixelPos / u_canvasSize) * 2.0 - 1.0;
            gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
            v_texCoord = (u_texPos + (a_position * u_texSize)) / u_atlasSize;
        }"#;

        let frag_code = r#"#version 300 es
        precision highp float;

        uniform sampler2D u_texture;
        uniform float u_spriteAlpha;

        in vec2 v_texCoord;
        out vec4 outColor;

        void main() {
            outColor = texture(u_texture, v_texCoord);
            outColor.a *= u_spriteAlpha;
        }"#;

        let txt_frag_code = r#"#version 300 es
        precision highp float;

        uniform sampler2D u_texture;
        uniform vec4 u_spriteColor;

        in vec2 v_texCoord;
        out vec4 outColor;

        void main() {
            vec4 texColor = texture(u_texture, v_texCoord);
            outColor = vec4(u_spriteColor.rgb, u_spriteColor.a * texColor.r);
            //outColor = texture(u_texture, v_texCoord);
        }"#;

        let program = link_program(&gl, vert_code, frag_code)?;
        let txt_program = link_program(&gl, vert_code, txt_frag_code)?;

        // --- Geometry
        let vertices: [f32; 8] = [0.0,0.0, 1.0,0.0, 1.0,1.0, 0.0,1.0];
        let buffer = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        unsafe {
            let view = js_sys::Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &view, GL::STATIC_DRAW);
        }

        let pos_loc = gl.get_attrib_location(&program, "a_position") as u32;
        gl.enable_vertex_attrib_array(pos_loc);
        gl.vertex_attrib_pointer_with_i32(pos_loc, 2, GL::FLOAT, false, 0, 0);

        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);

        // --- Texture ---
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        let font_texture = gl.create_texture().unwrap();
        gl.bind_texture(GL::TEXTURE_2D, Some(&font_texture));
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::LINEAR_MIPMAP_LINEAR as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        let (width, height) = (900.0, 1600.0);
        gl.use_program(Some(&program));
        let canvas_size_loc = gl.get_uniform_location(&program, "u_canvasSize");
        gl.uniform2f(canvas_size_loc.as_ref(), width, height);

        gl.use_program(Some(&txt_program));
        let canvas_size_loc = gl.get_uniform_location(&txt_program, "u_canvasSize");
        gl.uniform2f(canvas_size_loc.as_ref(), width, height);

        Ok(GlRenderer {
            gl, program, txt_program, texture, font_texture,
            texture_size: Rc::new(Cell::new((0, 0))),
            font_size: Rc::new(Cell::new((0, 0))),
        })
    }

    pub fn set_sizes(&self, width: f32, height: f32) {
        self.gl.viewport(0, 0, width as i32, height as i32);

        self.gl.use_program(Some(&self.program));
        let canvas_size_loc = self.gl.get_uniform_location(&self.program, "u_canvasSize");
        self.gl.uniform2f(canvas_size_loc.as_ref(), width, height);

        self.gl.use_program(Some(&self.txt_program));
        let canvas_size_loc = self.gl.get_uniform_location(&self.txt_program, "u_canvasSize");
        self.gl.uniform2f(canvas_size_loc.as_ref(), width, height);
    }

    pub fn set_texture(&self, img: &HtmlImageElement) {
        self.texture_size.replace((img.width(), img.height()));
        self.gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture));
        self.gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            GL::TEXTURE_2D, 0, GL::RGBA as i32, GL::RGBA, GL::UNSIGNED_BYTE, img
        ).unwrap();
        self.gl.generate_mipmap(GL::TEXTURE_2D);
    }

    pub fn set_font_texture(&self, width: u32, height: u32, data: &[u8]) {
        self.font_size.replace((width, height));
        self.gl.bind_texture(GL::TEXTURE_2D, Some(&self.font_texture));
        self.gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_u8_array_and_src_offset(
            GL::TEXTURE_2D, 0, GL::LUMINANCE as i32, width as i32, height as i32, 0,
            GL::LUMINANCE, GL::UNSIGNED_BYTE, data, 0
        ).unwrap();
        self.gl.generate_mipmap(GL::TEXTURE_2D);
    }

    pub fn render(&self, sprites: &[Sprite], txt_sprites: &[Sprite]) {
        self.gl.clear_color(1.0, 1.0, 1.0, 1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT);

        if !sprites.is_empty() {
            self.gl.bind_texture(GL::TEXTURE_2D, Some(&self.texture));
            self.gl.use_program(Some(&self.program));
            let atlas_pos_loc = self.gl.get_uniform_location(&self.program, "u_atlasSize");
            let (width, height) = self.texture_size.get();
            self.gl.uniform2f(atlas_pos_loc.as_ref(), width as f32, height as f32);
            for sprite in sprites {
                let sprite_pos_loc = self.gl.get_uniform_location(&self.program, "u_spritePos");
                let sprite_size_loc = self.gl.get_uniform_location(&self.program, "u_spriteSize");
                self.gl.uniform2f(sprite_pos_loc.as_ref(), sprite.state.x, sprite.state.y);
                self.gl.uniform2f(sprite_size_loc.as_ref(), sprite.state.width, sprite.state.height);

                let sprite_alpha_loc = self.gl.get_uniform_location(&self.program, "u_spriteAlpha");
                self.gl.uniform1f(sprite_alpha_loc.as_ref(), sprite.state.color[3]);

                let tex_pos_loc = self.gl.get_uniform_location(&self.program, "u_texPos");
                let tex_size_loc = self.gl.get_uniform_location(&self.program, "u_texSize");
                self.gl.uniform2f(tex_pos_loc.as_ref(), sprite.atlas_item.x as f32, sprite.atlas_item.y as f32);
                self.gl.uniform2f(tex_size_loc.as_ref(), sprite.atlas_item.width as f32, sprite.atlas_item.height as f32);

                self.gl.draw_arrays(GL::TRIANGLE_FAN, 0, 4);
            }
        }

        if !txt_sprites.is_empty() {
            self.gl.bind_texture(GL::TEXTURE_2D, Some(&self.font_texture));
            self.gl.use_program(Some(&self.txt_program));
            let atlas_pos_loc = self.gl.get_uniform_location(&self.txt_program, "u_atlasSize");
            let (width, height) = self.font_size.get();
            self.gl.uniform2f(atlas_pos_loc.as_ref(), width as f32, height as f32);
            for sprite in txt_sprites {
                let sprite_pos_loc = self.gl.get_uniform_location(&self.txt_program, "u_spritePos");
                let sprite_size_loc = self.gl.get_uniform_location(&self.txt_program, "u_spriteSize");
                self.gl.uniform2f(sprite_pos_loc.as_ref(), sprite.state.x, sprite.state.y);
                self.gl.uniform2f(sprite_size_loc.as_ref(), sprite.state.width, sprite.state.height);

                let sprite_alpha_loc = self.gl.get_uniform_location(&self.txt_program, "u_spriteColor");
                self.gl.uniform4f(sprite_alpha_loc.as_ref(), sprite.state.color[0], sprite.state.color[1], sprite.state.color[2], sprite.state.color[3]);

                let tex_pos_loc = self.gl.get_uniform_location(&self.txt_program, "u_texPos");
                let tex_size_loc = self.gl.get_uniform_location(&self.txt_program, "u_texSize");
                self.gl.uniform2f(tex_pos_loc.as_ref(), sprite.atlas_item.x as f32, sprite.atlas_item.y as f32);
                self.gl.uniform2f(tex_size_loc.as_ref(), sprite.atlas_item.width as f32, sprite.atlas_item.height as f32);

                self.gl.draw_arrays(GL::TRIANGLE_FAN, 0, 4);
            }
        }
    }
}

fn link_program(gl: &GL, vert: &str, frag: &str) -> Result<WebGlProgram, JsValue> {
    let program = gl.create_program().ok_or("Unable to create program")?;
    let v_shader = compile_shader(gl, GL::VERTEX_SHADER, vert)?;
    let f_shader = compile_shader(gl, GL::FRAGMENT_SHADER, frag)?;
    gl.attach_shader(&program, &v_shader);
    gl.attach_shader(&program, &f_shader);
    gl.link_program(&program);
    Ok(program)
}

fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, JsValue> {
    let shader = gl.create_shader(shader_type).ok_or("Unable to create shader")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);
    Ok(shader)
}
