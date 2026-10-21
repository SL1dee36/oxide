use crate::core::color::Color;
use crate::core::input::InputState;
use crate::core::time::Time;
use glow::HasContext;

pub struct Context<'a> {
    pub gl: &'a glow::Context,
    pub width: u32,
    pub height: u32,
    pub time: &'a Time,
    pub input: &'a InputState,
}

impl<'a> Context<'a> {
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

    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    #[inline]
    pub fn clear(&self, color: Color) {
        unsafe {
            self.gl.clear_color(color.r, color.g, color.b, color.a);
            self.gl
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        }
    }

    #[inline]
    pub fn set_viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            self.gl.viewport(x, y, width, height);
        }
    }

    #[inline]
    pub fn delta_time(&self) -> f32 {
        self.time.delta_time()
    }

    #[inline]
    pub fn elapsed(&self) -> f32 {
        self.time.elapsed()
    }

    #[inline]
    pub fn fps(&self) -> f32 {
        self.time.fps()
    }
}
