use crate::core::color::Color;
use crate::core::error::OxideError;
use crate::renderer::buffer::{IndexBuffer, VertexArray, VertexBuffer};
use crate::renderer::shader::ShaderProgram;
use crate::text::atlas::FontAtlas;
use glam::{Mat4, Vec2};
use glow::HasContext;

const VERTEX_SHADER_TEXT: &str = r#"#version 330 core
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_uv;

uniform mat4 u_projection;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = u_projection * vec4(a_position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER_TEXT: &str = r#"#version 330 core
in vec2 v_uv;

uniform vec4 u_color;
uniform sampler2D u_atlas;

out vec4 o_color;

void main() {
    float alpha = texture(u_atlas, v_uv).a;
    o_color = vec4(u_color.rgb, u_color.a * alpha);
}
"#;

#[repr(C)]
#[derive(Clone, Copy)]
struct TextVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

pub struct PipelineText {
    shader: ShaderProgram,
    projection: Mat4,
}

impl PipelineText {
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        let shader = ShaderProgram::new(gl, VERTEX_SHADER_TEXT, FRAGMENT_SHADER_TEXT)?;
        Ok(Self {
            shader,
            projection: Mat4::IDENTITY,
        })
    }

    pub fn begin(&mut self, gl: &glow::Context, screen_width: f32, screen_height: f32) {
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        self.projection = Mat4::orthographic_rh_gl(
            0.0,
            screen_width,
            screen_height,
            0.0,
            -1.0,
            1.0,
        );

        self.shader.bind(gl);
        self.shader.set_mat4(gl, "u_projection", &self.projection);
        self.shader.set_int(gl, "u_atlas", 0);
    }

    pub fn draw_text(
        &self,
        gl: &glow::Context,
        atlas: &FontAtlas,
        text: &str,
        origin: Vec2,
        color: Color,
        scale: f32,
    ) -> Result<(), OxideError> {
        if text.is_empty() {
            return Ok(());
        }

        let mut vertices: Vec<TextVertex> = Vec::with_capacity(text.len() * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(text.len() * 6);

        let mut cursor_x = origin.x;
        let mut cursor_y = origin.y;
        let line_height = atlas.line_height * scale;

        for c in text.chars() {
            if c == '\n' {
                cursor_x = origin.x;
                cursor_y += line_height;
                continue;
            }

            if let Some(glyph) = atlas.glyphs.get(&c) {
                if glyph.size.x > 0.0 && glyph.size.y > 0.0 {
                    let gx = cursor_x + glyph.offset.x * scale;
                    let gy = cursor_y + (atlas.font_size - glyph.offset.y - glyph.size.y) * scale;
                    let gw = glyph.size.x * scale;
                    let gh = glyph.size.y * scale;

                    let base_idx = vertices.len() as u32;

                    vertices.push(TextVertex {
                        position: [gx, gy],
                        uv: [glyph.uv_min.x, glyph.uv_min.y],
                    });
                    vertices.push(TextVertex {
                        position: [gx + gw, gy],
                        uv: [glyph.uv_max.x, glyph.uv_min.y],
                    });
                    vertices.push(TextVertex {
                        position: [gx + gw, gy + gh],
                        uv: [glyph.uv_max.x, glyph.uv_max.y],
                    });
                    vertices.push(TextVertex {
                        position: [gx, gy + gh],
                        uv: [glyph.uv_min.x, glyph.uv_max.y],
                    });

                    indices.extend_from_slice(&[
                        base_idx,
                        base_idx + 1,
                        base_idx + 2,
                        base_idx + 2,
                        base_idx + 3,
                        base_idx,
                    ]);
                }

                cursor_x += glyph.advance_x * scale;
            } else {
                cursor_x += atlas.font_size * 0.6 * scale;
            }
        }

        if vertices.is_empty() {
            return Ok(());
        }

        self.shader.set_vec4(gl, "u_color", color.to_vec4());
        atlas.texture.bind(gl, 0);

        let vao = VertexArray::new(gl)?;
        let vbo = VertexBuffer::new(gl, &vertices)?;
        let ebo = IndexBuffer::new(gl, &indices)?;

        vao.bind(gl);
        vbo.bind(gl);
        ebo.bind(gl);

        unsafe {
            let stride = std::mem::size_of::<TextVertex>() as i32;

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

            gl.enable_vertex_attrib_array(1);
            let uv_offset = 2 * std::mem::size_of::<f32>() as i32;
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, uv_offset);

            gl.draw_elements(
                glow::TRIANGLES,
                ebo.count() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }

        ebo.unbind(gl);
        vbo.unbind(gl);
        vao.unbind(gl);

        ebo.destroy(gl);
        vbo.destroy(gl);
        vao.destroy(gl);

        Ok(())
    }

    pub fn end(&self, gl: &glow::Context) {
        self.shader.unbind(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.shader.destroy(gl);
    }
}
