use gl;
use gl::types::*;
use std::mem;
use std::ptr;

use fonts::Font;
use graphics::{
    Program,
    Shader,
};
use math::Vec2;

macro_rules! offset_of {
    ($ty: ty, $field: ident) => {{
        (&(*(ptr::null() as *const $ty)).$field as *const _ as usize)
    }}
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32
}

#[derive(Clone, Copy)]
pub struct Viewport{
    pub p: Vec2,
    pub w: f32,
    pub h: f32
}

#[derive(Clone, Copy, Debug)]
struct RectVertex {
    bound: [f32; 4],
    color: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
struct CircleVertex {
    center: [f32; 2],
    radius: f32,
    color: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
struct LineVertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone, Copy, Debug)]
struct TextVertex {
    bound: [f32; 4],
    uv_bound: [f32; 4],
    color: [f32; 4],
}

struct RectTechnique {
    program: GLuint,
    uniform_viewport: GLint,
    vao: GLuint,
    vbo: GLuint,
    vertices: Vec<RectVertex>,
}

struct CircleTechnique {
    program: GLuint, 
    uniform_viewport: GLint,
    vao: GLuint,
    vbo: GLuint,
    vertices: Vec<CircleVertex>,
}

struct LineTechnique {
    program: GLuint, 
    uniform_viewport: GLint,
    vao: GLuint,
    vbo: GLuint,
    vertices: Vec<LineVertex>,
}

struct TextTechnique {
    program: GLuint, 
    uniform_viewport: GLint,
    uniform_font_tex: GLint,
    vao: GLuint,
    vbo: GLuint,
}

pub struct Renderer {
    rect_tech: RectTechnique,
    circle_tech: CircleTechnique,
    line_tech: LineTechnique,
    text_tech: TextTechnique,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    fn to_array(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

impl Viewport {
    pub fn new(p: Vec2, w: f32, h: f32) -> Self {
        Viewport { p, w, h }
    }

    fn to_array(&self) -> [f32; 4] {
        [self.p.x, self.p.y, self.w, self.h]
    }
}

impl RectTechnique {
    pub fn new() -> Self { unsafe {
        let program = Program::new(&[
            Shader::from_file(gl::VERTEX_SHADER, "res/shaders/rect.vert").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::GEOMETRY_SHADER, "res/shaders/rect.geom").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::FRAGMENT_SHADER, "res/shaders/rect.frag").expect("Cannot initialize GL resources"),
        ]).expect("Cannot initialize GL resources");
        let uniform_viewport = program.get_uniform_location("viewport");
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<RectVertex>() as _,
            offset_of!(RectVertex, bound) as _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<RectVertex>() as _,
            offset_of!(RectVertex, color) as _,
        );
        RectTechnique { program: program.id, uniform_viewport, vao, vbo, vertices: Vec::new() }
    }}

    pub fn begin_batch(&mut self) {
        self.vertices.clear();
    }

    pub fn draw(&mut self, x: f32, y: f32, w: f32, h: f32, color: &[f32; 4]) {
        self.vertices.push(RectVertex { bound: [x, y, w, h], color: color.clone() });
    }

    pub fn end_batch(&mut self, viewport: &[f32; 4]) { unsafe {
        gl::UseProgram(self.program);
        gl::Uniform4f(self.uniform_viewport, viewport[0], viewport[1], viewport[2], viewport[3]);
        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<RectVertex>() * self.vertices.len()) as _,
            self.vertices.as_ptr() as _,
            gl::STREAM_DRAW,
        );
        gl::DrawArrays(gl::POINTS, 0, self.vertices.len() as _);
    }}
}

impl CircleTechnique {
    pub fn new() -> Self { unsafe {
        let program = Program::new(&[
            Shader::from_file(gl::VERTEX_SHADER, "res/shaders/circle.vert").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::GEOMETRY_SHADER, "res/shaders/circle.geom").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::FRAGMENT_SHADER, "res/shaders/circle.frag").expect("Cannot initialize GL resources"),
        ]).expect("Cannot initialize GL resources");
        let uniform_viewport = program.get_uniform_location("viewport");
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<CircleVertex>() as _,
            offset_of!(CircleVertex, center) as _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            1,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<CircleVertex>() as _,
            offset_of!(CircleVertex, radius) as _,
        );
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<CircleVertex>() as _,
            offset_of!(CircleVertex, color) as _,
        );
        CircleTechnique { program: program.id, uniform_viewport, vao, vbo, vertices: Vec::new() }
    }}

    pub fn begin_batch(&mut self) {
        self.vertices.clear();
    }

    pub fn draw(&mut self, x: f32, y: f32, radius: f32, color: &[f32; 4]) {
        self.vertices.push(CircleVertex { center: [x, y], radius, color: color.clone() });
    }

    pub fn end_batch(&mut self, viewport: &[f32; 4]) { unsafe {
        gl::UseProgram(self.program);
        gl::Uniform4f(self.uniform_viewport, viewport[0], viewport[1], viewport[2], viewport[3]);
        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<CircleVertex>() * self.vertices.len()) as _,
            self.vertices.as_ptr() as _,
            gl::STREAM_DRAW,
        );
        gl::DrawArrays(gl::POINTS, 0, self.vertices.len() as _);
    }}
}

