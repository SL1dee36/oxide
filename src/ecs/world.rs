use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::collision::manifold::ContactManifold;
use crate::collision::ray::{Ray3D, RaycastHit};
use crate::core::context::Context;
use crate::ecs::components::{Component, Name, NodeBehavior, Transform, Visible};
use crate::ecs::entity::EntityId;
use crate::ecs::systems::{update_behaviors_system, update_hierarchy_system};
use crate::physics::rigid_body::{RigidBody, RigidBodyType};
use crate::physics::soft_body::SoftBody;
use crate::physics::systems::step_physics_system;
use crate::renderer::buffer::{IndexBuffer, VertexArray, VertexBuffer};
use crate::renderer::pipeline_3d::Light3D;
use crate::spatial::camera::Camera;
use crate::text::text2d::Text2D;
use glam::{Mat4, Vec3};
use slotmap::{SecondaryMap, SlotMap};

/// Represents allocated GPU buffers orphaned from an ECS entity upon despawn,
/// pending explicit OpenGL deallocation within the active rendering context.
#[derive(Debug)]
pub struct PendingGpuMeshCleanup {
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}

impl PendingGpuMeshCleanup {
    /// Destroys the encapsulated OpenGL buffers and array objects.
    pub fn destroy(self, gl: &glow::Context) {
        if let Some(vao) = self.vao {
            vao.destroy(gl);
        }
        if let Some(vbo) = self.vbo {
            vbo.destroy(gl);
        }
        if let Some(ebo) = self.ebo {
            ebo.destroy(gl);
        }
    }
}

/// Central Entity-Component-System (ECS) data store managing entities, decoupled component tables, and world resources.
pub struct World {
    /// Active entity registry.
    pub entities: SlotMap<EntityId, ()>,
    /// Local TRS transformation components.
    pub transform: SecondaryMap<EntityId, Transform>,
    /// Cached world transformation matrix components.
    pub global_transform: SecondaryMap<EntityId, Mat4>,
    /// Hierarchy parent components.
    pub parent: SecondaryMap<EntityId, EntityId>,
    /// Hierarchy children components.
    pub children: SecondaryMap<EntityId, Vec<EntityId>>,
    /// Rendering visibility toggle components.
    pub visible: SecondaryMap<EntityId, Visible>,
    /// Geometric collision bounding volumes.
    pub collider: SecondaryMap<EntityId, Collider>,
    /// 6-DOF rigid body physics components.
    pub rigid_body: SecondaryMap<EntityId, RigidBody>,
    /// Deformable Position-Based Dynamics (PBD) soft body components.
    pub soft_body: SecondaryMap<EntityId, SoftBody>,
    /// 3D polygonal mesh rendering payloads.
    pub mesh_3d: SecondaryMap<EntityId, Mesh3D>,
    /// 2D quad surface rendering payloads.
    pub surface_2d: SecondaryMap<EntityId, Surface2D>,
    /// 2D typography text rendering payloads.
    pub text_2d: SecondaryMap<EntityId, Text2D>,

    /// Diagnostic entity names.
    pub names: SecondaryMap<EntityId, Name>,
    /// Dynamic procedural behaviors.
    pub behaviors: SecondaryMap<EntityId, Box<dyn NodeBehavior>>,

    /// Queue of orphaned GPU resources pending OpenGL deallocation.
    pub pending_gpu_cleanup: Vec<PendingGpuMeshCleanup>,

    /// Active viewing camera resource.
    pub camera: Camera,
    /// Active directional light resource.
    pub light: Light3D,
}

impl World {
    /// Creates an empty ECS world with default camera and directional light.
    pub fn new() -> Self {
        Self {
            entities: SlotMap::with_key(),
            transform: SecondaryMap::new(),
            global_transform: SecondaryMap::new(),
            parent: SecondaryMap::new(),
            children: SecondaryMap::new(),
            visible: SecondaryMap::new(),
            collider: SecondaryMap::new(),
            rigid_body: SecondaryMap::new(),
            soft_body: SecondaryMap::new(),
            mesh_3d: SecondaryMap::new(),
            surface_2d: SecondaryMap::new(),
            text_2d: SecondaryMap::new(),
            names: SecondaryMap::new(),
            behaviors: SecondaryMap::new(),
            pending_gpu_cleanup: Vec::new(),
            camera: Camera::default(),
            light: Light3D::default(),
        }
    }

