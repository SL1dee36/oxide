use glam::{Mat3, Quat, Vec3};

/// Classifies physical motion simulation mode of a rigid body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RigidBodyType {
    /// Immobile body with infinite mass; unaffected by forces, gravity, or impulses.
    Static,
    /// Fully simulated dynamic body obeying forces, gravity, and impulses.
    #[default]
    Dynamic,
    /// Kinematically controlled body with velocity unaffected by forces or impulses.
    Kinematic,
}

/// 6-DOF rigid body dynamics component.
#[derive(Debug, Clone, PartialEq)]
pub struct RigidBody {
    /// Linear velocity vector in world units per second.
    pub velocity: Vec3,
    /// Angular velocity vector in radians per second.
    pub angular_velocity: Vec3,
    /// Force accumulator in Newtons, applied during next integration step.
    pub force_accumulator: Vec3,
    /// Torque accumulator in Newton-meters, applied during next integration step.
    pub torque_accumulator: Vec3,
    /// Total mass in kilograms.
    pub mass: f32,
    /// Inverse mass ($1 / m$), 0.0 for static or infinite mass bodies.
    pub inv_mass: f32,
    /// Inverse inertia tensor in body local Cartesian coordinates.
    pub inv_inertia_local: Mat3,
    /// Coefficient of restitution (bounciness) in [0.0, 1.0].
    pub restitution: f32,
    /// Coulomb surface friction coefficient in [0.0, 1.0].
    pub friction: f32,
    /// Motion simulation type classification.
    pub body_type: RigidBodyType,
    /// Linear velocity damping ratio per second.
    pub linear_damping: f32,
    /// Angular velocity damping ratio per second.
    pub angular_damping: f32,
}

impl RigidBody {
    /// Constructs a rigid body with the specified mass and rigid body type.
    pub fn new(mass: f32, body_type: RigidBodyType) -> Self {
        match body_type {
            RigidBodyType::Static => Self::new_static(),
            RigidBodyType::Dynamic => Self::new_dynamic(mass),
            RigidBodyType::Kinematic => Self::new_kinematic(),
        }
    }

