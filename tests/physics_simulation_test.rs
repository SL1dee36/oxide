use oxide::prelude::*;
use oxide::ffi::*;

#[test]
fn test_rigid_body_free_fall_gravity() {
    let mut world = World::new();
    let initial_y = 10.0;
    let entity = world.spawn_rigid_body(
        2.0,
        Transform::from_translation(Vec3::new(0.0, initial_y, 0.0)),
        Collider::sphere(0.5),
    );

    let dt = 0.05;
    for _ in 0..10 {
        world.step_physics(dt);
    }

    let rb = world.rigid_body(entity).expect("RigidBody must exist");
    let transform = world.transform(entity).expect("Transform must exist");

    // After 0.5s of gravity (g = -9.81 m/s^2), velocity y should be approx -4.9 m/s
    assert!(rb.velocity.y < -4.0);
    // Position y should have fallen well below initial position
    assert!(transform.translation.y < initial_y - 0.5);
}

#[test]
fn test_rigid_body_impulse_and_momentum() {
    let mut world = World::new();
    let mass = 2.5;
    let entity = world.spawn_rigid_body(
        mass,
        Transform::from_translation(Vec3::ZERO),
        Collider::sphere(1.0),
    );

    let rb = world.rigid_body_mut(entity).expect("RigidBody must exist");
    assert_eq!(rb.velocity, Vec3::ZERO);

    // Apply linear impulse along X axis
    let impulse = Vec3::new(10.0, 0.0, 0.0);
    rb.apply_impulse(impulse);

    // delta_v = impulse / mass = 10.0 / 2.5 = 4.0 m/s
    assert!((rb.velocity.x - 4.0).abs() < 1e-4);

    // Advance simulation
    world.step_physics(0.1);

    let transform = world.transform(entity).expect("Transform must exist");
    assert!(transform.translation.x > 0.3);
}

