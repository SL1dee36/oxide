# Модуль `spatial` — Пространственный граф сцены и узлы

Модуль `spatial` реализует иерархическое дерево узлов (Node Graph, аналогично архитектуре Godot), математику пространственных трансформаций (TRS) и камеры.

Исходный код: [`src/spatial/`](file:///e:/PRJCT/Oxide/src/spatial/)

---

## 1. Трансформация (`Transform`)

Структура `Transform` хранит локальное пространственное состояние объекта через тройку: Позиция (`translation: Vec3`), Ориентация (`rotation: Quat`), Масштаб (`scale: Vec3`).

```rust
// Создание
let mut transform = Transform::from_translation(Vec3::new(0.0, 2.0, -5.0));

// Манипуляции
transform.rotate_y(1.5 * dt);
transform.rotate_x(0.5 * dt);
transform.translate(Vec3::new(0.1, 0.0, 0.0));

// Матрица модели 4x4
let mat: Mat4 = transform.matrix();
```

---

## 2. Камера (`Camera`)

Поддерживает перспективную и ортографическую проекции:

```rust
// Перспективная камера для 3D сцены
let mut camera = Camera::perspective(60.0, ctx.aspect_ratio(), 0.1, 100.0);
camera.look_at(
    Vec3::new(0.0, 5.0, 10.0), // Позиция глаза
    Vec3::new(0.0, 0.0, 0.0),  // Точка фокуса
    Vec3::Y,                   // Вектор "вверх"
);

// Матрицы
let view = camera.view_matrix();
let proj = camera.projection_matrix(ctx.aspect_ratio());
```

---

## 3. Узлы сцены (`SpatialNode`)

Каждый элемент в сцене представлен структурой `SpatialNode`:
- `id: NodeId` — уникальный дескриптор узла в сцене.
- `parent: Option<NodeId>` — ссылка на родительский узел.
- `children: Vec<NodeId>` — список дочерних узлов.
- `transform: Transform` — локальная трансформация относительно родителя.
- `global_transform: Mat4` — кэшированная глобальная матрица в мировом пространстве.
- `kind: NodeKind`:
  - `Empty` — служебный узел (точка привязки, группировка).
  - `Mesh3D(Mesh3D)` — 3D-геометрия.
  - `Surface2D(Surface2D)` — 2D-поверхность.
  - `Text2D(Text2D)` — 2D-текст.
- `collider: Option<Collider>` — объем коллизии для пространственных запросов.
- `behavior: Option<Box<dyn NodeBehavior>>` — автономный скрипт/поведение.

### Трейт поведения `NodeBehavior`:
```rust
pub trait NodeBehavior {
    fn on_process(&mut self, node: &mut SpatialNode, ctx: &mut Context, dt: f32) {}
}
```

---

## 4. Контейнер сцены (`Scene`)

`Scene` управляет плоским массивом узлов, деревом иерархии и циклами обновления:

### Наследование трансформаций:
При вызове `scene.update_transforms()` или перед каждым рендером, сцена рекурсивно вычисляет мировые матрицы:
$$\text{global\_transform}_{\text{child}} = \text{global\_transform}_{\text{parent}} \times \text{local\_matrix}_{\text{child}}$$

### Спавн узлов:
```rust
let cube_id = scene.spawn_mesh_3d(mesh, transform, collider);
let panel_id = scene.spawn_surface_2d(surface, transform, collider);
let text_id = scene.spawn_text_2d(text, transform);

// Привязка в иерархию (parent-child)
scene.attach_child(parent_id, child_id);
```

### Пространственные запросы:
- `scene.check_intersection(node_a, node_b) -> Option<ContactManifold>` — проверяет пересечение коллидеров узлов.
- `scene.resolve_penetration(node_a, node_b, &contact)` — кинематически раздвигает узлы вдоль нормали контакта.
- `scene.raycast(&ray) -> Option<(NodeId, RaycastHit)>` — находит ближайший узел, пересекаемый лучом.
