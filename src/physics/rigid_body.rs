use glam::Vec3;

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

    /// Constructs a dynamic rigid body with given mass in kilograms.
    pub fn new_dynamic(mass: f32) -> Self {
        let safe_mass = mass.max(1e-5);
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass: safe_mass,
            inv_mass: 1.0 / safe_mass,
            restitution: 0.2,
            friction: 0.5,
            body_type: RigidBodyType::Dynamic,
            linear_damping: 0.02,
            angular_damping: 0.05,
        }
    }

    /// Constructs an immobile static rigid body with zero inverse mass.
    pub fn new_static() -> Self {
        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            mass: 0.0,
            inv_mass: 0.0,
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
            restitution: 0.2,
            friction: 0.5,
            body_type: RigidBodyType::Kinematic,
            linear_damping: 0.0,
            angular_damping: 0.0,
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

    /// Applies an instantaneous angular impulse.
    #[inline]
    pub fn apply_angular_impulse(&mut self, angular_impulse: Vec3) {
        if self.body_type == RigidBodyType::Dynamic {
            self.angular_velocity += angular_impulse * self.inv_mass;
        }
    }

    /// Reconfigures mass and recalculates inverse mass.
    pub fn set_mass(&mut self, mass: f32) {
        if self.body_type == RigidBodyType::Dynamic {
            let safe_mass = mass.max(1e-5);
            self.mass = safe_mass;
            self.inv_mass = 1.0 / safe_mass;
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
