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
#[derive(Debug)]
pub struct VertexArray {
    id: glow::VertexArray,
}

impl VertexArray {
    /// Creates a wrapper around an existing raw OpenGL VAO identifier.
    pub fn from_raw(id: glow::VertexArray) -> Self {
        Self { id }
    }

    /// Creates a dummy VAO for testing without active OpenGL context.
    pub fn dummy_for_test() -> Self {
        Self {
            id: glow::NativeVertexArray(std::num::NonZeroU32::new(1).unwrap()),
        }
    }

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
#[derive(Debug)]
pub struct VertexBuffer {
    id: glow::Buffer,
    count: usize,
    capacity: usize,
}

impl VertexBuffer {
    /// Creates a wrapper around an existing raw OpenGL buffer identifier and vertex count.
    pub fn from_raw(id: glow::Buffer, count: usize) -> Self {
        Self {
            id,
            count,
            capacity: count,
        }
    }

    /// Creates a dummy VBO for testing without active OpenGL context.
    pub fn dummy_for_test(count: usize) -> Self {
        Self {
            id: glow::NativeBuffer(std::num::NonZeroU32::new(1).unwrap()),
            count,
            capacity: count,
        }
    }

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
                capacity: data.len(),
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
                capacity: data.len(),
            })
        }
    }

    /// Allocates an empty vertex buffer with preallocated capacity and `DYNAMIC_DRAW` usage.
    pub fn with_capacity<T: Pod + Zeroable>(gl: &glow::Context, capacity: usize) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(id));
            if capacity > 0 {
                let byte_size = (capacity * std::mem::size_of::<T>()) as i32;
                gl.buffer_data_size(glow::ARRAY_BUFFER, byte_size, glow::DYNAMIC_DRAW);
            }
            gl.bind_buffer(glow::ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: 0,
                capacity,
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
            self.capacity = data.len();
        }
    }

    /// Updates an in-place sub-slice of existing buffer data using `gl.buffer_sub_data_u8_slice`.
    /// Reallocates GPU buffer storage with extra capacity only when the new data exceeds current capacity.
    pub fn update_sub_data<T: Pod + Zeroable>(&mut self, gl: &glow::Context, offset_bytes: i32, data: &[T]) {
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.id));
            let slice = bytemuck::cast_slice(data);
            let elem_size = std::mem::size_of::<T>();
            let cur_cap_bytes = self.capacity * elem_size;
            let req_bytes = (offset_bytes as usize) + slice.len();

            if req_bytes <= cur_cap_bytes {
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset_bytes, slice);
                if offset_bytes == 0 {
                    self.count = data.len();
                } else {
                    self.count = self.count.max((offset_bytes as usize / elem_size) + data.len());
                }
            } else {
                let new_capacity = (self.capacity * 2).max(data.len()).max(16);
                let new_byte_size = (new_capacity * elem_size) as i32;
                gl.buffer_data_size(glow::ARRAY_BUFFER, new_byte_size, glow::DYNAMIC_DRAW);
                gl.buffer_sub_data_u8_slice(glow::ARRAY_BUFFER, offset_bytes, slice);
                self.capacity = new_capacity;
                if offset_bytes == 0 {
                    self.count = data.len();
                } else {
                    self.count = self.count.max((offset_bytes as usize / elem_size) + data.len());
                }
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

    /// Returns allocated capacity in elements of the vertex buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Deallocates the buffer from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

/// GPU Element Array Buffer (EBO / Index Buffer) storing triangle indices.
#[derive(Debug)]
pub struct IndexBuffer {
    id: glow::Buffer,
    count: usize,
    capacity: usize,
}

impl IndexBuffer {
    /// Creates a wrapper around an existing raw OpenGL buffer identifier and index count.
    pub fn from_raw(id: glow::Buffer, count: usize) -> Self {
        Self {
            id,
            count,
            capacity: count,
        }
    }

    /// Creates a dummy EBO for testing without active OpenGL context.
    pub fn dummy_for_test(count: usize) -> Self {
        Self {
            id: glow::NativeBuffer(std::num::NonZeroU32::new(1).unwrap()),
            count,
            capacity: count,
        }
    }

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
                capacity: indices.len(),
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
                capacity: indices.len(),
            })
        }
    }

    /// Allocates an empty index buffer with preallocated capacity and `DYNAMIC_DRAW` usage.
    pub fn with_capacity(gl: &glow::Context, capacity: usize) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_buffer()
                .map_err(|e| OxideError::ContextError(e.to_string()))?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(id));
            if capacity > 0 {
                let byte_size = (capacity * std::mem::size_of::<u32>()) as i32;
                gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, byte_size, glow::DYNAMIC_DRAW);
            }
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

            Ok(Self {
                id,
                count: 0,
                capacity,
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
            self.capacity = indices.len();
        }
    }

    /// Updates an in-place sub-slice of existing index buffer data using `gl.buffer_sub_data_u8_slice`.
    /// Reallocates GPU buffer storage with extra capacity only when the new data exceeds current capacity.
    pub fn update_sub_data(&mut self, gl: &glow::Context, offset_bytes: i32, indices: &[u32]) {
        unsafe {
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.id));
            let slice = bytemuck::cast_slice(indices);
            let elem_size = std::mem::size_of::<u32>();
            let cur_cap_bytes = self.capacity * elem_size;
            let req_bytes = (offset_bytes as usize) + slice.len();

            if req_bytes <= cur_cap_bytes {
                gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, offset_bytes, slice);
                if offset_bytes == 0 {
                    self.count = indices.len();
                } else {
                    self.count = self.count.max((offset_bytes as usize / elem_size) + indices.len());
                }
            } else {
                let new_capacity = (self.capacity * 2).max(indices.len()).max(16);
                let new_byte_size = (new_capacity * elem_size) as i32;
                gl.buffer_data_size(glow::ELEMENT_ARRAY_BUFFER, new_byte_size, glow::DYNAMIC_DRAW);
                gl.buffer_sub_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, offset_bytes, slice);
                self.capacity = new_capacity;
                if offset_bytes == 0 {
                    self.count = indices.len();
                } else {
                    self.count = self.count.max((offset_bytes as usize / elem_size) + indices.len());
                }
            }
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
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

    /// Returns allocated capacity of the index buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Deallocates the index buffer from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.id);
        }
    }
}

