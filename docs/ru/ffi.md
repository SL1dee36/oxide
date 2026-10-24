# Модуль `ffi` — Внешний интерфейс C ABI и модель управления памятью

Модуль `ffi` экспортирует стабильный ANSI C99-совместимый интерфейс (`extern "C"`), позволяющий вызывать функции Oxide из C, C++, Python, C#, Zig и любых языков с поддержкой C ABI.

Исходный код: [`src/ffi/`](../../src/ffi/)  
Заголовочный файл C/C++: [`include/oxide.h`](../../include/oxide.h)  
Python-обертка: [`bindings/python/oxide.py`](../../bindings/python/oxide.py)

---

## 1. Модель владения и времени жизни памяти (Ownership & Lifetimes)

Взаимодействие через границу FFI подчинено строгим правилам управления памятью между кучей Rust и клиентским кодом:

```mermaid
flowchart TD
    subgraph RustHeap [Куча Rust (Heap Allocation)]
        AppAlloc["oxide_app_create()<br/>Box::into_raw(Box::new(OxideApp))"]
        SceneAlloc["oxide_scene_create()<br/>Box::into_raw(Box::new(OxideScene))"]
    end

    subgraph ClientCode [Клиентский код (C / Python)]
        AppPtr["*mut OxideApp (Непрозрачный указатель)"]
        ScenePtr["*mut OxideScene (Непрозрачный указатель)"]
        CtxPtr["*mut OxideContext (Временная ссылка)"]
    end

    AppAlloc --> AppPtr
    SceneAlloc --> ScenePtr

    AppPtr -->|"oxide_app_run(app, callbacks)"| AppConsumed["Box::from_raw(app) -> Освобождается при выходе из цикла"]
    AppPtr -->|"oxide_app_destroy(app)<br/>(только если не вызывался run)"| AppFreed["drop(Box::from_raw(app))"]
    ScenePtr -->|"oxide_scene_destroy(scene)<br/>(Обязательный вызов)"| SceneFreed["drop(Box::from_raw(scene))"]

    AppConsumed -.-> CtxPtr
    NoteCtx["OxideContext валиден ТОЛЬКО внутри callback-функции!"] -.-> CtxPtr
```

### 1.1. Создание и освобождение приложения (`OxideApp`):
- `oxide_app_create(title, width, height) -> *mut OxideApp` выделяет память под структуру `OxideApp` в куче Rust через `Box::into_raw` и возвращает непрозрачный указатель.
- **Потребление при запуске:** Функция `oxide_app_run(app, callbacks)` принимает владение указателем `app`, восстанавливает `Box::from_raw(app)` и **автоматически освобождает память** при выходе из цикла событий.
- **Ручное освобождение:** Функция `oxide_app_destroy(app)` вызывается **только в том случае, если `oxide_app_run` не вызывалась** (например, при отмене запуска). Повторный вызов `oxide_app_destroy` после `oxide_app_run` недопустим и приводит к двойному освобождению (double-free).

### 1.2. Создание и освобождение сцены (`OxideScene`):
- `oxide_scene_create() -> *mut OxideScene` аллоцирует граф сцены в куче Rust.
- Клиентский код **обязан явно освободить память сцены** вызовом `oxide_scene_destroy(scene)` после завершения работы.

### 1.3. Контекст выполнения кадра (`OxideContext`):
- Передается по указателю в функции обратного вызова `on_init`, `on_update`, `on_render`.
- Является временной ссылкой на стек Rust-рантайма. **Запрещено сохранять указатель `OxideContext`** для использования вне тела обратного вызова.

