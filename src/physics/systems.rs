use crate::collision::collider::resolve_penetration_weighted;
use crate::collision::systems::check_collider_intersection;
use crate::ecs::entity::EntityId;
use crate::ecs::systems::update_hierarchy_system;
use crate::ecs::world::World;
use crate::physics::rigid_body::RigidBodyType;
use glam::{Mat3, Quat, Vec3};

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
    // Phase 1: Force and torque integration (velocity & angular velocity update)
    for (_entity, body) in &mut world.rigid_body {
        if body.is_dynamic() {
            let accel = config.gravity + body.force_accumulator * body.inv_mass;
            body.velocity += accel * dt;
            let ang_accel = body.inv_inertia_local * body.torque_accumulator;
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

    // Phase 3: Collision detection (narrowphase) & collection of contacts
    let collider_entities: Vec<EntityId> = world
        .collider
        .keys()
        .filter(|&e| world.is_alive(e))
        .collect();

    let mut contacts = Vec::new();

    for i in 0..collider_entities.len() {
        for j in (i + 1)..collider_entities.len() {
            let a = collider_entities[i];
            let b = collider_entities[j];

            // Prune static-static pairs immediately before intersection testing
            if world.is_static(a) && world.is_static(b) {
                continue;
            }

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
                contacts.push((a, b, contact));
            }
        }
    }

    // Phase 4: Velocity iterations (normal and friction impulses with angular momentum)
    for _ in 0..config.velocity_iterations {
        for &(a, b, ref contact) in &contacts {
            resolve_rigid_body_velocity_impulse(world, a, b, contact);
        }
    }

    // Phase 5: Position iterations (penetration relaxation)
    let pos_iters = config.position_iterations.max(1);
    for _ in 0..config.position_iterations {
        for &(a, b, ref contact) in &contacts {
            resolve_rigid_body_position_penetration(world, a, b, contact, pos_iters);
        }
    }
}

