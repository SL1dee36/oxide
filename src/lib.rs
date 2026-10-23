//! Oxide engine root crate.
//!
//! A modular 2D/3D rendering, spatial hierarchy, and collision detection engine
//! for real-time graphics, transform hierarchies, and interactive applications.

#![allow(clippy::missing_safety_doc, clippy::too_many_arguments, clippy::explicit_counter_loop)]

/// Asset management, mesh definitions, and file loading.
pub mod assets;
/// Geometric collision detection, contact manifold generation, and raycasting.
pub mod collision;
/// Runtime lifecycle, application context, event loop, and time tracking.
pub mod core;
/// C-compatible foreign function interface (FFI) bindings.
pub mod ffi;
/// Mathematics re-exports and angle conversion utilities.
pub mod math;
/// Commonly used engine types and functions for unified import.
pub mod prelude;
/// OpenGL 3.3 Core Profile graphics pipelines, shaders, and GPU buffers.
pub mod renderer;
/// Hierarchical scene graph, spatial nodes, camera projections, and transforms.
pub mod spatial;
/// Font parsing, glyph atlas rasterization, and batched 2D text rendering.
pub mod text;
/// Window creation, OS event handling, and OpenGL graphics context setup.
pub mod window;
