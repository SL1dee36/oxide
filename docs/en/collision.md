# Module `collision` — Spatial Queries & Collision Engine

The `collision` module provides a lightweight, analytical query engine for intersection testing, contact manifold generation, raycasting, and kinematic separation.

Source files: [`src/collision/`](file:///e:/PRJCT/Oxide/src/collision/)

---

## 1. Colliders (`Collider`)

The `Collider` enum designates local volume bounds attached to scene nodes:

```rust
pub enum Collider {
    Box3D { half_extents: Vec3 },
    Sphere { radius: f32 },
    Box2D { half_extents: Vec2 },
}

// Constructors
let box3d = Collider::box_3d(Vec3::new(2.0, 2.0, 2.0)); // half_extents = (1, 1, 1)
let sphere = Collider::sphere(1.5);
let box2d = Collider::box_2d(Vec2::new(100.0, 50.0));
```

---

## 2. Bounding Volumes

### `Aabb3D` (Axis-Aligned Bounding Box)
- `from_center_half_extents(center: Vec3, half: Vec3) -> Self`: Constructs an AABB from center point and extents.
- `transform(&self, matrix: &Mat4) -> Self`: Transforms an AABB through an arbitrary 4x4 matrix, computing tight rotated bounds.
- `intersects(&self, other: &Self) -> bool`: Fast boolean overlap test.
- `contact(&self, other: &Self) -> Option<ContactManifold>`: Calculates the minimum penetration axis, normal, and point.

### `BoundingSphere`
- `intersects(&self, other: &Self) -> bool`: Center distance squared evaluation against combined radii.
- `contact(&self, other: &Self) -> Option<ContactManifold>`: Generates a contact manifold along the radial center offset.

### `Aabb2D` (Screen-Space Bounding Box)
- Analytical rectangle intersection checks for 2D UI elements.

---

## 3. The `ContactManifold` Structure

Upon detecting an intersection, a `ContactManifold` is generated:

```rust
pub struct ContactManifold {
    pub normal: Vec3, // Unit separation normal
    pub depth: f32,   // Penetration depth (> 0.0)
    pub point: Vec3,  // World space contact point
}
```

---

## 4. Kinematic Separation (`resolve_penetration` & `resolve_penetration_weighted`)

Displaces overlapping bodies along the contact normal to eliminate penetration, taking the `is_static` flag into account:

```rust
if let Some(contact) = scene.check_intersection(wall_node, sphere_node) {
    // If one node is static (is_static == true), 100% of separation displacement
    // is applied to the dynamic node.
    scene.resolve_penetration(wall_node, sphere_node, &contact);
}
```

- When both nodes are dynamic: separation is split equally (50% / 50%).
- When one node is static (e.g. maze wall, obstacle): the static node remains fixed (0%), and the dynamic node receives 100% of the displacement.
- When both are static: no displacement occurs.

Analytical contact generation is supported for: Box3D $\leftrightarrow$ Box3D, Sphere $\leftrightarrow$ Sphere, and Box3D $\leftrightarrow$ Sphere.

---

## 5. Raycasting (`Ray3D`, `RaycastHit`)

Supports spatial queries and mouse object picking:

```rust
let ray = Ray3D::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

// Query against all scene colliders
if let Some((hit_node_id, hit_info)) = scene.raycast(&ray) {
    println!("Hit node {:?} at distance {:.2}", hit_node_id, hit_info.distance);
    println!("Intersection point: {:?}", hit_info.point);
    println!("Surface normal: {:?}", hit_info.normal);
}
```
