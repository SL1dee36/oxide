use oxide::prelude::*;

#[test]
fn test_transform_hierarchy_propagation() {
    let mut scene = Scene::new();

    let parent_id = scene.spawn_node(
        "Parent",
        NodeKind::Empty,
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        None,
    );

    let child_id = scene.spawn_node(
        "Child",
        NodeKind::Empty,
        Transform::from_translation(Vec3::new(5.0, 2.0, 0.0)),
        None,
    );

    scene.attach_child(parent_id, child_id);
    scene.update_transforms();

    let child_global = scene.node(child_id).global_transform;
    let expected_pos = Vec3::new(15.0, 2.0, 0.0);
    let actual_pos = child_global.transform_point3(Vec3::ZERO);

    assert!((actual_pos - expected_pos).length() < 1e-4);
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
