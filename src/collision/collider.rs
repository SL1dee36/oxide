use crate::collision::manifold::ContactManifold;
use crate::collision::ray::{Ray3D, RaycastHit};
use crate::spatial::transform::Transform;
use glam::{Mat3, Mat4, Vec2, Vec3};

/// Supported collision bounding volume primitives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider {
    /// 3D Axis-Aligned Bounding Box specified by half-extents along X, Y, and Z axes.
    Box3D {
        /// Half-extents vector $(h_x, h_y, h_z)$.
        half_extents: Vec3,
    },
    /// 3D spherical bounding volume defined by radius.
    Sphere {
        /// Sphere radius in world units.
        radius: f32,
    },
    /// 2D rectangular bounding box in screen pixels.
    Box2D {
        /// Half-extents vector $(h_x, h_y)$.
        half_extents: Vec2,
    },
}

impl Collider {
    /// Constructs a 3D box collider from total dimensions `(dx, dy, dz)`.
    #[inline]
    pub fn box_3d(size: Vec3) -> Self {
        Self::Box3D {
            half_extents: size * 0.5,
        }
    }

    /// Constructs a 3D spherical bounding volume with given `radius`.
    #[inline]
    pub fn sphere(radius: f32) -> Self {
        Self::Sphere { radius }
    }

    /// Constructs a 2D box collider from total pixel dimensions `(width, height)`.
    #[inline]
    pub fn box_2d(size: Vec2) -> Self {
        Self::Box2D {
            half_extents: size * 0.5,
        }
    }

    /// Computes a conservative world-space 3D AABB enclosing this collider under the given transformation matrix.
    pub fn world_aabb(&self, matrix: &Mat4) -> Aabb3D {
        match self {
            Self::Box3D { half_extents } => {
                Aabb3D::from_center_half_extents(Vec3::ZERO, *half_extents).transform(matrix)
            }
            Self::Sphere { radius } => {
                let center = matrix.transform_point3(Vec3::ZERO);
                let (scale, _, _) = matrix.to_scale_rotation_translation();
                let r_eff = *radius * scale.x.abs().max(scale.y.abs()).max(scale.z.abs());
                Aabb3D::from_center_half_extents(center, Vec3::splat(r_eff))
            }
            Self::Box2D { half_extents } => {
                let (scale, _, trans) = matrix.to_scale_rotation_translation();
                let h = Vec3::new(
                    half_extents.x * scale.x.abs(),
                    half_extents.y * scale.y.abs(),
                    0.0,
                );
                Aabb3D::from_center_half_extents(trans, h)
            }
        }
    }
}

/// 3D Axis-Aligned Bounding Box (AABB) defined by minimum and maximum extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb3D {
    /// Minimum corner coordinate $(x_{\min}, y_{\min}, z_{\min})$.
    pub min: Vec3,
    /// Maximum corner coordinate $(x_{\max}, y_{\max}, z_{\max})$.
    pub max: Vec3,
}

impl Aabb3D {
    /// Constructs an AABB from explicit min and max points.
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Constructs an AABB from center point and half-extents vector.
    pub fn from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Computes the geometric center point of the box.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Returns the half-extents vector $(h_x, h_y, h_z)$.
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Transforms the AABB by a 4x4 affine matrix, computing a conservative enclosing AABB.
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

    /// Returns `true` if this AABB overlaps with `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Computes the contact manifold along the axis of minimum penetration against `other` AABB.
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

    /// Performs slab-method ray intersection against this AABB.
    pub fn raycast(&self, ray: &Ray3D) -> Option<RaycastHit> {
        if !ray.origin.is_finite() || !ray.direction.is_finite() || ray.direction.length_squared() < 1e-12 {
            return None;
        }

        let mut tmin = f32::NEG_INFINITY;
        let mut tmax = f32::INFINITY;
        let mut normal = Vec3::ZERO;

        // X axis slab test
        if ray.direction.x.abs() < 1e-8 {
            if ray.origin.x < self.min.x || ray.origin.x > self.max.x {
                return None;
            }
        } else {
            let inv_d = 1.0 / ray.direction.x;
            let mut t0 = (self.min.x - ray.origin.x) * inv_d;
            let mut t1 = (self.max.x - ray.origin.x) * inv_d;
            let mut n = Vec3::new(-1.0, 0.0, 0.0);
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
                n = Vec3::new(1.0, 0.0, 0.0);
            }
            if t0.is_nan() || t1.is_nan() {
                return None;
            }
            if t0 > tmin {
                tmin = t0;
                normal = n;
            }
            if t1 < tmax {
                tmax = t1;
            }
            if tmin > tmax {
                return None;
            }
        }

