# Module `collision` — Geometric Collision Detection, OBB/SAT, Broadphase, and Raycasting

The `collision` module delivers high-performance geometric intersection algorithms, Oriented Bounding Boxes (`Obb3D`) powered by the Separating Axis Theorem (SAT), a conservative Sort-and-Sweep broadphase stage (`BroadphaseGrid`), kinematic penetration resolution, and raycasting.

Source code: [`src/collision/`](../../src/collision/)

---

## 1. ECS Entity and Collider Integration

In Oxide's ECS architecture, collision components reside in the component table `world.collider: SecondaryMap<EntityId, Collider>`.

During collision evaluation, `Collider` primitives are evaluated in world space using `world.global_transform_matrix(entity)`:
- **Box3D:** Evaluated as an oriented bounding box `Obb3D::from_matrix`, accurately preserving rotation angles and scale without artificial bounding box expansion.
- **Sphere:** Center translates to `matrix.transform_point3(Vec3::ZERO)`, with radius scaled by the maximum coordinate scale factor.
- **Box2D:** Evaluated as screen-space rectangular extents.

---

## 2. Bounding Volume Primitives (`Collider`)

The `Collider` enum defines geometry in the entity's local coordinate frame:

```rust
pub enum Collider {
    Box3D { half_extents: glam::Vec3 },
    Sphere { radius: f32 },
    Box2D { half_extents: glam::Vec2 },
}
```

### Constructors & Methods:
- `Collider::box_3d(size: Vec3) -> Self` — Takes total dimensions `(dx, dy, dz)` and stores half-extents `size * 0.5`.
- `Collider::sphere(radius: f32) -> Self` — Defines a bounding sphere with radius.
- `Collider::box_2d(size: Vec2) -> Self` — Defines a 2D planar rectangle in screen pixels.
- `Collider::world_aabb(&self, matrix: &Mat4) -> Aabb3D` — Computes a conservative world-space AABB for broadphase acceleration.

---

## 3. Mathematical Volumes

### `Obb3D` (Oriented Bounding Box)
```rust
pub struct Obb3D {
    pub center: Vec3,
    pub axes: [Vec3; 3],       // Orthonormal world-space axes X, Y, Z
    pub half_extents: Vec3,  // Scaled half-extents
}
```
- `from_matrix(half_extents: Vec3, matrix: &Mat4) -> Self` — Constructs an OBB from half-extents and a world transform matrix.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — Exact OBB vs OBB test using the **Separating Axis Theorem (SAT)** across 15 candidate axes (6 face normals + 9 edge-edge cross products).
- `contact_sphere(&self, sphere: &BoundingSphere) -> Option<ContactManifold>` — Exact OBB vs Sphere intersection by transforming sphere center into the OBB's local orthonormal basis.
- `raycast(&self, ray: &Ray3D) -> Option<RaycastHit>` — Local-space raycast via slab method on the local AABB.

### `Aabb3D` (Axis-Aligned Bounding Box)
```rust
pub struct Aabb3D {
    pub min: Vec3,
    pub max: Vec3,
}
```
Provides conservative volume wrapping for broadphase culling (`BroadphaseGrid`), hierarchical acceleration, and spatial partitioning.

### `BoundingSphere` (Spherical Volume)
```rust
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}
```
- `intersects(&self, other: &Self) -> bool` — Distance squared comparison: $\|\mathbf{c}_1 - \mathbf{c}_2\|^2 \le (r_1 + r_2)^2$.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — Computes contact normal along the center-to-center vector.

---

## 4. Broadphase Stage (`BroadphaseGrid` — Sort-and-Sweep)

To eliminate $O(N^2)$ pairwise checking over large scenes, the broadphase stage utilizes a Sweep-and-Prune / Sort-and-Sweep algorithm:

```rust
pub struct BroadphaseGrid {
    proxies: Vec<BroadphaseProxy>,
    pairs: Vec<BroadphasePair>,
}
```

1. Each alive collider generates a conservative world-space `Aabb3D`.
2. Proxies are sorted along the principal X-axis by `min.x` in $O(N \log N)$.
3. Linear interval sweep along X prunes up to 99.9% of distant pairs before any costly SAT calculations occur.
4. Pairs consisting of two immovable static objects are discarded immediately.
5. Remaining candidate pairs are dispatched to narrowphase intersection testing.

---

## 5. Contact Manifold (`ContactManifold`)

```rust
pub struct ContactManifold {
    pub normal: glam::Vec3, // Unit normal pointing from body A to body B
    pub depth: f32,         // Penetration depth along normal (> 0.0)
    pub point: glam::Vec3,  // World-space contact position
}
```

---

## 6. Kinematic Penetration Resolution (`resolve_penetration_system`)

Resolves mutual geometric penetration along the contact normal:
- **Both bodies dynamic:** Displacement is shared equally (50% / 50%).
- **One body static:** The static obstacle remains immovable (0%), while the dynamic body receives 100% displacement.
- **Both bodies static:** Displacements are ignored.

---

## 7. Raycasting (`Ray3D`, `RaycastHit`)

```rust
pub struct Ray3D {
    pub origin: Vec3,
    pub direction: Vec3, // Automatically normalized
}

pub struct RaycastHit {
    pub distance: f32,   // Parametric hit distance t >= 0.0
    pub point: Vec3,      // World-space hit position
    pub normal: Vec3,     // Outward surface normal at hit point
}
```

The system `raycast_system(world: &World, ray: &Ray3D) -> Option<(EntityId, RaycastHit)>` queries all visible entities with colliders and returns the closest hit.
