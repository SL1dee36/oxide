# Модуль `spatial` — Иерархический граф сцены, узлы и камеры

Модуль `spatial` реализует иерархическое дерево узлов (Node Graph), вычисление каскадных пространственных трансформаций (TRS) и модели перспективных и ортографических камер.

Исходный код: [`src/spatial/`](../../src/spatial/)

---

## 1. Пространственные трансформации (`Transform`)

Структура `Transform` представляет локальное положение объекта в трехмерном пространстве:

```rust
pub struct Transform {
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}
```

- **Модельная матрица:** `transform.matrix() -> Mat4` формирует аффинную матрицу $4 \times 4$ по формуле $M = T \cdot R \cdot S$.
- **Манипуляторы вращения:** `rotate_x(rad)`, `rotate_y(rad)`, `rotate_z(rad)` умножают текущий кватернион на инкрементальное вращение.
- **Эйлеровы углы:** `set_euler_angles(x_rad, y_rad, z_rad)` задает ориентацию в порядке конвенции YXZ.

---

## 2. Камеры и проекции (`Camera`, `Projection`)

Структура `Camera` объединяет пространственную позу (`transform: Transform`) и геометрию оптической проекции (`projection: Projection`):

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

### Основные методы `Camera`:
- `Camera::perspective(fov_y_deg, z_near, z_far)` — создает перспективную камеру с углом обзора в градусах.
- `Camera::orthographic(width, height, z_near, z_far)` — создает ортографическую камеру с заданными размерами параллелепипеда видимости.
- `look_at(eye: Vec3, target: Vec3, up: Vec3)` — пересчитывает поворот и позицию камеры для наведения на целевую точку.
- `view_matrix() -> Mat4` — возвращает правую матрицу вида (обратную матрицу мирового положения камеры).
- `projection_matrix(aspect_ratio: f32) -> Mat4` — вычисляет правую матрицу проекции OpenGL для заданного соотношения сторон.

---

---

## 3. Идентификация узлов и компоненты (`NodeId`, `Transform`)

В современной архитектуре Oxide каждый узел сцены является сущностью ECS ([`EntityId`](../ecs.md), для обратной совместимости доступен псевдоним `NodeId = EntityId`), а его свойства распределены по разреженным таблицам компонентов:

- **Локальная трансформация:** `Transform` (положение, ориентация, масштаб).
- **Мировая трансформация:** `GlobalTransform` (матрица $4 \times 4$).
- **Иерархические связи:** компоненты `Parent(EntityId)` и `Children(Vec<EntityId>)`.
- **Имя и видимость:** компоненты `Name(String)` и `Visible(bool)`.
- **Автономное поведение (`NodeBehavior`):**
  ```rust
  pub trait NodeBehavior {
      fn on_process(&mut self, entity: EntityId, world: &mut World, ctx: &mut Context, dt: f32);
  }
  ```

---

## 4. Граф сцены (`Scene`) и система иерархии (`update_hierarchy_system`)

Структура `Scene` объединяет камеру, цвет очистки и поколенческий мир ECS:

```rust
pub struct Scene {
    pub world: World,
    pub camera: Camera,
    pub clear_color: Color,
}
```

### Иерархический пересчет трансформаций (`update_hierarchy_system`):
Система обходит граф сцены в ширину (BFS), начиная с корневых узлов (`Parent == None`), гарантируя вычисление матриц родителей до детей без переполнения стека рекурсии:
$$M_{\text{global}}(\text{child}) = M_{\text{global}}(\text{parent}) \times M_{\text{local}}(\text{child})$$

### Спавн и привязка:
```rust
let parent_id = scene.spawn_empty("Anchor", Transform::from_xyz(0.0, 5.0, 0.0));
let mesh_id = scene.spawn_mesh_3d(Mesh3D::cube(1.0), Transform::IDENTITY, Some(Collider::box_3d(Vec3::ONE)));

// Построение иерархии (parent-child)
scene.attach_child(parent_id, mesh_id);
```

### Взаимодействие со сценой:
- `scene.update(ctx, dt)` — выполняет иерархический пересчет мировых матриц, физическую симуляцию (включая OBB SAT и Broadphase) и вызывает `on_process` для всех зарегистрированных поведений.
- `scene.render(ctx)` — выполняет фазы отрисовки 3D мешей, 2D поверхностей и батчированного 2D текста.
- `scene.despawn_node(id)` — безопасно удаляет узел и все его дочерние элементы, освобождая занятые GPU-ресурсы через очередь очистки VRAM.
- `scene.check_intersection(id_a, id_b)` — проверяет геометрическое перекрытие ограничивающих объемов узлов.
- `scene.resolve_penetration(id_a, id_b, &contact)` — кинематически разделяет пересекающиеся узлы.
- `scene.raycast(&ray)` — выполняет трассировку луча по всей сцене.