        // Y axis slab test
        if ray.direction.y.abs() < 1e-8 {
            if ray.origin.y < self.min.y || ray.origin.y > self.max.y {
                return None;
            }
        } else {
            let inv_d = 1.0 / ray.direction.y;
            let mut t0 = (self.min.y - ray.origin.y) * inv_d;
            let mut t1 = (self.max.y - ray.origin.y) * inv_d;
            let mut n = Vec3::new(0.0, -1.0, 0.0);
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
                n = Vec3::new(0.0, 1.0, 0.0);
            }
            if t0.is_nan() || t1.is_nan() {
                return None;
            }
            if t0 > tmin {
                tmin = t0;
                normal = n;
            }
            if t1 < tmax {
                tmax = t1;
            }
            if tmin > tmax {
                return None;
            }
        }

        // Z axis slab test
        if ray.direction.z.abs() < 1e-8 {
            if ray.origin.z < self.min.z || ray.origin.z > self.max.z {
                return None;
            }
        } else {
            let inv_d = 1.0 / ray.direction.z;
            let mut t0 = (self.min.z - ray.origin.z) * inv_d;
            let mut t1 = (self.max.z - ray.origin.z) * inv_d;
            let mut n = Vec3::new(0.0, 0.0, -1.0);
            if t0 > t1 {
                std::mem::swap(&mut t0, &mut t1);
                n = Vec3::new(0.0, 0.0, 1.0);
            }
            if t0.is_nan() || t1.is_nan() {
                return None;
            }
            if t0 > tmin {
                tmin = t0;
                normal = n;
            }
            if t1 < tmax {
                tmax = t1;
            }
            if tmin > tmax {
                return None;
            }
        }

        if tmax < 0.0 || tmin.is_nan() || tmax.is_nan() {
            return None;
        }

        let hit_dist = if tmin >= 0.0 { tmin } else { 0.0 };

        Some(RaycastHit {
            distance: hit_dist,
            point: ray.point_at(hit_dist),
            normal,
        })
    }

    /// Evaluates contact between this AABB and a bounding sphere.
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

/// 3D Oriented Bounding Box (OBB) defined by center, 3 orthonormal world-space axes, and scaled half-extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Obb3D {
    /// Center of the box in world Cartesian coordinates.
    pub center: Vec3,
    /// Orthonormal local axes X, Y, Z in world space.
    pub axes: [Vec3; 3],
    /// Scaled half-extents along each local axis.
    pub half_extents: Vec3,
}

impl Obb3D {
    /// Constructs an OBB from unscaled half-extents and a world transformation matrix.
    pub fn from_matrix(half_extents: Vec3, matrix: &Mat4) -> Self {
        let (scale, rot, translation) = matrix.to_scale_rotation_translation();
        let rot_mat = Mat3::from_quat(rot);
        let axes = [
            rot_mat.x_axis.normalize_or_zero(),
            rot_mat.y_axis.normalize_or_zero(),
            rot_mat.z_axis.normalize_or_zero(),
        ];
        Self {
            center: translation,
            axes,
            half_extents: half_extents * scale.abs(),
        }
    }

