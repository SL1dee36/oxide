# Module `ecs` — Entity-Component-System Architecture

The `ecs` module provides a generational Entity-Component-System (ECS) architectural pattern, featuring a centralized world registry, decoupled component tables, and an efficient systems execution pipeline.

Source code: [`src/ecs/`](../../src/ecs/)

---

## 1. Architectural Concept

Traditional monolithic scene graphs suffer from deep inheritance coupling and pointer-chasing memory fragmentation. Oxide adopts a strict ECS paradigm:

- **Entity:** `EntityId` — A lightweight 64-bit identifier storing a slot index and generational counter (`slotmap::new_key_type!`), fully preventing ABA aliasing bugs when slots are recycled.
- **Component:** Pure data structs stored in sparse secondary storage arrays (`slotmap::SecondaryMap<EntityId, T>`).
- **System:** Stateless update passes operating over entities and their associated component slices (`update_hierarchy_system`, `update_behaviors_system`, `step_physics_system`, `RenderSystem`).

---

## 2. Central Registry: `World`

The `World` struct coordinates entity lifetimes, component mappings, and shared engine resources:

```rust
pub struct World {
    pub entities: SlotMap<EntityId, ()>,
    pub transform: SecondaryMap<EntityId, Transform>,
    pub global_transform: SecondaryMap<EntityId, Mat4>,
    pub parent: SecondaryMap<EntityId, EntityId>,
    pub children: SecondaryMap<EntityId, Vec<EntityId>>,
    pub visible: SecondaryMap<EntityId, Visible>,
    pub collider: SecondaryMap<EntityId, Collider>,
    pub rigid_body: SecondaryMap<EntityId, RigidBody>,
    pub soft_body: SecondaryMap<EntityId, SoftBody>,
    pub mesh_3d: SecondaryMap<EntityId, Mesh3D>,
    pub surface_2d: SecondaryMap<EntityId, Surface2D>,
    pub text_2d: SecondaryMap<EntityId, Text2D>,
    pub behavior: SecondaryMap<EntityId, Box<dyn NodeBehavior>>,
    pub pending_gpu_cleanup: Vec<PendingGpuMeshCleanup>,
    camera: Camera,
    light: Light3D,
}
```

### Key Lifecycle Methods:
- `spawn() -> EntityId` — Allocates a new alive entity slot.
- `despawn(entity: EntityId) -> bool` — Removes an entity, unbinds child nodes, and safely transfers allocated GPU buffers to the deferred cleanup queue.
- `attach_child(parent: EntityId, child: EntityId)` — Establishes parent-child spatial hierarchy relationships.
- `is_alive(entity: EntityId) -> bool` — Validates entity index and generation.

---

## 3. Component Tables

| Component | Storage Type | Description |
| :--- | :--- | :--- |
| **`Transform`** | `Transform` | Local translation (`Vec3`), orientation quaternion (`Quat`), and scale (`Vec3`). |
| **`global_transform`** | `Mat4` | Cached world-space affine matrix ($M_{\text{parent}} \times M_{\text{local}}$). |
| **`Visible`** | `Visible(pub bool)` | Visibility toggle for rendering and raycast queries. |
| **`Name`** | `Name(pub String)` | Debug name identifier. |
| **`Collider`** | `Collider` | Bounding volume geometry (`Box3D`, `Sphere`, `Box2D`). |
| **`RigidBody`** | `RigidBody` | 6-DOF dynamic properties (mass, velocity, inertia tensors, friction, restitution). |
| **`SoftBody`** | `SoftBody` | XPBD cloth particles, bilateral distance constraints, and compliance. |
| **`Mesh3D`** | `Mesh3D` | Polygonal geometry, texture handle `Arc<Texture2D>`, and GPU buffer handles (`VAO`, `VBO`, `EBO`). |
| **`Surface2D`** | `Surface2D` | Planar UI rectangle with corner radius and background color. |
| **`Text2D`** | `Text2D` | Text string with scale and color attributes. |
| **`NodeBehavior`** | `Box<dyn NodeBehavior>` | User frame update hook (`on_update(ctx, dt)`). |

---

## 4. Safe GPU Buffer Cleanup (`PendingGpuMeshCleanup`)

When an entity is despawned via `World::despawn`, its CPU RAM structures are freed immediately. However, OpenGL buffer objects (`glGenVertexArrays`, `glGenBuffers`) cannot be destroyed outside of an active graphics context.

To prevent GPU memory leaks:
1. Upon `despawn`, existing buffer handles (`vao`, `vbo`, `ebo`) are pushed to `world.pending_gpu_cleanup`.
2. At the beginning of each frame, `RenderSystem::render_world` drains this queue and invokes `.destroy(gl)` within the active OpenGL rendering thread.

---

## 5. Execution Systems

- **`update_hierarchy_system(world: &mut World)`:** Computes global matrices from root nodes down to leaves.
- **`update_behaviors_system(world: &mut World, ctx: &mut Context, dt: f32)`:** Dispatches per-frame updates to attached `NodeBehavior` instances.
- **`world.update(ctx, dt)`:** Orchestrates physics simulation (`step_physics_system`), hierarchy recalculation, and behavior scripts in deterministic order.
