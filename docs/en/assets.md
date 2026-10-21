# Module `assets` — Geometry, Surfaces & Asset Loading

The `assets` module manages 3D mesh geometry (`Mesh3D`), 2D planar surfaces (`Surface2D`), procedural primitive generators, and disk file parsing.

Source files: [`src/assets/`](file:///e:/PRJCT/Oxide/src/assets/)

---

## 1. 3D Mesh Representation (`Mesh3D`)

`Mesh3D` holds CPU-side vertex and index data and lazily uploads them to GPU buffers (VAO/VBO/EBO):

```rust
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}
```

### Built-in Procedural Generators:
- `Mesh3D::cube(size: f32)`: Cube geometry with distinct face normals across all 6 sides and texture UVs.
- `Mesh3D::plane(width: f32, depth: f32)`: Planar horizontal surface with an upward-pointing normal `[0, 1, 0]`.
- `Mesh3D::sphere(radius: f32, rings: u32, sectors: u32)`: UV sphere with configurable radial tessellation.

### GPU Buffer Allocation:
The `mesh.ensure_gpu_buffers(gl)` method is invoked lazily by `Scene` upon first render, binding the VAO, VBO, and EBO.

---

## 2. Planar Surfaces (`Surface2D`)

Used for UI panels, projection planes, and HUD cards:

```rust
let panel = Surface2D::from_color(
    Vec2::new(300.0, 100.0),
    Color::rgba(0.1, 0.15, 0.25, 0.9),
);
```

---

## 3. Asset Loaders (`loader.rs`)

Supports loading assets from both the filesystem and in-memory byte buffers:

### Loading 3D Wavefront OBJ Geometry:
Uses `tobj` with automatic triangulation and normal generation:
```rust
// From disk
let mesh = load_obj_file("assets/models/teapot.obj")?;

// From memory
let mesh = load_obj_from_memory(OBJ_BYTE_DATA)?;
```

### Loading Image Textures (PNG, JPEG):
Uses the `image` crate:
```rust
// GPU texture from disk
let texture = load_texture_file(ctx.gl, "assets/textures/diffuse.png")?;

// From memory
let texture = load_texture_from_memory(ctx.gl, PNG_BYTES)?;
```
