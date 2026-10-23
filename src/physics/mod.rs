//! Custom high-performance 2D/3D physics engine and simulation solvers.
//!
//! Features 6-DOF rigid bodies, Position-Based Dynamics (PBD) soft bodies, mass-spring lattices,
//! velocity impulses, Coulomb friction, and constraint relaxation.

pub mod pbd_solver;
pub mod rigid_body;
pub mod soft_body;
pub mod systems;

pub use pbd_solver::SoftBodySystem;
pub use rigid_body::{RigidBody, RigidBodyType};
pub use soft_body::{DistanceConstraint, SoftBody, SoftParticle, SpringConstraint};
pub use systems::{
    step_physics_system, step_physics_system_with_config, step_rigid_body_system,
    step_soft_body_system, PhysicsConfig,
};
