use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::{resolve_penetration_weighted, Aabb2D, Aabb3D, BoundingSphere, Collider};
use crate::collision::manifold::ContactManifold;
use crate::collision::ray::{Ray3D, RaycastHit};
use crate::core::context::Context;
use crate::renderer::pipeline_2d::Pipeline2D;
use crate::renderer::pipeline_3d::{Light3D, Pipeline3D};
use crate::spatial::camera::Camera;
use crate::spatial::node::{NodeId, NodeKind, SpatialNode};
use crate::spatial::transform::Transform;
use crate::text::atlas::FontAtlas;
use crate::text::font::Font;
use crate::text::pipeline_text::PipelineText;
use crate::text::text2d::Text2D;
use glam::{Mat4, Vec2, Vec3};
use std::sync::Arc;

/// Hierarchical scene graph container coordinating spatial nodes, cameras, lights, and rendering passes.
pub struct Scene {
    nodes: Vec<Option<SpatialNode>>,
    /// Active viewing camera.
    pub camera: Camera,
    /// Active 3D directional light source.
    pub light: Light3D,
    pipeline_3d: Option<Pipeline3D>,
    pipeline_2d: Option<Pipeline2D>,
    pipeline_text: Option<PipelineText>,
    font_atlas: Option<Arc<FontAtlas>>,
}