    /// Spawns a new entity into the world with default [`Transform`], `global_transform`, and [`Visible`].
    pub fn spawn(&mut self) -> EntityId {
        let entity = self.entities.insert(());
        self.transform.insert(entity, Transform::IDENTITY);
        self.global_transform.insert(entity, Mat4::IDENTITY);
        self.visible.insert(entity, Visible(true));
        entity
    }

    /// Checks if `entity` is currently alive in the world.
    #[inline]
    pub fn is_alive(&self, entity: EntityId) -> bool {
        self.entities.contains_key(entity)
    }

    /// Despawns an entity, cleanly detaching it from its parent and recursively despawning all child entities.
    ///
    /// Cleans up all component records across all secondary maps. Returns `true` if entity existed.
    pub fn despawn(&mut self, entity: EntityId) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        // Cleanly detach from parent if present
        if let Some(parent_entity) = self.parent.remove(entity) {
            if let Some(c_list) = self.children.get_mut(parent_entity) {
                c_list.retain(|&c| c != entity);
            }
        }

        self.despawn_recursive(entity);
        true
    }

    fn despawn_recursive(&mut self, entity: EntityId) {
        // Collect and recursively despawn children first
        if let Some(c_list) = self.children.remove(entity) {
            for child in c_list {
                self.despawn_recursive(child);
            }
        }

        // Remove from all component maps
        self.transform.remove(entity);
        self.global_transform.remove(entity);
        self.parent.remove(entity);
        self.visible.remove(entity);
        self.collider.remove(entity);
        self.rigid_body.remove(entity);
        self.soft_body.remove(entity);
        if let Some(mut mesh) = self.mesh_3d.remove(entity) {
            if mesh.vao.is_some() || mesh.vbo.is_some() || mesh.ebo.is_some() {
                self.pending_gpu_cleanup.push(PendingGpuMeshCleanup {
                    vao: mesh.vao.take(),
                    vbo: mesh.vbo.take(),
                    ebo: mesh.ebo.take(),
                });
            }
        }
        self.surface_2d.remove(entity);
        self.text_2d.remove(entity);
        self.names.remove(entity);
        self.behaviors.remove(entity);

        self.entities.remove(entity);
    }

    /// Attaches `child` under `parent` in the transform hierarchy.
    pub fn attach_child(&mut self, parent: EntityId, child: EntityId) {
        if !self.is_alive(parent) || !self.is_alive(child) || parent == child {
            return;
        }

        // Detach from previous parent if any
        if let Some(old_parent) = self.parent.get(child).copied() {
            if old_parent != parent {
                if let Some(old_children) = self.children.get_mut(old_parent) {
                    old_children.retain(|&c| c != child);
                }
            }
        }

        self.parent.insert(child, parent);
        let c_list = self.children.entry(parent).unwrap().or_default();
        if !c_list.contains(&child) {
            c_list.push(child);
        }
    }

    /// Detaches `child` from its parent, making it a root entity in the hierarchy.
    pub fn detach_child(&mut self, child: EntityId) {
        if let Some(parent) = self.parent.remove(child) {
            if let Some(c_list) = self.children.get_mut(parent) {
                c_list.retain(|&c| c != child);
            }
        }
    }

    // ==========================================
    // Generic ECS Component Accessors
    // ==========================================

    /// Inserts or replaces component `C` on `entity`.
    #[inline]
    pub fn insert<C: Component>(&mut self, entity: EntityId, component: C) {
        if self.is_alive(entity) {
            C::insert(self, entity, component);
        }
    }

    /// Retrieves an immutable reference to component `C` on `entity`.
    #[inline]
    pub fn get<C: Component>(&self, entity: EntityId) -> Option<&C> {
        C::get(self, entity)
    }

    /// Retrieves a mutable reference to component `C` on `entity`.
    #[inline]
    pub fn get_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
        C::get_mut(self, entity)
    }

    /// Removes and returns component `C` from `entity`.
    #[inline]
    pub fn remove<C: Component>(&mut self, entity: EntityId) -> Option<C> {
        C::remove(self, entity)
    }

    /// Checks if `entity` has component `C`.
    #[inline]
    pub fn has<C: Component>(&self, entity: EntityId) -> bool {
        C::get(self, entity).is_some()
    }

    // ==========================================
    // Convenient Direct Accessors
    // ==========================================

    /// Returns an immutable reference to the entity's local [`Transform`].
    #[inline]
    pub fn transform(&self, entity: EntityId) -> Option<&Transform> {
        self.transform.get(entity)
    }

    /// Returns a mutable reference to the entity's local [`Transform`].
    #[inline]
    pub fn transform_mut(&mut self, entity: EntityId) -> Option<&mut Transform> {
        self.transform.get_mut(entity)
    }

    /// Sets the local [`Transform`] on `entity`.
    #[inline]
    pub fn set_transform(&mut self, entity: EntityId, transform: Transform) {
        if self.is_alive(entity) {
            self.transform.insert(entity, transform);
            if self.parent.get(entity).is_none() {
                self.global_transform.insert(entity, transform.matrix());
            }
        }
    }

    /// Returns the entity's cached `global_transform` matrix.
    #[inline]
    pub fn global_transform(&self, entity: EntityId) -> Option<&Mat4> {
        self.global_transform.get(entity)
    }

    /// Returns the entity's cached `global_transform` matrix, or computes it from local transform if unparented.
    #[inline]
    pub fn global_transform_matrix(&self, entity: EntityId) -> Mat4 {
        if let Some(&gt) = self.global_transform.get(entity) {
            gt
        } else if let Some(t) = self.transform.get(entity) {
            t.matrix()
        } else {
            Mat4::IDENTITY
        }
    }

    /// Returns an optional reference to the entity's diagnostic name.
    #[inline]
    pub fn name(&self, entity: EntityId) -> Option<&str> {
        self.names.get(entity).map(|n| n.0.as_str())
    }

    /// Sets or replaces the entity's diagnostic name.
    #[inline]
    pub fn set_name(&mut self, entity: EntityId, name: impl Into<String>) {
        if self.is_alive(entity) {
            self.names.insert(entity, Name(name.into()));
        }
    }

    /// Returns `true` if `entity` is marked visible for rendering.
    #[inline]
    pub fn is_visible(&self, entity: EntityId) -> bool {
        self.visible.get(entity).map(|v| v.0).unwrap_or(true)
    }

    /// Sets the visibility flag on `entity`.
    #[inline]
    pub fn set_visible(&mut self, entity: EntityId, visible: bool) {
        if self.is_alive(entity) {
            self.visible.insert(entity, Visible(visible));
        }
    }

    /// Returns `true` if `entity` is marked as a static obstacle or static rigid body.
    #[inline]
    pub fn is_static(&self, entity: EntityId) -> bool {
        if let Some(rb) = self.rigid_body.get(entity) {
            rb.is_static()
        } else {
            false
        }
    }

    /// Sets or configures the static obstacle state on `entity`.
    #[inline]
    pub fn set_static(&mut self, entity: EntityId, is_static: bool) {
        if self.is_alive(entity) {
            if let Some(rb) = self.rigid_body.get_mut(entity) {
                rb.body_type = if is_static {
                    RigidBodyType::Static
                } else {
                    RigidBodyType::Dynamic
                };
                if is_static {
                    rb.inv_mass = 0.0;
                } else if rb.mass > 0.0 {
                    rb.inv_mass = 1.0 / rb.mass;
                }
            } else if is_static {
                self.rigid_body.insert(entity, RigidBody::new_static());
            }
        }
    }

    /// Returns an optional reference to rigid body component.
    #[inline]
    pub fn rigid_body(&self, entity: EntityId) -> Option<&RigidBody> {
        self.rigid_body.get(entity)
    }

    /// Returns an optional mutable reference to rigid body component.
    #[inline]
    pub fn rigid_body_mut(&mut self, entity: EntityId) -> Option<&mut RigidBody> {
        self.rigid_body.get_mut(entity)
    }

    /// Returns an optional reference to soft body component.
    #[inline]
    pub fn soft_body(&self, entity: EntityId) -> Option<&SoftBody> {
        self.soft_body.get(entity)
    }

    /// Returns an optional mutable reference to soft body component.
    #[inline]
    pub fn soft_body_mut(&mut self, entity: EntityId) -> Option<&mut SoftBody> {
        self.soft_body.get_mut(entity)
    }

    /// Returns an optional reference to 3D mesh component.
    #[inline]
    pub fn mesh_3d(&self, entity: EntityId) -> Option<&Mesh3D> {
        self.mesh_3d.get(entity)
    }

    /// Returns an optional mutable reference to 3D mesh component.
    #[inline]
    pub fn mesh_3d_mut(&mut self, entity: EntityId) -> Option<&mut Mesh3D> {
        self.mesh_3d.get_mut(entity)
    }

    /// Returns an optional reference to 2D quad surface component.
    #[inline]
    pub fn surface_2d(&self, entity: EntityId) -> Option<&Surface2D> {
        self.surface_2d.get(entity)
    }

    /// Returns an optional mutable reference to 2D quad surface component.
    #[inline]
    pub fn surface_2d_mut(&mut self, entity: EntityId) -> Option<&mut Surface2D> {
        self.surface_2d.get_mut(entity)
    }

    /// Returns an optional reference to 2D text component.
    #[inline]
    pub fn text_2d(&self, entity: EntityId) -> Option<&Text2D> {
        self.text_2d.get(entity)
    }

    /// Returns an optional mutable reference to 2D text component.
    #[inline]
    pub fn text_2d_mut(&mut self, entity: EntityId) -> Option<&mut Text2D> {
        self.text_2d.get_mut(entity)
    }

    /// Returns an optional reference to collision bounding volume.
    #[inline]
    pub fn collider(&self, entity: EntityId) -> Option<&Collider> {
        self.collider.get(entity)
    }

    /// Returns an optional mutable reference to collision bounding volume.
    #[inline]
    pub fn collider_mut(&mut self, entity: EntityId) -> Option<&mut Collider> {
        self.collider.get_mut(entity)
    }

    /// Returns entity's parent in the hierarchy if attached.
    #[inline]
    pub fn parent(&self, entity: EntityId) -> Option<EntityId> {
        self.parent.get(entity).copied()
    }

    /// Returns slice of child entities attached under `entity`.
    #[inline]
    pub fn children(&self, entity: EntityId) -> Option<&[EntityId]> {
        self.children.get(entity).map(|c| c.as_slice())
    }

    // ==========================================
    // Convenience Entity Builders
    // ==========================================

    /// Spawns a 3D polygon mesh entity with an attached collider.
    pub fn spawn_mesh_3d(
        &mut self,
        mesh: Mesh3D,
        transform: Transform,
        collider: Collider,
    ) -> EntityId {
        let entity = self.spawn();
        self.set_name(entity, "Mesh3D");
        self.set_transform(entity, transform);
        self.mesh_3d.insert(entity, mesh);
        self.collider.insert(entity, collider);
        entity
    }

    /// Spawns an immobile 3D mesh entity with static rigid body.
    pub fn spawn_static_mesh_3d(
        &mut self,
        mesh: Mesh3D,
        transform: Transform,
        collider: Collider,
    ) -> EntityId {
        let entity = self.spawn_mesh_3d(mesh, transform, collider);
        self.set_name(entity, "StaticMesh3D");
        self.rigid_body.insert(entity, RigidBody::new_static());
        entity
    }

    /// Spawns a 3D box mesh entity with an attached box collider.
    pub fn spawn_box_3d(
        &mut self,
        width: f32,
        height: f32,
        depth: f32,
        transform: Transform,
    ) -> EntityId {
        self.spawn_mesh_3d(
            Mesh3D::box_mesh(width, height, depth),
            transform,
            Collider::box_3d(Vec3::new(width, height, depth)),
        )
    }

    /// Spawns an immobile 3D box mesh entity with static rigid body.
    pub fn spawn_static_box_3d(
        &mut self,
        width: f32,
        height: f32,
        depth: f32,
        transform: Transform,
    ) -> EntityId {
        let entity = self.spawn_box_3d(width, height, depth, transform);
        self.set_name(entity, "StaticBox3D");
        self.rigid_body.insert(entity, RigidBody::new_static());
        entity
    }

    /// Spawns a 2D planar quad surface in screen coordinates with an attached collider.
    pub fn spawn_surface_2d(
        &mut self,
        surface: Surface2D,
        transform: Transform,
        collider: Collider,
    ) -> EntityId {
        let entity = self.spawn();
        self.set_name(entity, "Surface2D");
        self.set_transform(entity, transform);
        self.surface_2d.insert(entity, surface);
        self.collider.insert(entity, collider);
        entity
    }

    /// Spawns a 2D screen-space typography text element.
    pub fn spawn_text_2d(&mut self, text: Text2D, transform: Transform) -> EntityId {
        let entity = self.spawn();
        self.set_name(entity, "Text2D");
        self.set_transform(entity, transform);
        self.text_2d.insert(entity, text);
        entity
    }

    /// Spawns a dynamic rigid body entity with a collider.
    pub fn spawn_rigid_body(
        &mut self,
        mass: f32,
        transform: Transform,
        collider: Collider,
    ) -> EntityId {
        let entity = self.spawn();
        self.set_transform(entity, transform);
        self.collider.insert(entity, collider);
        self.rigid_body.insert(entity, RigidBody::new_dynamic(mass));
        entity
    }

    /// Spawns a rigid body entity with an attached 3D mesh, collider, and custom rigid body settings.
    pub fn spawn_rigid_body_with_mesh(
        &mut self,
        mesh: Mesh3D,
        transform: Transform,
        collider: Collider,
        rigid_body: RigidBody,
    ) -> EntityId {
        let entity = self.spawn();
        self.set_transform(entity, transform);
        self.mesh_3d.insert(entity, mesh);
        self.collider.insert(entity, collider);
        self.rigid_body.insert(entity, rigid_body);
        entity
    }

    /// Spawns a deformable cloth sheet soft body with an associated deformable [`Mesh3D`].
    pub fn spawn_soft_body_cloth(
        &mut self,
        width: f32,
        height: f32,
        res_x: usize,
        res_y: usize,
        mass: f32,
        stiffness: f32,
        transform: Transform,
    ) -> EntityId {
        let (mut soft_body, mut mesh) = SoftBody::cloth(width, height, res_x, res_y, mass, stiffness);
        let mat = transform.matrix();
        for p in &mut soft_body.particles {
            p.position = mat.transform_point3(p.position);
            p.old_position = p.position;
            p.predicted_position = p.position;
        }
        for v in &mut mesh.vertices {
            let pos = mat.transform_point3(Vec3::from_array(v.position));
            v.position = pos.to_array();
        }
        for c in &mut soft_body.constraints {
            c.rest_length = (soft_body.particles[c.particle_a].position
                - soft_body.particles[c.particle_b].position)
                .length();
        }

        let entity = self.spawn();
        self.set_name(entity, "SoftBodyCloth");
        self.set_transform(entity, Transform::IDENTITY);
        self.soft_body.insert(entity, soft_body);
        self.mesh_3d.insert(entity, mesh);
        entity
    }

    // ==========================================
    // Camera & Light Resources
    // ==========================================

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

    // ==========================================
    // Systems & Backwards Compatibility Aliases
    // ==========================================

    /// Advances physics simulation sub-steps (rigid bodies, soft bodies, constraints).
    pub fn step_physics(&mut self, dt: f32) {
        step_physics_system(self, dt);
    }

    /// Traverses the scene hierarchy and recalculates world transformation matrices for all entities.
    pub fn update_transforms(&mut self) {
        update_hierarchy_system(self);
    }

    /// Evaluates geometric intersection between colliders of two entities.
    pub fn check_intersection(&self, a: EntityId, b: EntityId) -> Option<ContactManifold> {
        crate::collision::systems::check_entity_intersection(self, a, b)
    }

    /// Kinematically separates two overlapping entities along the contact normal taking rigid body types into account.
    pub fn resolve_penetration(&mut self, a: EntityId, b: EntityId, contact: &ContactManifold) {
        crate::collision::systems::resolve_penetration_system(self, a, b, contact);
    }

    /// Casts a ray through the world, returning the closest intersecting entity and hit information.
    pub fn raycast(&self, ray: &Ray3D) -> Option<(EntityId, RaycastHit)> {
        crate::collision::systems::raycast_system(self, ray)
    }

    /// Backwards-compatibility alias for [`Self::despawn`].
    #[inline]
    pub fn despawn_node(&mut self, entity: EntityId) -> bool {
        self.despawn(entity)
    }

    /// Updates physics simulation, hierarchical transforms, and runs attached entity behaviors with delta time `dt`.
    pub fn update(&mut self, ctx: &mut Context, dt: f32) {
        step_physics_system(self, dt);
        update_hierarchy_system(self);
        update_behaviors_system(self, ctx, dt);
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
