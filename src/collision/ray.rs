use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
}

impl Ray3D {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}
