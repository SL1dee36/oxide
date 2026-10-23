//! Vector, matrix, quaternion, and geometric mathematics primitives.
//!
//! Re-exports foundational linear algebra types from [`glam`] and provides
//! basic angular unit conversion helpers.

pub use glam::{
    vec2, vec3, vec4, Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4,
};

/// Floating-point tolerance threshold for geometrical comparisons.
pub const EPSILON: f32 = 1e-5;

/// Converts an angle from degrees to radians.
#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * (std::f32::consts::PI / 180.0)
}

/// Converts an angle from radians to degrees.
#[inline]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * (180.0 / std::f32::consts::PI)
}
