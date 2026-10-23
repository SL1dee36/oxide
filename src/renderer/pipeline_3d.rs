use crate::core::color::Color;
use crate::core::error::OxideError;
use crate::renderer::buffer::{IndexBuffer, Vertex3D, VertexArray};
use crate::renderer::shader::ShaderProgram;
use crate::renderer::texture::Texture2D;
use glam::{Mat4, Vec3};
use glow::HasContext;

const VERTEX_SHADER_3D: &str = r#"#version 450 core
layout(location = 0) in vec3 a_position;
layout(location = 1) in vec3 a_normal;
layout(location = 2) in vec2 a_uv;

uniform mat4 u_model;
uniform mat3 u_normal_matrix;
uniform mat4 u_view;
uniform mat4 u_projection;

out vec3 v_normal;
out vec2 v_uv;
out vec3 v_frag_pos;

void main() {
    vec4 world_pos = u_model * vec4(a_position, 1.0);
    v_frag_pos = world_pos.xyz;
    v_normal = normalize(u_normal_matrix * a_normal);
    v_uv = a_uv;
    gl_Position = u_projection * u_view * world_pos;
}
"#;

const FRAGMENT_SHADER_3D: &str = r#"#version 450 core
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

/// Directional and ambient lighting parameters for 3D rendering.
#[derive(Clone, Copy, Debug)]
pub struct Light3D {
    /// Unit vector pointing toward the directional light source.
    pub direction: Vec3,
    /// Direct illumination color / intensity.
    pub color: Vec3,
    /// Ambient background illumination floor.
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

/// 3D polygon graphics pipeline supporting perspective projection, Lambertian lighting, and depth testing.
pub struct Pipeline3D {
    shader: ShaderProgram,
    /// Active lighting configuration.
    pub light: Light3D,
}

impl Pipeline3D {
    /// Compiles 3D vertex and fragment shaders and initializes default lighting.
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        let shader = ShaderProgram::new(gl, VERTEX_SHADER_3D, FRAGMENT_SHADER_3D)?;
        Ok(Self {
            shader,
            light: Light3D::default(),
        })
    }

    /// Configures vertex attribute pointers (0: position, 1: normal, 2: uv) for [`Vertex3D`].
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

    /// Binds shader program, activates depth test with `GL_LEQUAL`, and sets view/projection/light uniforms.
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

    /// Submits indexed mesh geometry for rendering with specified transform, albedo, and optional texture.
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
        let normal_matrix = glam::Mat3::from_mat4(*model).inverse().transpose();
        self.shader.set_mat4(gl, "u_model", model);
        self.shader.set_mat3(gl, "u_normal_matrix", &normal_matrix);
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

    /// Unbinds active 3D shader program.
    pub fn end(&self, gl: &glow::Context) {
        self.shader.unbind(gl);
    }

    /// Deallocates the 3D shader program from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        self.shader.destroy(gl);
    }
}
