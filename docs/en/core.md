# Subsystem `core` — Application Lifecycle and Execution Context

The `core` module manages application lifecycle, the operating system event loop, per-frame execution context, timing calculation, input device polling, and typed error handling.

Source code: [`src/core/`](../../src/core/)

---

## 1. Separation of Responsibilities

| Entity | Subsystem | Domain of Responsibility |
| :--- | :--- | :--- |
| **`App` / `AppBuilder`** | `core::app` | Application configuration (title, dimensions), `winit::event_loop::EventLoop` initialization, dispatching lifecycle events to `AppHandler`. Does not issue direct graphics commands. |
| **`Context<'a>`** | `core::context` | Ephemeral per-frame execution context provided to `AppHandler` callbacks. Supplies safe access to the OpenGL driver interface (`&glow::Context`), viewport bounds, timing (`Time`), and input states (`InputState`). Exists strictly for the duration of the callback. |
| **`WindowContext`** | `window::window_context` | Owner of OS window handles (`winit::window::Window`), render surfaces (`glutin::surface::Surface`), and the hardware OpenGL context (`PossiblyCurrentContext`). Manages framebuffer swaps (`swap_buffers`) and surface resizing. |

---

## 2. Application Lifecycle (`App`, `AppBuilder`, `AppHandler`)

### `AppBuilder` Pattern
Configures parameters before entering the blocking event loop:
- `AppBuilder::new() -> Self` — Instantiates builder with default values (1280x720, "Oxide Runtime").
- `with_title(title: impl Into<String>) -> Self` — Configures OS window title.
- `with_size(width: u32, height: u32) -> Self` — Configures logical window resolution in pixels.
- `run<H: AppHandler + 'static>(self, handler: H) -> Result<(), OxideError>` — Initializes window subsystem, creates OpenGL context, and enters the blocking event loop.

### `AppHandler` Trait
Callback interface implemented by user applications:
```rust
use oxide::prelude::*;

pub struct SimulationHandler;

impl AppHandler for SimulationHandler {
    fn on_init(&mut self, ctx: &mut Context) {
        // Runs once after OpenGL context is current
        ctx.set_viewport(0, 0, ctx.width as i32, ctx.height as i32);
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        // Simulation logic updates
    }

    fn on_render(&mut self, ctx: &mut Context) {
        // Submits rendering commands
        ctx.clear(Color::rgb(0.08, 0.10, 0.14));
    }
}
```

---

## 3. Execution Context (`Context`)

`Context<'a>` aggregates frame-specific engine state:

```rust
pub struct Context<'a> {
    pub gl: &'a glow::Context,
    pub width: u32,
    pub height: u32,
    pub time: &'a Time,
    pub input: &'a InputState,
}
```

### `Context` Methods:
- `aspect_ratio(&self) -> f32` — Computes viewport aspect ratio (`width as f32 / height as f32`).
- `clear(&self, color: Color)` — Clears color (`GL_COLOR_BUFFER_BIT`) and depth (`GL_DEPTH_BUFFER_BIT`) buffers.
- `set_viewport(&self, x: i32, y: i32, width: i32, height: i32)` — Sets the active OpenGL rasterization viewport.
- `delta_time(&self) -> f32` — Returns duration of the preceding frame in seconds ($\Delta t$).
- `elapsed(&self) -> f32` — Returns total elapsed time since application startup.
- `fps(&self) -> f32` — Returns rolling average frames per second calculation over the last 1.0s window.

---

## 4. Time Calculation (`Time`)

The `Time` structure utilizes high-precision `std::time::Instant` monotonic clocks:
- **Lag Spiral Prevention:** Delta time $\Delta t$ is capped at `0.1` seconds (`duration.as_secs_f32().min(0.1)`), preventing physics jumps during window drags or stalls.
- **FPS Computation:** Rolling average calculated over accumulator intervals $\ge 1.0$ second.
- **Frame Counter:** `frame_count() -> u64` tracks total frames rendered.

---

## 5. Input Device Polling (`InputState`)

Maintains keyboard and mouse button transition sets via `winit`:

### Keyboard:
- `is_key_down(key: KeyCode) -> bool` — Key is continuously held down.
- `is_key_just_pressed(key: KeyCode) -> bool` — Key transitioned from released to pressed this frame.
- `is_key_just_released(key: KeyCode) -> bool` — Key transitioned from pressed to released this frame.

### Mouse:
- `is_mouse_button_down(button: MouseButton) -> bool` — Mouse button is continuously held down.
- `is_mouse_button_just_pressed(button: MouseButton) -> bool` — Mouse button was pressed this frame.
- `is_mouse_button_just_released(button: MouseButton) -> bool` — Mouse button was released this frame.
- `mouse_position() -> Vec2` — Current cursor coordinates in physical screen pixels.
- `mouse_delta() -> Vec2` — Cursor displacement vector since the preceding frame.
- `mouse_wheel_delta() -> Vec2` — Accumulated scroll wheel delta vector during the current frame.

---

## 6. Color Representation (`Color`)

`Color` encapsulates 4 floating-point channels in the `[0.0, 1.0]` range:
```rust
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```
Constants: `Color::WHITE`, `Color::BLACK`, `Color::RED`, `Color::GREEN`, `Color::BLUE`, `Color::TRANSPARENT`.  
Conversions: `to_array() -> [f32; 4]` and `to_vec4() -> glam::Vec4`.

---

## 7. Error Handling (`OxideError`)

Subsystem errors are strictly typed via the `OxideError` enumeration based on `thiserror`:
- `WindowCreation(String)` — OS window or event loop initialization failure.
- `ContextError(String)` — Framebuffer configuration, context activation, or buffer swap failure.
- `ShaderError(String)` — Vertex/fragment shader compilation or linking error.
- `TextureError(String)` — Image decoding or GPU texture memory allocation failure.
- `MeshError(String)` — Geometry parser failure (e.g., within `tobj`).
- `Io(std::io::Error)` — Filesystem read/write failure.
