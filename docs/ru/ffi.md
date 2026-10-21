# Модуль `ffi` — Внешний интерфейс C-ABI и мультиязычные биндинги

Модуль `ffi` предоставляет стабильный C-совместимый интерфейс (`extern "C"`), позволяющий использовать Oxide из **Python**, **C**, **C++**, **C#**, **Zig** и любых других языков.

Исходный код: [`src/ffi/`](file:///e:/PRJCT/Oxide/src/ffi/)  
Заголовочный файл C/C++: [`include/oxide.h`](file:///e:/PRJCT/Oxide/include/oxide.h)  
Python-обертка: [`bindings/python/oxide.py`](file:///e:/PRJCT/Oxide/bindings/python/oxide.py)

---

## 1. Сборка динамической библиотеки

Чтобы собрать библиотеку `oxide.dll` (Windows) или `liboxide.so` (Linux), выполните:

```bash
cargo build --lib --release
```

Файл библиотеки появится в каталоге `target/release/`.

---

## 2. Разработка на Python

Пакет `bindings/python/oxide.py` работает на стандартной библиотеке `ctypes` без необходимости установки сторонних пакетов через `pip`:

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

Запуск:
```bash
python examples/python_simulation.py
```

---

## 3. Разработка на C / C++

Подключите заголовок `oxide.h` и слинкуйте исполняемый файл с `oxide.lib` / `oxide.dll`:

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

## 4. Спецификация C-функций API

| Функция | Назначение |
| :--- | :--- |
| `oxide_app_create(title, w, h)` | Выделяет структуру приложения в куче |
| `oxide_app_run(app, callbacks)` | Запускает окно и цикл событий, передавая управление функциям обратного вызова |
| `oxide_scene_create()` | Создает независимый граф сцены |
| `oxide_scene_spawn_cube_3d(scene, size, transform)` | Создает 3D куб с коллидером |
| `oxide_scene_spawn_sphere_3d(scene, radius, transform)` | Создает 3D сферу с коллидером |
| `oxide_scene_spawn_surface_2d(scene, size, color, transform)` | Создает 2D поверхность |
| `oxide_scene_spawn_text_2d(scene, text, color, scale, transform)` | Создает 2D текст |
| `oxide_scene_check_collision(scene, id_a, id_b, out_contact)` | Возвращает `true`, если обнаружено пересечение |
| `oxide_scene_resolve_penetration(scene, id_a, id_b, contact)` | Кинематически расталкивает объекты |
| `oxide_context_clear(ctx, color)` | Очищает цвет и глубину кадра |
| `oxide_context_get_fps(ctx)` | Возвращает актуальный FPS |
