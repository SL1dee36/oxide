//! Application lifecycle, OS event loop, context, time, input, and errors.

pub mod app;
pub mod color;
pub mod context;
pub mod error;
pub mod input;
pub mod time;

pub use app::{App, AppBuilder, AppConfig, AppHandler};
pub use color::Color;
pub use context::Context;
pub use error::OxideError;
pub use input::{InputState, KeyCode, MouseButton};
pub use time::Time;