### 1.4. Защита от паник и обработка ошибок:
- **Макрос `ffi_boundary!`:** Проброс Rust `panic!` через границу `extern "C"` является неопределённым поведением (Undefined Behavior). Все FFI-функции Oxide защищены макросом `ffi_boundary!`, который перехватывает паники с помощью `std::panic::catch_unwind` и возвращает безопасные значения по умолчанию (дефолтные указатели, `false`, `0` или `OXIDE_INVALID_NODE_ID`).
- **Идентификатор ошибки узла `OXIDE_INVALID_NODE_ID`:** Константа `UINT64_MAX` (`0xFFFFFFFFFFFFFFFF`, в Python `(1 << 64) - 1`) возвращается функциями создания сущностей (`oxide_scene_spawn_*`) в случае передачи невалидного указателя сцены или внутренней ошибки.
- **Статус выполнения операций:** Функции `oxide_scene_update`, `oxide_scene_render` и `oxide_scene_step_physics` возвращают `bool` (`true` при успешном выполнении, `false` при невалидном указателе или перехваченной панике).

---

## 2. Разработка на Python (`bindings/python/oxide.py`)

Обертка на базе стандартной библиотеки `ctypes` не требует сторонних зависимостей (`pip`) и реализует автоматическое освобождение памяти через деструкторы и контекстные менеджеры:

```python
from oxide import App, Scene, Vec3, Color, Transform

class Simulation:
    def __init__(self):
        # Автоматическое управление: Scene.__del__ вызовет oxide_scene_destroy
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

### Контекстный менеджер для сцены:
```python
with Scene() as scene:
    cube = scene.spawn_cube_3d(1.0)
    # По выходу из блока with автоматически вызывается scene.destroy()
```

---

## 3. Разработка на C / C++

Заголовочный файл: [`include/oxide.h`](../../include/oxide.h)  
Пример приложения: [`examples/c_app/main.c`](../../examples/c_app/main.c)

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

    // oxide_app_run поглощает и освобождает указатель app
    int result = oxide_app_run(app, callbacks);

    // Обязательное ручное освобождение сцены
    oxide_scene_destroy(user_app.scene);
    return result;
}
```

---

## 4. Сборка C-приложения

### Сборка библиотеки Oxide:
```bash
cargo build --release
```

### Сборка через CMake:
```bash
cd examples/c_app
cmake -B build
cmake --build build --config Release
```

### Сборка через Makefile:
```bash
cd examples/c_app
make
./c_app
```

---

## 5. Физический C ABI и симуляция

Oxide предоставляет полный набор функций для работы с 6-DOF твердыми телами и XPBD-тканями через C ABI:

```c
// Спавн твердого тела в форме параллелепипеда
uint64_t oxide_scene_spawn_rigid_body_box(
    OxideScene* scene,
    OxideVec3 half_extents,
    float mass,
    OxideRigidBodyType body_type,
    OxideTransform transform
);

// Спавн твердого тела в форме сферы
uint64_t oxide_scene_spawn_rigid_body_sphere(
    OxideScene* scene,
    float radius,
    float mass,
    OxideRigidBodyType body_type,
    OxideTransform transform
);

// Воздействие силами и импульсами
void oxide_scene_rigid_body_apply_force(OxideScene* scene, uint64_t node, OxideVec3 force);
void oxide_scene_rigid_body_apply_torque(OxideScene* scene, uint64_t node, OxideVec3 torque);
void oxide_scene_rigid_body_apply_impulse(OxideScene* scene, uint64_t node, OxideVec3 impulse);

// Линейная и угловая скорость
void oxide_scene_rigid_body_set_velocity(OxideScene* scene, uint64_t node, OxideVec3 velocity);
bool oxide_scene_rigid_body_get_velocity(const OxideScene* scene, uint64_t node, OxideVec3* out_velocity);
void oxide_scene_rigid_body_set_angular_velocity(OxideScene* scene, uint64_t node, OxideVec3 angular_velocity);
bool oxide_scene_rigid_body_get_angular_velocity(const OxideScene* scene, uint64_t node, OxideVec3* out_angular_velocity);

// Шаг физической симуляции (автоматически вызывается внутри oxide_scene_update)
bool oxide_scene_step_physics(OxideScene* scene, float dt);

// XPBD-ткань
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