impl Scene {
    /// Creates an empty scene graph with default camera and directional light.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            camera: Camera::default(),
            light: Light3D::default(),
            pipeline_3d: None,
            pipeline_2d: None,
            pipeline_text: None,
            font_atlas: None,
        }
    }

    /// Replaces the active viewing camera.
    pub fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    /// Returns an immutable reference to the active camera.
    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    /// Returns a mutable reference to the active camera.
    pub fn camera_mut(&mut self) -> &mut Camera {
        &mut self.camera
    }

    /// Replaces the directional lighting configuration.
    pub fn set_light(&mut self, light: Light3D) {
        self.light = light;
    }

    /// Returns an immutable reference to directional lighting.
    pub fn light(&self) -> &Light3D {
        &self.light
    }

    /// Returns a mutable reference to directional lighting.
    pub fn light_mut(&mut self) -> &mut Light3D {
        &mut self.light
    }

    /// Instantiates a generic node into the scene hierarchy and returns its unique [`NodeId`].
    pub fn spawn_node(
        &mut self,
        name: impl Into<String>,
        kind: NodeKind,
        transform: Transform,
        collider: Option<Collider>,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        let mut node = SpatialNode::new(id, name, kind)
            .with_transform(transform);
        if let Some(col) = collider {
            node = node.with_collider(col);
        }
        self.nodes.push(Some(node));
        id
    }

    /// Spawns a 3D polygon mesh node with an attached collider.
    pub fn spawn_mesh_3d(
        &mut self,
        mesh: Mesh3D,
        transform: Transform,
        collider: Collider,
    ) -> NodeId {
        self.spawn_node("Mesh3D", NodeKind::Mesh3D(mesh), transform, Some(collider))
    }

    /// Spawns an immobile 3D mesh node with `is_static = true`.
    pub fn spawn_static_mesh_3d(
        &mut self,
        mesh: Mesh3D,
        transform: Transform,
        collider: Collider,
    ) -> NodeId {
        let id = self.spawn_node("StaticMesh3D", NodeKind::Mesh3D(mesh), transform, Some(collider));
        self.node_mut(id).is_static = true;
        id
    }

    /// Spawns a 2D planar quad surface in screen coordinates.
    pub fn spawn_surface_2d(
        &mut self,
        surface: Surface2D,
        transform: Transform,
        collider: Collider,
    ) -> NodeId {
        self.spawn_node("Surface2D", NodeKind::Surface2D(surface), transform, Some(collider))
    }

    /// Assigns a shared font atlas for typography rendering in this scene.
    pub fn set_font_atlas(&mut self, atlas: Arc<FontAtlas>) {
        self.font_atlas = Some(atlas);
    }

    /// Returns the active shared font atlas if assigned.
    pub fn font_atlas(&self) -> Option<Arc<FontAtlas>> {
        self.font_atlas.clone()
    }

    /// Spawns a 2D screen-space typography text element.
    pub fn spawn_text_2d(
        &mut self,
        text: Text2D,
        transform: Transform,
    ) -> NodeId {
        self.spawn_node("Text2D", NodeKind::Text2D(text), transform, None)
    }

    /// Attaches `child_id` under `parent_id` in the transform tree.
    pub fn attach_child(&mut self, parent_id: NodeId, child_id: NodeId) {
        if parent_id.0 >= self.nodes.len() || child_id.0 >= self.nodes.len() {
            return;
        }

        if let Some(child) = self.nodes[child_id.0].as_mut() {
            child.parent = Some(parent_id);
        }

        if let Some(parent) = self.nodes[parent_id.0].as_mut() {
            if !parent.children.contains(&child_id) {
                parent.children.push(child_id);
            }
        }
    }

    /// Retrieves an immutable reference to the node with identifier `id`.
    pub fn node(&self, id: NodeId) -> &SpatialNode {
        self.nodes
            .get(id.0)
            .and_then(|opt| opt.as_ref())
            .expect("Invalid NodeId accessed in Scene")
    }

    /// Retrieves a mutable reference to the node with identifier `id`.
    pub fn node_mut(&mut self, id: NodeId) -> &mut SpatialNode {
        self.nodes
            .get_mut(id.0)
            .and_then(|opt| opt.as_mut())
            .expect("Invalid NodeId accessed in Scene")
    }

    /// Traverses the scene hierarchy and recalculates world transformation matrices for all nodes.
    pub fn update_transforms(&mut self) {
        let root_ids: Vec<NodeId> = self
            .nodes
            .iter()
            .flatten()
            .filter(|n| n.parent.is_none())
            .map(|n| n.id)
            .collect();

        for root_id in root_ids {
            self.propagate_transform(root_id, Mat4::IDENTITY);
        }
    }

    fn propagate_transform(&mut self, current_id: NodeId, parent_global: Mat4) {
        let (global, children) = if let Some(node) = self.nodes[current_id.0].as_mut() {
            let global = parent_global * node.transform.matrix();
            node.global_transform = global;
            (global, node.children.clone())
        } else {
            return;
        };

        for child_id in children {
            self.propagate_transform(child_id, global);
        }
    }

    /// Updates hierarchical transforms and runs attached node behaviors with delta time `dt`.
    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        self.update_transforms();

        for i in 0..self.nodes.len() {
            if let Some(node) = self.nodes[i].as_mut() {
                if let Some(mut behavior) = node.behavior.take() {
                    behavior.on_process(node, ctx, dt);
                    if let Some(current_node) = self.nodes[i].as_mut() {
                        current_node.behavior = Some(behavior);
                    }
                }
            }
        }
    }

    /// Executes the multi-pass frame rendering pipeline (3D meshes, 2D surfaces, and 2D text overlay).
    pub fn render(&mut self, ctx: &mut Context) {
        self.update_transforms();

        if self.pipeline_3d.is_none() {
            self.pipeline_3d = Pipeline3D::new(ctx.gl).ok();
        }
        if self.pipeline_2d.is_none() {
            self.pipeline_2d = Pipeline2D::new(ctx.gl).ok();
        }

        // Phase 1: 3D Render Pass
        if let Some(ref mut p3d) = self.pipeline_3d {
            p3d.light = self.light;
            let view = self.camera.view_matrix();
            let proj = self.camera.projection_matrix(ctx.aspect_ratio());
            p3d.begin(ctx.gl, &view, &proj);

            for node in self.nodes.iter_mut().flatten() {
                if !node.visible {
                    continue;
                }

                if let NodeKind::Mesh3D(ref mut mesh) = node.kind {
                    let _ = mesh.ensure_gpu_buffers(ctx.gl);
                    if let (Some(vao), Some(ebo)) = (&mesh.vao, &mesh.ebo) {
                        p3d.draw(
                            ctx.gl,
                            &node.global_transform,
                            mesh.color,
                            mesh.emissive,
                            None,
                            vao,
                            ebo,
                        );
                    }
                }
            }
            p3d.end(ctx.gl);
        }

        // Phase 2: 2D Render Pass (Overlay)
        if let Some(ref mut p2d) = self.pipeline_2d {
            p2d.begin(ctx.gl, ctx.width as f32, ctx.height as f32);

            for node in self.nodes.iter().flatten() {
                if !node.visible {
                    continue;
                }

                if let NodeKind::Surface2D(ref surface) = node.kind {
                    let pos = node.transform.translation;
                    let scale = Vec3::new(
                        node.transform.scale.x * surface.size.x,
                        node.transform.scale.y * surface.size.y,
                        1.0,
                    );
                    let model_2d = Mat4::from_scale_rotation_translation(
                        scale,
                        node.transform.rotation,
                        Vec3::new(pos.x, pos.y, 0.0),
                    );
                    p2d.draw(ctx.gl, &model_2d, surface.color, None);
                }
            }
            p2d.end(ctx.gl);
        }

        // Phase 3: Text Render Pass (Overlay)
        if self.pipeline_text.is_none() {
            self.pipeline_text = PipelineText::new(ctx.gl).ok();
        }
        if self.font_atlas.is_none() {
            if let Ok(font) = Font::default_font() {
                if let Ok(atlas) = FontAtlas::new(ctx.gl, font, 24.0) {
                    self.font_atlas = Some(Arc::new(atlas));
                }
            }
        }

        if let (Some(ref mut ptext), Some(ref atlas)) = (&mut self.pipeline_text, &self.font_atlas) {
            ptext.begin(ctx.gl, ctx.width as f32, ctx.height as f32);

            for node in self.nodes.iter().flatten() {
                if !node.visible {
                    continue;
                }

                if let NodeKind::Text2D(ref text) = node.kind {
                    let pos = Vec2::new(node.transform.translation.x, node.transform.translation.y);
                    let scale = text.scale * node.transform.scale.x;
                    let _ = ptext.draw_text(ctx.gl, atlas, &text.content, pos, text.color, scale);
                }
            }
            ptext.end(ctx.gl);
        }
    }

    /// Evaluates geometric intersection between colliders of two nodes.
    pub fn check_intersection(&self, a_id: NodeId, b_id: NodeId) -> Option<ContactManifold> {
        let node_a = self.node(a_id);
        let node_b = self.node(b_id);

        let col_a = node_a.collider.as_ref()?;
        let col_b = node_b.collider.as_ref()?;

        let mat_a = node_a.global_matrix();
        let mat_b = node_b.global_matrix();

        match (col_a, col_b) {
            (Collider::Box3D { half_extents: ha }, Collider::Box3D { half_extents: hb }) => {
                let aabb_a = Aabb3D::from_center_half_extents(Vec3::ZERO, *ha).transform(&mat_a);
                let aabb_b = Aabb3D::from_center_half_extents(Vec3::ZERO, *hb).transform(&mat_b);
                aabb_a.contact(&aabb_b)
            }
            (Collider::Sphere { radius: ra }, Collider::Sphere { radius: rb }) => {
                let pos_a = mat_a.transform_point3(Vec3::ZERO);
                let pos_b = mat_b.transform_point3(Vec3::ZERO);
                let sphere_a = BoundingSphere::new(pos_a, *ra);
                let sphere_b = BoundingSphere::new(pos_b, *rb);
                sphere_a.contact(&sphere_b)
            }
            (Collider::Box2D { half_extents: ha }, Collider::Box2D { half_extents: hb }) => {
                let pos_a = node_a.transform.translation.truncate();
                let scale_a = node_a.transform.scale.truncate();
                let pos_b = node_b.transform.translation.truncate();
                let scale_b = node_b.transform.scale.truncate();
                let aabb_a = Aabb2D::from_center_half_extents(pos_a, *ha * scale_a);
                let aabb_b = Aabb2D::from_center_half_extents(pos_b, *hb * scale_b);
                aabb_a.contact(&aabb_b)
            }
            (Collider::Box3D { half_extents: h }, Collider::Sphere { radius: r }) => {
                let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, *h).transform(&mat_a);
                let pos_sphere = mat_b.transform_point3(Vec3::ZERO);
                let sphere = BoundingSphere::new(pos_sphere, *r);
                aabb.contact_sphere(&sphere)
            }
            (Collider::Sphere { radius: r }, Collider::Box3D { half_extents: h }) => {
                let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, *h).transform(&mat_b);
                let pos_sphere = mat_a.transform_point3(Vec3::ZERO);
                let sphere = BoundingSphere::new(pos_sphere, *r);
                if let Some(mut contact) = aabb.contact_sphere(&sphere) {
                    contact.normal = -contact.normal;
                    Some(contact)
                } else {
                    None
                }
            }
            (Collider::Box3D { half_extents: h3d }, Collider::Box2D { half_extents: h2d }) => {
                let pos_3d = mat_a.transform_point3(Vec3::ZERO);
                let pos_2d = Vec3::new(node_b.transform.translation.x, node_b.transform.translation.y, 0.0);
                let aabb_a = Aabb3D::from_center_half_extents(pos_3d, *h3d);
                let aabb_b = Aabb3D::from_center_half_extents(pos_2d, Vec3::new(h2d.x, h2d.y, 10.0));
                aabb_a.contact(&aabb_b)
            }
            (Collider::Box2D { half_extents: h2d }, Collider::Box3D { half_extents: h3d }) => {
                let pos_2d = Vec3::new(node_a.transform.translation.x, node_a.transform.translation.y, 0.0);
                let pos_3d = mat_b.transform_point3(Vec3::ZERO);
                let aabb_a = Aabb3D::from_center_half_extents(pos_2d, Vec3::new(h2d.x, h2d.y, 10.0));
                let aabb_b = Aabb3D::from_center_half_extents(pos_3d, *h3d);
                aabb_a.contact(&aabb_b)
            }
            _ => None,
        }
    }

    /// Kinematically separates two overlapping nodes along the contact normal taking `is_static` into account.
    pub fn resolve_penetration(&mut self, a_id: NodeId, b_id: NodeId, contact: &ContactManifold) {
        if a_id == b_id {
            return;
        }
        let (node_a, node_b) = if a_id.0 < b_id.0 {
            let (left, right) = self.nodes.split_at_mut(b_id.0);
            (
                left[a_id.0].as_mut().unwrap(),
                right[0].as_mut().unwrap(),
            )
        } else {
            let (left, right) = self.nodes.split_at_mut(a_id.0);
            (
                right[0].as_mut().unwrap(),
                left[b_id.0].as_mut().unwrap(),
            )
        };

        resolve_penetration_weighted(
            &mut node_a.transform,
            node_a.is_static,
            &mut node_b.transform,
            node_b.is_static,
            contact,
        );
        node_a.global_transform = node_a.transform.matrix();
        node_b.global_transform = node_b.transform.matrix();
    }

    /// Casts a ray through the scene, returning the closest intersecting node and hit information.
    pub fn raycast(&self, ray: &Ray3D) -> Option<(NodeId, RaycastHit)> {
        let mut closest_hit: Option<(NodeId, RaycastHit)> = None;

        for node in self.nodes.iter().flatten() {
            if !node.visible {
                continue;
            }
            if let Some(ref collider) = node.collider {
                let hit = match collider {
                    Collider::Box3D { half_extents } => {
                        let aabb = Aabb3D::from_center_half_extents(Vec3::ZERO, *half_extents)
                            .transform(&node.global_transform);
                        aabb.raycast(ray)
                    }
                    Collider::Sphere { radius } => {
                        let pos = node.global_transform.transform_point3(Vec3::ZERO);
                        let sphere = BoundingSphere::new(pos, *radius);
                        sphere.raycast(ray)
                    }
                    _ => None,
                };

                if let Some(hit_info) = hit {
                    if closest_hit.as_ref().is_none_or(|(_, prev)| hit_info.distance < prev.distance) {
                        closest_hit = Some((node.id, hit_info));
                    }
                }
            }
        }

        closest_hit
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}
