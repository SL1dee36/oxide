# Subsystem `ffi` — C-ABI External Interface and Memory Model

The `ffi` module exports a stable ANSI C99-compatible interface (`extern "C"`), enabling applications in C, C++, Python, C#, Zig, and any language with C ABI support to drive Oxide subsystems.

Source code: [`src/ffi/`](../../src/ffi/)  
C/C++ Header: [`include/oxide.h`](../../include/oxide.h)  
Python Wrapper: [`bindings/python/oxide.py`](../../bindings/python/oxide.py)

---

## 1. Ownership and Lifetime Memory Model

Interaction across the FFI boundary adheres to explicit heap ownership rules:

```mermaid
flowchart TD
    subgraph RustHeap [Rust Heap Allocation]
        AppAlloc["oxide_app_create()<br/>Box::into_raw(Box::new(OxideApp))"]
        SceneAlloc["oxide_scene_create()<br/>Box::into_raw(Box::new(OxideScene))"]
    end

    subgraph ClientCode [Client Runtime (C / Python)]
        AppPtr["*mut OxideApp (Opaque Pointer)"]
        ScenePtr["*mut OxideScene (Opaque Pointer)"]
        CtxPtr["*mut OxideContext (Transient Reference)"]
    end

    AppAlloc --> AppPtr
    SceneAlloc --> ScenePtr

    AppPtr -->|"oxide_app_run(app, callbacks)"| AppConsumed["Box::from_raw(app) -> Deallocated upon loop exit"]
    AppPtr -->|"oxide_app_destroy(app)<br/>(Only if run is never invoked)"| AppFreed["drop(Box::from_raw(app))"]
    ScenePtr -->|"oxide_scene_destroy(scene)<br/>(Mandatory destructor)"| SceneFreed["drop(Box::from_raw(scene))"]

    AppConsumed -.-> CtxPtr
    NoteCtx["OxideContext is valid ONLY inside the callback body!"] -.-> CtxPtr
```

### 1.1. Application Lifetime (`OxideApp`):
- `oxide_app_create(title, width, height) -> *mut OxideApp` allocates the `OxideApp` structure on the Rust heap via `Box::into_raw` and returns an opaque pointer.
- **Consumption on Execution:** `oxide_app_run(app, callbacks)` assumes ownership of the `app` pointer, reconstructs `Box::from_raw(app)`, and **automatically deallocates it** upon event loop termination.
- **Manual Deallocation:** `oxide_app_destroy(app)` must be invoked **only if `oxide_app_run` was never called** (e.g., if startup was canceled). Invoking `oxide_app_destroy` after `oxide_app_run` is invalid and results in a double-free defect.

### 1.2. Scene Graph Lifetime (`OxideScene`):
- `oxide_scene_create() -> *mut OxideScene` allocates a scene graph container on the Rust heap.
- The caller **is required to explicitly release the scene** via `oxide_scene_destroy(scene)` when it is no longer needed.

### 1.3. Frame Context Lifetime (`OxideContext`):
- Passed by raw pointer to `on_init`, `on_update`, and `on_render` callbacks.
- Represents a transient stack reference within the frame loop. **Storing or accessing an `OxideContext` pointer outside the scope of the callback is strictly invalid**.

### 1.4. Panic Safety and Error Handling:
- **`ffi_boundary!` Macro:** Unwinding a Rust `panic!` across an `extern "C"` boundary is Undefined Behavior. All Oxide C FFI entry points are guarded by `ffi_boundary!`, which intercepts panics via `std::panic::catch_unwind` and returns safe defaults (`false`, `null`, `0`, or `OXIDE_INVALID_NODE_ID`).
- **Invalid Node Identifier `OXIDE_INVALID_NODE_ID`:** Constant `UINT64_MAX` (`0xFFFFFFFFFFFFFFFF`, in Python `(1 << 64) - 1`) is returned by node creation routines (`oxide_scene_spawn_*`) upon null pointers or runtime failures.
- **Operation Status Returns:** `oxide_scene_update`, `oxide_scene_render`, and `oxide_scene_step_physics` return `bool` (`true` on success, `false` on invalid handle or caught panic).

---

## 2. Python Integration (`bindings/python/oxide.py`)

The Python wrapper utilizes the standard `ctypes` module with zero third-party dependencies, implementing RAII deallocation via finalizers and context managers:

```python
from oxide import App, Scene, Vec3, Color, Transform

class Simulation:
    def __init__(self):
        # Scene.__del__ automatically invokes oxide_scene_destroy on garbage collection
        self.scene = Scene()
        self.cube = None

    def on_init(self, ctx):
        self.scene.set_camera_perspective(
            fov_deg=60.0,
            aspect=ctx.aspect_ratio,
            near=0.1,
            far=100.0,
            eye=Vec3(0.0, 3.0, 8.0),
            target=Vec3(0.0, 0.0, 0.0),
        )
        self.cube = self.scene.spawn_cube_3d(1.5, Transform.from_xyz(0.0, 0.0, 0.0))

    def on_update(self, ctx, dt):
        self.scene.node_rotate_y(self.cube, 1.2 * dt)
        self.scene.update(ctx, dt)

    def on_render(self, ctx):
        ctx.clear(Color(0.08, 0.10, 0.14))
        self.scene.render(ctx)

if __name__ == "__main__":
    app = App(title="Oxide Python Workspace", width=1280, height=720)
    app.run(Simulation())
```

