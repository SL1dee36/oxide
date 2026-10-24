//! Geometric collision detection, bounding volume hierarchies, contact manifolds, and raycasting.

pub mod broadphase;
pub mod collider;
pub mod manifold;
pub mod ray;
pub mod systems;

pub use broadphase::{BroadphaseGrid, BroadphasePair, BroadphaseProxy};
pub use collider::{
    resolve_penetration, resolve_penetration_weighted, Aabb2D, Aabb3D, BoundingSphere, Collider,
    Obb3D,
};
pub use manifold::ContactManifold;
pub use ray::{Ray3D, RaycastHit};
pub use systems::{
    check_collider_intersection, check_entity_intersection, raycast_system,
    resolve_penetration_system,
};
