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
- `Camera::perspective(fov_y_deg, aspect, z_near, z_far)` — создает перспективную камеру с углом обзора в градусах.
- `Camera::orthographic(width, height, z_near, z_far)` — создает ортографическую камеру с заданными размерами параллелепипеда видимости.
- `look_at(eye: Vec3, target: Vec3, up: Vec3)` — пересчитывает поворот и позицию камеры для наведения на целевую точку.
- `view_matrix() -> Mat4` — возвращает правую матрицу вида (обратную матрицу мирового положения камеры).
- `projection_matrix(aspect_ratio: f32) -> Mat4` — вычисляет правую матрицу проекции OpenGL для заданного соотношения сторон.

---

## 3. Узел сцены (`SpatialNode`)

Каждый объект в графе сцены представлен структурой `SpatialNode`:

```rust
pub struct SpatialNode {
    pub id: NodeId,
    pub name: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub transform: Transform,
    pub global_transform: Mat4,
    pub kind: NodeKind,
    pub collider: Option<Collider>,
    pub visible: bool,
    pub is_static: bool,
    pub behavior: Option<Box<dyn NodeBehavior>>,
}
```

### Разновидности полезной нагрузки (`NodeKind`):
- `NodeKind::Empty` — точка привязки, вспомогательный шарнир или родительская группа.
- `NodeKind::Mesh3D(Mesh3D)` — трехмерная полигональная сетка.
- `NodeKind::Surface2D(Surface2D)` — плоская прямоугольная панель в экранных координатах.
- `NodeKind::Text2D(Text2D)` — экранный текстовый элемент.

### Автономное поведение (`NodeBehavior`):
Трейт для привязки логики симуляции к отдельным узлам:
```rust
pub trait NodeBehavior {
    fn on_process(&mut self, node: &mut SpatialNode, ctx: &mut Context, dt: f32);
}
```

---

## 4. Граф сцены (`Scene`)

`Scene` управляет иерархией узлов, источниками света, камерой и циклами обновления:

### Рекурсивный пересчет трансформаций:
Метод `update_transforms()` обходит иерархию узлов сверху вниз:
$$M_{\text{global}}(\text{child}) = M_{\text{global}}(\text{parent}) \times M_{\text{local}}(\text{child})$$
Если узел не имеет родителя (`parent == None`), его глобальная матрица совпадает с локальной матрицей `transform.matrix()`.

### Спавн и привязка:
```rust
let parent_id = scene.spawn_node("Anchor", NodeKind::Empty, Transform::from_xyz(0.0, 5.0, 0.0), None);
let mesh_id = scene.spawn_mesh_3d(Mesh3D::cube(1.0), Transform::IDENTITY, Collider::box_3d(Vec3::ONE));

// Построение иерархии (parent-child)
scene.attach_child(parent_id, mesh_id);
```

### Взаимодействие со сценой:
- `scene.update(ctx, dt)` — пересчитывает мировые матрицы узлов и вызывает `on_process` для всех зарегистрированных поведений.
- `scene.render(ctx)` — выполняет фазы отрисовки 3D мешей, 2D поверхностей и 2D текста.
- `scene.check_intersection(id_a, id_b)` — проверяет геометрическое перекрытие ограничивающих объемов узлов.
- `scene.resolve_penetration(id_a, id_b, &contact)` — кинематически разделяет пересекающиеся узлы.
- `scene.raycast(&ray)` — выполняет трассировку луча по всей сцене.
