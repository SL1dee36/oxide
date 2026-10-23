use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::core::context::Context;
use crate::spatial::transform::Transform;
use crate::text::text2d::Text2D;
use glam::Mat4;

/// Strongly-typed identifier for a node stored within a [`super::scene::Scene`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    /// Sentinel identifier representing an uninitialized or invalid node reference.
    pub const INVALID: Self = Self(usize::MAX);
}

/// Custom procedural behavior attached to a spatial node, invoked during scene processing steps.
pub trait NodeBehavior {
    /// Executed every frame during [`super::scene::Scene::update`].
    fn on_process(&mut self, _node: &mut SpatialNode, _ctx: &mut Context, _dt: f32) {}
}

/// Specific payload data represented by a spatial node.
pub enum NodeKind {
    /// Empty structural transform pivot or organizational grouping node.
    Empty,
    /// 3D polygon mesh.
    Mesh3D(Mesh3D),
    /// 2D planar quad in screen coordinates.
    Surface2D(Surface2D),
    /// 2D screen-space typography text element.
    Text2D(Text2D),
}

/// Element in the hierarchical scene graph containing transform, visual representation, and collider.
pub struct SpatialNode {
    /// Unique identifier in the scene.
    pub id: NodeId,
    /// Descriptive name for diagnostic identification.
    pub name: String,
    /// Identifier of parent node if attached to an existing hierarchy.
    pub parent: Option<NodeId>,
    /// Identifiers of child nodes attached to this node.
    pub children: Vec<NodeId>,
    /// Local transform relative to parent.
    pub transform: Transform,
    /// Cached world transformation matrix ($M_{\text{parent}} \cdot M_{\text{local}}$).
    pub global_transform: Mat4,
    /// Attached payload kind (geometry, surface, text, or empty).
    pub kind: NodeKind,
    /// Bounding volume collider for intersection testing and raycasting.
    pub collider: Option<Collider>,
    /// Controls whether node is submitted for rendering.
    pub visible: bool,
    /// Static obstacle flag: static nodes never receive penetration displacements during collision resolution.
    pub is_static: bool,
    /// Optional dynamic behavior script.
    pub behavior: Option<Box<dyn NodeBehavior>>,
}

impl SpatialNode {
    /// Creates a new spatial node.
    pub fn new(id: NodeId, name: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id,
            name: name.into(),
            parent: None,
            children: Vec::new(),
            transform: Transform::IDENTITY,
            global_transform: Mat4::IDENTITY,
            kind,
            collider: None,
            visible: true,
            is_static: false,
            behavior: None,
        }
    }

    /// Sets the local transform and initializes global transform.
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self.global_transform = transform.matrix();
        self
    }

    /// Returns the active world transformation matrix.
    #[inline]
    pub fn global_matrix(&self) -> Mat4 {
        if self.parent.is_none() {
            self.transform.matrix()
        } else {
            self.global_transform
        }
    }

    /// Configures the static obstacle flag.
    pub fn with_static(mut self, is_static: bool) -> Self {
        self.is_static = is_static;
        self
    }

    /// Attaches a collision volume.
    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }

    /// Attaches a custom procedural behavior component.
    pub fn with_behavior<B: NodeBehavior + 'static>(mut self, behavior: B) -> Self {
        self.behavior = Some(Box::new(behavior));
        self
    }

    /// Replaces or assigns a procedural behavior component.
    pub fn set_behavior<B: NodeBehavior + 'static>(&mut self, behavior: B) {
        self.behavior = Some(Box::new(behavior));
    }
}
