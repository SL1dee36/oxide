use glam::Vec3;

/// Half-infinite 3D ray defined by origin point and normalized direction vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray3D {
    /// Starting point of the ray in world coordinates.
    pub origin: Vec3,
    /// Unit direction vector along which the ray propagates.
    pub direction: Vec3,
}

/// Intersection record produced by raycasting against a bounding volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
    /// Parametric distance $t$ from ray origin to the intersection point along direction vector.
    pub distance: f32,
    /// World Cartesian coordinates of the hit point.
    pub point: Vec3,
    /// Outward unit surface normal vector at the hit point.
    pub normal: Vec3,
}

impl Ray3D {
    /// Constructs a ray, normalizing `direction`.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    /// Evaluates point along the ray at parametric distance $t$ ($P = \text{origin} + \text{direction} \cdot t$).
    pub fn point_at(&self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
}
