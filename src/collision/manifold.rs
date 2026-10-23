use glam::Vec3;

/// Contact manifold describing geometric penetration between two overlapping colliders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactManifold {
    /// Unit separation normal pointing from body A toward body B.
    pub normal: Vec3,
    /// Minimum translation distance required along `normal` to resolve overlap (> 0.0).
    pub depth: f32,
    /// Estimated contact point in world Cartesian space.
    pub point: Vec3,
}

impl ContactManifold {
    /// Constructs a contact manifold instance.
    pub fn new(normal: Vec3, depth: f32, point: Vec3) -> Self {
        Self {
            normal,
            depth,
            point,
        }
    }
}
