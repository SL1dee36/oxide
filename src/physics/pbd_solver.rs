use crate::collision::collider::Collider;
use crate::ecs::world::World;
use crate::physics::soft_body::SoftBody;
use crate::physics::systems::PhysicsConfig;
use glam::{Mat4, Vec3};

/// Metadata caching a static geometric collider for fast particle-boundary projection.
#[derive(Debug, Clone, Copy)]
pub struct StaticColliderInfo {
    pub collider: Collider,
    pub transform: Mat4,
    pub inv_transform: Mat4,
}

/// Position-Based Dynamics (PBD / XPBD) simulation subsystem for soft bodies and cloth.
pub struct SoftBodySystem;

impl SoftBodySystem {
    /// Steps all soft body components in `world` using Position-Based Dynamics.
    pub fn step(world: &mut World, dt: f32, config: &PhysicsConfig) {
        if dt <= 1e-6 {
            return;
        }

        // Cache static colliders in the world for boundary projections
        let static_colliders: Vec<StaticColliderInfo> = world
            .collider
            .keys()
            .filter(|&e| world.is_alive(e) && world.rigid_body.get(e).is_none_or(|rb| !rb.is_dynamic()))
            .filter_map(|e| {
                let collider = *world.collider.get(e)?;
                let transform = world.global_transform_matrix(e);
                let inv_transform = transform.inverse();
                Some(StaticColliderInfo {
                    collider,
                    transform,
                    inv_transform,
                })
            })
            .collect();

        for (entity, soft_body) in &mut world.soft_body {
            Self::step_single_soft_body(soft_body, dt, config, &static_colliders);

            // Synchronize deformed vertices to associated Mesh3D if present
            if let Some(mesh) = world.mesh_3d.get_mut(entity) {
                soft_body.sync_to_mesh(mesh);
            }
        }
    }

