# Module `renderer` — Graphics Backend & OpenGL Pipelines

The `renderer` module encapsulates OpenGL 3.3 Core Profile calls, GLSL shader compilation, GPU buffer allocations, and specialized rendering pipelines for 3D and 2D elements.

Source files: [`src/renderer/`](file:///e:/PRJCT/Oxide/src/renderer/)

---

## 1. 3D Render Pipeline (`Pipeline3D`)

`Pipeline3D` draws mesh geometry with directional and ambient lighting and depth testing.

### Key Characteristics:
- **Shader Model:** GLSL 330 core vertex and fragment shaders.
- **Lighting:** Blinn-Phong/Lambertian model using a directional light vector (`u_light_dir`, `u_light_color`) combined with an ambient tone (`u_ambient_color`).
- **Normal Matrix:** Dynamically evaluates `mat3(transpose(inverse(u_model)))` in vertex shader code to preserve normal fidelity under non-uniform scaling.
- **Depth Testing:** Manages `GL_DEPTH_TEST` with `GL_LEQUAL` function.

### `Light3D` Definition:
```rust
pub struct Light3D {
    pub direction: Vec3,
    pub color: Vec3,
    pub ambient: Vec3,
}
```

---

## 2. 2D Render Pipeline (`Pipeline2D`)

`Pipeline2D` renders planar panels, HUDs, and interface surfaces in screen pixel coordinates.

### Key Characteristics:
- **Orthographic Projection:** Origin `(0, 0)` is placed at the top-left corner; X spans rightward and Y extends downward.
- **Alpha Blending:** Enables `GL_BLEND` with `gl.blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)`.
- **Shared Unit Quad:** Vertices `[0,0]`, `[1,0]`, `[1,1]`, `[0,1]` are cached in a single GPU VAO and scaled via the 2D model transform.

---

## 3. Shader Program (`ShaderProgram`)

`ShaderProgram` coordinates compilation stages and program linking with descriptive error diagnostics:

```rust
let shader = ShaderProgram::new(gl, vertex_glsl, fragment_glsl)?;

shader.bind(gl);
shader.set_mat4(gl, "u_model", &matrix);
shader.set_vec4(gl, "u_color", color.to_vec4());
shader.set_int(gl, "u_texture", 0);
shader.unbind(gl);
```

---

## 4. Hardware Buffer Objects (`buffer.rs`)

Safe RAII abstractions over raw OpenGL buffer targets:

| Type | Purpose |
| :--- | :--- |
| `Vertex3D` | Vertex struct with `position: [f32; 3]`, `normal: [f32; 3]`, and `uv: [f32; 2]` (`repr(C)`) |
| `VertexArray` | Wrapper around an OpenGL VAO (`glGenVertexArrays`) |
| `VertexBuffer` | Wrapper around an OpenGL VBO (`GL_ARRAY_BUFFER`, `GL_STATIC_DRAW`) |
| `IndexBuffer` | Wrapper around an OpenGL EBO (`GL_ELEMENT_ARRAY_BUFFER`) for indexed triangles |

---

## 5. Texture Abstraction (`Texture2D`)

The `Texture2D` type encapsulates `GL_TEXTURE_2D`:
- Trilinear mipmapped filtering (`GL_LINEAR_MIPMAP_LINEAR`).
- `Texture2D::white(gl)` provisions a 1x1 solid white texture for untextured rendering.
- `bind(gl, slot)` assigns the texture to hardware sampler units (`GL_TEXTURE0 + slot`).
