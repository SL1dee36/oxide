# Subsystem `assets` — Host RAM Geometry and Resource Loaders

The `assets` module manages polygonal geometry resident in host memory (`Mesh3D`), planar 2D surface parameters (`Surface2D`), procedural primitive generation, and deserialization of Wavefront OBJ models and raster images.

Source code: [`src/assets/`](../../src/assets/)

---

## 1. Polygonal Mesh Representation (`Mesh3D`)

`Mesh3D` stores vertex and index arrays in host system RAM. GPU VRAM buffers (`vao`, `vbo`, `ebo`) are lazily allocated:

```rust
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub color: Color,
    pub emissive: Color,
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}
```

### Procedural Primitive Generators:
- `Mesh3D::cube(size: f32) -> Self` — Generates a rectangular cuboid with 24 vertices and 36 indices, with independent normals per face and normalized UVs in `[0.0, 1.0]`.
- `Mesh3D::plane(width: f32, depth: f32) -> Self` — Generates a horizontal quad in the XZ plane (normal `[0.0, 1.0, 0.0]`) with 4 vertices and 6 indices.
- `Mesh3D::sphere(radius: f32, rings: u32, sectors: u32) -> Self` — Generates a parametric UV sphere with configurable latitude rings and longitude sectors.

### GPU Buffer Allocation:
```rust
pub fn ensure_gpu_buffers(&mut self, gl: &glow::Context) -> Result<(), OxideError>;
```
Verifies whether `self.vao` is present. If unallocated, it creates `VertexArray`, `VertexBuffer`, and `IndexBuffer`, binds vertex attributes via `Pipeline3D::setup_vertex_attributes`, and stores the buffer handles.

---

## 2. Planar 2D Surfaces (`Surface2D`)

`Surface2D` specifies pixel dimensions and fill color for a planar screen quad without allocating dedicated meshes, since `Pipeline2D` reuses a persistent unit quad:

```rust
pub struct Surface2D {
    pub size: glam::Vec2,
    pub color: Color,
}
```

Constructors:
- `Surface2D::new(size: Vec2) -> Self`
- `Surface2D::from_color(size: Vec2, color: Color) -> Self`

---

## 3. Resource Loading Functions (`loader.rs`)

Resource loading functions deserialize assets from files on disk or in-memory byte slices (suitable for `include_bytes!`):

```rust
// Parses Wavefront OBJ geometry from memory
pub fn load_obj_from_memory(bytes: &[u8]) -> Result<Mesh3D, OxideError>;

// Reads and parses Wavefront OBJ geometry from disk
pub fn load_obj_file(path: impl AsRef<Path>) -> Result<Mesh3D, OxideError>;

// Decodes image data (PNG/JPEG) and uploads it into a GPU 2D texture from memory
pub fn load_texture_from_memory(gl: &glow::Context, bytes: &[u8]) -> Result<Texture2D, OxideError>;

// Reads image file from disk and uploads it into a GPU 2D texture
pub fn load_texture_file(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Texture2D, OxideError>;
```

---

## 4. Pipeline Submission Stages

### Submitting a 3D Mesh to `Pipeline3D`:
1. Load model geometry into host RAM:
   ```rust
   let mut mesh = load_obj_file("assets/models/component.obj")?;
   ```
2. Allocate GPU buffers with the active OpenGL context:
   ```rust
   mesh.ensure_gpu_buffers(ctx.gl)?;
   ```
3. Issue pipeline draw call:
   ```rust
   if let (Some(vao), Some(ebo)) = (&mesh.vao, &mesh.ebo) {
       pipeline_3d.draw(ctx.gl, &model_matrix, mesh.color, mesh.emissive, None, vao, ebo);
   }
   ```

### Submitting a 2D Surface to `Pipeline2D`:
1. Define surface dimensions and color:
   ```rust
   let surface = Surface2D::from_color(Vec2::new(300.0, 80.0), Color::rgba(0.1, 0.15, 0.25, 0.9));
   ```
2. Build 2D transformation matrix:
   ```rust
   let model_2d = Mat4::from_scale_rotation_translation(
       Vec3::new(surface.size.x, surface.size.y, 1.0),
       Quat::IDENTITY,
       Vec3::new(20.0, 20.0, 0.0),
   );
   ```
3. Issue pipeline draw call:
   ```rust
   pipeline_2d.draw(ctx.gl, &model_2d, surface.color, None);
   ```