### Scene Context Manager:
```python
with Scene() as scene:
    cube = scene.spawn_cube_3d(1.0)
    # scene.destroy() is automatically executed when exiting the with block
```

---

## 3. C / C++ Integration

Header file: [`include/oxide.h`](../../include/oxide.h)  
Example application: [`examples/c_app/main.c`](../../examples/c_app/main.c)

```c
#include "oxide.h"
#include <stdio.h>

typedef struct {
    OxideScene* scene;
    uint64_t cube_id;
} UserApp;

void on_init(OxideContext* ctx, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    app->scene = oxide_scene_create();

    OxideVec3 eye = {0.0f, 3.0f, 8.0f};
    OxideVec3 target = {0.0f, 0.0f, 0.0f};
    oxide_scene_set_camera_perspective(
        app->scene, 60.0f, oxide_context_get_aspect_ratio(ctx), 0.1f, 100.0f, eye, target
    );

    OxideTransform t = { {-1.2f, 0.0f, 0.0f}, {0.0f, 0.0f, 0.0f, 1.0f}, {1.0f, 1.0f, 1.0f} };
    app->cube_id = oxide_scene_spawn_cube_3d(app->scene, 1.5f, t);
}

void on_update(OxideContext* ctx, float dt, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    oxide_scene_node_rotate_y(app->scene, app->cube_id, 1.2f * dt);
    oxide_scene_update(app->scene, ctx, dt);
}

void on_render(OxideContext* ctx, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    OxideColor clear_col = {0.08f, 0.10f, 0.14f, 1.0f};
    oxide_context_clear(ctx, clear_col);
    oxide_scene_render(app->scene, ctx);
}

int main(void) {
    UserApp user_app = {0};
    OxideApp* app = oxide_app_create("Oxide Native C", 1280, 720);

    OxideCallbacks callbacks = {
        .on_init = on_init,
        .on_update = on_update,
        .on_render = on_render,
        .user_data = &user_app
    };

    // oxide_app_run consumes and frees the app pointer
    int result = oxide_app_run(app, callbacks);

    // Explicit deallocation of the scene graph
    oxide_scene_destroy(user_app.scene);
    return result;
}
```

---

## 4. Building the C Example

### Compile Shared Library:
```bash
cargo build --release
```

### Build via CMake:
```bash
cd examples/c_app
cmake -B build
cmake --build build --config Release
```

### Build via Makefile:
```bash
cd examples/c_app
make
./c_app
```

---

## 5. Physics C ABI and Simulation

Oxide exports a complete suite of C ABI functions for 6-DOF rigid bodies and XPBD soft body cloth simulations:

```c
// Spawn rigid body box
uint64_t oxide_scene_spawn_rigid_body_box(
    OxideScene* scene,
    OxideVec3 half_extents,
    float mass,
    OxideRigidBodyType body_type,
    OxideTransform transform
);

// Spawn rigid body sphere
uint64_t oxide_scene_spawn_rigid_body_sphere(
    OxideScene* scene,
    float radius,
    float mass,
    OxideRigidBodyType body_type,
    OxideTransform transform
);

// Force and impulse application
void oxide_scene_rigid_body_apply_force(OxideScene* scene, uint64_t node, OxideVec3 force);
void oxide_scene_rigid_body_apply_torque(OxideScene* scene, uint64_t node, OxideVec3 torque);
void oxide_scene_rigid_body_apply_impulse(OxideScene* scene, uint64_t node, OxideVec3 impulse);

// Linear and angular velocity
void oxide_scene_rigid_body_set_velocity(OxideScene* scene, uint64_t node, OxideVec3 velocity);
bool oxide_scene_rigid_body_get_velocity(const OxideScene* scene, uint64_t node, OxideVec3* out_velocity);
void oxide_scene_rigid_body_set_angular_velocity(OxideScene* scene, uint64_t node, OxideVec3 angular_velocity);
bool oxide_scene_rigid_body_get_angular_velocity(const OxideScene* scene, uint64_t node, OxideVec3* out_angular_velocity);

// Physics step invocation (also automatically performed within oxide_scene_update)
bool oxide_scene_step_physics(OxideScene* scene, float dt);

// XPBD Cloth
uint64_t oxide_scene_spawn_cloth_grid(
    OxideScene* scene,
    float width,
    float height,
    uint32_t cols,
    uint32_t rows,
    float mass_per_particle,
    float compliance,
    OxideTransform transform
);
void oxide_scene_soft_body_pin_particle(OxideScene* scene, uint64_t node, uint32_t particle_index);
```
