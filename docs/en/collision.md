# Subsystem `collision` — Geometric Collision Detection and Raycasting

The `collision` module provides geometric intersection testing algorithms, contact manifold generation, kinematic penetration resolution, and analytical raycasting.

> **Architectural Note:** The `collision` subsystem performs geometric collision detection and is **not a rigid body dynamics physics engine**. It does not model momentum, angular inertia tensors, friction, or dynamic constraint solvers. It solely evaluates geometric overlap queries and applies kinematic separation.

Source code: [`src/collision/`](../../src/collision/)

---

## 1. Relationship Between Spatial Transforms and Colliders

Each scene graph entity (`SpatialNode`) optionally holds a local bounding volume (`collider: Option<Collider>`).

During intersection testing, `Collider` geometries are transformed into world coordinates via `SpatialNode::global_matrix()`:
- **Box3D:** The local axis-aligned box is transformed by the $4 \times 4$ affine matrix into a conservative world `Aabb3D` using absolute column vector projections.
- **Sphere:** The center is transformed via `matrix.transform_point3(Vec3::ZERO)`, and the radius scales uniformly.
- **Box2D:** Dimensions and translation map directly to 2D pixel coordinates.

---

## 2. Bounding Volume Types (`Collider`)

The `Collider` enumeration specifies bounding geometry in the node's local coordinate frame:

```rust
pub enum Collider {
    Box3D { half_extents: glam::Vec3 },
    Sphere { radius: f32 },
    Box2D { half_extents: glam::Vec2 },
}
```

Constructors:
- `Collider::box_3d(size: Vec3) -> Self` — Takes full dimensions `(dx, dy, dz)` and stores half-extents `size * 0.5`.
- `Collider::sphere(radius: f32) -> Self` — Defines a sphere of radius `radius`.
- `Collider::box_2d(size: Vec2) -> Self` — Takes full pixel dimensions `(w, h)` and stores half-extents `size * 0.5`.

---

## 3. Mathematical Bounding Volumes

### `Aabb3D` (3D Axis-Aligned Bounding Box)
```rust
pub struct Aabb3D {
    pub min: Vec3,
    pub max: Vec3,
}
```
- `from_center_half_extents(center: Vec3, half_extents: Vec3) -> Self`
- `transform(&self, matrix: &Mat4) -> Self` — Evaluates a conservative world AABB enclosing the transformed box.
- `intersects(&self, other: &Self) -> bool` — Overlap evaluation along X, Y, and Z axes.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — Computes minimum overlap axis, penetration depth, and normal.
- `contact_sphere(&self, sphere: &BoundingSphere) -> Option<ContactManifold>` — Computes closest-point distance between AABB and sphere.

### `BoundingSphere` (3D Bounding Sphere)
```rust
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}
```
- `intersects(&self, other: &Self) -> bool` — Evaluates center distance squared: $\|\mathbf{c}_1 - \mathbf{c}_2\|^2 \le (r_1 + r_2)^2$.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — Generates contact along the center-to-center vector.

### `Aabb2D` (2D Screen-Space AABB)
```rust
pub struct Aabb2D {
    pub min: Vec2,
    pub max: Vec2,
}
```
Analytical overlap testing between 2D rectangles in screen coordinates.

---

## 4. Contact Manifold (`ContactManifold`)

Upon intersection, an analytical `ContactManifold` is generated:

```rust
pub struct ContactManifold {
    pub normal: glam::Vec3,
    pub depth: f32,
    pub point: glam::Vec3,
}
```

- **`normal: Vec3`**: Unit vector along minimum separation axis, pointing from body A toward body B.
- **`depth: f32`**: Penetration distance along separation normal ($> 0.0$).
- **`point: Vec3`**: Estimated world-space contact location.

---

## 5. Kinematic Penetration Resolution (`resolve_penetration`)

Separates intersecting nodes along the contact normal without dynamic force modeling:

```rust
pub fn resolve_penetration_weighted(
    transform_a: &mut Transform,
    is_static_a: bool,
    transform_b: &mut Transform,
    is_static_b: bool,
    contact: &ContactManifold,
);
```

### Displacement Allocation:
1. **Both bodies dynamic (`is_static == false`):** Separation displacement is split equally (50% / 50%):
   $$\mathbf{T}_a \leftarrow \mathbf{T}_a - \mathbf{n} \cdot \frac{d}{2}, \quad \mathbf{T}_b \leftarrow \mathbf{T}_b + \mathbf{n} \cdot \frac{d}{2}$$
2. **One body static (`is_static == true`):** The static obstacle receives 0% displacement, and the dynamic body receives 100% of the displacement vector ($d$).
3. **Both bodies static:** Displacements are ignored.

---

## 6. Raycasting (`Ray3D`, `RaycastHit`)

Provides spatial intersection testing for line-of-sight checks and entity picking:

```rust
pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3, // Automatically normalized
}

pub struct RaycastHit {
    pub distance: f32,  // Parametric distance t >= 0.0
    pub point: Vec3,     // World-space hit location
    pub normal: Vec3,    // Outward surface normal at hit point
}
```

- **Slab Method:** `Aabb3D::raycast` computes analytical ray-plane intersection intervals.
- **Quadratic Solution:** `BoundingSphere::raycast` solves the ray-sphere intersection quadratic equation.
- **Scene Traversal:** `Scene::raycast(&self, ray: &Ray3D) -> Option<(NodeId, RaycastHit)>` tests all visible colliders and returns the hit record with minimal `distance`.
