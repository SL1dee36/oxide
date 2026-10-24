# Subsystem `spatial` — Hierarchical Scene Graph, Nodes, and Cameras

The `spatial` module provides a hierarchical scene graph (Node Graph), cascading spatial transform evaluation (TRS), and perspective/orthographic camera projection models.

Source code: [`src/spatial/`](../../src/spatial/)

---

## 1. Spatial Transformations (`Transform`)

`Transform` represents the local spatial state of an entity in 3D Cartesian space:

```rust
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}
```

- **Model Matrix:** `transform.matrix() -> Mat4` evaluates the $4 \times 4$ affine matrix via $M = T \cdot R \cdot S$.
- **Incremental Rotation:** `rotate_x(rad)`, `rotate_y(rad)`, and `rotate_z(rad)` multiply the current quaternion by an incremental rotation.
- **Euler Angles:** `set_euler_angles(x_rad, y_rad, z_rad)` configures orientation using YXZ convention.

---

## 2. Viewing Cameras and Projections (`Camera`, `Projection`)

`Camera` combines spatial pose (`transform: Transform`) with geometric projection configuration (`projection: Projection`):

```rust
pub enum Projection {
    Perspective {
        fov_y_rad: f32,
        z_near: f32,
        z_far: f32,
    },
    Orthographic {
        half_width: f32,
        half_height: f32,
        z_near: f32,
        z_far: f32,
    },
}
```

### Key `Camera` Methods:
- `Camera::perspective(fov_y_deg, z_near, z_far)` — Constructs a perspective camera with field-of-view in degrees.
- `Camera::orthographic(width, height, z_near, z_far)` — Constructs an orthographic camera with specified view bounds.
- `look_at(eye: Vec3, target: Vec3, up: Vec3)` — Computes rotation and position to orient the camera toward a target.
- `view_matrix() -> Mat4` — Returns right-handed view matrix (inverse of camera's world transform).
- `projection_matrix(aspect_ratio: f32) -> Mat4` — Evaluates right-handed OpenGL projection matrix for the given aspect ratio.

---

---

## 3. Node Identification and Components (`NodeId`, `Transform`)

In Oxide's modern ECS architecture, every scene node is an ECS entity ([`EntityId`](../ecs.md), with `NodeId = EntityId` provided as a backwards-compatible alias). Node properties are stored in sparse component tables:

- **Local Transform:** `Transform` (translation, rotation, scale).
- **World Transform:** `GlobalTransform` ($4 \times 4$ affine matrix).
- **Hierarchy Links:** `Parent(EntityId)` and `Children(Vec<EntityId>)` components.
- **Identity & Visibility:** `Name(String)` and `Visible(bool)` components.
- **Autonomous Behaviors (`NodeBehavior`):**
  ```rust
  pub trait NodeBehavior {
      fn on_process(&mut self, entity: EntityId, world: &mut World, ctx: &mut Context, dt: f32);
  }
  ```

---

## 4. Scene Graph (`Scene`) and Hierarchy System (`update_hierarchy_system`)

`Scene` encapsulates the active camera, viewport clear color, and the ECS world:

```rust
pub struct Scene {
    pub world: World,
    pub camera: Camera,
    pub clear_color: Color,
}
```

### Cascading Transform Propagation (`update_hierarchy_system`):
The hierarchy system performs a breadth-first traversal (BFS) starting from root nodes (`Parent == None`), guaranteeing parent matrices are evaluated prior to children without recursion overhead:
$$M_{\text{global}}(\text{child}) = M_{\text{global}}(\text{parent}) \times M_{\text{local}}(\text{child})$$

### Spawning and Hierarchical Linking:
```rust
let parent_id = scene.spawn_empty("Anchor", Transform::from_xyz(0.0, 5.0, 0.0));
let mesh_id = scene.spawn_mesh_3d(Mesh3D::cube(1.0), Transform::IDENTITY, Some(Collider::box_3d(Vec3::ONE)));

// Link parent-child relationship
scene.attach_child(parent_id, mesh_id);
```

### Scene Operations:
- `scene.update(ctx, dt)` — Evaluates hierarchical world matrices, executes physics simulation (including OBB SAT and Broadphase), and invokes active `NodeBehavior` callbacks.
- `scene.render(ctx)` — Submits 3D mesh, 2D surface, and batched 2D text render passes.
- `scene.despawn_node(id)` — Removes a node and all recursive descendants, safely queuing GPU meshes into VRAM cleanup.
- `scene.check_intersection(id_a, id_b)` — Evaluates geometric overlap between node colliders.
- `scene.resolve_penetration(id_a, id_b, &contact)` — Kinematically separates intersecting nodes.
- `scene.raycast(&ray)` — Performs closest-hit raycasting through all colliders in the scene.

