use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::core::context::Context;
use crate::spatial::transform::Transform;
use crate::text::text2d::Text2D;
use glam::Mat4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

impl NodeId {
    pub const INVALID: Self = Self(usize::MAX);
}

pub trait NodeBehavior {
    fn on_process(&mut self, _node: &mut SpatialNode, _ctx: &mut Context, _dt: f32) {}
}

pub enum NodeKind {
    Empty,
    Mesh3D(Mesh3D),
    Surface2D(Surface2D),
    Text2D(Text2D),
}

pub struct SpatialNode {
    pub id: NodeId,
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub transform: Transform,
    pub global_transform: Mat4,
    pub kind: NodeKind,
    pub collider: Option<Collider>,
    pub visible: bool,
    pub is_static: bool,
    pub behavior: Option<Box<dyn NodeBehavior>>,
}

impl SpatialNode {
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

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self.global_transform = transform.matrix();
        self
    }

    #[inline]
    pub fn global_matrix(&self) -> Mat4 {
        if self.parent.is_none() {
            self.transform.matrix()
        } else {
            self.global_transform
        }
    }

    pub fn with_static(mut self, is_static: bool) -> Self {
        self.is_static = is_static;
        self
    }

    pub fn with_collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }

    pub fn with_behavior<B: NodeBehavior + 'static>(mut self, behavior: B) -> Self {
        self.behavior = Some(Box::new(behavior));
        self
    }

    pub fn set_behavior<B: NodeBehavior + 'static>(&mut self, behavior: B) {
        self.behavior = Some(Box::new(behavior));
    }
}