    /// Advances a single [`SoftBody`] by `dt` seconds across multiple sub-steps.
    pub fn step_single_soft_body(
        soft_body: &mut SoftBody,
        dt: f32,
        config: &PhysicsConfig,
        static_colliders: &[StaticColliderInfo],
    ) {
        let substeps = soft_body.substeps.max(1);
        let sub_dt = dt / substeps as f32;
        let damping = soft_body.damping;
        let compliance = soft_body.compliance;

        for _ in 0..substeps {
            // -------------------------------------------------------------
            // 1. Prediction Phase (Explicit Euler / Verlet position prediction)
            // -------------------------------------------------------------
            for p in &mut soft_body.particles {
                if p.inv_mass > 0.0 {
                    // Integrate external forces (gravity)
                    p.velocity += config.gravity * sub_dt;
                    p.velocity *= damping;
                    p.predicted_position = p.position + p.velocity * sub_dt;
                } else {
                    p.predicted_position = p.position;
                    p.velocity = Vec3::ZERO;
                }
            }

            // -------------------------------------------------------------
            // 2. Collision / Boundary Constraints (Pre-solve projections)
            // -------------------------------------------------------------
            for p in &mut soft_body.particles {
                if p.inv_mass == 0.0 {
                    continue;
                }

                for sc in static_colliders {
                    match sc.collider {
                        Collider::Sphere { radius } => {
                            let center = sc.transform.transform_point3(Vec3::ZERO);
                            let (scale, _, _) = sc.transform.to_scale_rotation_translation();
                            let eff_radius = radius * scale.x.abs().max(scale.y.abs()).max(scale.z.abs());

                            let diff = p.predicted_position - center;
                            let dist = diff.length();
                            if dist < eff_radius && dist > 1e-6 {
                                let normal = diff / dist;
                                p.predicted_position = center + normal * eff_radius;

                                // Dissipate inward velocity component
                                let v_dot_n = p.velocity.dot(normal);
                                if v_dot_n < 0.0 {
                                    p.velocity -= normal * v_dot_n;
                                }
                            }
                        }
                        Collider::Box3D { half_extents } => {
                            let p_local = sc.inv_transform.transform_point3(p.predicted_position);
                            let hx = half_extents.x;
                            let hy = half_extents.y;
                            let hz = half_extents.z;

                            if p_local.x.abs() < hx && p_local.y.abs() < hy && p_local.z.abs() < hz {
                                let dx = hx - p_local.x.abs();
                                let dy = hy - p_local.y.abs();
                                let dz = hz - p_local.z.abs();

                                let (corrected_local, local_normal) = if dx < dy && dx < dz {
                                    (
                                        Vec3::new(hx * p_local.x.signum(), p_local.y, p_local.z),
                                        Vec3::new(p_local.x.signum(), 0.0, 0.0),
                                    )
                                } else if dy < dz {
                                    (
                                        Vec3::new(p_local.x, hy * p_local.y.signum(), p_local.z),
                                        Vec3::new(0.0, p_local.y.signum(), 0.0),
                                    )
                                } else {
                                    (
                                        Vec3::new(p_local.x, p_local.y, hz * p_local.z.signum()),
                                        Vec3::new(0.0, 0.0, p_local.z.signum()),
                                    )
                                };

                                p.predicted_position = sc.transform.transform_point3(corrected_local);
                                let world_normal = sc.transform.transform_vector3(local_normal).normalize_or_zero();
                                let v_dot_n = p.velocity.dot(world_normal);
                                if v_dot_n < 0.0 {
                                    p.velocity -= world_normal * v_dot_n;
                                }
                            }
                        }
                        Collider::Box2D { half_extents } => {
                            let p_local = sc.inv_transform.transform_point3(p.predicted_position);
                            let hx = half_extents.x;
                            let hy = half_extents.y;

                            if p_local.x.abs() < hx && p_local.y.abs() < hy {
                                let dx = hx - p_local.x.abs();
                                let dy = hy - p_local.y.abs();

                                let mut corrected_local = p_local;
                                if dx < dy {
                                    corrected_local.x = hx * p_local.x.signum();
                                } else {
                                    corrected_local.y = hy * p_local.y.signum();
                                }
                                p.predicted_position = sc.transform.transform_point3(corrected_local);
                            }
                        }
                    }
                }
            }

            // -------------------------------------------------------------
            // 3. Constraint Iteration Phase (Sequential Impulse / Gauss-Seidel)
            // -------------------------------------------------------------
            let compliance_term = compliance / (sub_dt * sub_dt);

            for c in &soft_body.constraints {
                let i_a = c.particle_a;
                let i_b = c.particle_b;

                if i_a >= soft_body.particles.len() || i_b >= soft_body.particles.len() {
                    continue;
                }

                let w_a = soft_body.particles[i_a].inv_mass;
                let w_b = soft_body.particles[i_b].inv_mass;
                let w_sum = w_a + w_b;

                if w_sum <= 0.0 {
                    continue;
                }

                let p_a = soft_body.particles[i_a].predicted_position;
                let p_b = soft_body.particles[i_b].predicted_position;

                // Vector from i_a to i_b
                let d = p_b - p_a;
                let dist = d.length();

                if dist > 1e-6 {
                    // Constraint error C(x*) = d - d_0
                    let c_val = dist - c.rest_length;
                    // Lagrange multiplier correction scaling with XPBD compliance
                    let lambda = c_val / (w_sum + compliance_term);
                    let delta_x = (d / dist) * (lambda * c.stiffness);

                    // If stretched (dist > rest_length, c_val > 0), delta_x points toward i_b.
                    // i_a moves toward i_b (+ delta_x * w_a), and i_b moves toward i_a (- delta_x * w_b).
                    soft_body.particles[i_a].predicted_position += delta_x * w_a;
                    soft_body.particles[i_b].predicted_position -= delta_x * w_b;
                }
            }

            // -------------------------------------------------------------
            // 4. Velocity Update & Position Commit
            // -------------------------------------------------------------
            for p in &mut soft_body.particles {
                if p.inv_mass > 0.0 {
                    p.velocity = (p.predicted_position - p.position) / sub_dt;
                    p.old_position = p.position;
                    p.position = p.predicted_position;
                } else {
                    p.predicted_position = p.position;
                    p.velocity = Vec3::ZERO;
                }
            }
        }
    }
}
