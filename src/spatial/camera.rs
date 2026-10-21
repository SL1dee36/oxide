use crate::math::deg_to_rad;
use crate::spatial::transform::Transform;
use glam::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub enum Projection {
    Perspective {
        fov_y_rad: f32,
        z_near: f32,
        z_far: f32,
    },
    Orthographic {
        half_width: f32,
        half_height: f32,
        z_near: f32,
        z_far: f32,
    },
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub transform: Transform,
    pub projection: Projection,
}

impl Camera {
    pub fn perspective(fov_y_deg: f32, _aspect: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            transform: Transform::IDENTITY,
            projection: Projection::Perspective {
                fov_y_rad: deg_to_rad(fov_y_deg),
                z_near,
                z_far,
            },
        }
    }

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

    pub fn look_at(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.transform.translation = eye;
        let view = Mat4::look_at_rh(eye, target, up);
        let inv = view.inverse();
        let (_scale, rot, trans) = inv.to_scale_rotation_translation();
        self.transform.rotation = rot;
        self.transform.translation = trans;
    }

    #[inline]
    pub fn view_matrix(&self) -> Mat4 {
        self.transform.matrix().inverse()
    }

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
        Self::perspective(60.0, 16.0 / 9.0, 0.1, 1000.0)
    }
}
