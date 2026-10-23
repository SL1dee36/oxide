use crate::core::color::Color;
use crate::core::input::InputState;
use crate::core::time::Time;
use glow::HasContext;

/// Per-frame execution context granting access to graphics primitives, viewport geometry, input, and timing.
pub struct Context<'a> {
    /// Underlying OpenGL 3.3 Core interface.
    pub gl: &'a glow::Context,
    /// Physical viewport width in pixels.
    pub width: u32,
    /// Physical viewport height in pixels.
    pub height: u32,
    /// Read-only reference to the engine time tracking system.
    pub time: &'a Time,
    /// Read-only reference to current frame input device states.
    pub input: &'a InputState,
}

impl<'a> Context<'a> {
    /// Instantiates a frame execution context wrapper.
    pub fn new(
        gl: &'a glow::Context,
        width: u32,
        height: u32,
        time: &'a Time,
        input: &'a InputState,
    ) -> Self {
        Self {
            gl,
            width,
            height,
            time,
            input,
        }
    }

    /// Calculates viewport aspect ratio (`width / height`).
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Clears both color and depth attachments of current framebuffer using `color`.
    #[inline]
    pub fn clear(&self, color: Color) {
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    /// Configures the active OpenGL rasterization viewport.
    #[inline]
    pub fn set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            self.gl.viewport(x, y, width, height);
        }
    }

    /// Returns time elapsed during the preceding frame in seconds ($\Delta t$).
    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.time.delta_time()
    }

    /// Returns total runtime elapsed since startup in seconds.
    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.time.elapsed()
    }

    /// Returns average frames per second calculation.
    #[inline]
    pub fn fps(&self) -> f32 {
        self.time.fps()
    }
}
