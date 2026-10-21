use glam::{EulerRot, Mat4, Quat, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    #[inline]
    pub fn translate(&mut self, delta: Vec3) {
        self.translation += delta;
    }

    #[inline]
    pub fn rotate_x(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_x(rad);
    }

    #[inline]
    pub fn rotate_y(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_y(rad);
    }

    #[inline]
    pub fn rotate_z(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_z(rad);
    }

    #[inline]
    pub fn set_euler_angles(&mut self, x_rad: f32, y_rad: f32, z_rad: f32) {
        self.rotation = Quat::from_euler(EulerRot::YXZ, y_rad, x_rad, z_rad);
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}
