# Oxide

[![Rust](https://img.shields.io/badge/Rust-2021%20Edition-orange.svg)](https://www.rust-lang.org/)
[![OpenGL](https://img.shields.io/badge/OpenGL-4.5%20Core-blue.svg)](https://www.khronos.org/opengl/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-green.svg)](LICENSE-MIT)

Oxide is a modular 2D/3D rendering, spatial hierarchy, and collision detection engine written in Rust with an OpenGL 4.5 Core Profile graphics backend. It provides decoupled subsystems for real-time graphics, transform hierarchies, geometric collision queries, and multi-language embedding via a C99 ABI.

---

## Architecture and Subsystems

The engine is structured as a collection of decoupled subsystems exposed through the `src/` tree:

| Module | Purpose | Primary Structures & Traits |
| :--- | :--- | :--- |
| [`core`](src/core/) | Application lifecycle, frame event loop, timing, input polling, typed errors | [`App`](src/core/app.rs), [`AppBuilder`](src/core/app.rs), [`AppHandler`](src/core/app.rs), [`Context`](src/core/context.rs), [`Time`](src/core/time.rs), [`InputState`](src/core/input.rs), [`Color`](src/core/color.rs), [`OxideError`](src/core/error.rs) |
| [`ecs`](src/ecs/) | Generational ECS entity storage, component secondary maps, node behaviors, GPU cleanup | [`World`](src/ecs/world.rs), [`EntityId`](src/ecs/entity.rs), [`Component`](src/ecs/components.rs), [`Visible`](src/ecs/components.rs), [`Name`](src/ecs/components.rs), [`NodeBehavior`](src/ecs/components.rs), [`PendingGpuMeshCleanup`](src/ecs/world.rs) |
| [`window`](src/window/) | System window creation, display management, OpenGL context binding | [`WindowContext`](src/window/window_context.rs) |
| [`renderer`](src/renderer/) | OpenGL 4.5 Core pipelines, GLSL shaders, textures, GPU vertex/index buffers | [`Pipeline3D`](src/renderer/pipeline_3d.rs), [`Pipeline2D`](src/renderer/pipeline_2d.rs), [`ShaderProgram`](src/renderer/shader.rs), [`Texture2D`](src/renderer/texture.rs), [`VertexArray`](src/renderer/buffer.rs), [`VertexBuffer`](src/renderer/buffer.rs), [`IndexBuffer`](src/renderer/buffer.rs), [`Light3D`](src/renderer/pipeline_3d.rs), [`Vertex3D`](src/renderer/buffer.rs) |
| [`spatial`](src/spatial/) | Hierarchical transforms, viewing projections, camera controls | [`Scene`](src/spatial/scene.rs), [`Transform`](src/spatial/transform.rs), [`Camera`](src/spatial/camera.rs), [`Projection`](src/spatial/camera.rs), [`update_hierarchy_system`](src/ecs/systems.rs) |
| [`collision`](src/collision/) | Geometric collision detection, OBB SAT solver, Sort-and-Sweep broadphase, raycasting | [`Collider`](src/collision/collider.rs), [`Obb3D`](src/collision/collider.rs), [`Aabb3D`](src/collision/collider.rs), [`BoundingSphere`](src/collision/collider.rs), [`Aabb2D`](src/collision/collider.rs), [`BroadphaseGrid`](src/collision/broadphase.rs), [`BroadphasePair`](src/collision/broadphase.rs), [`ContactManifold`](src/collision/manifold.rs), [`Ray3D`](src/collision/ray.rs), [`RaycastHit`](src/collision/ray.rs) |
| [`physics`](src/physics/) | 6-DOF rigid body dynamics with inertia tensors, XPBD soft body cloth simulation | [`RigidBody`](src/physics/rigid_body.rs), [`RigidBodyType`](src/physics/rigid_body.rs), [`SoftBody`](src/physics/soft_body.rs), [`SoftParticle`](src/physics/soft_body.rs), [`DistanceConstraint`](src/physics/soft_body.rs), [`PhysicsConfig`](src/physics/systems.rs), [`SoftBodySystem`](src/physics/pbd_solver.rs) |
| [`assets`](src/assets/) | Resource deserialization, procedural geometry primitives, image decoding | [`Mesh3D`](src/assets/mesh.rs), [`Surface2D`](src/assets/surface.rs), `load_obj_file`, `load_obj_from_memory`, `load_texture_file`, `load_texture_from_memory` |
| [`text`](src/text/) | TrueType/OpenType font parsing, glyph atlas rasterization, single-draw-call quad batching | [`Font`](src/text/font.rs), [`FontAtlas`](src/text/atlas.rs), [`Glyph`](src/text/atlas.rs), [`Text2D`](src/text/text2d.rs), [`PipelineText`](src/text/pipeline_text.rs) |
| [`math`](src/math/) | Linear algebra primitives, coordinate transformations, angular conversions | [`Vec2`](src/math/mod.rs), [`Vec3`](src/math/mod.rs), [`Vec4`](src/math/mod.rs), [`Mat4`](src/math/mod.rs), [`Quat`](src/math/mod.rs), `deg_to_rad`, `rad_to_deg` |
| [`ffi`](src/ffi/) | C ABI symbol export, opaque pointers, panic boundaries, extern C function bindings | [`OxideApp`](src/ffi/mod.rs), [`OxideContext`](src/ffi/mod.rs), [`OxideScene`](src/ffi/mod.rs), [`OxideCallbacks`](src/ffi/mod.rs), [`OxideTransform`](src/ffi/mod.rs), [`OxideContact`](src/ffi/mod.rs), `OXIDE_INVALID_NODE_ID` |

---

## System Requirements and Dependencies

- **Rust:** Rust 1.75+ (2021 Edition).
- **Graphics API:** OpenGL 4.5 Core Profile.
- **Primary Dependencies:**
  - `winit` (0.30) — OS window creation and event processing.
  - `glutin` (0.32) + `glutin-winit` (0.5) — OpenGL context creation and surface configuration.
  - `glow` (0.16) — OpenGL function loader and direct GL call bindings.
  - `glam` (0.29) — SIMD-aligned vector, matrix, and quaternion mathematics.
  - `image` (0.25) — PNG, JPEG, and GIF decoding into raw pixel buffers.
  - `tobj` (4.0) — Wavefront `.obj` file format parsing.
  - `fontdue` (0.9) — Font rasterization and glyph metrics computation.
  - `thiserror` (2.0) — Typed error derivation.

---

## Quickstart (Rust)

Add `oxide` to your `Cargo.toml`:

```toml
[dependencies]
oxide = { path = "." }
```

Implement an [`AppHandler`](src/core/app.rs) to handle initialization, frame updates, and rendering:

```rust
use oxide::prelude::*;

struct SimulationWorkspace;

impl AppHandler for SimulationWorkspace {
    fn on_init(&mut self, ctx: &mut Context) {
        ctx.set_viewport(0, 0, ctx.width as i32, ctx.height as i32);
    }

    fn on_update(&mut self, _ctx: &mut Context, _dt: f32) {
        // Update simulation state
    }

    fn on_render(&mut self, ctx: &mut Context) {
        ctx.clear(Color::rgb(0.08, 0.10, 0.14));
    }
}

fn main() -> Result<(), OxideError> {
    App::build()
        .with_title("Oxide Simulation")
        .with_size(1280, 720)
        .run(SimulationWorkspace)
}
```

Run the minimal window example:

```bash
cargo run --example minimal_window
```

---

## C ABI and Python Bindings

Oxide compiles simultaneously as a Rust static crate (`rlib`) and a shared dynamic library (`cdylib`): `oxide.dll` (Windows), `liboxide.so` (Linux), or `liboxide.dylib` (macOS).

Build the release dynamic library:

```bash
cargo build --release
```

### 1. Native C99 Application

Header file: [`include/oxide.h`](include/oxide.h)  
Source example: [`examples/c_app/main.c`](examples/c_app/main.c)

#### Compilation via GCC/Clang (Linux):
```bash
gcc -std=c99 -Iinclude examples/c_app/main.c -Ltarget/release -loxide -Wl,-rpath,target/release -lm -o c_app
./c_app
```

#### Compilation via MSVC (Windows):
```cmd
cl /Iinclude examples\c_app\main.c /Fe:c_app.exe /link target\release\oxide.dll.lib
copy target\release\oxide.dll .
c_app.exe
```

Alternatively, build using CMake:
```bash
cd examples/c_app
cmake -B build
cmake --build build --config Release
```

### 2. Python Environment via `ctypes`

Module: [`bindings/python/oxide.py`](bindings/python/oxide.py)  
Simulation example: [`examples/python_simulation.py`](examples/python_simulation.py)

Run the Python simulation directly:

```bash
python examples/python_simulation.py
```

Code structure:

```python
from oxide import App, Scene, Vec3, Color, Transform

class Simulation:
    def __init__(self):
        self.scene = Scene()
        self.cube = None

    def on_init(self, ctx):
        self.scene.set_camera_perspective(
            fov_deg=60.0,
            aspect=ctx.aspect_ratio,
            near=0.1,
            far=100.0,
            eye=Vec3(0.0, 3.0, 8.0),
            target=Vec3(0.0, 0.0, 0.0),
        )
        self.cube = self.scene.spawn_cube_3d(1.5, Transform.from_xyz(0.0, 0.0, 0.0))

    def on_update(self, ctx, dt):
        self.scene.node_rotate_y(self.cube, 1.0 * dt)
        self.scene.update(ctx, dt)

    def on_render(self, ctx):
        ctx.clear(Color(0.08, 0.10, 0.14))
        self.scene.render(ctx)

if __name__ == "__main__":
    app = App(title="Oxide Python Runtime", width=1280, height=720)
    app.run(Simulation())
```

---

## Documentation

Full subsystem specifications and architecture references:

- 🇷🇺 **Russian Documentation:** [`docs/ru/index.md`](docs/ru/index.md)
  - [`core.md`](docs/ru/core.md) • [`ecs.md`](docs/ru/ecs.md) • [`physics.md`](docs/ru/physics.md) • [`window.md`](docs/ru/window.md) • [`renderer.md`](docs/ru/renderer.md) • [`spatial.md`](docs/ru/spatial.md) • [`collision.md`](docs/ru/collision.md) • [`assets.md`](docs/ru/assets.md) • [`text.md`](docs/ru/text.md) • [`math.md`](docs/ru/math.md) • [`ffi.md`](docs/ru/ffi.md)
- 🇬🇧 **English Documentation:** [`docs/en/index.md`](docs/en/index.md)
  - [`core.md`](docs/en/core.md) • [`ecs.md`](docs/en/ecs.md) • [`physics.md`](docs/en/physics.md) • [`window.md`](docs/en/window.md) • [`renderer.md`](docs/en/renderer.md) • [`spatial.md`](docs/en/spatial.md) • [`collision.md`](docs/en/collision.md) • [`assets.md`](docs/en/assets.md) • [`text.md`](docs/en/text.md) • [`math.md`](docs/en/math.md) • [`ffi.md`](docs/en/ffi.md)

Generate and open the Rustdoc API documentation locally:

```bash
cargo doc --no-deps --open
```

---

## Testing and Verification

Run the entire integration and unit test suite:

```bash
cargo test
```

Test targets include:
- `tests/spatial_and_collision_test.rs` — Scene graph hierarchy transform propagation, AABB/Sphere contacts, OBB SAT contact manifolds, weighted penetration resolution, and raycasting.
- `tests/physics_simulation_test.rs` — 6-DOF rigid body dynamics (gravity, linear/angular momentum, torque, inertia tensor), XPBD cloth particle simulation and distance constraints, C ABI physics lifecycle, and 200+ bodies broadphase culling performance.
- `tests/text_engine_test.rs` — TrueType font loading, glyph metrics calculation, and Latin/Cyrillic rasterization.
- `tests/ffi_lifecycle_test.rs` — C ABI heap allocations, scene manipulation, collision queries, panic boundaries (`ffi_boundary!`), and memory deallocation.
- `tests/assets_loader_test.rs` — Wavefront OBJ in-memory parsing, invalid format handling, and procedural primitive vertex/index generation.

---

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT License ([`LICENSE-MIT`](LICENSE-MIT))

at your option.
