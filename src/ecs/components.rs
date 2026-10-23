use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::core::context::Context;
use crate::ecs::entity::EntityId;
use crate::ecs::world::World;
use crate::physics::rigid_body::RigidBody;
use crate::physics::soft_body::SoftBody;
pub use crate::spatial::transform::Transform;
use crate::text::text2d::Text2D;
use glam::Mat4;

/// Rendering visibility toggle component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Visible(pub bool);

impl Default for Visible {
    fn default() -> Self {
        Self(true)
    }
}

/// Diagnostic name string component.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Name(pub String);

impl<S: Into<String>> From<S> for Name {
    fn from(s: S) -> Self {
        Self(s.into())
    }
}

/// Custom procedural behavior attached to an entity.
pub trait NodeBehavior: 'static {
    /// Executed every frame during [`World::update`].
    fn on_process(&mut self, entity: EntityId, world: &mut World, ctx: &mut Context, dt: f32);
}

/// Trait defining decoupled component storage dispatch for generic ECS operations on [`World`].
pub trait Component: 'static + Sized {
    /// Retrieves an immutable reference to this component on `entity` if present.
    fn get(world: &World, entity: EntityId) -> Option<&Self>;
    /// Retrieves a mutable reference to this component on `entity` if present.
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self>;
    /// Inserts or replaces this component on `entity`.
    fn insert(world: &mut World, entity: EntityId, component: Self);
    /// Removes and returns this component from `entity` if present.
    fn remove(world: &mut World, entity: EntityId) -> Option<Self>;
}

impl Component for Transform {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.transform.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.transform.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.transform.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.transform.remove(entity)
    }
}

impl Component for Mat4 {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.global_transform.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.global_transform.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.global_transform.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.global_transform.remove(entity)
    }
}

impl Component for Visible {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.visible.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.visible.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.visible.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.visible.remove(entity)
    }
}

impl Component for Collider {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.collider.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.collider.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.collider.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.collider.remove(entity)
    }
}

impl Component for RigidBody {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.rigid_body.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.rigid_body.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.rigid_body.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.rigid_body.remove(entity)
    }
}

impl Component for SoftBody {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.soft_body.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.soft_body.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.soft_body.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.soft_body.remove(entity)
    }
}

impl Component for Mesh3D {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.mesh_3d.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.mesh_3d.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.mesh_3d.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.mesh_3d.remove(entity)
    }
}

impl Component for Surface2D {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.surface_2d.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.surface_2d.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.surface_2d.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.surface_2d.remove(entity)
    }
}

impl Component for Text2D {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.text_2d.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.text_2d.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.text_2d.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.text_2d.remove(entity)
    }
}

impl Component for Name {
    fn get(world: &World, entity: EntityId) -> Option<&Self> {
        world.names.get(entity)
    }
    fn get_mut(world: &mut World, entity: EntityId) -> Option<&mut Self> {
        world.names.get_mut(entity)
    }
    fn insert(world: &mut World, entity: EntityId, component: Self) {
        world.names.insert(entity, component);
    }
    fn remove(world: &mut World, entity: EntityId) -> Option<Self> {
        world.names.remove(entity)
    }
}
