# Module `window` — Window Management & OpenGL Context

The `window` module handles operating system window initialization, OpenGL 3.3 Core Profile configuration, and `glow` loader binding.

Source files: [`src/window/`](file:///e:/PRJCT/Oxide/src/window/)

---

## 1. Integration Stack

Cross-platform windowing and context initialization rely on:
- **`winit 0.29`**: Native window creation and OS event processing (input, resizing, window closing).
- **`glutin 0.31` + `glutin-winit 0.4`**: Hardware-accelerated OpenGL context creation, pixel format negotiation (RGBA8, 24-bit depth buffer, 8-bit stencil).
- **`raw-window-handle 0.5`**: Low-level platform handles connecting `glutin` and `winit`.
- **`glow 0.13`**: Zero-overhead safe OpenGL bindings.

---

## 2. The `WindowContext` Structure

```rust
pub struct WindowContext {
    pub window: Window,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    pub glow_gl: glow::Context,
}
```

### Fields:
- `window`: Active `winit::window::Window`.
- `gl_context`: The activated OpenGL context (`glutin::context::PossiblyCurrentContext`).
- `gl_surface`: The window framebuffer surface (`glutin::surface::Surface<WindowSurface>`).
- `glow_gl`: Loaded `glow::Context` instance.

---

## 3. Initialization Lifecycle

When calling `WindowContext::new(...)`:
1. Constructs a `WindowBuilder` with requested title and dimensions.
2. `DisplayBuilder` queries GPU drivers for hardware configurations with multisampling (MSAA) and hardware acceleration.
3. Requests an **OpenGL 3.3 Core Profile** context.
4. Generates the window surface and makes the context active (`make_current`).
5. Enforces vertical synchronization (`SwapInterval::Wait(1)`).
6. Loads all OpenGL function pointers through `glow::Context::from_loader_function`.

---

## 4. Control Methods

```rust
// Resizes the underlying framebuffer when window bounds change
pub fn resize(&self, width: u32, height: u32);

// Swaps front and back framebuffers (VSync swap)
pub fn swap_buffers(&self) -> Result<(), OxideError>;
```