fn resolve_rigid_body_velocity_impulse(
    world: &mut World,
    a: EntityId,
    b: EntityId,
    contact: &crate::collision::manifold::ContactManifold,
) {
    let (body_a_type, body_a_inv_mass, body_a_restitution, body_a_friction, inv_i_a_world) =
        if let Some(rb) = world.rigid_body.get(a) {
            let rot_a = world.transform.get(a).map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
            (rb.body_type, rb.inv_mass, rb.restitution, rb.friction, rb.inv_inertia_world(rot_a))
        } else {
            (RigidBodyType::Static, 0.0, 0.2, 0.5, Mat3::ZERO)
        };

    let (body_b_type, body_b_inv_mass, body_b_restitution, body_b_friction, inv_i_b_world) =
        if let Some(rb) = world.rigid_body.get(b) {
            let rot_b = world.transform.get(b).map(|t| t.rotation).unwrap_or(Quat::IDENTITY);
            (rb.body_type, rb.inv_mass, rb.restitution, rb.friction, rb.inv_inertia_world(rot_b))
        } else {
            (RigidBodyType::Static, 0.0, 0.2, 0.5, Mat3::ZERO)
        };

    let is_dynamic_a = body_a_type == RigidBodyType::Dynamic && body_a_inv_mass > 0.0;
    let is_dynamic_b = body_b_type == RigidBodyType::Dynamic && body_b_inv_mass > 0.0;

    if !is_dynamic_a && !is_dynamic_b {
        return;
    }

    let pos_a = world.global_transform_matrix(a).transform_point3(Vec3::ZERO);
    let pos_b = world.global_transform_matrix(b).transform_point3(Vec3::ZERO);

    let r_a = contact.point - pos_a;
    let r_b = contact.point - pos_b;

    // Contact velocities
    let (vel_a, ang_a) = if let Some(rb) = world.rigid_body.get(a) {
        (rb.velocity, rb.angular_velocity)
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };
    let (vel_b, ang_b) = if let Some(rb) = world.rigid_body.get(b) {
        (rb.velocity, rb.angular_velocity)
    } else {
        (Vec3::ZERO, Vec3::ZERO)
    };

    let vel_a_contact = if is_dynamic_a {
        vel_a + ang_a.cross(r_a)
    } else {
        vel_a
    };
    let vel_b_contact = if is_dynamic_b {
        vel_b + ang_b.cross(r_b)
    } else {
        vel_b
    };

    let rel_vel = vel_b_contact - vel_a_contact;
    let normal = contact.normal;
    let vel_along_normal = rel_vel.dot(normal);

    // Only resolve if bodies are approaching each other
    if vel_along_normal >= 0.0 {
        return;
    }

    let r_a_cross_n = r_a.cross(normal);
    let r_b_cross_n = r_b.cross(normal);

    let ang_term_a = if is_dynamic_a {
        r_a_cross_n.dot(inv_i_a_world * r_a_cross_n)
    } else {
        0.0
    };
    let ang_term_b = if is_dynamic_b {
        r_b_cross_n.dot(inv_i_b_world * r_b_cross_n)
    } else {
        0.0
    };

    let k_normal = body_a_inv_mass + body_b_inv_mass + ang_term_a + ang_term_b;
    if k_normal <= 1e-8 {
        return;
    }

    let restitution = body_a_restitution.min(body_b_restitution);
    let j_n = -(1.0 + restitution) * vel_along_normal / k_normal;
    let impulse_n = normal * j_n;

    // Calculate tangent friction impulse
    let mut impulse_t = Vec3::ZERO;
    let delta_v_a_n = if is_dynamic_a { -impulse_n * body_a_inv_mass } else { Vec3::ZERO };
    let delta_w_a_n = if is_dynamic_a { -inv_i_a_world * r_a.cross(impulse_n) } else { Vec3::ZERO };
    let delta_v_b_n = if is_dynamic_b { impulse_n * body_b_inv_mass } else { Vec3::ZERO };
    let delta_w_b_n = if is_dynamic_b { inv_i_b_world * r_b.cross(impulse_n) } else { Vec3::ZERO };

    let vel_a_after = vel_a_contact + delta_v_a_n + delta_w_a_n.cross(r_a);
    let vel_b_after = vel_b_contact + delta_v_b_n + delta_w_b_n.cross(r_b);
    let rel_vel_after = vel_b_after - vel_a_after;

    let tangent = rel_vel_after - normal * rel_vel_after.dot(normal);
    let tangent_len = tangent.length();
    if tangent_len > 1e-5 {
        let tangent_dir = tangent / tangent_len;
        let r_a_cross_t = r_a.cross(tangent_dir);
        let r_b_cross_t = r_b.cross(tangent_dir);

        let ang_term_t_a = if is_dynamic_a {
            r_a_cross_t.dot(inv_i_a_world * r_a_cross_t)
        } else {
            0.0
        };
        let ang_term_t_b = if is_dynamic_b {
            r_b_cross_t.dot(inv_i_b_world * r_b_cross_t)
        } else {
            0.0
        };

        let k_tangent = body_a_inv_mass + body_b_inv_mass + ang_term_t_a + ang_term_t_b;
        if k_tangent > 1e-8 {
            let j_t_raw = -rel_vel_after.dot(tangent_dir) / k_tangent;
            let friction = (body_a_friction * body_b_friction).sqrt();
            let max_friction = j_n * friction;
            let j_t = j_t_raw.clamp(-max_friction, max_friction);
            impulse_t = tangent_dir * j_t;
        }
    }

    let total_impulse = impulse_n + impulse_t;

    if is_dynamic_a && is_dynamic_b {
        if let Some([rb_a, rb_b]) = world.rigid_body.get_disjoint_mut([a, b]) {
            rb_a.velocity -= total_impulse * body_a_inv_mass;
            rb_a.angular_velocity -= inv_i_a_world * r_a.cross(total_impulse);
            rb_b.velocity += total_impulse * body_b_inv_mass;
            rb_b.angular_velocity += inv_i_b_world * r_b.cross(total_impulse);
        }
    } else if is_dynamic_a {
        if let Some(rb_a) = world.rigid_body.get_mut(a) {
            rb_a.velocity -= total_impulse * body_a_inv_mass;
            rb_a.angular_velocity -= inv_i_a_world * r_a.cross(total_impulse);
        }
    } else if is_dynamic_b {
        if let Some(rb_b) = world.rigid_body.get_mut(b) {
            rb_b.velocity += total_impulse * body_b_inv_mass;
            rb_b.angular_velocity += inv_i_b_world * r_b.cross(total_impulse);
        }
    }
}

fn resolve_rigid_body_position_penetration(
    world: &mut World,
    a: EntityId,
    b: EntityId,
    contact: &crate::collision::manifold::ContactManifold,
    iterations: usize,
) {
    let is_static_a = world.is_static(a);
    let is_static_b = world.is_static(b);

    if is_static_a && is_static_b {
        return;
    }

    let Some([trans_a, trans_b]) = world.transform.get_disjoint_mut([a, b]) else {
        return;
    };

    let mut sub_contact = *contact;
    sub_contact.depth = contact.depth / (iterations as f32);

    resolve_penetration_weighted(
        trans_a,
        is_static_a,
        trans_b,
        is_static_b,
        &sub_contact,
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
}

/// Simulates deformable soft bodies using Position-Based Dynamics (PBD) and mass-spring constraints.
pub fn step_soft_body_system(world: &mut World, dt: f32, config: &PhysicsConfig) {
    crate::physics::pbd_solver::SoftBodySystem::step(world, dt, config);
}
