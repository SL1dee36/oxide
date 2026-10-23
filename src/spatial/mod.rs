//! Spatial scene hierarchy, camera models, and TRS transformations.

pub mod camera;
pub mod node;
pub mod scene;
pub mod systems;
pub mod transform;

pub use camera::{Camera, Projection};
pub use node::{NodeBehavior, NodeId};
pub use scene::Scene;
pub use systems::update_hierarchy_system;
pub use transform::Transform;
