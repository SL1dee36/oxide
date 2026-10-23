use oxide::prelude::*;

#[test]
fn test_transform_hierarchy_propagation() {
    let mut world = World::new();

    let parent = world.spawn();
    world.set_name(parent, "Parent");
    world.set_transform(parent, Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)));

    let child = world.spawn();
    world.set_name(child, "Child");
    world.set_transform(child, Transform::from_translation(Vec3::new(5.0, 2.0, 0.0)));

    world.attach_child(parent, child);
    update_hierarchy_system(&mut world);

    let child_global = world.global_transform(child).expect("Child global transform must exist");
    let expected_pos = Vec3::new(15.0, 2.0, 0.0);
    let actual_pos = child_global.transform_point3(Vec3::ZERO);

    assert!((actual_pos - expected_pos).length() < 1e-4);
}

#[test]
fn test_ecs_component_insertion_and_retrieval() {
    let mut world = World::new();
    let entity = world.spawn();

    // Test generic insert & get
    world.insert(entity, Name("TestEntity".to_string()));
    assert_eq!(world.get::<Name>(entity).unwrap().0, "TestEntity");
    assert_eq!(world.name(entity).unwrap(), "TestEntity");

    world.insert(entity, Visible(false));
    assert!(world.has::<Visible>(entity));
    assert!(!world.is_visible(entity));

    // Test component removal
    let removed = world.remove::<Visible>(entity);
    assert!(removed.is_some());
    assert!(!world.has::<Visible>(entity));
    assert!(world.is_visible(entity));
}

#[test]
fn test_aabb_collision_and_contact() {
    let box_a = Aabb3D::from_center_half_extents(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(1.0));
    let box_b = Aabb3D::from_center_half_extents(Vec3::new(1.5, 0.0, 0.0), Vec3::splat(1.0));

    assert!(box_a.intersects(&box_b));

    let contact = box_a.contact(&box_b).expect("Contact manifold expected");
    assert!((contact.depth - 0.5).abs() < 1e-4);
    assert_eq!(contact.normal, Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_sphere_collision_and_contact() {
    let s1 = BoundingSphere::new(Vec3::new(0.0, 0.0, 0.0), 1.0);
    let s2 = BoundingSphere::new(Vec3::new(0.0, 1.5, 0.0), 1.0);

    assert!(s1.intersects(&s2));

    let contact = s1.contact(&s2).expect("Contact manifold expected");
    assert!((contact.depth - 0.5).abs() < 1e-4);
    assert_eq!(contact.normal, Vec3::new(0.0, 1.0, 0.0));
}

#[test]
fn test_penetration_resolution() {
    let mut t_a = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    let mut t_b = Transform::from_translation(Vec3::new(1.0, 0.0, 0.0));

    let contact = ContactManifold::new(Vec3::new(1.0, 0.0, 0.0), 0.5, Vec3::new(0.75, 0.0, 0.0));
    resolve_penetration(&mut t_a, &mut t_b, &contact);

    assert!((t_a.translation.x - (-0.25)).abs() < 1e-4);
    assert!((t_b.translation.x - 1.25).abs() < 1e-4);
}

#[test]
fn test_raycast_against_volumes() {
    let aabb = Aabb3D::from_center_half_extents(Vec3::new(0.0, 0.0, 5.0), Vec3::splat(1.0));
    let ray = Ray3D::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));

    let hit = aabb.raycast(&ray).expect("Ray should hit AABB");
    assert!((hit.distance - 4.0).abs() < 1e-4);
    assert_eq!(hit.normal, Vec3::new(0.0, 0.0, -1.0));

    let sphere = BoundingSphere::new(Vec3::new(0.0, 0.0, 10.0), 2.0);
    let hit_sphere = sphere.raycast(&ray).expect("Ray should hit Sphere");
    assert!((hit_sphere.distance - 8.0).abs() < 1e-4);
}

