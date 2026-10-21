# Module `core` — Runtime Lifecycle & Engine Core

The `core` module oversees application lifecycle management, OS event loops, graphics execution context, frame timing, and input polling.

Source files: [`src/core/`](file:///e:/PRJCT/Oxide/src/core/)

---

## 1. Application Configuration & Runner (`App`, `AppBuilder`)

### Overview
`App` exposes a fluent `AppBuilder` to configure window parameters and launch an event loop driven by an `AppHandler` implementation.

```rust
use oxide::prelude::*;

struct SimulationWorkspace;

impl AppHandler for SimulationWorkspace {
    fn on_init(&mut self, ctx: &mut Context) {
        // Runs once upon startup
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        // Simulation step (dt in seconds)
    }

    fn on_render(&mut self, ctx: &mut Context) {
        // Frame draw pass
        ctx.clear(Color::rgb(0.08, 0.09, 0.12));
    }
}

fn main() -> Result<(), OxideError> {
    App::build()
        .with_title("Oxide Workspace")
        .with_size(1280, 720)
        .run(SimulationWorkspace)
}
```

### `AppBuilder` Methods
- `new() -> Self`: Constructs default configuration (1280x720, "Oxide Runtime").
- `with_title(title: impl Into<String>) -> Self`: Sets the window title.
- `with_size(width: u32, height: u32) -> Self`: Sets the logical resolution in pixels.
- `run<H: AppHandler + 'static>(self, handler: H) -> Result<(), OxideError>`: Initializes the window and OpenGL context, entering the event loop.

---

## 2. Lifecycle Trait (`AppHandler`)

The callback interface for simulation and rendering logic:

```rust
pub trait AppHandler {
    fn on_init(&mut self, _ctx: &mut Context) {}
    fn on_update(&mut self, _ctx: &mut Context, _dt: f32) {}
    fn on_render(&mut self, _ctx: &mut Context) {}
}
```

- **`on_init`**: Executed immediately after OpenGL context creation. Cameras, geometry, and textures are initialized here.
- **`on_update`**: Invoked before rendering each frame. Receives `dt: f32` (frame delta time).
- **`on_render`**: Invoked for drawing passes. Typically calls `ctx.clear(...)` and `scene.render(ctx)`.

---

## 3. Execution Context (`Context`)

`Context` is passed by reference to all `AppHandler` callbacks, exposing hardware state and utilities:

| Method | Return Type | Description |
| :--- | :--- | :--- |
| `ctx.clear(color: Color)` | `()` | Clears both color and depth buffers using the specified color |
| `ctx.aspect_ratio()` | `f32` | Returns the current screen aspect ratio (`width / height`) |
| `ctx.set_viewport(x, y, w, h)` | `()` | Configures the OpenGL viewport rectangle |
| `ctx.delta_time()` | `f32` | Time elapsed since the previous frame in seconds |
| `ctx.elapsed()` | `f32` | Total runtime elapsed since application start |
| `ctx.fps()` | `f32` | Smoothed frames-per-second metric |

---

## 4. Time Tracking (`Time`)

The `Time` system leverages `std::time::Instant` for high-resolution timing:
- Spiral-of-death mitigation: `delta_time` is clamped to a maximum of `0.1s` during window drag/lag.
- Accumulator for accurate 1-second rolling FPS calculation.
- Total frame count via `frame_count()`.

---

## 5. Input System (`InputState`)

Available via `ctx.input`. Supports key polling and mouse tracking:

```rust
// Keyboard polling
if ctx.input.is_key_down(KeyCode::Space) {
    // Key is currently held
}
if ctx.input.is_key_just_pressed(KeyCode::KeyR) {
    // Key was pressed on this exact frame
}

// Mouse polling
let mouse_pos = ctx.input.mouse_position(); // glam::Vec2 in screen pixels
let mouse_delta = ctx.input.mouse_delta();   // Frame cursor delta
if ctx.input.is_mouse_button_down(MouseButton::Left) {
    // Left mouse button held
}
```

---

## 6. Color Representation (`Color`)

`Color` stores red, green, blue, and alpha in normalized `[0.0, 1.0]` floats:

```rust
let col = Color::rgba(0.2, 0.6, 1.0, 0.85);
let white = Color::WHITE;
let transparent = Color::TRANSPARENT;

let array: [f32; 4] = col.to_array();
let vec4: glam::Vec4 = col.to_vec4();
```
