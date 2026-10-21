use crate::core::context::Context;
use crate::core::error::OxideError;
use crate::core::input::InputState;
use crate::core::time::Time;
use crate::window::window_context::WindowContext;
use glam::Vec2;
use winit::event::{ElementState, Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::PhysicalKey;

pub trait AppHandler {
    fn on_init(&mut self, _ctx: &mut Context) {}
    fn on_update(&mut self, _ctx: &mut Context, _dt: f32) {}
    fn on_render(&mut self, _ctx: &mut Context) {}
}

pub struct AppConfig {
    pub title: String,
    pub width: u32,
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

pub struct AppBuilder {
    config: AppConfig,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            config: AppConfig::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.config.title = title.into();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.config.width = width;
        self.config.height = height;
        self
    }

    pub fn run<H: AppHandler + 'static>(self, mut handler: H) -> Result<(), OxideError> {
        let event_loop = EventLoop::new().map_err(|e| OxideError::WindowCreation(e.to_string()))?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let window_ctx = WindowContext::new(
            &self.config.title,
            self.config.width,
            self.config.height,
            &event_loop,
        )?;

        let mut time = Time::new();
        let mut input = InputState::new();
        let mut width = self.config.width;
        let mut height = self.config.height;

        {
            let mut init_ctx = Context::new(
                &window_ctx.glow_gl,
                width,
                height,
                &time,
                &input,
            );
            init_ctx.set_viewport(0, 0, width as i32, height as i32);
            handler.on_init(&mut init_ctx);
        }

        event_loop
            .run(move |event, target| match event {
                Event::WindowEvent { event, window_id } if window_id == window_ctx.window.id() => {
                    match event {
                        WindowEvent::CloseRequested => {
                            target.exit();
                        }
                        WindowEvent::Resized(new_size) => {
                            width = new_size.width;
                            height = new_size.height;
                            window_ctx.resize(width, height);
                            unsafe {
                                glow::HasContext::viewport(
                                    &window_ctx.glow_gl,
                                    0,
                                    0,
                                    width as i32,
                                    height as i32,
                                );
                            }
                        }
                        WindowEvent::KeyboardInput { event: key_event, .. } => {
                            if let PhysicalKey::Code(code) = key_event.physical_key {
                                input.on_key_event(code, key_event.state == ElementState::Pressed);
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            input.on_mouse_button_event(button, state == ElementState::Pressed);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            input.on_cursor_moved(Vec2::new(position.x as f32, position.y as f32));
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            let (dx, dy) = match delta {
                                winit::event::MouseScrollDelta::LineDelta(x, y) => (x, y),
                                winit::event::MouseScrollDelta::PixelDelta(pos) => {
                                    (pos.x as f32, pos.y as f32)
                                }
                            };
                            input.on_mouse_wheel(Vec2::new(dx, dy));
                        }
                        WindowEvent::RedrawRequested => {
                            time.update();
                            let dt = time.delta_time();

                            let mut frame_ctx = Context::new(
                                &window_ctx.glow_gl,
                                width,
                                height,
                                &time,
                                &input,
                            );

                            handler.on_update(&mut frame_ctx, dt);
                            handler.on_render(&mut frame_ctx);

                            let _ = window_ctx.swap_buffers();
                            input.reset_frame_deltas();
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    window_ctx.window.request_redraw();
                }
                _ => {}
            })
            .map_err(|e| OxideError::WindowCreation(e.to_string()))?;

        Ok(())
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App;

impl App {
    pub fn build() -> AppBuilder {
        AppBuilder::new()
    }
}
