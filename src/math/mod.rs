pub use glam::{
    vec2, vec3, vec4, Mat2, Mat3, Mat4, Quat, Vec2, Vec3, Vec4,
};

pub const EPSILON: f32 = 1e-5;

#[inline]
pub fn deg_to_rad(deg: f32) -> f32 {
    deg * (std::f32::consts::PI / 180.0)
}

#[inline]
pub fn rad_to_deg(rad: f32) -> f32 {
    rad * (180.0 / std::f32::consts::PI)
}