#[test]
fn test_box_sphere_contact() {
    let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
    let sphere = BoundingSphere::new(Vec3::new(1.5, 0.0, 0.0), 1.0);

    let contact = aabb.contact_sphere(&sphere).expect("Box and Sphere must intersect");
    assert!((contact.depth - 0.5).abs() < 1e-4);
    assert_eq!(contact.normal, Vec3::new(1.0, 0.0, 0.0));
}

#[test]
fn test_static_obstacle_penetration_resolution() {
    let mut wall_transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    let mut agent_transform = Transform::from_translation(Vec3::new(1.5, 0.0, 0.0));

    let contact = ContactManifold::new(Vec3::new(1.0, 0.0, 0.0), 0.5, Vec3::new(1.0, 0.0, 0.0));
    // Wall is static (true), Agent is dynamic (false)
    resolve_penetration_weighted(&mut wall_transform, true, &mut agent_transform, false, &contact);

    // Wall must not move at all
    assert_eq!(wall_transform.translation, Vec3::ZERO);
    // Agent must take full displacement (+0.5)
    assert!((agent_transform.translation.x - 2.0).abs() < 1e-4);
}

#[test]
fn test_sphere_scale_collision() {
    let mut world = World::new();

    // Sphere A has radius 1.0, but scale 2.0 (effective radius 2.0)
    let s_a = world.spawn();
    world.set_name(s_a, "SphereA");
    world.set_transform(
        s_a,
        Transform {
            translation: Vec3::ZERO,
            rotation: glam::Quat::IDENTITY,
            scale: Vec3::splat(2.0),
        },
    );
    world.insert(s_a, Collider::sphere(1.0));

    // Sphere B has radius 1.0, scale 1.0, at translation (2.5, 0.0, 0.0)
    let s_b = world.spawn();
    world.set_name(s_b, "SphereB");
    world.set_transform(s_b, Transform::from_translation(Vec3::new(2.5, 0.0, 0.0)));
    world.insert(s_b, Collider::sphere(1.0));

    world.update_transforms();

    let contact = world.check_intersection(s_a, s_b).expect("Scaled sphere must intersect");
    assert!((contact.depth - 0.5).abs() < 1e-4);
    assert_eq!(contact.normal, Vec3::new(1.0, 0.0, 0.0));

    // Also verify raycast accounts for sphere scaling
    let ray = Ray3D::new(Vec3::new(0.0, 0.0, -10.0), Vec3::new(0.0, 0.0, 1.0));
    let hit = world.raycast(&ray).expect("Ray should hit scaled sphere");
    // Center at (0, 0, 0), effective radius 2.0 -> hit at z = -2.0 -> distance = 8.0
    assert!((hit.1.distance - 8.0).abs() < 1e-4);
}

#[test]
fn test_despawn_node() {
    let mut world = World::new();

    let parent = world.spawn();
    world.set_name(parent, "Parent");
    world.set_transform(parent, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)));

    let child_1 = world.spawn();
    world.set_name(child_1, "Child1");

    let child_2 = world.spawn();
    world.set_name(child_2, "Child2");

    world.attach_child(parent, child_1);
    world.attach_child(parent, child_2);

    assert_eq!(world.children(parent).unwrap().len(), 2);
    assert!(world.is_alive(child_1));
    assert!(world.is_alive(child_2));

    // Despawn child_1
    assert!(world.despawn(child_1));
    assert!(!world.is_alive(child_1));
    assert!(!world.children(parent).unwrap().contains(&child_1));
    assert_eq!(world.children(parent).unwrap().len(), 1);

    // Despawning again returns false
    assert!(!world.despawn(child_1));

    // Despawn parent should recursively despawn child_2
    assert!(world.despawn(parent));
    assert!(!world.is_alive(parent));
    assert!(!world.is_alive(child_2));

    // Invalid / out-of-bounds entity returns false
    assert!(!world.despawn(Entity::from_ffi(9999)));

    // Collision check between despawned entities gracefully returns None without panicking
    assert!(world.check_intersection(parent, child_1).is_none());

    // Penetration resolution between despawned entities is safe no-op
    let dummy_contact = ContactManifold::new(Vec3::X, 1.0, Vec3::ZERO);
    world.resolve_penetration(parent, child_1, &dummy_contact);
}
