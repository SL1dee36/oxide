# Модуль `collision` — Аналитический движок коллизий и пространственных запросов

Модуль `collision` предоставляет легковесную, математически точную систему проверки пересечений, формирования контактов, лучевых запросов (Raycasting) и кинематического расталкивания объектов.

Исходный код: [`src/collision/`](file:///e:/PRJCT/Oxide/src/collision/)

---

## 1. Коллидеры (`Collider`)

Перечисление `Collider` задает форму границы объема узла в локальных координатах:

```rust
pub enum Collider {
    Box3D { half_extents: Vec3 },
    Sphere { radius: f32 },
    Box2D { half_extents: Vec2 },
}

// Конструкторы
let box3d = Collider::box_3d(Vec3::new(2.0, 2.0, 2.0)); // half_extents = (1, 1, 1)
let sphere = Collider::sphere(1.5);
let box2d = Collider::box_2d(Vec2::new(100.0, 50.0));
```

---

## 2. Пространственные объемы

### `Aabb3D` (Ориентированный на оси ограничивающий параллелепипед)
- `from_center_half_extents(center: Vec3, half: Vec3) -> Self` — создание из центра и полуразмеров.
- `transform(&self, matrix: &Mat4) -> Self` — трансформация AABB произвольной матрицей 4x4 (с учетом масштабирования и поворота).
- `intersects(&self, other: &Self) -> bool` — быстрая булева проверка пересечения.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — вычисление глубины проникновения, нормали разделения и точки контакта.

### `BoundingSphere` (Ограничивающая сфера)
- `intersects(&self, other: &Self) -> bool` — проверка по квадрату расстояния между центрами.
- `contact(&self, other: &Self) -> Option<ContactManifold>` — генерация контакта вдоль вектора между центрами сфер.

### `Aabb2D` (Двумерный AABB для интерфейса)
- Аналитическое пересечение плоских прямоугольников в пиксельных координатах.

---

## 3. Структура контакта (`ContactManifold`)

При обнаружении пересечения генерируется структура:

```rust
pub struct ContactManifold {
    pub normal: Vec3, // Нормаль разделения (единичный вектор)
    pub depth: f32,   // Глубина проникновения (> 0.0)
    pub point: Vec3,  // Примерная мировая точка контакта
}
```

---

## 4. Кинематическое расталкивание (`resolve_penetration` & `resolve_penetration_weighted`)

Функция расталкивает два перекрывающихся объекта вдоль нормали контакта, устраняя проникновение с учетом статического флага (`is_static`):

```rust
if let Some(contact) = scene.check_intersection(wall_node, sphere_node) {
    // Если один из узлов помечен как статический (is_static == true),
    // все 100% смещения разделения применяются к динамическому телу.
    scene.resolve_penetration(wall_node, sphere_node, &contact);
}
```

- Если оба объекта динамические: смещение делится поровну (50% / 50%).
- Если один объект статический (стена, препятствие): статический объект остается неподвижным (0%), а динамический выталкивается на 100% глубины контакта.
- Если оба статические: смещение игнорируется.

Поддерживаются точные контакты: Box3D $\leftrightarrow$ Box3D, Sphere $\leftrightarrow$ Sphere, Box3D $\leftrightarrow$ Sphere.

---

## 5. Лучевые запросы (`Ray3D`, `RaycastHit`)

Используются для трассировки лучей и выбора объектов мышью в пространстве:

```rust
let ray = Ray3D::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));

// Запрос ко всей сцене
if let Some((hit_node_id, hit_info)) = scene.raycast(&ray) {
    println!("Попадание в узел {:?} на расстоянии {:.2}", hit_node_id, hit_info.distance);
    println!("Точка контакта: {:?}", hit_info.point);
    println!("Нормаль поверхности: {:?}", hit_info.normal);
}
```
