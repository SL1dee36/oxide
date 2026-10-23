use crate::core::error::OxideError;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext, Version};
use glutin::display::{GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, WindowSurface, SwapInterval};
use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle;
use std::ffi::CString;
use std::num::NonZeroU32;
use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// Encapsulates OS window handle, Glutin context, surface, and Glow OpenGL bindings.
pub struct WindowContext {
    /// Winit window reference.
    pub window: Window,
    /// Active OpenGL context.
    pub gl_context: PossiblyCurrentContext,
    /// Glutin window render surface.
    pub gl_surface: Surface<WindowSurface>,
    /// Glow OpenGL API function table loaded from graphics driver.
    pub glow_gl: glow::Context,
}

impl WindowContext {
    /// Creates a window and initializes an OpenGL 3.3 Core Profile graphics context.
    pub fn new(
        title: &str,
        width: u32,
        height: u32,
        event_loop: &ActiveEventLoop,
    ) -> Result<Self, OxideError> {
        let window_attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width as f64, height as f64));

        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .prefer_hardware_accelerated(Some(true));

        let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

        let (window, gl_config) = display_builder
            .build(event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .map_err(|e| OxideError::WindowCreation(e.to_string()))?;

        let window = window.ok_or_else(|| {
            OxideError::WindowCreation("Failed to create winit window".to_string())
        })?;

        let raw_window_handle = window
            .window_handle()
            .map_err(|e| OxideError::WindowCreation(e.to_string()))?
            .as_raw();

        let gl_display = gl_config.display();

        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(raw_window_handle));

        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .map_err(|e| OxideError::ContextError(e.to_string()))?
        };

        let attrs = window
            .build_surface_attributes(Default::default())
            .map_err(|e| OxideError::ContextError(e.to_string()))?;
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &attrs)
                .map_err(|e| OxideError::ContextError(e.to_string()))?
        };

        let gl_context = not_current_gl_context
            .make_current(&gl_surface)
            .map_err(|e| OxideError::ContextError(e.to_string()))?;

        let _ = gl_surface.set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()));

        let glow_gl = unsafe {
            glow::Context::from_loader_function(|s| {
                let c_str = CString::new(s).unwrap();
                gl_display.get_proc_address(c_str.as_c_str())
            })
        };

        Ok(Self {
            window,
            gl_context,
            gl_surface,
            glow_gl,
        })
    }

    /// Resizes the underlying OpenGL surface when the OS window dimensions change.
    pub fn resize(&self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.gl_surface.resize(
                &self.gl_context,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
        }
    }

    /// Swaps the front and back framebuffers, displaying rendered primitives on screen.
    pub fn swap_buffers(&self) -> Result<(), OxideError> {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .map_err(|e| OxideError::ContextError(e.to_string()))
    }
}