    /// Evaluates contact between this OBB and a bounding sphere.
    pub fn contact_sphere(&self, sphere: &BoundingSphere) -> Option<ContactManifold> {
        let d = sphere.center - self.center;
        let local_p = Vec3::new(
            d.dot(self.axes[0]),
            d.dot(self.axes[1]),
            d.dot(self.axes[2]),
        );

        let local_q = Vec3::new(
            local_p.x.clamp(-self.half_extents.x, self.half_extents.x),
            local_p.y.clamp(-self.half_extents.y, self.half_extents.y),
            local_p.z.clamp(-self.half_extents.z, self.half_extents.z),
        );

        let world_q = self.center
            + self.axes[0] * local_q.x
            + self.axes[1] * local_q.y
            + self.axes[2] * local_q.z;

        let diff = sphere.center - world_q;
        let dist_sq = diff.length_squared();

        if dist_sq >= sphere.radius * sphere.radius {
            return None;
        }

        let dist = dist_sq.sqrt();
        let (normal, depth, contact_point) = if dist > 1e-5 {
            (diff / dist, sphere.radius - dist, world_q)
        } else {
            // Sphere center is inside or on the boundary of the OBB
            let d_min = (local_p + self.half_extents).abs();
            let d_max = (self.half_extents - local_p).abs();

            let mut min_dist = d_min.x;
            let mut face_axis_idx = 0;
            let mut face_sign = -1.0;

            if d_max.x < min_dist {
                min_dist = d_max.x;
                face_axis_idx = 0;
                face_sign = 1.0;
            }
            if d_min.y < min_dist {
                min_dist = d_min.y;
                face_axis_idx = 1;
                face_sign = -1.0;
            }
            if d_max.y < min_dist {
                min_dist = d_max.y;
                face_axis_idx = 1;
                face_sign = 1.0;
            }
            if d_min.z < min_dist {
                min_dist = d_min.z;
                face_axis_idx = 2;
                face_sign = -1.0;
            }
            if d_max.z < min_dist {
                min_dist = d_max.z;
                face_axis_idx = 2;
                face_sign = 1.0;
            }

            let normal = self.axes[face_axis_idx] * face_sign;
            let depth = sphere.radius + min_dist;
            let mut local_face_pt = local_q;
            match face_axis_idx {
                0 => local_face_pt.x = face_sign * self.half_extents.x,
                1 => local_face_pt.y = face_sign * self.half_extents.y,
                _ => local_face_pt.z = face_sign * self.half_extents.z,
            }
            let pt = self.center
                + self.axes[0] * local_face_pt.x
                + self.axes[1] * local_face_pt.y
                + self.axes[2] * local_face_pt.z;
            (normal, depth, pt)
        };

        Some(ContactManifold::new(normal, depth, contact_point))
    }

    /// Performs raycast against this OBB by transforming the ray into local space.
    pub fn raycast(&self, ray: &Ray3D) -> Option<RaycastHit> {
        let oc = ray.origin - self.center;
        let local_origin = Vec3::new(
            oc.dot(self.axes[0]),
            oc.dot(self.axes[1]),
            oc.dot(self.axes[2]),
        );
        let local_direction = Vec3::new(
            ray.direction.dot(self.axes[0]),
            ray.direction.dot(self.axes[1]),
            ray.direction.dot(self.axes[2]),
        );

        let local_ray = Ray3D::new(local_origin, local_direction);
        let local_aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, self.half_extents);
        let hit = local_aabb.raycast(&local_ray)?;

        // Rotate normal back to world space
        let world_normal = (self.axes[0] * hit.normal.x
            + self.axes[1] * hit.normal.y
            + self.axes[2] * hit.normal.z)
            .normalize();

