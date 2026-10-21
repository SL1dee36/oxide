# Module `ffi` — C-ABI Foreign Function Interface & Multi-Language Bindings

The `ffi` module delivers a stable C-compatible interface (`extern "C"`), enabling native application development in **Python**, **C**, **C++**, **C#**, **Zig**, and other environments.

Source files: [`src/ffi/`](file:///e:/PRJCT/Oxide/src/ffi/)  
C/C++ Header: [`include/oxide.h`](file:///e:/PRJCT/Oxide/include/oxide.h)  
Python Wrapper: [`bindings/python/oxide.py`](file:///e:/PRJCT/Oxide/bindings/python/oxide.py)

---

## 1. Building the Shared Library

To compile `oxide.dll` (Windows) or `liboxide.so` (Linux), run:

```bash
cargo build --lib --release
```

The compiled shared binary is emitted to `target/release/`.

---

## 2. Python Development

The `bindings/python/oxide.py` layer operates via the standard library `ctypes` without requiring any external packages:

```python
import sys
sys.path.append("bindings/python")
from oxide import App, Scene, Vec3, Vec2, Color, Transform

class Simulation:
    def __init__(self):
        self.scene = Scene()
        self.cube = None

    def on_init(self, ctx):
        self.scene.set_camera_perspective(
            fov_deg=60.0, aspect=ctx.aspect_ratio, near=0.1, far=100.0,
            eye=Vec3(0, 3, 8), target=Vec3(0, 0, 0)
        )
        self.cube = self.scene.spawn_cube_3d(1.5, Transform.from_xyz(-1.2, 0, 0))

    def on_update(self, ctx, dt):
        self.scene.node_rotate_y(self.cube, 1.2 * dt)
        self.scene.update(ctx, dt)

    def on_render(self, ctx):
        ctx.clear(Color(0.08, 0.09, 0.12))
        self.scene.render(ctx)

if __name__ == "__main__":
    app = App(title="Python Oxide Simulation", width=1280, height=720)
    app.run(Simulation())
```

Execute with:
```bash
python examples/python_simulation.py
```

---

## 3. C / C++ Development

Include `oxide.h` and link against `oxide.lib` / `oxide.dll`:

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
    OxideColor clear_col = {0.08f, 0.09f, 0.12f, 1.0f};
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

    int result = oxide_app_run(app, callbacks);
    oxide_scene_destroy(user_app.scene);
    return result;
}
```

---

## 4. C Function API Specification

| Function | Description |
| :--- | :--- |
| `oxide_app_create(title, w, h)` | Allocates heap application instance |
| `oxide_app_run(app, callbacks)` | Starts window and event loop, invoking C function pointers |
| `oxide_scene_create()` | Allocates an independent scene graph |
| `oxide_scene_spawn_cube_3d(scene, size, transform)` | Spawns a 3D cube node with Box3D collider |
| `oxide_scene_spawn_sphere_3d(scene, radius, transform)` | Spawns a 3D sphere node with Sphere collider |
| `oxide_scene_spawn_surface_2d(scene, size, color, transform)` | Spawns a 2D planar surface node |
| `oxide_scene_spawn_text_2d(scene, text, color, scale, transform)` | Spawns a 2D text node |
| `oxide_scene_check_collision(scene, id_a, id_b, out_contact)` | Returns `true` if volumes intersect, outputting contact info |
| `oxide_scene_resolve_penetration(scene, id_a, id_b, contact)` | Kinematically pushes overlapping bodies apart |
| `oxide_context_clear(ctx, color)` | Clears framebuffer color and depth targets |
| `oxide_context_get_fps(ctx)` | Retrieves smoothed FPS metric |
