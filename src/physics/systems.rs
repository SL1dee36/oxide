use crate::collision::collider::resolve_penetration_weighted;
use crate::collision::systems::check_collider_intersection;
use crate::ecs::entity::EntityId;
use crate::ecs::systems::update_hierarchy_system;
use crate::ecs::world::World;
use crate::physics::rigid_body::RigidBodyType;
use glam::{Quat, Vec3};

/// Global environment physics parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhysicsConfig {
    /// Gravitational acceleration vector in m/s^2 (default: [0, -9.81, 0]).
    pub gravity: Vec3,
    /// Number of velocity/impulse solver iterations per step.
    pub velocity_iterations: usize,
    /// Number of positional penetration solver iterations per step.
    pub position_iterations: usize,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            velocity_iterations: 4,
            position_iterations: 2,
        }
    }
}

/// Executes a complete physics simulation sub-step across all rigid bodies and soft bodies in `world`.
pub fn step_physics_system(world: &mut World, dt: f32) {
    step_physics_system_with_config(world, dt, &PhysicsConfig::default());
}

/// Executes physics simulation with custom environment configuration.
pub fn step_physics_system_with_config(world: &mut World, dt: f32, config: &PhysicsConfig) {
    if dt <= 0.0 {
        return;
    }

    // Step 1: Rigid body simulation
    step_rigid_body_system(world, dt, config);

    // Step 2: Soft body simulation
    step_soft_body_system(world, dt, config);

    // Step 3: Update hierarchy transforms
    update_hierarchy_system(world);
}

/// Simulates 6-DOF rigid body dynamics: force integration, position update, collision detection, and impulse solving.
pub fn step_rigid_body_system(world: &mut World, dt: f32, config: &PhysicsConfig) {
    // Phase 1: Force integration (velocity update)
    for (_entity, body) in &mut world.rigid_body {
        if body.is_dynamic() {
            let accel = config.gravity + body.force_accumulator * body.inv_mass;
            body.velocity += accel * dt;
            let ang_accel = body.torque_accumulator * body.inv_mass;
            body.angular_velocity += ang_accel * dt;

            // Apply damping
            body.velocity *= (1.0 - body.linear_damping * dt).clamp(0.0, 1.0);
            body.angular_velocity *= (1.0 - body.angular_damping * dt).clamp(0.0, 1.0);

            // Reset accumulators
            body.force_accumulator = Vec3::ZERO;
            body.torque_accumulator = Vec3::ZERO;
        }
    }

    // Phase 2: Position update
    for (entity, body) in &world.rigid_body {
        if body.is_dynamic() || body.is_kinematic() {
            if let Some(trans) = world.transform.get_mut(entity) {
                trans.translation += body.velocity * dt;

                let ang_speed_sq = body.angular_velocity.length_squared();
                if ang_speed_sq > 1e-8 {
                    let axis = body.angular_velocity.normalize();
                    let angle = ang_speed_sq.sqrt() * dt;
                    let rot_delta = Quat::from_axis_angle(axis, angle);
                    trans.rotation = (rot_delta * trans.rotation).normalize();
                }
            }
        }
    }

    // Update global transform matrices before collision testing
    update_hierarchy_system(world);

    // Phase 3 & 4: Collision detection & constraint solving
    let collider_entities: Vec<EntityId> = world
        .collider
        .keys()
        .filter(|&e| world.is_alive(e))
        .collect();

    for i in 0..collider_entities.len() {
        for j in (i + 1)..collider_entities.len() {
            let a = collider_entities[i];
            let b = collider_entities[j];

            let col_a = match world.collider.get(a) {
                Some(c) => *c,
                None => continue,
            };
            let col_b = match world.collider.get(b) {
                Some(c) => *c,
                None => continue,
            };

            let mat_a = world.global_transform_matrix(a);
            let mat_b = world.global_transform_matrix(b);

            if let Some(contact) = check_collider_intersection(&col_a, &mat_a, &col_b, &mat_b) {
                resolve_rigid_body_collision(world, a, b, &contact);
            }
        }
    }
}

