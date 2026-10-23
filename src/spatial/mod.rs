//! Spatial scene hierarchy, node graph, camera representations, and TRS transformations.

pub mod camera;
pub mod node;
pub mod scene;
pub mod transform;

pub use camera::{Camera, Projection};
pub use node::{NodeBehavior, NodeId, NodeKind, SpatialNode};
pub use scene::Scene;
pub use transform::Transform;
