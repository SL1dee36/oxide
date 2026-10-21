use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContactManifold {
    pub normal: Vec3,
    pub depth: f32,
    pub point: Vec3,
}

impl ContactManifold {
    pub fn new(normal: Vec3, depth: f32, point: Vec3) -> Self {
        Self {
            normal,
            depth,
            point,
        }
    }
}
