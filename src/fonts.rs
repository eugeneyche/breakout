use freetype as ft;
use gl;
use gl::types::*;
use std::path::Path;

use math::Vec2;

#[derive(Clone, Copy, Debug, Default)]
pub struct GlyphInfo {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub advance: Vec2,
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub has_bitmap: bool,
}

pub struct Font {
    pub texture: GLuint,
    pub line_height: f32,
    pub glyph_infos: Vec<GlyphInfo>,
}

#[derive(Clone, Copy, Debug)]
pub struct FontHandle(u32);

pub struct FontMeasure {
    pub min: Vec2,
    pub max: Vec2,
}


pub struct FontLibrary {
    ft_library: ft::Library,
    font_store: Vec<Font>,
}

impl Font {
    pub fn measure(&self, text: &str) -> FontMeasure {
        let mut cursor = Vec2::default();
        let mut measure = FontMeasure {
            min: Vec2::default(),
            max: Vec2::default(),
        };
        for c in text.chars() {
            if c == '\n' {
                cursor.x = 0.;
                cursor.y += self.line_height;
                continue;
            }
            let ci = c as u8;
            let ref glyph_info = self.glyph_infos[ci as usize];
            if glyph_info.has_bitmap {
                if measure.min.x < cursor.x + glyph_info.x as f32 {
                    measure.min.x = cursor.x + glyph_info.x as f32;
                }
                if measure.min.y < cursor.y + glyph_info.y as f32 - glyph_info.h as f32 {
                    measure.min.y = cursor.y + glyph_info.y as f32 - glyph_info.h as f32;
                }
                if measure.max.x > cursor.x + glyph_info.x as f32 + glyph_info.w as f32 {
                    measure.max.x = cursor.x + glyph_info.x as f32 + glyph_info.w as f32;
                }
                if measure.max.y > cursor.y + glyph_info.y as f32 {
                    measure.max.y = cursor.y + glyph_info.y as f32;
                }
            }
            cursor = cursor + glyph_info.advance;
        }
        measure
    }
}

impl FontLibrary {
    pub fn new() -> Self {
        FontLibrary {
            ft_library: ft::Library::init().expect("Cannot initialize Freetype."),
            font_store: Vec::new(),
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(&mut self, path: P, size: u32) -> FontHandle {
        let path = path.as_ref();
        let face = self.ft_library.new_face(path, 0).expect("Cannot load Freetype face");
        face.set_pixel_sizes(0, size).expect("Cannot set Freetype face pixel size");
        let line_height = face.height() as f32 / 64.;
        let mut glyph_infos = vec![GlyphInfo::default(); 256];
        let mut bitmaps = vec![None; 256];
        let mut total_w = 0;
        let mut max_glyph_w = 0;
        let mut max_glyph_h = 0;
        for (i, glyph_info) in glyph_infos.iter_mut().enumerate() {
            if !face.load_char(i, ft::face::LoadFlag::RENDER).is_ok() {
                continue;
            }
            let glyph = face.glyph();
            glyph_info.x = glyph.bitmap_left() as _;
            glyph_info.y = glyph.bitmap_top() as _;
            glyph_info.w = glyph.bitmap().width() as _;
            glyph_info.h = glyph.bitmap().rows() as _;
            glyph_info.advance = Vec2::new(glyph.advance().x as f32 / 64., glyph.advance().y as f32 / 64.);
            glyph_info.has_bitmap = true;
            bitmaps[i] = Some(glyph.bitmap().buffer().to_vec());
            total_w += glyph_info.w;
            if max_glyph_w < glyph_info.w {
                max_glyph_w = glyph_info.w;
            }
            if max_glyph_h < glyph_info.h {
                max_glyph_h = glyph_info.h;
            }
        }
        let ratio = (total_w as f32 / max_glyph_h as f32).sqrt() as i32;
        let unpacked_tex_width = total_w / ratio + max_glyph_w;
        let tex_width = (unpacked_tex_width / 4 + 1) * 4;
        let tex_height = max_glyph_h as i32 * ratio;
        let mut tex_buffer = vec![0u8; (tex_width * tex_height) as usize];
        let mut cursor_x = 0;
        let mut cursor_y = 0;
        for (i, glyph_info) in glyph_infos.iter_mut().enumerate() {
            if let Some(buffer) = bitmaps[i].as_ref() {
                if cursor_x + glyph_info.w > tex_width as i32 {
                    cursor_x = 0;
                    cursor_y += max_glyph_h;
                }
                let min_tex_y = tex_height - cursor_y - glyph_info.h;
                glyph_info.uv_min = Vec2::new(
                    cursor_x as f32 / tex_width as f32,
                    min_tex_y as f32 / tex_height as f32,
                );

                glyph_info.uv_max = Vec2::new(
                    (cursor_x as f32 + glyph_info.w as f32) / tex_width as f32,
                    (min_tex_y as f32 + glyph_info.h as f32) / tex_height as f32,
                );

                for row in 0..glyph_info.h {
                    let buffer_begin = ((glyph_info.h - row - 1) * glyph_info.w) as usize;
                    let buffer_end = buffer_begin + glyph_info.w as usize;
                    let tex_begin = ((min_tex_y + row) * tex_width + cursor_x) as usize;
                    let tex_end = tex_begin + glyph_info.w as usize;
                    tex_buffer[tex_begin..tex_end].copy_from_slice(&buffer[buffer_begin..buffer_end]);
                }
                cursor_x += glyph_info.w;
            }
        }
        unsafe {
            let mut tex = 0;
            gl::GenTextures(1, &mut tex);
            gl::BindTexture(gl::TEXTURE_2D, tex);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::COMPRESSED_RED as _,
                tex_width as _,
                tex_height as _,
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                tex_buffer.as_ptr() as _
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as _);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as _);
            self.font_store.push(Font {
                texture: tex,
                line_height: line_height,
                glyph_infos
            });
            FontHandle((self.font_store.len() - 1) as _)
        }
    }

    pub fn get<'a>(&'a self, handle: FontHandle) -> &'a Font {
        let FontHandle(index) = handle;
        &self.font_store[index as usize]
    }
}
