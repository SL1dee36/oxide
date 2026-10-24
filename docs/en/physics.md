# Module `physics` — 6-DOF Rigid Body Dynamics and XPBD Soft Body Simulation

The `physics` module delivers a complete real-time physics engine combining 6-DOF rigid body dynamics with an iterative impulse solver and an Extended Position-Based Dynamics (XPBD) soft body cloth solver.

Source code: [`src/physics/`](../../src/physics/)

---

## 1. 6-DOF Rigid Body Dynamics

The `RigidBody` component encapsulates kinematic and dynamic properties for 3D objects with six degrees of freedom (3 translational + 3 rotational).

```rust
pub struct RigidBody {
    pub body_type: RigidBodyType, // Dynamic, Static, Kinematic
    pub mass: f32,
    pub inv_mass: f32,
    pub inv_inertia_local: Mat3,  // Local inverse inertia tensor
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    pub restitution: f32,         // Coefficient of restitution [0..1]
    pub friction: f32,            // Surface friction coefficient [0..1]
    pub linear_damping: f32,
    pub angular_damping: f32,
}
```

### 1.1. Inertia Tensors
When spawning a rigid body, the engine analytically computes its diagonal local inertia tensor $I_{\text{local}}$:
- **Sphere with radius $r$:** $I = \frac{2}{5} m r^2 \cdot \mathbf{I}_{3\times3}$
- **Box with half-extents $(h_x, h_y, h_z)$:**  
  $I_{xx} = \frac{1}{12} m ((2h_y)^2 + (2h_z)^2)$,  
  $I_{yy} = \frac{1}{12} m ((2h_x)^2 + (2h_z)^2)$,  
  $I_{zz} = \frac{1}{12} m ((2h_x)^2 + (2h_y)^2)$.

In world space, the inverse inertia tensor transforms dynamically with orientation:
$$I_{\text{world}}^{-1} = R \cdot I_{\text{local}}^{-1} \cdot R^T$$

### 1.2. Physics Configuration (`PhysicsConfig`)
```rust
pub struct PhysicsConfig {
    pub gravity: Vec3,                 // Environmental gravity (default: [0, -9.81, 0])
    pub velocity_iterations: usize,    // Velocity impulse solver iterations (default: 4)
    pub position_iterations: usize,    // Positional penetration relaxation iterations (default: 2)
}
```

---

## 2. Physics Step Pipeline (`step_rigid_body_system`)

The simulation executes each physics step across 5 deterministic phases:

1. **Phase 1: Numerical Integration (Semi-Implicit Euler):**
   - Linear acceleration: $\mathbf{a} = \mathbf{g} + m^{-1} \mathbf{F}_{\text{accum}}$.
   - Angular acceleration: $\boldsymbol{\alpha} = I_{\text{world}}^{-1} \cdot \boldsymbol{\tau}_{\text{accum}}$.
   - Velocities: $\mathbf{v} \mathrel{+}= \mathbf{a} \Delta t$, $\boldsymbol{\omega} \mathrel{+}= \boldsymbol{\alpha} \Delta t$.
   - Position: $\mathbf{x} \mathrel{+}= \mathbf{v} \Delta t$.
   - Orientation: $q \mathrel{+}= \frac{1}{2} (\boldsymbol{\omega}, 0) \otimes q$, followed by normalization.

2. **Phase 2: Hierarchy Synchronization:**  
   Calls `update_hierarchy_system` to compute world matrices `global_transform`.

3. **Phase 3: Collision Detection (Broadphase + Narrowphase):**
   - **Broadphase (`BroadphaseGrid`):** Sort-and-Sweep along the X-axis culls up to 99.9% of non-overlapping candidate pairs and static-static pairs in $O(N \log N)$.
   - **Narrowphase (`check_collider_intersection`):** Exact OBB SAT solver computes contact manifolds with contact normal, penetration depth, and exact contact point $\mathbf{p}$.

4. **Phase 4: Velocity Iterations (`velocity_iterations`):**
   - Lever arms: $\mathbf{r}_a = \mathbf{p} - \mathbf{c}_a$, $\mathbf{r}_b = \mathbf{p} - \mathbf{c}_b$.
   - Relative contact velocity: $\mathbf{v}_{\text{rel}} = (\mathbf{v}_b + \boldsymbol{\omega}_b \times \mathbf{r}_b) - (\mathbf{v}_a + \boldsymbol{\omega}_a \times \mathbf{r}_a)$.
   - Effective contact mass along normal:
     $$K_n = m_a^{-1} + m_b^{-1} + (\mathbf{r}_a \times \mathbf{n})^T I_a^{-1} (\mathbf{r}_a \times \mathbf{n}) + (\mathbf{r}_b \times \mathbf{n})^T I_b^{-1} (\mathbf{r}_b \times \mathbf{n})$$
   - Computes normal restitution impulses and Coulomb friction tangential impulses.

5. **Phase 5: Position Iterations (`position_iterations`):**  
   Resolves residual geometric penetrations along contact normals respecting immovable body flags.

---

## 3. XPBD Soft Body and Cloth Simulation

The `SoftBody` subsystem implements Extended Position-Based Dynamics (XPBD) for unconditionally stable cloth and deformable surface simulation:

- **Particles (`SoftParticle`):** Track position, previous position $\mathbf{x}_{\text{prev}}$, velocity, and pin flags (`is_fixed`).
- **Distance Constraints (`DistanceConstraint`):** Maintain structural and shear mesh connections with compliance $\alpha = \text{compliance} / \Delta t^2$.
- **Sub-stepping:** Divides each frame step into 4–8 sub-steps to enforce constraint projection and static obstacle collision response.
- **Mesh Deformation Feedback:** Particle positions feed directly into vertex buffers of `Mesh3D`, rendering deformable geometry in real time.
