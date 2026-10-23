use glam::{EulerRot, Mat4, Quat, Vec3};

/// Affine spatial transformation in 3D containing translation, orientation quaternion, and scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// Position translation vector in 3D Cartesian coordinates.
    pub translation: Vec3,
    /// Orientation represented as a unit quaternion.
    pub rotation: Quat,
    /// Scale factor along X, Y, and Z axes.
    pub scale: Vec3,
}

impl Transform {
    /// Identity transformation with zero translation, identity rotation, and unit scale.
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    /// Constructs a transform from explicit translation, rotation, and scale components.
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Constructs a transform with specified translation, identity rotation, and unit scale.
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Constructs a transform from Cartesian coordinates `(x, y, z)`.
    pub fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from_translation(Vec3::new(x, y, z))
    }

    /// Constructs a transform with specified rotation, zero translation, and unit scale.
    pub fn from_rotation(rotation: Quat) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation,
            scale: Vec3::ONE,
        }
    }

    /// Constructs a transform with specified scale, zero translation, and identity rotation.
    pub fn from_scale(scale: Vec3) -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }

    /// Builder method to update the scale component.
    pub fn with_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    /// Computes the 4x4 affine model matrix corresponding to scale, rotation, and translation ($M = T \cdot R \cdot S$).
    #[inline]
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    /// Translates position by vector `delta`.
    #[inline]
    pub fn translate(&mut self, delta: Vec3) {
        self.translation += delta;
    }

    /// Applies rotation around the X-axis by `rad` radians.
    #[inline]
    pub fn rotate_x(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_x(rad);
    }

    /// Applies rotation around the Y-axis by `rad` radians.
    #[inline]
    pub fn rotate_y(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_y(rad);
    }

    /// Applies rotation around the Z-axis by `rad` radians.
    #[inline]
    pub fn rotate_z(&mut self, rad: f32) {
        self.rotation *= Quat::from_rotation_z(rad);
    }

    /// Configures orientation from Euler angles (YXZ order) in radians.
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
