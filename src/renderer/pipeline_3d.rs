use crate::core::color::Color;
use crate::core::error::OxideError;
use crate::renderer::buffer::{IndexBuffer, Vertex3D, VertexArray};
use crate::renderer::shader::ShaderProgram;
use crate::renderer::texture::Texture2D;
use glam::{Mat4, Vec3};
use glow::HasContext;

const VERTEX_SHADER_3D: &str = r#"#version 330 core
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;

uniform mat4 u_model;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_normal;
out vec2 v_uv;
out vec3 v_frag_pos;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_frag_pos = world_pos.xyz;
    mat3 normal_matrix = mat3(transpose(inverse(u_model)));
    v_normal = normalize(normal_matrix * a_normal);
    v_uv = a_uv;
    gl_Position = u_projection * u_view * world_pos;
}
"#;

const FRAGMENT_SHADER_3D: &str = r#"#version 330 core
in vec3 v_normal;
in vec2 v_uv;
in vec3 v_frag_pos;

uniform vec3 u_light_dir;
uniform vec3 u_light_color;
uniform vec3 u_ambient_color;
uniform vec4 u_albedo_color;
uniform vec3 u_emissive_color;
uniform sampler2D u_texture;
uniform int u_use_texture;

out vec4 o_color;

void main() {
    vec3 N = normalize(v_normal);
    vec3 L = normalize(u_light_dir);
    float diff = max(dot(N, L), 0.0);
    vec3 diffuse = diff * u_light_color;
    vec3 lighting = u_ambient_color + diffuse;

    vec4 tex_color = vec4(1.0);
    if (u_use_texture == 1) {
        tex_color = texture(u_texture, v_uv);
    }

    vec4 final_color = u_albedo_color * tex_color;
    vec3 lit = final_color.rgb * lighting + u_emissive_color;
    o_color = vec4(lit, final_color.a);
}
"#;

#[derive(Clone, Copy, Debug)]
pub struct Light3D {
    pub direction: Vec3,
    pub color: Vec3,
    pub ambient: Vec3,
}

impl Default for Light3D {
    fn default() -> Self {
        Self {
            direction: Vec3::new(0.5, 1.0, 0.7).normalize(),
            color: Vec3::new(1.0, 0.98, 0.95),
            ambient: Vec3::new(0.25, 0.25, 0.28),
        }
    }
}

pub struct Pipeline3D {
    shader: ShaderProgram,
    pub light: Light3D,
}

impl Pipeline3D {
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        let shader = ShaderProgram::new(gl, VERTEX_SHADER_3D, FRAGMENT_SHADER_3D)?;
        Ok(Self {
            shader,
            light: Light3D::default(),
        })
    }

    pub fn setup_vertex_attributes(gl: &glow::Context) {
        unsafe {
            let stride = std::mem::size_of::<Vertex3D>() as i32;

            gl.enable_vertex_attrib_array(0);
            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, stride, 0);

            gl.enable_vertex_attrib_array(1);
            let normal_offset = 3 * std::mem::size_of::<f32>() as i32;
            gl.vertex_attrib_pointer_f32(1, 3, glow::FLOAT, false, stride, normal_offset);

            gl.enable_vertex_attrib_array(2);
            let uv_offset = 6 * std::mem::size_of::<f32>() as i32;
            gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, stride, uv_offset);
        }
    }

    pub fn begin(
        &self,
        gl: &glow::Context,
        view: &Mat4,
        projection: &Mat4,
    ) {
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LEQUAL);
        }
        self.shader.bind(gl);
        self.shader.set_mat4(gl, "u_view", view);
        self.shader.set_mat4(gl, "u_projection", projection);
        self.shader.set_vec3(gl, "u_light_dir", self.light.direction);
        self.shader.set_vec3(gl, "u_light_color", self.light.color);
        self.shader.set_vec3(gl, "u_ambient_color", self.light.ambient);
        self.shader.set_int(gl, "u_texture", 0);
    }

    pub fn draw(
        &self,
        gl: &glow::Context,
        model: &Mat4,
        albedo: Color,
        emissive: Color,
        texture: Option<&Texture2D>,
        vao: &VertexArray,
        ebo: &IndexBuffer,
    ) {
        self.shader.set_mat4(gl, "u_model", model);
        self.shader.set_vec4(gl, "u_albedo_color", albedo.to_vec4());
        self.shader.set_vec3(gl, "u_emissive_color", Vec3::new(emissive.r, emissive.g, emissive.b));

        if let Some(tex) = texture {
            tex.bind(gl, 0);
            self.shader.set_int(gl, "u_use_texture", 1);
        } else {
            self.shader.set_int(gl, "u_use_texture", 0);
        }

        vao.bind(gl);
        ebo.bind(gl);
        unsafe {
            gl.draw_elements(
                glow::TRIANGLES,
                ebo.count() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
        ebo.unbind(gl);
        vao.unbind(gl);
    }

    pub fn end(&self, gl: &glow::Context) {
        self.shader.unbind(gl);
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.shader.destroy(gl);
    }
}
