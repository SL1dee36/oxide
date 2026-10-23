# Subsystem `renderer` — Graphics Pipelines and VRAM Resources

The `renderer` module implements OpenGL 4.5 Core Profile graphics pipelines, GLSL shader compilation, GPU buffer allocations, and texture resource management.

Source code: [`src/renderer/`](../../src/renderer/)

---

## 1. Segregation of Host RAM and GPU VRAM Data

Oxide enforces a strict boundary between host RAM geometry and image representations (the `assets` module) and allocated GPU VRAM objects (the `renderer` module):

```mermaid
graph LR
    subgraph RAM [Host System Memory (RAM) - assets]
        HostMesh["assets::Mesh3D<br/>(Vec<Vertex3D>, Vec<u32>)"]
        HostSurface["assets::Surface2D<br/>(Vec2 size, Color)"]
        RawImage["Decoded Pixel Buffer<br/>(Vec<u8> RGBA)"]
    end

    subgraph VRAM [GPU Video Memory (VRAM) - renderer]
        VAO["renderer::VertexArray<br/>(glGenVertexArrays)"]
        VBO["renderer::VertexBuffer<br/>(GL_ARRAY_BUFFER)"]
        EBO["renderer::IndexBuffer<br/>(GL_ELEMENT_ARRAY_BUFFER)"]
        Texture["renderer::Texture2D<br/>(GL_TEXTURE_2D)"]
    end

    HostMesh -->|"ensure_gpu_buffers(gl)"| VAO & VBO & EBO
    RawImage -->|"Texture2D::new(gl, ...)"| Texture
```

---

## 2. GPU Buffer Abstractions (`buffer.rs`)

| Type | Description | Key Methods |
| :--- | :--- | :--- |
| `Vertex3D` | Vertex memory layout `#[repr(C)]`: `position: [f32; 3]`, `normal: [f32; 3]`, `uv: [f32; 2]`. Total: 32 bytes. | `new(pos, normal, uv)` |
| `VertexArray` | Encapsulates an OpenGL VAO (`glGenVertexArrays`), tracking vertex attribute formats and buffer links. | `new(gl)`, `bind(gl)`, `unbind(gl)`, `destroy(gl)` |
| `VertexBuffer` | Encapsulates a VBO (`GL_ARRAY_BUFFER`, `GL_STATIC_DRAW`), storing a contiguous array of vertices. | `new<T>(gl, data: &[T])`, `bind(gl)`, `unbind(gl)`, `count()`, `destroy(gl)` |
| `IndexBuffer` | Encapsulates an EBO (`GL_ELEMENT_ARRAY_BUFFER`, `GL_STATIC_DRAW`), storing `u32` triangle indices. | `new(gl, indices: &[u32])`, `bind(gl)`, `unbind(gl)`, `count()`, `destroy(gl)` |

---

## 3. Shader Programs (`ShaderProgram`)

`ShaderProgram` coordinates vertex and fragment GLSL 450 core compilation and uniform variable updates:

```rust
pub struct ShaderProgram { /* id: glow::Program */ }
```

### Uniform Parameter Methods:
- `set_mat4(gl, name: &str, matrix: &Mat4)` — Uploads a 4x4 matrix (`glUniformMatrix4fv`).
- `set_vec3(gl, name: &str, vec: Vec3)` — Uploads a 3D vector (`glUniform3f`).
- `set_vec4(gl, name: &str, vec: Vec4)` — Uploads a 4D vector / color (`glUniform4f`).
- `set_float(gl, name: &str, value: f32)` — Uploads a scalar `f32`.
- `set_int(gl, name: &str, value: i32)` — Uploads an integer value (e.g. texture unit index).

---

## 4. GPU Textures (`Texture2D`)

`Texture2D` encapsulates a `GL_TEXTURE_2D` allocation with trilinear mipmap filtering:
- `Texture2D::new(gl, width, height, pixels: &[u8]) -> Result<Self, OxideError>` — Allocates texture, uploads RGBA8 pixel data, and generates mipmaps (`glGenerateMipmap`).
- `Texture2D::white(gl) -> Result<Self, OxideError>` — Creates a 1x1 solid white texture for untextured materials.
- `bind(&self, gl, slot: u32)` — Activates texture unit `GL_TEXTURE0 + slot` and binds texture.
- `destroy(&self, gl)` — Releases GPU memory (`glDeleteTextures`).

---

## 5. 3D Graphics Pipeline (`Pipeline3D`)

`Pipeline3D` renders polygonal geometry with perspective projection, depth testing, and directional lighting:

```rust
pub struct Pipeline3D {
    pub light: Light3D,
}
```

### Illumination Formulation:
- **Directional Light (`Light3D`):**
  $$\mathbf{I} = \mathbf{I}_{\text{ambient}} + \max(\mathbf{N} \cdot \mathbf{L}, 0.0) \cdot \mathbf{I}_{\text{light}}$$
  $$\mathbf{C}_{\text{final}} = \mathbf{C}_{\text{albedo}} \cdot \mathbf{I} + \mathbf{C}_{\text{emissive}}$$
- **Normal Matrix:** Evaluated in vertex shader as `mat3(transpose(inverse(u_model)))`, preserving normal orthogonality under non-uniform scaling.

### Pipeline Signatures:
```rust
// 1. Activates pipeline, passes view/projection matrices and lighting uniforms
pub fn begin(&self, gl: &glow::Context, view: &Mat4, projection: &Mat4);

// 2. Submits draw call for an indexed mesh
pub fn draw(
    &self,
    gl: &glow::Context,
    model: &Mat4,
    albedo: Color,
    emissive: Color,
    texture: Option<&Texture2D>,
    vao: &VertexArray,
    ebo: &IndexBuffer,
);

// 3. Completes 3D pass
pub fn end(&self, gl: &glow::Context);
```

---

## 6. 2D Graphics Pipeline (`Pipeline2D`)

`Pipeline2D` renders screen-space planar quads and interface elements:
- **Coordinate Model:** Screen orthographic projection with origin $(0, 0)$ at the top-left, axes: $X \in [0, \text{width}]$, $Y \in [0, \text{height}]$ pointing downwards.
- **Alpha Blending:** `gl.enable(GL_BLEND)` with function `gl.blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)`.
- **Shared Unit Quad:** Vertices `[0,0]`, `[1,0]`, `[1,1]`, `[0,1]` reside in persistent pipeline VBO/EBO buffers and are positioned/scaled via the model matrix.

```rust
pub fn begin(&mut self, gl: &glow::Context, screen_width: f32, screen_height: f32);
pub fn draw(&self, gl: &glow::Context, model: &Mat4, color: Color, texture: Option<&Texture2D>);
pub fn end(&self, gl: &glow::Context);
```
