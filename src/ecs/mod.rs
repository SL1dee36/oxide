//! Entity-Component-System (ECS) architecture.
//!
//! Provides a data-oriented [`World`](crate::ecs::World) storing decoupled components in generational [`slotmap`] tables.

pub mod components;
pub mod entity;
pub mod systems;
pub mod world;

pub use components::{Component, Name, NodeBehavior, Transform, Visible};
pub use entity::{Entity, EntityId};
pub use systems::{update_behaviors_system, update_hierarchy_system};
pub use world::{PendingGpuMeshCleanup, World};

/// Backwards-compatibility alias for [`World`].
pub type Scene = World;
/// Backwards-compatibility alias for [`EntityId`].
pub type NodeId = EntityId;