        Some(RaycastHit {
            distance: hit.distance,
            point: ray.point_at(hit.distance),
            normal: world_normal,
        })
    }

    /// Computes contact manifold between two OBBs using Separating Axis Theorem (SAT).
    pub fn contact(&self, other: &Self) -> Option<ContactManifold> {
        let diff = other.center - self.center;

        // 15 candidate axes: 3 from self, 3 from other, 9 cross products
        let axes_to_test: [Vec3; 15] = [
            self.axes[0],
            self.axes[1],
            self.axes[2],
            other.axes[0],
            other.axes[1],
            other.axes[2],
            self.axes[0].cross(other.axes[0]),
            self.axes[0].cross(other.axes[1]),
            self.axes[0].cross(other.axes[2]),
            self.axes[1].cross(other.axes[0]),
            self.axes[1].cross(other.axes[1]),
            self.axes[1].cross(other.axes[2]),
            self.axes[2].cross(other.axes[0]),
            self.axes[2].cross(other.axes[1]),
            self.axes[2].cross(other.axes[2]),
        ];

        let mut min_overlap = f32::INFINITY;
        let mut best_axis = Vec3::Y;

        for raw_axis in axes_to_test {
            let len_sq = raw_axis.length_squared();
            if len_sq < 1e-6 {
                // Parallel / degenerate edge cross product
                continue;
            }
            let axis = raw_axis / len_sq.sqrt();

            let r_a = self.half_extents.x * self.axes[0].dot(axis).abs()
                + self.half_extents.y * self.axes[1].dot(axis).abs()
                + self.half_extents.z * self.axes[2].dot(axis).abs();

            let r_b = other.half_extents.x * other.axes[0].dot(axis).abs()
                + other.half_extents.y * other.axes[1].dot(axis).abs()
                + other.half_extents.z * other.axes[2].dot(axis).abs();

            let s = diff.dot(axis).abs();
            let overlap = (r_a + r_b) - s;

            if overlap <= 0.0 {
                // Separating axis found
                return None;
            }

            if overlap < min_overlap {
                min_overlap = overlap;
                let oriented_axis = if diff.dot(axis) < 0.0 { -axis } else { axis };
                best_axis = oriented_axis;
            }
        }

        if min_overlap.is_infinite() {
            return None;
        }

        let r_a_best = self.half_extents.x * self.axes[0].dot(best_axis).abs()
            + self.half_extents.y * self.axes[1].dot(best_axis).abs()
            + self.half_extents.z * self.axes[2].dot(best_axis).abs();

        let p_n = self.center.dot(best_axis) + r_a_best - min_overlap * 0.5;

        // Two orthonormal tangent axes in the plane perpendicular to best_axis
        let t1 = if best_axis.x.abs() > 0.9 {
            best_axis.cross(Vec3::Y).normalize()
        } else {
            best_axis.cross(Vec3::X).normalize()
        };
        let t2 = best_axis.cross(t1).normalize();

        // Project both OBBs onto t1
        let r_a_1 = self.half_extents.x * self.axes[0].dot(t1).abs()
            + self.half_extents.y * self.axes[1].dot(t1).abs()
            + self.half_extents.z * self.axes[2].dot(t1).abs();
        let r_b_1 = other.half_extents.x * other.axes[0].dot(t1).abs()
            + other.half_extents.y * other.axes[1].dot(t1).abs()
            + other.half_extents.z * other.axes[2].dot(t1).abs();
        let min_a_1 = self.center.dot(t1) - r_a_1;
        let max_a_1 = self.center.dot(t1) + r_a_1;
        let min_b_1 = other.center.dot(t1) - r_b_1;
        let max_b_1 = other.center.dot(t1) + r_b_1;
        let p_1 = (min_a_1.max(min_b_1) + max_a_1.min(max_b_1)) * 0.5;

        // Project both OBBs onto t2
        let r_a_2 = self.half_extents.x * self.axes[0].dot(t2).abs()
            + self.half_extents.y * self.axes[1].dot(t2).abs()
            + self.half_extents.z * self.axes[2].dot(t2).abs();
        let r_b_2 = other.half_extents.x * other.axes[0].dot(t2).abs()
            + other.half_extents.y * other.axes[1].dot(t2).abs()
            + other.half_extents.z * other.axes[2].dot(t2).abs();
        let min_a_2 = self.center.dot(t2) - r_a_2;
        let max_a_2 = self.center.dot(t2) + r_a_2;
        let min_b_2 = other.center.dot(t2) - r_b_2;
        let max_b_2 = other.center.dot(t2) + r_b_2;
        let p_2 = (min_a_2.max(min_b_2) + max_a_2.min(max_b_2)) * 0.5;

        let contact_point = best_axis * p_n + t1 * p_1 + t2 * p_2;

        Some(ContactManifold::new(best_axis, min_overlap, contact_point))
    }

    /// Evaluates whether this OBB intersects with `other`.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool {
        self.contact(other).is_some()
    }
}

/// 3D spherical bounding volume defined by center point and radius.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingSphere {
    /// Center of the sphere in world Cartesian coordinates.
    pub center: Vec3,
    /// Sphere radius in world units.
    pub radius: f32,
}

impl BoundingSphere {
    /// Constructs a bounding sphere from center position and radius.
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }

    /// Evaluates whether this sphere intersects with `other`.
    pub fn intersects(&self, other: &Self) -> bool {
        let dist_sq = self.center.distance_squared(other.center);
        let rad_sum = self.radius + other.radius;
        dist_sq <= rad_sum * rad_sum
    }

    /// Computes contact manifold between two spheres.
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

    /// Performs quadratic ray-sphere intersection test.
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

/// 2D Axis-Aligned Bounding Box for planar screen-space elements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb2D {
    /// Minimum corner in screen coordinates (pixels).
    pub min: Vec2,
    /// Maximum corner in screen coordinates (pixels).
    pub max: Vec2,
}

impl Aabb2D {
    /// Constructs a 2D AABB from explicit min and max points.
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    /// Constructs a 2D AABB from center point and half-extents.
    pub fn from_center_half_extents(center: Vec2, half_extents: Vec2) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Evaluates overlap between two 2D AABBs.
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Computes 2D contact manifold along minimum separation axis.
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

/// Displaces transforms of two dynamic bodies equally (50% / 50%) along contact normal to resolve geometric overlap.
pub fn resolve_penetration(
    transform_a: &mut Transform,
    transform_b: &mut Transform,
    contact: &ContactManifold,
) {
    resolve_penetration_weighted(transform_a, false, transform_b, false, contact);
}

/// Displaces transforms along contact normal respecting static immovable flags (`is_static_a`, `is_static_b`).
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