    /// Constructs a dynamic rigid body with given mass in kilograms and isotropic unit sphere inertia tensor.
    pub fn new_dynamic(mass: f32) -> Self {
        let safe_mass = mass.max(1e-5);
        let inv_mass = 1.0 / safe_mass;
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass: safe_mass,
            inv_mass,
            inv_inertia_local: Mat3::from_diagonal(Vec3::splat(inv_mass * 2.5)),
            restitution: 0.2,
            friction: 0.5,
            body_type: RigidBodyType::Dynamic,
            linear_damping: 0.02,
            angular_damping: 0.05,
        }
    }

    /// Constructs an immobile static rigid body with zero inverse mass and zero inertia tensor.
    pub fn new_static() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass: 0.0,
            inv_mass: 0.0,
            inv_inertia_local: Mat3::ZERO,
            restitution: 0.2,
            friction: 0.5,
            body_type: RigidBodyType::Static,
            linear_damping: 0.0,
            angular_damping: 0.0,
        }
    }

    /// Constructs a kinematic rigid body with controlled velocity unaffected by forces.
    pub fn new_kinematic() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass: 0.0,
            inv_mass: 0.0,
            inv_inertia_local: Mat3::ZERO,
            restitution: 0.2,
            friction: 0.5,
            body_type: RigidBodyType::Kinematic,
            linear_damping: 0.0,
            angular_damping: 0.0,
        }
    }

    /// Calculates and configures the inverse inertia tensor for a solid cuboid of dimensions `(width, height, depth)`.
    ///
    /// Principle moments: $I_{xx} = \frac{1}{12} m (h^2 + d^2)$, $I_{yy} = \frac{1}{12} m (w^2 + d^2)$, $I_{zz} = \frac{1}{12} m (w^2 + h^2)$.
    pub fn with_box_inertia(mut self, size: Vec3) -> Self {
        if self.body_type == RigidBodyType::Dynamic && self.inv_mass > 0.0 {
            let w = size.x.max(1e-4);
            let h = size.y.max(1e-4);
            let d = size.z.max(1e-4);

            let i_xx = (1.0 / 12.0) * self.mass * (h * h + d * d);
            let i_yy = (1.0 / 12.0) * self.mass * (w * w + d * d);
            let i_zz = (1.0 / 12.0) * self.mass * (w * w + h * h);

            self.inv_inertia_local = Mat3::from_diagonal(Vec3::new(
                1.0 / i_xx,
                1.0 / i_yy,
                1.0 / i_zz,
            ));
        } else {
            self.inv_inertia_local = Mat3::ZERO;
        }
        self
    }

    /// Calculates and configures the inverse inertia tensor for a solid uniform sphere with given `radius`.
    ///
    /// Principle moment: $I = \frac{2}{5} m r^2$.
    pub fn with_sphere_inertia(mut self, radius: f32) -> Self {
        if self.body_type == RigidBodyType::Dynamic && self.inv_mass > 0.0 {
            let r = radius.max(1e-4);
            let i = 0.4 * self.mass * (r * r);
            self.inv_inertia_local = Mat3::from_diagonal(Vec3::splat(1.0 / i));
        } else {
            self.inv_inertia_local = Mat3::ZERO;
        }
        self
    }

    /// Sets explicit local inverse inertia tensor matrix.
    pub fn with_inertia_tensor(mut self, inv_inertia_local: Mat3) -> Self {
        if self.body_type == RigidBodyType::Dynamic {
            self.inv_inertia_local = inv_inertia_local;
        } else {
            self.inv_inertia_local = Mat3::ZERO;
        }
        self
    }

    /// Transforms local inverse inertia tensor into world space coordinates via body orientation quaternion.
    #[inline]
    pub fn inv_inertia_world(&self, rotation: Quat) -> Mat3 {
        if self.body_type != RigidBodyType::Dynamic {
            Mat3::ZERO
        } else {
            let rot_mat = Mat3::from_quat(rotation);
            rot_mat * self.inv_inertia_local * rot_mat.transpose()
        }
    }

    /// Configures restitution (bounciness).
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self
    }

    /// Configures friction coefficient.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.max(0.0);
        self
    }

    /// Sets initial linear velocity.
    pub fn with_velocity(mut self, velocity: Vec3) -> Self {
        self.velocity = velocity;
        self
    }

    /// Sets initial angular velocity.
    pub fn with_angular_velocity(mut self, angular_velocity: Vec3) -> Self {
        self.angular_velocity = angular_velocity;
        self
    }

    /// Applies a continuous force in Newtons accumulated for the upcoming simulation step.
    #[inline]
    pub fn apply_force(&mut self, force: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.force_accumulator += force;
        }
    }

    /// Applies a continuous torque in Newton-meters accumulated for the upcoming simulation step.
    #[inline]
    pub fn apply_torque(&mut self, torque: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.torque_accumulator += torque;
        }
    }

    /// Applies an instantaneous linear impulse ($J = \Delta p = m \cdot \Delta v$).
    #[inline]
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.velocity += impulse * self.inv_mass;
        }
    }

    /// Applies an instantaneous angular impulse using local inverse inertia tensor ($\Delta \boldsymbol{\omega} = I^{-1} \cdot \mathbf{L}$).
    #[inline]
    pub fn apply_angular_impulse(&mut self, angular_impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.angular_velocity += self.inv_inertia_local * angular_impulse;
        }
    }

    /// Reconfigures mass and recalculates inverse mass and default inertia.
    pub fn set_mass(&mut self, mass: f32) {
        if self.body_type == RigidBodyType::Dynamic {
            let safe_mass = mass.max(1e-5);
            self.mass = safe_mass;
            self.inv_mass = 1.0 / safe_mass;
            self.inv_inertia_local = Mat3::from_diagonal(Vec3::splat(self.inv_mass * 2.5));
        }
    }

    /// Returns `true` if body is dynamic.
    #[inline]
    pub fn is_dynamic(&self) -> bool {
        self.body_type == RigidBodyType::Dynamic
    }

    /// Returns `true` if body is static.
    #[inline]
    pub fn is_static(&self) -> bool {
        self.body_type == RigidBodyType::Static
    }

    /// Returns `true` if body is kinematic.
    #[inline]
    pub fn is_kinematic(&self) -> bool {
        self.body_type == RigidBodyType::Kinematic
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        Self::new_dynamic(1.0)
    }
}
