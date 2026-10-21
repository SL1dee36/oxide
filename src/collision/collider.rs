use crate::collision::manifold::ContactManifold;
use crate::collision::ray::{Ray3D, RaycastHit};
use crate::spatial::transform::Transform;
use glam::{Mat4, Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider {
    Box3D { half_extents: Vec3 },
    Sphere { radius: f32 },
    Box2D { half_extents: Vec2 },
}

impl Collider {
    #[inline]
    pub fn box_3d(size: Vec3) -> Self {
        Self::Box3D {
            half_extents: size * 0.5,
        }
    }

    #[inline]
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    #[inline]
    pub fn box_2d(size: Vec2) -> Self {
        Self::Box2D {
            half_extents: size * 0.5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3D {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb3D {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    pub fn transform(&self, matrix: &Mat4) -> Self {
        let center = self.center();
        let half = self.half_extents();

        let new_center = matrix.transform_point3(center);

        let cols = matrix.to_cols_array_2d();
        let new_half = Vec3::new(
            cols[0][0].abs() * half.x + cols[1][0].abs() * half.y + cols[2][0].abs() * half.z,
            cols[0][1].abs() * half.x + cols[1][1].abs() * half.y + cols[2][1].abs() * half.z,
            cols[0][2].abs() * half.x + cols[1][2].abs() * half.y + cols[2][2].abs() * half.z,
        );

        Self::from_center_half_extents(new_center, new_half)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn contact(&self, other: &Self) -> Option<ContactManifold> {
        let d1 = other.max - self.min;
        let d2 = self.max - other.min;

        if d1.x < 0.0 || d2.x < 0.0 || d1.y < 0.0 || d2.y < 0.0 || d1.z < 0.0 || d2.z < 0.0 {
            return None;
        }

        let overlap_x = d1.x.min(d2.x);
        let overlap_y = d1.y.min(d2.y);
        let overlap_z = d1.z.min(d2.z);

        let center_a = self.center();
        let center_b = other.center();
        let diff = center_b - center_a;

        if overlap_x < overlap_y && overlap_x < overlap_z {
            let sign = if diff.x >= 0.0 { 1.0 } else { -1.0 };
            let normal = Vec3::new(sign, 0.0, 0.0);
            let point = Vec3::new(
                if sign > 0.0 { self.max.x } else { self.min.x },
                center_a.y,
                center_a.z,
            );
            Some(ContactManifold::new(normal, overlap_x, point))
        } else if overlap_y < overlap_z {
            let sign = if diff.y >= 0.0 { 1.0 } else { -1.0 };
            let normal = Vec3::new(0.0, sign, 0.0);
            let point = Vec3::new(
                center_a.x,
                if sign > 0.0 { self.max.y } else { self.min.y },
                center_a.z,
            );
            Some(ContactManifold::new(normal, overlap_y, point))
        } else {
            let sign = if diff.z >= 0.0 { 1.0 } else { -1.0 };
            let normal = Vec3::new(0.0, 0.0, sign);
            let point = Vec3::new(
                center_a.x,
                center_a.y,
                if sign > 0.0 { self.max.z } else { self.min.z },
            );
            Some(ContactManifold::new(normal, overlap_z, point))
        }
    }

    pub fn raycast(&self, ray: &Ray3D) -> Option<RaycastHit> {
        let mut tmin = (self.min.x - ray.origin.x) / ray.direction.x;
        let mut tmax = (self.max.x - ray.origin.x) / ray.direction.x;
        let mut normal = Vec3::new(-1.0, 0.0, 0.0);

        if tmin > tmax {
            std::mem::swap(&mut tmin, &mut tmax);
            normal = Vec3::new(1.0, 0.0, 0.0);
        }

        let mut tymin = (self.min.y - ray.origin.y) / ray.direction.y;
        let mut tymax = (self.max.y - ray.origin.y) / ray.direction.y;
        let mut y_normal = Vec3::new(0.0, -1.0, 0.0);

        if tymin > tymax {
            std::mem::swap(&mut tymin, &mut tymax);
            y_normal = Vec3::new(0.0, 1.0, 0.0);
        }

        if (tmin > tymax) || (tymin > tmax) {
            return None;
        }

        if tymin > tmin {
            tmin = tymin;
            normal = y_normal;
        }

        if tymax < tmax {
            tmax = tymax;
        }

        let mut tzmin = (self.min.z - ray.origin.z) / ray.direction.z;
        let mut tzmax = (self.max.z - ray.origin.z) / ray.direction.z;
        let mut z_normal = Vec3::new(0.0, 0.0, -1.0);

        if tzmin > tzmax {
            std::mem::swap(&mut tzmin, &mut tzmax);
            z_normal = Vec3::new(0.0, 0.0, 1.0);
        }

        if (tmin > tzmax) || (tzmin > tmax) {
            return None;
        }

        if tzmin > tmin {
            tmin = tzmin;
            normal = z_normal;
        }

        if tmin < 0.0 {
            return None;
        }

        Some(RaycastHit {
            distance: tmin,
            point: ray.point_at(tmin),
            normal,
        })
    }

    pub fn contact_sphere(&self, sphere: &BoundingSphere) -> Option<ContactManifold> {
        let closest = Vec3::new(
            sphere.center.x.clamp(self.min.x, self.max.x),
            sphere.center.y.clamp(self.min.y, self.max.y),
            sphere.center.z.clamp(self.min.z, self.max.z),
        );

        let diff = sphere.center - closest;
        let dist_sq = diff.length_squared();

        if dist_sq >= sphere.radius * sphere.radius {
            return None;
        }

        let dist = dist_sq.sqrt();
        let (normal, depth) = if dist > 1e-5 {
            (diff / dist, sphere.radius - dist)
        } else {
            let d_min = (sphere.center - self.min).abs();
            let d_max = (self.max - sphere.center).abs();
            let mut min_dist = d_min.x;
            let mut norm = Vec3::new(-1.0, 0.0, 0.0);

            if d_max.x < min_dist {
                min_dist = d_max.x;
                norm = Vec3::new(1.0, 0.0, 0.0);
            }
            if d_min.y < min_dist {
                min_dist = d_min.y;
                norm = Vec3::new(0.0, -1.0, 0.0);
            }
            if d_max.y < min_dist {
                min_dist = d_max.y;
                norm = Vec3::new(0.0, 1.0, 0.0);
            }
            if d_min.z < min_dist {
                min_dist = d_min.z;
                norm = Vec3::new(0.0, 0.0, -1.0);
            }
            if d_max.z < min_dist {
                min_dist = d_max.z;
                norm = Vec3::new(0.0, 0.0, 1.0);
            }

            (norm, sphere.radius + min_dist)
        };

        Some(ContactManifold::new(normal, depth, closest))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let dist_sq = self.center.distance_squared(other.center);
        let rad_sum = self.radius + other.radius;
        dist_sq <= rad_sum * rad_sum
    }

    pub fn contact(&self, other: &Self) -> Option<ContactManifold> {
        let diff = other.center - self.center;
        let dist = diff.length();
        let rad_sum = self.radius + other.radius;

        if dist >= rad_sum {
            return None;
        }

        let normal = if dist > 1e-6 {
            diff / dist
        } else {
            Vec3::Y
        };

        let depth = rad_sum - dist;
        let point = self.center + normal * self.radius;

        Some(ContactManifold::new(normal, depth, point))
    }

    pub fn raycast(&self, ray: &Ray3D) -> Option<RaycastHit> {
        let oc = ray.origin - self.center;
        let a = ray.direction.dot(ray.direction);
        let b = 2.0 * oc.dot(ray.direction);
        let c = oc.dot(oc) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }

        let point = ray.point_at(t);
        let normal = (point - self.center).normalize();

        Some(RaycastHit {
            distance: t,
            point,
            normal,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2D {
    pub min: Vec2,
    pub max: Vec2,
}

impl Aabb2D {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn from_center_half_extents(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    pub fn contact(&self, other: &Self) -> Option<ContactManifold> {
        let d1 = other.max - self.min;
        let d2 = self.max - other.min;

        if d1.x < 0.0 || d2.x < 0.0 || d1.y < 0.0 || d2.y < 0.0 {
            return None;
        }

        let overlap_x = d1.x.min(d2.x);
        let overlap_y = d1.y.min(d2.y);

        let center_a = (self.min + self.max) * 0.5;
        let center_b = (other.min + other.max) * 0.5;
        let diff = center_b - center_a;

        if overlap_x < overlap_y {
            let sign = if diff.x >= 0.0 { 1.0 } else { -1.0 };
            Some(ContactManifold::new(
                Vec3::new(sign, 0.0, 0.0),
                overlap_x,
                Vec3::new(center_a.x, center_a.y, 0.0),
            ))
        } else {
            let sign = if diff.y >= 0.0 { 1.0 } else { -1.0 };
            Some(ContactManifold::new(
                Vec3::new(0.0, sign, 0.0),
                overlap_y,
                Vec3::new(center_a.x, center_a.y, 0.0),
            ))
        }
    }
}

pub fn resolve_penetration(
    transform_a: &mut Transform,
    transform_b: &mut Transform,
    contact: &ContactManifold,
) {
    resolve_penetration_weighted(transform_a, false, transform_b, false, contact);
}

pub fn resolve_penetration_weighted(
    transform_a: &mut Transform,
    is_static_a: bool,
    transform_b: &mut Transform,
    is_static_b: bool,
    contact: &ContactManifold,
) {
    if is_static_a && is_static_b {
        return;
    }
    if is_static_a {
        transform_b.translation += contact.normal * contact.depth;
    } else if is_static_b {
        transform_a.translation -= contact.normal * contact.depth;
    } else {
        let half = contact.normal * (contact.depth * 0.5);
        transform_a.translation -= half;
        transform_b.translation += half;
    }
}