impl LineTechnique {
    pub fn new() -> Self { unsafe {
        let program = Program::new(&[
            Shader::from_file(gl::VERTEX_SHADER, "res/shaders/line.vert").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::FRAGMENT_SHADER, "res/shaders/line.frag").expect("Cannot initialize GL resources"),
        ]).expect("Cannot initialize GL resources");
        let uniform_viewport = program.get_uniform_location("viewport");
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<LineVertex>() as _,
            offset_of!(LineVertex, position) as _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<LineVertex>() as _,
            offset_of!(LineVertex, color) as _,
        );
        LineTechnique { program: program.id, uniform_viewport, vao, vbo, vertices: Vec::new() }
    }}

    pub fn begin_batch(&mut self) {
        self.vertices.clear();
    }

    pub fn draw(&mut self, from_x: f32, from_y: f32, to_x: f32, to_y: f32, color: &[f32; 4]) {
        self.vertices.push(LineVertex { position: [from_x, from_y], color: color.clone() });
        self.vertices.push(LineVertex { position: [to_x, to_y], color: color.clone() });
    }

    pub fn end_batch(&mut self, viewport: &[f32; 4]) { unsafe {
        gl::UseProgram(self.program);
        gl::Uniform4f(self.uniform_viewport, viewport[0], viewport[1], viewport[2], viewport[3]);
        gl::BindVertexArray(self.vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (mem::size_of::<LineVertex>() * self.vertices.len()) as _,
            self.vertices.as_ptr() as _,
            gl::STREAM_DRAW,
        );
        gl::DrawArrays(gl::LINES, 0, self.vertices.len() as _);
    }}
}

impl TextTechnique {
    pub fn new() -> Self { unsafe {
        let program = Program::new(&[
            Shader::from_file(gl::VERTEX_SHADER, "res/shaders/text.vert").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::GEOMETRY_SHADER, "res/shaders/text.geom").expect("Cannot initialize GL resources"),
            Shader::from_file(gl::FRAGMENT_SHADER, "res/shaders/text.frag").expect("Cannot initialize GL resources"),
        ]).expect("Cannot initialize GL resources");
        let uniform_viewport = program.get_uniform_location("viewport");
        let uniform_font_tex = program.get_uniform_location("font_tex");
        let mut vao = 0;
        let mut vbo = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(
            0,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<TextVertex>() as _,
            offset_of!(TextVertex, bound) as _,
        );
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(
            1,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<TextVertex>() as _,
            offset_of!(TextVertex, uv_bound) as _,
        );
        gl::EnableVertexAttribArray(2);
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<TextVertex>() as _,
            offset_of!(TextVertex, color) as _,
        );
        TextTechnique { program: program.id, uniform_viewport, uniform_font_tex, vao, vbo }
    }}

    pub fn draw(&mut self, viewport: &[f32; 4], text: &str, p: Vec2, color: Color, font: &Font) {
        let mut cursor = p;
        let mut vertices = Vec::new();
        for c in text.chars() {
            if c == '\n' {
                cursor.x = p.x;
                cursor.y -= font.line_height as f32;
                continue;
            }
            let ci = c as u8;
            let ref glyph_info = font.glyph_infos[ci as usize];
            if glyph_info.has_bitmap {
                vertices.push(TextVertex {
                    bound: [
                        cursor.x + glyph_info.x as f32,
                        cursor.y + glyph_info.y as f32 - glyph_info.h as f32,
                        cursor.x + glyph_info.x as f32 + glyph_info.w as f32,
                        cursor.y + glyph_info.y as f32,
                    ],
                    uv_bound: [
                        glyph_info.uv_min.x,
                        glyph_info.uv_min.y,
                        glyph_info.uv_max.x,
                        glyph_info.uv_max.y,
                    ],
                    color: color.to_array(),
                });
                cursor = cursor + glyph_info.advance;
            }
        }
        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform4f(self.uniform_viewport, viewport[0], viewport[1], viewport[2], viewport[3]);
            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<TextVertex>() * vertices.len()) as _,
                vertices.as_ptr() as _,
                gl::STREAM_DRAW,
            );
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, font.texture);
            gl::Uniform1i(self.uniform_font_tex, 0);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            gl::DrawArrays(gl::POINTS, 0, vertices.len() as _);
            gl::Disable(gl::BLEND);
        }
    }
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            rect_tech: RectTechnique::new(),
            circle_tech: CircleTechnique::new(),
            line_tech: LineTechnique::new(),
            text_tech: TextTechnique::new(),
        }
    }

    pub fn begin_batch(&mut self) {
        self.rect_tech.begin_batch();
        self.circle_tech.begin_batch();
        self.line_tech.begin_batch();
    }

    pub fn draw_rectangle(&mut self, p: Vec2, w: f32, h: f32, color: Color) {
        self.rect_tech.draw(p.x, p.y, w, h, &color.to_array());
    }

    pub fn draw_circle(&mut self, c: Vec2, radius: f32, color: Color) {
        self.circle_tech.draw(c.x, c.y, radius, &color.to_array());
    }

    pub fn draw_line(&mut self, from: Vec2, to: Vec2, color: Color) {
        self.line_tech.draw(from.x, from.y, to.x, to.y, &color.to_array());
    }

    pub fn draw_text(&mut self, viewport: &Viewport, text: &str, p: Vec2, color: Color, font: &Font) {
        self.text_tech.draw(&viewport.to_array(), text, p, color, font);
    }

    pub fn end_batch(&mut self, viewport: &Viewport) {
        self.rect_tech.end_batch(&viewport.to_array());
        self.circle_tech.end_batch(&viewport.to_array());
        self.line_tech.end_batch(&viewport.to_array());
    }
}
