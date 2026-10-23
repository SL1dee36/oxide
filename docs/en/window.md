# Subsystem `window` — Window Subsystem and Graphics Context

The `window` module handles operating system window creation, display framebuffer configuration, hardware OpenGL 4.5 Core Profile context initialization, and double-buffer presentation management.

Source code: [`src/window/`](../../src/window/)

---

## 1. Window Integration Stack

The windowing and graphics driver connection is established using the following low-level crates:
- **`winit 0.30`**: OS-native window creation (Windows, Linux, macOS) and window manager event polling (resizing, focus, redraw requests, keyboard/mouse input).
- **`glutin 0.32` & `glutin-winit 0.5`**: Cross-platform OpenGL graphics context creation, framebuffer format negotiation (RGBA8, 24-bit depth, 8-bit stencil), and vertical synchronization configuration.
- **`raw-window-handle 0.6`**: Low-level platform window handle interoperability (`HWND`, `Xlib Window`, `NSWindow`) bridging `glutin` and `winit`.
- **`glow 0.16`**: Dynamic function pointer loading (`gl_display.get_proc_address`) exposing a type-safe Rust OpenGL interface.

---

## 2. Structure `WindowContext`

`WindowContext` encapsulates ownership of OS-level display and rendering resources:

```rust
pub struct WindowContext {
    pub window: Window,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    pub glow_gl: glow::Context,
}
```

### Field Definitions:
- `window: winit::window::Window` — Operating system window handle.
- `gl_context: glutin::context::PossiblyCurrentContext` — Active, current OpenGL context.
- `gl_surface: glutin::surface::Surface<WindowSurface>` — Render surface attached to the window framebuffer.
- `glow_gl: glow::Context` — Table of OpenGL function pointers loaded from the graphics driver.

---

## 3. Initialization Sequence

Calling `WindowContext::new(title, width, height, event_loop)` executes the following deterministic sequence:
1. **Window Configuration:** Configures `WindowAttributes` with the requested title and logical pixel resolution (`winit::dpi::LogicalSize`).
2. **Display Configuration Negotiation:** `ConfigTemplateBuilder` requests an 8-bit alpha channel and hardware acceleration (`prefer_hardware_accelerated(Some(true))`). A reduction filter selects the framebuffer configuration with maximum multisample anti-aliasing (MSAA) samples.
3. **OpenGL Profile Specification:** Constructs `ContextAttributesBuilder` requiring **OpenGL 4.5 Core Profile** (`ContextApi::OpenGl(Some(Version::new(4, 5)))`).
4. **Surface Allocation and Activation:** The window surface is created via `gl_config.create_window_surface`, after which the inactive context is made current (`make_current(&gl_surface)`).
5. **VSync Synchronization:** Enables vertical synchronization via `SwapInterval::Wait(NonZeroU32::new(1))` to prevent screen tearing.
6. **Driver Function Pointer Loading:** `glow::Context::from_loader_function` loads OpenGL function pointers using C strings passed to `gl_display.get_proc_address`.

---

## 4. Control Methods

### Surface Resizing (`resize`)
```rust
pub fn resize(&self, width: u32, height: u32);
```
Invoked upon receiving a `WindowEvent::Resized` event. If width and height are greater than zero, resizes the underlying `gl_surface` buffers to match new dimensions.

### Framebuffer Swap (`swap_buffers`)
```rust
pub fn swap_buffers(&self) -> Result<(), OxideError>;
```
Swaps the front and back framebuffers. On failure, returns `OxideError::ContextError`.
