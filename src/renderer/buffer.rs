use bytemuck::{Pod, Zeroable};
use crate::core::error::OxideError;
use glow::HasContext;

/// Memory layout for 3D polygon vertices containing position, normal, and texture coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, Pod, Zeroable)]
pub struct Vertex3D {
    /// Spatial Cartesian coordinates `[x, y, z]`.
    pub position: [f32; 3],
    /// Unit surface normal vector `[nx, ny, nz]`.
    pub normal: [f32; 3],
    /// Normalized UV texture coordinates `[u, v]`.
    pub uv: [f32; 2],
}

impl Vertex3D {
    /// Constructs a vertex from component arrays.
    pub const fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2]) -> Self {
        Self {
            position,
            normal,
            uv,
        }
    }
}

/// OpenGL Vertex Array Object (VAO) managing vertex attribute bindings.
pub struct VertexArray {
    id: glow::VertexArray,
}

impl VertexArray {
    /// Allocates a new vertex array object in GPU memory.
    pub fn new(gl: &glow::Context) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_vertex_array()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            Ok(Self { id })
        }
    }

    /// Binds this vertex array object to the current OpenGL context.
    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(Some(self.id));
        }
    }

    /// Unbinds any active vertex array object.
    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_vertex_array(None);
        }
    }

    /// Deallocates the vertex array object from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_vertex_array(self.id);
        }
    }
}

/// GPU Vertex Buffer Object (VBO) storing continuous vertex attribute data.
pub struct VertexBuffer {
    id: glow::Buffer,
    count: usize,
}

impl VertexBuffer {
    /// Allocates and uploads vertex data into a GPU buffer with `STATIC_DRAW` usage.
    pub fn new<T: Pod + Zeroable>(gl: &glow::Context, data: &[T]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(id));
            let slice = bytemuck::cast_slice(data);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: data.len(),
            })
        }
    }

    /// Allocates and uploads vertex data into a GPU buffer with `DYNAMIC_DRAW` usage.
    pub fn new_dynamic<T: Pod + Zeroable>(gl: &glow::Context, data: &[T]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(id));
            let slice = bytemuck::cast_slice(data);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: data.len(),
            })
        }
    }

    /// Updates existing buffer data with `DYNAMIC_DRAW` usage.
    pub fn update<T: Pod + Zeroable>(&mut self, gl: &glow::Context, data: &[T]) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
            let slice = bytemuck::cast_slice(data);
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            self.count = data.len();
        }
    }

    /// Updates an in-place sub-slice of existing buffer data using `gl.buffer_sub_data_u8_slice`.
    pub fn update_sub_data<T: Pod + Zeroable>(&mut self, gl: &glow::Context, offset_bytes: i32, data: &[T]) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
            let slice = bytemuck::cast_slice(data);
            if data.len() <= self.count && offset_bytes == 0 {
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset_bytes, slice);
            } else {
                gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
                self.count = data.len();
            }
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    /// Binds buffer to `GL_ARRAY_BUFFER` target.
    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
        }
    }

    /// Unbinds `GL_ARRAY_BUFFER` target.
    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
        }
    }

    /// Returns element count contained within the vertex buffer.
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Deallocates the buffer from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

/// GPU Element Array Buffer (EBO / Index Buffer) storing triangle indices.
pub struct IndexBuffer {
    id: glow::Buffer,
    count: usize,
}

impl IndexBuffer {
    /// Allocates and uploads index data into a GPU buffer with `STATIC_DRAW` usage.
    pub fn new(gl: &glow::Context, indices: &[u32]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(id));
            let slice = bytemuck::cast_slice(indices);
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, slice, glow::STATIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: indices.len(),
            })
        }
    }

    /// Allocates and uploads index data into a GPU buffer with `DYNAMIC_DRAW` usage.
    pub fn new_dynamic(gl: &glow::Context, indices: &[u32]) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(id));
            let slice = bytemuck::cast_slice(indices);
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: indices.len(),
            })
        }
    }

    /// Updates existing index buffer data with `DYNAMIC_DRAW` usage.
    pub fn update(&mut self, gl: &glow::Context, indices: &[u32]) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.id));
            let slice = bytemuck::cast_slice(indices);
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, slice, glow::DYNAMIC_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
            self.count = indices.len();
        }
    }

    /// Binds buffer to `GL_ELEMENT_ARRAY_BUFFER` target.
    #[inline]
    pub fn bind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.id));
        }
    }

    /// Unbinds `GL_ELEMENT_ARRAY_BUFFER` target.
    #[inline]
    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }

    /// Returns count of indices in the buffer.
    #[inline]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Deallocates the index buffer from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