#[test]
fn test_rigid_body_collision_and_penetration_resolution() {
    let mut world = World::new();

    // Static floor box at y = 0 with height 2.0 (top surface at y = 1.0)
    let floor_entity = world.spawn();
    world.set_transform(floor_entity, Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
    world.collider.insert(floor_entity, Collider::box_3d(Vec3::new(20.0, 2.0, 20.0)));
    world.rigid_body.insert(floor_entity, RigidBody::new_static());

    // Dynamic ball dropping right onto the floor at y = 1.8 with radius 1.0 (penetrating 0.2m)
    let ball_entity = world.spawn_rigid_body(
        1.0,
        Transform::from_translation(Vec3::new(0.0, 1.8, 0.0)),
        Collider::sphere(1.0),
    );

    // Set initial downward velocity
    if let Some(rb) = world.rigid_body_mut(ball_entity) {
        rb.velocity = Vec3::new(0.0, -2.0, 0.0);
    }

    world.step_physics(0.016);

    let ball_rb = world.rigid_body(ball_entity).expect("Ball RigidBody must exist");
    let ball_tf = world.transform(ball_entity).expect("Ball Transform must exist");

    // The collision impulse should push velocity upward or zero out downward velocity
    assert!(ball_rb.velocity.y > -0.5);
    // Position should be corrected so ball bottom surface (center - radius) is near or above 1.0
    assert!(ball_tf.translation.y >= 1.95);
}

#[test]
fn test_soft_body_cloth_pbd_simulation() {
    let mut world = World::new();

    // Spawn a 2x2 meter cloth with 5x5 grid resolution
    let cloth_entity = world.spawn_soft_body_cloth(
        2.0,
        2.0,
        5,
        5,
        1.0,
        0.9,
        Transform::IDENTITY,
    );

    assert_eq!(world.soft_body(cloth_entity).unwrap().particles.len(), 25);
    assert!(!world.soft_body(cloth_entity).unwrap().constraints.is_empty());
    assert!(world.mesh_3d(cloth_entity).is_some());

    // Pin top two corners: particle (0, 0) and (4, 0) in grid (top edge is y = 0)
    let top_left = 0;
    let top_right = 4;
    {
        let sb = world.soft_body_mut(cloth_entity).unwrap();
        sb.particles[top_left].inv_mass = 0.0;
        sb.particles[top_right].inv_mass = 0.0;
    }

    let initial_center_y = world.soft_body(cloth_entity).unwrap().particles[12].position.y;

    // Simulate several time steps under gravity
    for _ in 0..20 {
        world.step_physics(0.016);
    }

    let sb = world.soft_body(cloth_entity).unwrap();
    let mesh = world.mesh_3d(cloth_entity).unwrap();

    // Pinned corners should not have fallen
    assert_eq!(sb.particles[top_left].position.y, 1.0);
    assert_eq!(sb.particles[top_right].position.y, 1.0);

    // Free center particle (12) must have sagged downward under gravity
    let current_center_y = sb.particles[12].position.y;
    assert!(current_center_y < initial_center_y);

    // Mesh vertices must be synced to particle positions
    assert_eq!(mesh.vertices.len(), sb.particles.len());
    for (i, p) in sb.particles.iter().enumerate() {
        let v_pos = Vec3::from_slice(&mesh.vertices[i].position);
        assert!((v_pos - p.position).length() < 1e-4);
    }
}

#[test]
fn test_ffi_physics_lifecycle() {
    unsafe {
        let scene = oxide_scene_create();
        assert!(!scene.is_null());

        let box_size = OxideVec3 { x: 2.0, y: 2.0, z: 2.0 };
        let box_tf = OxideTransform {
            translation: OxideVec3 { x: 0.0, y: 10.0, z: 0.0 },
            rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        let node_id = oxide_scene_spawn_rigid_body_box(
            scene,
            box_size,
            box_tf,
            5.0,  // mass
            0.5,  // restitution
            0.3,  // friction
            false // dynamic
        );
        assert_ne!(node_id, u64::MAX);

        let mut vel = OxideVec3 { x: 0.0, y: 0.0, z: 0.0 };
        assert!(oxide_scene_rigid_body_get_velocity(scene, node_id, &mut vel));
        assert_eq!(vel.y, 0.0);

        // Apply impulse
        oxide_scene_rigid_body_apply_impulse(scene, node_id, OxideVec3 { x: 15.0, y: 0.0, z: 0.0 });
        assert!(oxide_scene_rigid_body_get_velocity(scene, node_id, &mut vel));
        // 15.0 / 5.0 kg = 3.0 m/s
        assert!((vel.x - 3.0).abs() < 1e-4);

        // Step physics
        oxide_scene_step_physics(scene, 0.1);

        let mut post_tf = OxideTransform {
            translation: OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
        };
        assert!(oxide_scene_node_get_transform(scene, node_id, &mut post_tf));
        assert!(post_tf.translation.x > 0.2);
        assert!(post_tf.translation.y < 10.0);

        oxide_scene_destroy(scene);
    }
}

#[test]
fn test_soft_body_pbd_static_obstacle_collision() {
    let mut world = World::new();

    // Create a static obstacle floor box at y = 0.0 with size 10x2x10 (top surface at y = 1.0)
    let floor = world.spawn();
    world.set_transform(floor, Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
    world.collider.insert(floor, Collider::box_3d(Vec3::new(10.0, 2.0, 10.0)));
    world.rigid_body.insert(floor, RigidBody::new_static());

    // Spawn a small cloth hovering just above the floor at y = 2.0
    let cloth_entity = world.spawn_soft_body_cloth(
        1.0,
        1.0,
        3,
        3,
        0.5,
        1.0,
        Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
    );

    // Unpin all particles so it falls freely onto the floor
    {
        let sb = world.soft_body_mut(cloth_entity).unwrap();
        for p in &mut sb.particles {
            p.inv_mass = 1.0 / 0.5;
            p.mass = 0.5;
        }
    }

    // Step physics multiple times allowing it to fall and hit the floor
    for _ in 0..40 {
        world.step_physics(0.016);
    }

    let sb = world.soft_body(cloth_entity).unwrap();
    // Every particle must remain on or above the top surface (y >= 1.0 - epsilon)
    for p in &sb.particles {
        assert!(p.position.y >= 0.99, "Particle penetrated static floor: y = {}", p.position.y);
    }
}

#[test]
fn test_ffi_soft_body_particle_manipulation() {
    unsafe {
        let scene = oxide_scene_create();
        assert!(!scene.is_null());

        let cloth_id = oxide_scene_spawn_soft_body_cloth(
            scene,
            2.0,
            2.0,
            4,
            4,
            1.0,
            0.9,
            OxideTransform {
                translation: OxideVec3 { x: 0.0, y: 5.0, z: 0.0 },
                rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
                scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
            },
        );
        assert_ne!(cloth_id, u64::MAX);

        let mut particle_count: u32 = 0;
        assert!(oxide_scene_soft_body_get_particle_count(scene, cloth_id, &mut particle_count));
        assert_eq!(particle_count, 16); // 4x4 grid

        let mut pos = OxideVec3 { x: 0.0, y: 0.0, z: 0.0 };
        assert!(oxide_scene_soft_body_get_particle_position(scene, cloth_id, 0, &mut pos));

        // Pin particle 0
        assert!(oxide_scene_soft_body_set_particle_fixed(scene, cloth_id, 0, true));
        // Configure substeps and compliance
        assert!(oxide_scene_soft_body_set_substeps(scene, cloth_id, 8));
        assert!(oxide_scene_soft_body_set_compliance(scene, cloth_id, 0.001));

        // Step physics
        for _ in 0..10 {
            oxide_scene_step_physics(scene, 0.016);
        }

        // Pinned particle position must be unchanged
        let mut pinned_pos = OxideVec3 { x: 0.0, y: 0.0, z: 0.0 };
        assert!(oxide_scene_soft_body_get_particle_position(scene, cloth_id, 0, &mut pinned_pos));
        assert_eq!(pinned_pos.y, pos.y);

        oxide_scene_destroy(scene);
    }
}

#[test]
fn test_rigid_body_off_center_collision_induces_rotation() {
    let mut world = World::new();

    // Static obstacle box at y = 0.0 with size 2x2x2 (top surface at y = 1.0, centered at x = 0.0)
    let obstacle = world.spawn();
    world.set_transform(obstacle, Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)));
    world.collider.insert(obstacle, Collider::box_3d(Vec3::new(2.0, 2.0, 2.0)));
    world.rigid_body.insert(obstacle, RigidBody::new_static());

    // Dynamic box falling off-center at x = 0.8, y = 1.4 with size 1x1x1 (bottom at y = 0.9 penetrating at y = 1.0)
    let box_entity = world.spawn_rigid_body(
        2.0,
        Transform::from_translation(Vec3::new(0.8, 1.4, 0.0)),
        Collider::box_3d(Vec3::new(1.0, 1.0, 1.0)),
    );

    // Initial downward velocity, zero initial angular velocity
    if let Some(rb) = world.rigid_body_mut(box_entity) {
        rb.velocity = Vec3::new(0.0, -2.0, 0.0);
        assert_eq!(rb.angular_velocity, Vec3::ZERO);
    }

    world.step_physics(0.016);

    let rb = world.rigid_body(box_entity).expect("RigidBody must exist");
    assert!(
        rb.angular_velocity.length() > 0.0,
        "Off-center collision must induce angular velocity, got: {:?}",
        rb.angular_velocity
    );
}

#[test]
fn test_broadphase_many_bodies_performance_and_accuracy() {
    let mut world = World::new();

    // Spawn 200 non-intersecting bodies spaced 10.0 units apart along X and Z
    for i in 0..200 {
        let x = (i % 20) as f32 * 10.0;
        let z = (i / 20) as f32 * 10.0;
        world.spawn_rigid_body(
            1.0,
            Transform::from_translation(Vec3::new(x, 0.0, z)),
            Collider::box_3d(Vec3::ONE),
        );
    }

    // Spawn Pair 1: two colliding dynamic boxes at x = 1000.0 and 1000.6 (size 1.0, penetration = 0.4)
    let b1 = world.spawn_rigid_body(
        1.0,
        Transform::from_translation(Vec3::new(1000.0, 0.0, 0.0)),
        Collider::box_3d(Vec3::ONE),
    );
    let b2 = world.spawn_rigid_body(
        1.0,
        Transform::from_translation(Vec3::new(1000.6, 0.0, 0.0)),
        Collider::box_3d(Vec3::ONE),
    );

    // Spawn Pair 2: dynamic sphere colliding with static box at x = 2000.0 and 2000.7
    let s1 = world.spawn_rigid_body(
        1.0,
        Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)),
        Collider::sphere(0.5),
    );
    let static_box = world.spawn();
    world.set_transform(static_box, Transform::from_translation(Vec3::new(2000.7, 0.0, 0.0)));
    world.collider.insert(static_box, Collider::box_3d(Vec3::ONE));
    world.rigid_body.insert(static_box, RigidBody::new_static());

    // Direct Broadphase test: out of 204 bodies (20,706 potential pairs), broadphase must find exactly 2 candidate pairs!
    let mut bp = BroadphaseGrid::new();
    for (entity, collider) in &world.collider {
        let is_static = world.rigid_body.get(entity).is_none_or(|rb| !rb.is_dynamic());
        let mat = world.global_transform_matrix(entity);
        let aabb = collider.world_aabb(&mat);
        bp.push(entity, aabb, is_static);
    }
    let candidate_pairs = bp.compute_pairs();
    assert_eq!(
        candidate_pairs.len(),
        2,
        "Broadphase must cull 99.99% of pairs, leaving exactly 2 colliding pairs"
    );

    // Check physics simulation step performance and contact resolution
    let start = std::time::Instant::now();
    world.step_physics(0.016);
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 50,
        "Physics step with 204 bodies must take under 50ms, took {:?}",
        elapsed
    );

    // Dynamic boxes b1 and b2 should have bounced/separated
    let tf1 = world.transform(b1).unwrap();
    let tf2 = world.transform(b2).unwrap();
    assert!(
        tf2.translation.x > tf1.translation.x,
        "Bodies b1 and b2 must separate properly"
    );

    // Dynamic sphere s1 should have pushed back from static obstacle
    let tf_s1 = world.transform(s1).unwrap();
    assert!(
        tf_s1.translation.x < 2000.0,
        "Sphere must be displaced away from static obstacle"
    );
}
