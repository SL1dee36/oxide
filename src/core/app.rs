use crate::core::context::Context;
use crate::core::error::OxideError;
use crate::core::input::InputState;
use crate::core::time::Time;
use crate::window::window_context::WindowContext;
use glam::Vec2;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;
use winit::window::WindowId;

/// Application lifecycle callback interface implemented by user simulation and runtime handlers.
pub trait AppHandler {
    /// Invoked once after OpenGL context initialization prior to entering the event loop.
    fn on_init(&mut self, _ctx: &mut Context) {}
    /// Invoked every frame before rendering; receives frame duration `_dt` in seconds.
    fn on_update(&mut self, _ctx: &mut Context, _dt: f32) {}
    /// Invoked every frame to submit draw commands to graphics pipelines.
    fn on_render(&mut self, _ctx: &mut Context) {}
}

/// Window and display configuration parameters.
pub struct AppConfig {
    /// Text displayed in the OS window title bar.
    pub title: String,
    /// Initial logical window width in pixels.
    pub width: u32,
    /// Initial logical window height in pixels.
    pub height: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: "Oxide Runtime".to_string(),
            width: 1280,
            height: 720,
        }
    }
}

struct AppRunner<H: AppHandler> {
    config: AppConfig,
    handler: H,
    window_ctx: Option<WindowContext>,
    time: Time,
    input: InputState,
    width: u32,
    height: u32,
    exit_error: Option<OxideError>,
}

impl<H: AppHandler> ApplicationHandler for AppRunner<H> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window_ctx.is_none() {
            match WindowContext::new(
                &self.config.title,
                self.config.width,
                self.config.height,
                event_loop,
            ) {
                Ok(window_ctx) => {
                    let mut init_ctx = Context::new(
                        &window_ctx.glow_gl,
                        &window_ctx.window,
                        self.width,
                        self.height,
                        &self.time,
                        &self.input,
                    );
                    init_ctx.set_viewport(0, 0, self.width as i32, self.height as i32);
                    self.handler.on_init(&mut init_ctx);
                    self.window_ctx = Some(window_ctx);
                }
                Err(err) => {
                    self.exit_error = Some(err);
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let window_ctx = match self.window_ctx.as_mut() {
            Some(ctx) => ctx,
            None => return,
        };

        if window_id != window_ctx.window.id() {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(new_size) => {
                self.width = new_size.width;
                self.height = new_size.height;
                window_ctx.resize(self.width, self.height);
                unsafe {
                    glow::HasContext::viewport(
                        &window_ctx.glow_gl,
                        0,
                        0,
                        self.width as i32,
                        self.height as i32,
                    );
                }
            }
            WindowEvent::KeyboardInput { event: key_event, .. } => {
                if let PhysicalKey::Code(code) = key_event.physical_key {
                    self.input.on_key_event(code, key_event.state == ElementState::Pressed);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.on_mouse_button_event(button, state == ElementState::Pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.on_cursor_moved(Vec2::new(position.x as f32, position.y as f32));
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                    winit::event::MouseScrollDelta::PixelDelta(pos) => {
                        (pos.x as f32, pos.y as f32)
                    }
                };
                self.input.on_mouse_wheel(Vec2::new(dx, dy));
            }
            WindowEvent::RedrawRequested => {
                self.time.update();
                let dt = self.time.delta_time();

                let mut frame_ctx = Context::new(
                    &window_ctx.glow_gl,
                    &window_ctx.window,
                    self.width,
                    self.height,
                    &self.time,
                    &self.input,
                );

                self.handler.on_update(&mut frame_ctx, dt);
                self.handler.on_render(&mut frame_ctx);

                let _ = window_ctx.swap_buffers();
                self.input.reset_frame_deltas();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window_ctx) = &self.window_ctx {
            window_ctx.window.request_redraw();
        }
    }
}

/// Builder pattern for configuring and launching an engine runtime application instance.
pub struct AppBuilder {
    config: AppConfig,
}

impl AppBuilder {
    /// Creates a builder instance initialized with default window settings (1280x720, "Oxide Runtime").
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    /// Sets the window title string.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    /// Sets the initial window dimensions in pixels.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    /// Initializes OS window, OpenGL 4.5 Core context, and executes the blocking event/frame loop.
    pub fn run<H: AppHandler + 'static>(self, handler: H) -> Result<(), OxideError> {
        let event_loop = EventLoop::new().map_err(|e| OxideError::WindowCreation(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let width = self.config.width;
        let height = self.config.height;

        let mut runner = AppRunner {
            config: self.config,
            handler,
            window_ctx: None,
            time: Time::new(),
            input: InputState::new(),
            width,
            height,
            exit_error: None,
        };

        event_loop
            .run_app(&mut runner)
            .map_err(|e| OxideError::WindowCreation(e.to_string()))?;

        if let Some(err) = runner.exit_error {
            return Err(err);
        }

        Ok(())
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Static factory providing entry points for configuring applications.
pub struct App;

impl App {
    /// Starts construction of an [`AppBuilder`] instance.
    pub fn build() -> AppBuilder {
        AppBuilder::new()
    }
}
