# Модуль `ecs` — Архитектура сущностей и компонентов (ECS)

Модуль `ecs` реализует поколенческий (generational) шаблон проектирования Entity-Component-System (ECS), предоставляя централизованное хранилище игровых объектов, раздельные таблицы компонентов и конвейер системных обновлений.

Исходный код: [`src/ecs/`](../../src/ecs/)

---

## 1. Концепция и архитектурная модель

Традиционные иерархии объектов сопряжены с проблемами множественного наследования, разброса данных в памяти и хрупких ссылок. В Oxide применен строгий ECS-подход:

- **Entity (Сущность):** `EntityId` — легковесный 64-битный идентификатор слота и поколения (`slotmap::new_key_type!`), исключающий проблему ABA при переиспользовании памяти удаленных сущностей.
- **Component (Компонент):** Чистые структуры данных, хранящиеся во вторичных разреженных массивах (`slotmap::SecondaryMap<EntityId, T>`).
- **System (Система):** Функции без внутреннего состояния, обрабатывающие срезы сущностей с нужными наборами компонентов (`update_hierarchy_system`, `update_behaviors_system`, `step_physics_system`, `RenderSystem`).

---

## 2. Центральный реестр `World`

Структура `World` управляет жизненным циклом сущностей и распределенными таблицами компонентов:

```rust
pub struct World {
    pub entities: SlotMap<EntityId, ()>,
    pub transform: SecondaryMap<EntityId, Transform>,
    pub global_transform: SecondaryMap<EntityId, Mat4>,
    pub parent: SecondaryMap<EntityId, EntityId>,
    pub children: SecondaryMap<EntityId, Vec<EntityId>>,
    pub visible: SecondaryMap<EntityId, Visible>,
    pub collider: SecondaryMap<EntityId, Collider>,
    pub rigid_body: SecondaryMap<EntityId, RigidBody>,
    pub soft_body: SecondaryMap<EntityId, SoftBody>,
    pub mesh_3d: SecondaryMap<EntityId, Mesh3D>,
    pub surface_2d: SecondaryMap<EntityId, Surface2D>,
    pub text_2d: SecondaryMap<EntityId, Text2D>,
    pub behavior: SecondaryMap<EntityId, Box<dyn NodeBehavior>>,
    pub pending_gpu_cleanup: Vec<PendingGpuMeshCleanup>,
    // Камера и направленный свет
    camera: Camera,
    light: Light3D,
}
```

### Основные методы жизненного цикла:
- `spawn() -> EntityId` — выделяет новую живую сущность.
- `despawn(entity: EntityId) -> bool` — удаляет сущность, рекурсивно отсоединяет дочерние узлы и переносит выделенные GPU-ресурсы меша в очередь очистки видеопамяти.
- `attach_child(parent: EntityId, child: EntityId)` — выстраивает пространственную иерархию отношений «родитель-потомок».
- `is_alive(entity: EntityId) -> bool` — проверяет валидность сущности и совпадение поколения слота.

---

## 3. Таблицы компонентов

| Компонент | Тип | Описание |
| :--- | :--- | :--- |
| **`Transform`** | `Transform` | Локальная трансляция (`Vec3`), ориентация (`Quat`) и масштаб (`Vec3`). |
| **`global_transform`** | `Mat4` | Кешированная мировая аффинная матрица трансформации ($M_{\text{parent}} \times M_{\text{local}}$). |
| **`Visible`** | `Visible(pub bool)` | Флаг видимости при рендеринге и рейкастинге. |
| **`Name`** | `Name(pub String)` | Человекочитаемое имя узла для отладки и инспекции. |
| **`Collider`** | `Collider` | Ограничивающий объем (`Box3D`, `Sphere`, `Box2D`). |
| **`RigidBody`** | `RigidBody` | 6-DOF физические параметры (масса, скорость, тензор инерции, трение). |
| **`SoftBody`** | `SoftBody` | XPBD-частицы ткани, структурные ограничения расстояния, податливость. |
| **`Mesh3D`** | `Mesh3D` | Полигональная геометрия, текстура `Arc<Texture2D>` и GPU-буферы (`VAO`, `VBO`, `EBO`). |
| **`Surface2D`** | `Surface2D` | Экранный прямоугольник интерфейса со скруглением и цветом. |
| **`Text2D`** | `Text2D` | Текстовая строка с параметрами масштаба и цвета. |
| **`NodeBehavior`** | `Box<dyn NodeBehavior>` | Пользовательская логика обновления кадра (`on_update(ctx, dt)`). |

---

## 4. Очистка видеопамяти (`PendingGpuMeshCleanup`)

При удалении сущности (`World::despawn`) из оперативной памяти Rust освобождает структуры `Mesh3D`. Однако OpenGL-объекты (`glGenVertexArrays`, `glGenBuffers`) не могут быть освобождены в произвольном потоке или без активного графического контекста.

Для предотвращения утечек VRAM внедрен механизм отложенного уничтожения:
1. При `despawn` дескрипторы `vao`, `vbo`, `ebo` перемещаются в очередь `world.pending_gpu_cleanup`.
2. В начале каждого кадра в `RenderSystem::render_world` накопившаяся очередь сбрасывается и вызывает `destroy(gl)` в активном потоке рендеринга.

---

## 5. Системные обновления

- **`update_hierarchy_system(world: &mut World)`:** Вычисляет мировые матрицы `global_transform` обходом графа в глубину от корневых узлов к листьям.
- **`update_behaviors_system(world: &mut World, ctx: &mut Context, dt: f32)`:** Запускает пользовательские обработчики `NodeBehavior`.
- **`world.update(ctx, dt)`:** Комплексный метод шага кадра, вызывающий физику (`step_physics_system`), пересчет иерархии и пользовательские скрипты поведения.
