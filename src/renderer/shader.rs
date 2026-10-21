use crate::core::error::OxideError;
use glow::HasContext;

pub struct ShaderProgram {
    id: glow::Program,
}

impl ShaderProgram {
    pub fn new(
        gl: &glow::Context,
        vertex_source: &str,
        fragment_source: &str,
    ) -> Result<Self, OxideError> {
        unsafe {
            let program = gl
                .create_program()
                .map_err(|e| OxideError::ShaderError(e.to_string()))?;

            let vertex_shader = Self::compile_shader(gl, glow::VERTEX_SHADER, vertex_source)?;
            let fragment_shader =
                Self::compile_shader(gl, glow::FRAGMENT_SHADER, fragment_source)?;

            gl.attach_shader(program, vertex_shader);
            gl.attach_shader(program, fragment_shader);
            gl.link_program(program);

            if !gl.get_program_link_status(program) {
                let log = gl.get_program_info_log(program);
                gl.delete_shader(vertex_shader);
                gl.delete_shader(fragment_shader);
                gl.delete_program(program);
                return Err(OxideError::ShaderError(format!(
                    "Program linking failed: {}",
                    log
                )));
            }

            gl.detach_shader(program, vertex_shader);
            gl.detach_shader(program, fragment_shader);
            gl.delete_shader(vertex_shader);
            gl.delete_shader(fragment_shader);

            Ok(Self { id: program })
        }
    }

    unsafe fn compile_shader(
        gl: &glow::Context,
        shader_type: u32,
        source: &str,
    ) -> Result<glow::Shader, OxideError> {
        let shader = gl
            .create_shader(shader_type)
            .map_err(|e| OxideError::ShaderError(e.to_string()))?;

        gl.shader_source(shader, source);
        gl.compile_shader(shader);

        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader);
            let stage = if shader_type == glow::VERTEX_SHADER {
                "Vertex"
            } else {
                "Fragment"
            };
            return Err(OxideError::ShaderError(format!(
                "{} shader compilation failed: {}",
                stage, log
            )));
        }

        Ok(shader)
    }

    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(Some(self.id));
        }
    }

    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.use_program(None);
        }
    }

    pub fn set_mat4(&self, gl: &glow::Context, name: &str, matrix: &glam::Mat4) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, name) {
                gl.uniform_matrix_4_f32_slice(Some(&loc), false, &matrix.to_cols_array());
            }
        }
    }

    pub fn set_vec3(&self, gl: &glow::Context, name: &str, vec: glam::Vec3) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, name) {
                gl.uniform_3_f32(Some(&loc), vec.x, vec.y, vec.z);
            }
        }
    }

    pub fn set_vec4(&self, gl: &glow::Context, name: &str, vec: glam::Vec4) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, name) {
                gl.uniform_4_f32(Some(&loc), vec.x, vec.y, vec.z, vec.w);
            }
        }
    }

    pub fn set_float(&self, gl: &glow::Context, name: &str, value: f32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, name) {
                gl.uniform_1_f32(Some(&loc), value);
            }
        }
    }

    pub fn set_int(&self, gl: &glow::Context, name: &str, value: i32) {
        unsafe {
            if let Some(loc) = gl.get_uniform_location(self.id, name) {
                gl.uniform_1_i32(Some(&loc), value);
            }
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.id);
        }
    }
}
