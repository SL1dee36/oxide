use crate::math::deg_to_rad;
use crate::spatial::transform::Transform;
use glam::{Mat4, Vec3};

/// Geometric projection model for a camera.
#[derive(Debug, Clone, Copy)]
pub enum Projection {
    /// Symmetrical perspective projection defined by vertical field-of-view, near and far clipping planes.
    Perspective {
        /// Vertical field of view in radians.
        fov_y_rad: f32,
        /// Distance to the near clipping plane along view axis.
        z_near: f32,
        /// Distance to the far clipping plane along view axis.
        z_far: f32,
    },
    /// Orthographic parallel projection defined by half extents and clipping planes.
    Orthographic {
        /// Half-width of view cuboid in world units.
        half_width: f32,
        /// Half-height of view cuboid in world units.
        half_height: f32,
        /// Distance to near clipping plane.
        z_near: f32,
        /// Distance to far clipping plane.
        z_far: f32,
    },
}

/// 3D viewing camera combining world transform with perspective or orthographic projection.
#[derive(Debug, Clone)]
pub struct Camera {
    /// Spatial pose of the camera in world coordinates.
    pub transform: Transform,
    /// Projection configuration.
    pub projection: Projection,
}

impl Camera {
    /// Constructs a perspective camera with vertical field of view specified in degrees.
    pub fn perspective(fov_y_deg: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            transform: Transform::IDENTITY,
            projection: Projection::Perspective {
                fov_y_rad: deg_to_rad(fov_y_deg),
                z_near,
                z_far,
            },
        }
    }

    /// Constructs an orthographic camera with specified width and height.
    pub fn orthographic(width: f32, height: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            transform: Transform::IDENTITY,
            projection: Projection::Orthographic {
                half_width: width * 0.5,
                half_height: height * 0.5,
                z_near,
                z_far,
            },
        }
    }

    /// Orients the camera to look from `eye` position toward `target` position with specified `up` direction vector.
    pub fn look_at(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.transform.translation = eye;
        let view = Mat4::look_at_rh(eye, target, up);
        let inv = view.inverse();
        let (_scale, rot, trans) = inv.to_scale_rotation_translation();
        self.transform.rotation = rot;
        self.transform.translation = trans;
    }

    /// Calculates the 4x4 right-handed view matrix (inverse of camera's world transform matrix).
    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.matrix().inverse()
    }

    /// Computes the 4x4 right-handed OpenGL projection matrix for the given viewport `aspect_ratio`.
    #[inline]
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        match self.projection {
            Projection::Perspective {
                fov_y_rad,
                z_near,
                z_far,
            } => Mat4::perspective_rh_gl(fov_y_rad, aspect_ratio, z_near, z_far),
            Projection::Orthographic {
                half_width,
                half_height,
                z_near,
                z_far,
            } => Mat4::orthographic_rh_gl(
                -half_width * aspect_ratio,
                half_width * aspect_ratio,
                -half_height,
                half_height,
                z_near,
                z_far,
            ),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::perspective(60.0, 0.1, 1000.0)
    }
}
