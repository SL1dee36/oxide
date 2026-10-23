use crate::core::color::Color;
use crate::core::error::OxideError;
use crate::renderer::buffer::{IndexBuffer, VertexArray, VertexBuffer};
use crate::renderer::shader::ShaderProgram;
use crate::renderer::texture::Texture2D;
use glam::Mat4;
use glow::HasContext;

const VERTEX_SHADER_2D: &str = r#"#version 450 core
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_uv;

uniform mat4 u_projection;
uniform mat4 u_model;

out vec2 v_uv;

void main() {
    v_uv = a_uv;
    gl_Position = u_projection * u_model * vec4(a_position, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER_2D: &str = r#"#version 450 core
in vec2 v_uv;

uniform vec4 u_color;
uniform sampler2D u_texture;
uniform int u_use_texture;

out vec4 o_color;

void main() {
    vec4 tex_color = vec4(1.0);
    if (u_use_texture == 1) {
        tex_color = texture(u_texture, v_uv);
    }
    o_color = u_color * tex_color;
}
"#;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex2D {
    position: [f32; 2],
    uv: [f32; 2],
}

/// 2D graphics rendering pipeline utilizing screen-space orthographic projection and alpha blending.
pub struct Pipeline2D {
    shader: ShaderProgram,
    quad_vao: VertexArray,
    quad_vbo: VertexBuffer,
    quad_ebo: IndexBuffer,
    projection: Mat4,
}

impl Pipeline2D {
    /// Compiles shaders and initializes shared unit-quad geometry buffers.
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        let shader = ShaderProgram::new(gl, VERTEX_SHADER_2D, FRAGMENT_SHADER_2D)?;

        let quad_vertices = [
            Vertex2D {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            },
            Vertex2D {
                position: [1.0, 0.0],
                uv: [1.0, 0.0],
            },
            Vertex2D {
                position: [1.0, 1.0],
                uv: [1.0, 1.0],
            },
            Vertex2D {
                position: [0.0, 1.0],
                uv: [0.0, 1.0],
            },
        ];

        let quad_indices: [u32; 6] = [0, 1, 2, 2, 3, 0];

        let quad_vao = VertexArray::new(gl)?;
        let quad_vbo = VertexBuffer::new(gl, &quad_vertices)?;
        let quad_ebo = IndexBuffer::new(gl, &quad_indices)?;

        quad_vao.bind(gl);
        quad_vbo.bind(gl);
        quad_ebo.bind(gl);

        unsafe {
            let stride = std::mem::size_of::<Vertex2D>() as i32;

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);

            gl.enable_vertex_attrib_array(1);
            let uv_offset = 2 * std::mem::size_of::<f32>() as i32;
            gl.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, uv_offset);
        }

        quad_vao.unbind(gl);
        quad_vbo.unbind(gl);
        quad_ebo.unbind(gl);

        Ok(Self {
            shader,
            quad_vao,
            quad_vbo,
            quad_ebo,
            projection: Mat4::IDENTITY,
        })
    }

    /// Sets up 2D orthographic projection (top-left origin, Y-down) and alpha blending state.
    pub fn begin(&mut self, gl: &glow::Context, screen_width: f32, screen_height: f32) {
        unsafe {
            gl.disable(glow::DEPTH_TEST);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        // Top-left origin: X right [0, width], Y down [0, height]
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
        self.shader.set_int(gl, "u_texture", 0);
    }

    /// Submits a textured or solid-color transformed quad draw call.
    pub fn draw(
        &self,
        gl: &glow::Context,
        model: &Mat4,
        color: Color,
        texture: Option<&Texture2D>,
    ) {
        self.shader.set_mat4(gl, "u_model", model);
        self.shader.set_vec4(gl, "u_color", color.to_vec4());

        if let Some(tex) = texture {
            tex.bind(gl, 0);
            self.shader.set_int(gl, "u_use_texture", 1);
        } else {
            self.shader.set_int(gl, "u_use_texture", 0);
        }

        self.quad_vao.bind(gl);
        unsafe {
            gl.draw_elements(
                glow::TRIANGLES,
                self.quad_ebo.count() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
        self.quad_vao.unbind(gl);
    }

    /// Restores OpenGL blend and depth testing states after 2D pass completion.
    pub fn end(&self, gl: &glow::Context) {
        self.shader.unbind(gl);
        unsafe {
            gl.disable(glow::BLEND);
            gl.enable(glow::DEPTH_TEST);
        }
    }

    /// Deallocates pipeline shaders and unit-quad GPU geometry buffers.
    pub fn destroy(&self, gl: &glow::Context) {
        self.quad_ebo.destroy(gl);
        self.quad_vbo.destroy(gl);
        self.quad_vao.destroy(gl);
        self.shader.destroy(gl);
    }
}