fn resolve_rigid_body_collision(
    world: &mut World,
    a: EntityId,
    b: EntityId,
    contact: &crate::collision::manifold::ContactManifold,
) {
    let (body_a_type, body_a_inv_mass, body_a_restitution, body_a_friction) =
        if let Some(rb) = world.rigid_body.get(a) {
            (rb.body_type, rb.inv_mass, rb.restitution, rb.friction)
        } else {
            (RigidBodyType::Static, 0.0, 0.2, 0.5)
        };

    let (body_b_type, body_b_inv_mass, body_b_restitution, body_b_friction) =
        if let Some(rb) = world.rigid_body.get(b) {
            (rb.body_type, rb.inv_mass, rb.restitution, rb.friction)
        } else {
            (RigidBodyType::Static, 0.0, 0.2, 0.5)
        };

    let is_static_a = body_a_type != RigidBodyType::Dynamic || body_a_inv_mass == 0.0;
    let is_static_b = body_b_type != RigidBodyType::Dynamic || body_b_inv_mass == 0.0;

    // 1. Positional penetration resolution
    let Some([trans_a, trans_b]) = world.transform.get_disjoint_mut([a, b]) else {
        return;
    };

    resolve_penetration_weighted(
        trans_a,
        is_static_a,
        trans_b,
        is_static_b,
        contact,
    );

    if world.parent.get(a).is_none() {
        if let Some(t) = world.transform.get(a) {
            world.global_transform.insert(a, t.matrix());
        }
    }
    if world.parent.get(b).is_none() {
        if let Some(t) = world.transform.get(b) {
            world.global_transform.insert(b, t.matrix());
        }
    }

    // 2. Velocity impulse resolution
    let w_sum = body_a_inv_mass + body_b_inv_mass;
    if w_sum <= 0.0 {
        return;
    }

    let Some([rb_a, rb_b]) = world.rigid_body.get_disjoint_mut([a, b]) else {
        return;
    };

    let vel_a = rb_a.velocity;
    let vel_b = rb_b.velocity;
    // Velocity of B relative to A
    let rel_vel = vel_b - vel_a;

    let vel_along_normal = rel_vel.dot(contact.normal);
    // Only resolve if bodies are moving toward each other (vel_along_normal < 0)
    if vel_along_normal < 0.0 {
        let restitution = body_a_restitution.min(body_b_restitution);
        let j = -(1.0 + restitution) * vel_along_normal / w_sum;
        let impulse = contact.normal * j;

        if rb_a.is_dynamic() {
            rb_a.velocity -= impulse * body_a_inv_mass;
        }
        if rb_b.is_dynamic() {
            rb_b.velocity += impulse * body_b_inv_mass;
        }

        // Surface friction tangent impulse
        let tangent = rel_vel - contact.normal * vel_along_normal;
        let tangent_len = tangent.length();
        if tangent_len > 1e-5 {
            let tangent_dir = tangent / tangent_len;
            let jt = -rel_vel.dot(tangent_dir) / w_sum;
            let friction = (body_a_friction * body_b_friction).sqrt();
            let clamped_jt = jt.clamp(-j * friction, j * friction);
            let friction_impulse = tangent_dir * clamped_jt;

            if rb_a.is_dynamic() {
                rb_a.velocity -= friction_impulse * body_a_inv_mass;
            }
            if rb_b.is_dynamic() {
                rb_b.velocity += friction_impulse * body_b_inv_mass;
            }
        }
    }
}

/// Simulates deformable soft bodies using Position-Based Dynamics (PBD) and mass-spring constraints.
pub fn step_soft_body_system(world: &mut World, dt: f32, config: &PhysicsConfig) {
    crate::physics::pbd_solver::SoftBodySystem::step(world, dt, config);
}
