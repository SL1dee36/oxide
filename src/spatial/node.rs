//! Legacy node identification types and behavior interfaces.

pub use crate::ecs::components::NodeBehavior;
pub use crate::ecs::entity::Entity;

/// Backwards-compatibility alias for [`Entity`].
pub type NodeId = Entity;
