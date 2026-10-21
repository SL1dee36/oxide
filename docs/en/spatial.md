# Module `spatial` — Scene Graph, Nodes & Transformations

The `spatial` module implements a hierarchical node tree (conceptually modeled after Godot's node architecture), TRS spatial mathematics, and camera projections.

Source files: [`src/spatial/`](file:///e:/PRJCT/Oxide/src/spatial/)

---

## 1. Spatial Transformation (`Transform`)

The `Transform` struct stores the local spatial state: Position (`translation: Vec3`), Orientation (`rotation: Quat`), and Scale (`scale: Vec3`).

```rust
// Construction
let mut transform = Transform::from_translation(Vec3::new(0.0, 2.0, -5.0));

// Manipulation
transform.rotate_y(1.5 * dt);
transform.rotate_x(0.5 * dt);
transform.translate(Vec3::new(0.1, 0.0, 0.0));

// 4x4 Model Matrix
let mat: Mat4 = transform.matrix();
```

---

## 2. Camera System (`Camera`)

Supports perspective and orthographic camera projections:

```rust
// Perspective 3D Camera
let mut camera = Camera::perspective(60.0, ctx.aspect_ratio(), 0.1, 100.0);
camera.look_at(
    Vec3::new(0.0, 5.0, 10.0), // Eye location
    Vec3::new(0.0, 0.0, 0.0),  // Target focus point
    Vec3::Y,                   // Up direction
);

// View and Projection matrices
let view = camera.view_matrix();
let proj = camera.projection_matrix(ctx.aspect_ratio());
```

---

## 3. Node Hierarchy (`SpatialNode`)

All scene entities are structured as a `SpatialNode`:
- `id: NodeId`: Unique identifier within the scene.
- `parent: Option<NodeId>`: Parent node index.
- `children: Vec<NodeId>`: Array of attached child node IDs.
- `transform: Transform`: Local transform relative to parent.
- `global_transform: Mat4`: Cached world space transformation matrix.
- `kind: NodeKind`:
  - `Empty`: Anchor/grouping transform node.
  - `Mesh3D(Mesh3D)`: 3D geometry node.
  - `Surface2D(Surface2D)`: 2D planar element.
  - `Text2D(Text2D)`: 2D typography element.
- `collider: Option<Collider>`: Bounding shape for spatial intersection tests.
- `behavior: Option<Box<dyn NodeBehavior>>`: Custom processing callback.

### The `NodeBehavior` Trait:
```rust
pub trait NodeBehavior {
    fn on_process(&mut self, node: &mut SpatialNode, ctx: &mut Context, dt: f32) {}
}
```

---

## 4. Scene Graph Container (`Scene`)

`Scene` coordinates node lifetimes, hierarchy tree propagation, and rendering passes:

### Transform Cascading:
Calling `scene.update_transforms()` recursively computes global world matrices:
$$\text{global\_transform}_{\text{child}} = \text{global\_transform}_{\text{parent}} \times \text{local\_matrix}_{\text{child}}$$

### Spawning Nodes:
```rust
let cube_id = scene.spawn_mesh_3d(mesh, transform, collider);
let panel_id = scene.spawn_surface_2d(surface, transform, collider);
let text_id = scene.spawn_text_2d(text, transform);

// Parenting
scene.attach_child(parent_id, child_id);
```

### Spatial Queries:
- `scene.check_intersection(node_a, node_b) -> Option<ContactManifold>`: Evaluates collider overlaps.
- `scene.resolve_penetration(node_a, node_b, &contact)`: Kinematically separates overlapping nodes along the contact normal.
- `scene.raycast(&ray) -> Option<(NodeId, RaycastHit)>`: Identifies the closest node intersected by a 3D ray.
