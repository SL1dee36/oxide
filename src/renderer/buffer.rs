use crate::core::error::OxideError;
use glow::HasContext;

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vertex3D {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex3D {
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

pub struct VertexArray {
    id: glow::VertexArray,
}

impl VertexArray {
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_vertex_array()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            Ok(Self { id })
        }
    }

    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.id));
        }
    }

    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.id);
        }
    }
}

pub struct VertexBuffer {
    id: glow::Buffer,
    count: usize,
}

impl VertexBuffer {
    pub fn new<T>(gl: &glow::Context, data: &[T]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(id));
            let slice = std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                std::mem::size_of_val(data),
            );
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: data.len(),
            })
        }
    }

    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
        }
    }

    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

pub struct IndexBuffer {
    id: glow::Buffer,
    count: usize,
}

impl IndexBuffer {
    pub fn new(gl: &glow::Context, indices: &[u32]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(id));
            let slice = std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                std::mem::size_of_val(indices),
            );
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, slice, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: indices.len(),
            })
        }
    }

    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.id));
        }
    }

    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}
