use crate::collision::collider::{
    resolve_penetration_weighted, Aabb2D, Aabb3D, BoundingSphere, Collider,
};
use crate::collision::manifold::ContactManifold;
use crate::collision::ray::{Ray3D, RaycastHit};
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::spatial::transform::Transform;
use glam::{Mat4, Vec3};

/// Evaluates geometric intersection between two colliders in world space.
pub fn check_collider_intersection(
    col_a: &Collider,
    mat_a: &Mat4,
    col_b: &Collider,
    mat_b: &Mat4,
) -> Option<ContactManifold> {
    match (col_a, col_b) {
        (Collider::Box3D { half_extents: ha }, Collider::Box3D { half_extents: hb }) => {
            let aabb_a = Aabb3D::from_center_half_extents(Vec3::ZERO, *ha).transform(mat_a);
            let aabb_b = Aabb3D::from_center_half_extents(Vec3::ZERO, *hb).transform(mat_b);
            aabb_a.contact(&aabb_b)
        }
        (Collider::Sphere { radius: ra }, Collider::Sphere { radius: rb }) => {
            let pos_a = mat_a.transform_point3(Vec3::ZERO);
            let pos_b = mat_b.transform_point3(Vec3::ZERO);
            let (scale_a, _, _) = mat_a.to_scale_rotation_translation();
            let (scale_b, _, _) = mat_b.to_scale_rotation_translation();
            let max_scale_a = scale_a.x.abs().max(scale_a.y.abs()).max(scale_a.z.abs());
            let max_scale_b = scale_b.x.abs().max(scale_b.y.abs()).max(scale_b.z.abs());
            let sphere_a = BoundingSphere::new(pos_a, *ra * max_scale_a);
            let sphere_b = BoundingSphere::new(pos_b, *rb * max_scale_b);
            sphere_a.contact(&sphere_b)
        }
        (Collider::Box2D { half_extents: ha }, Collider::Box2D { half_extents: hb }) => {
            let (scale_a, _, trans_a) = mat_a.to_scale_rotation_translation();
            let (scale_b, _, trans_b) = mat_b.to_scale_rotation_translation();
            let aabb_a =
                Aabb2D::from_center_half_extents(trans_a.truncate(), *ha * scale_a.truncate());
            let aabb_b =
                Aabb2D::from_center_half_extents(trans_b.truncate(), *hb * scale_b.truncate());
            aabb_a.contact(&aabb_b)
        }
        (Collider::Box3D { half_extents: h }, Collider::Sphere { radius: r }) => {
            let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, *h).transform(mat_a);
            let pos_sphere = mat_b.transform_point3(Vec3::ZERO);
            let (scale_b, _, _) = mat_b.to_scale_rotation_translation();
            let max_scale_b = scale_b.x.abs().max(scale_b.y.abs()).max(scale_b.z.abs());
            let sphere = BoundingSphere::new(pos_sphere, *r * max_scale_b);
            aabb.contact_sphere(&sphere)
        }
        (Collider::Sphere { radius: r }, Collider::Box3D { half_extents: h }) => {
            let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, *h).transform(mat_b);
            let pos_sphere = mat_a.transform_point3(Vec3::ZERO);
            let (scale_a, _, _) = mat_a.to_scale_rotation_translation();
            let max_scale_a = scale_a.x.abs().max(scale_a.y.abs()).max(scale_a.z.abs());
            let sphere = BoundingSphere::new(pos_sphere, *r * max_scale_a);
            if let Some(mut contact) = aabb.contact_sphere(&sphere) {
                contact.normal = -contact.normal;
                Some(contact)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Evaluates geometric intersection between colliders of two entities in the world.
pub fn check_entity_intersection(world: &World, a: EntityId, b: EntityId) -> Option<ContactManifold> {
    if !world.is_alive(a) || !world.is_alive(b) || a == b {
        return None;
    }
    let col_a = world.collider.get(a)?;
    let col_b = world.collider.get(b)?;
    let mat_a = world.global_transform_matrix(a);
    let mat_b = world.global_transform_matrix(b);

    check_collider_intersection(col_a, &mat_a, col_b, &mat_b)
}

/// Kinematically separates two overlapping entities along the contact normal taking static bodies into account.
pub fn resolve_penetration_system(
    world: &mut World,
    a: EntityId,
    b: EntityId,
    contact: &ContactManifold,
) {
    if !world.is_alive(a) || !world.is_alive(b) || a == b {
        return;
    }

    let is_static_a = world.is_static(a);
    let is_static_b = world.is_static(b);

    let (Some(trans_a), Some(trans_b)) = (
        world.transform.get_mut(a).map(|t| t as *mut Transform),
        world.transform.get_mut(b).map(|t| t as *mut Transform),
    ) else {
        return;
    };

    unsafe {
        resolve_penetration_weighted(
            &mut *trans_a,
            is_static_a,
            &mut *trans_b,
            is_static_b,
            contact,
        );
    }

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

/// Casts a ray through all colliders in the world, returning the closest intersecting entity and hit details.
pub fn raycast_system(world: &World, ray: &Ray3D) -> Option<(EntityId, RaycastHit)> {
    let mut closest_hit: Option<(EntityId, RaycastHit)> = None;

    for (entity, collider) in &world.collider {
        if !world.is_visible(entity) {
            continue;
        }

        let mat = world.global_transform_matrix(entity);
        let hit = match collider {
            Collider::Box3D { half_extents } => {
                let aabb =
                    Aabb3D::from_center_half_extents(Vec3::ZERO, *half_extents).transform(&mat);
                aabb.raycast(ray)
            }
            Collider::Sphere { radius } => {
                let pos = mat.transform_point3(Vec3::ZERO);
                let (scale, _, _) = mat.to_scale_rotation_translation();
                let max_scale = scale.x.abs().max(scale.y.abs()).max(scale.z.abs());
                let sphere = BoundingSphere::new(pos, *radius * max_scale);
                sphere.raycast(ray)
            }
            _ => None,
        };

        if let Some(hit_info) = hit {
            if closest_hit
                .as_ref()
                .is_none_or(|(_, prev)| hit_info.distance < prev.distance)
            {
                closest_hit = Some((entity, hit_info));
            }
        }
    }

    closest_hit
}
