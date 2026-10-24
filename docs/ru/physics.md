# Модуль `physics` — Динамика твёрдых тел и XPBD-симуляция мягких тел

Модуль `physics` предоставляет полноценную систему симуляции физики в реальном времени, объединяющую 6-DOF динамику твердых тел (RigidBody) с импульсным солвером и подшаговый XPBD-солвер деформируемых тел и ткани (SoftBody).

Исходный код: [`src/physics/`](../../src/physics/)

---

## 1. Динамика твёрдых тел (RigidBody, 6-DOF)

Компонент `RigidBody` описывает кинематическое и динамическое поведение трехмерных объектов с 6 степенями свободы (3 поступательные + 3 вращательные).

```rust
pub struct RigidBody {
    pub body_type: RigidBodyType, // Dynamic, Static, Kinematic
    pub mass: f32,
    pub inv_mass: f32,
    pub inv_inertia_local: Mat3,  // Локальный обратный тензор инерции
    pub velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    pub restitution: f32,         // Коэффициент упругости [0..1]
    pub friction: f32,            // Коэффициент трения [0..1]
    pub linear_damping: f32,
    pub angular_damping: f32,
}
```

### 1.1. Тензоры инерции
При спавне твердого тела движок аналитически рассчитывает локальный тензор инерции $I_{\text{local}}$:
- **Сфера радиуса $r$:** $I = \frac{2}{5} m r^2 \cdot \mathbf{I}_{3\times3}$
- **Параллелепипед полуразмеров $(h_x, h_y, h_z)$:**  
  $I_{xx} = \frac{1}{12} m ((2h_y)^2 + (2h_z)^2)$,  
  $I_{yy} = \frac{1}{12} m ((2h_x)^2 + (2h_z)^2)$,  
  $I_{zz} = \frac{1}{12} m ((2h_x)^2 + (2h_y)^2)$.

В мировом пространстве обратный тензор пересчитывается с учетом текущей ориентации:
$$I_{\text{world}}^{-1} = R \cdot I_{\text{local}}^{-1} \cdot R^T$$

### 1.2. Конфигурация физики (`PhysicsConfig`)
```rust
pub struct PhysicsConfig {
    pub gravity: Vec3,                 // Вектор гравитации (по умолчанию: [0, -9.81, 0])
    pub velocity_iterations: usize,    // Итерации импульсного солвера (по умолчанию: 4)
    pub position_iterations: usize,    // Итерации релаксации проникновения (по умолчанию: 2)
}
```

---

## 2. Конвейер физического шага (`step_rigid_body_system`)

Симуляция каждого шага выполняется в 5 строго детерминированных фаз:

1. **Фаза 1: Численное интегрирование (Semi-Implicit Euler):**
   - Линейное ускорение: $\mathbf{a} = \mathbf{g} + m^{-1} \mathbf{F}_{\text{accum}}$.
   - Угловое ускорение: $\boldsymbol{\alpha} = I_{\text{world}}^{-1} \cdot \boldsymbol{\tau}_{\text{accum}}$.
   - Скорости: $\mathbf{v} \mathrel{+}= \mathbf{a} \Delta t$, $\boldsymbol{\omega} \mathrel{+}= \boldsymbol{\alpha} \Delta t$.
   - Позиция: $\mathbf{x} \mathrel{+}= \mathbf{v} \Delta t$.
   - Ориентация: $q \mathrel{+}= \frac{1}{2} (\boldsymbol{\omega}, 0) \otimes q$, с последующей нормализацией.

2. **Фаза 2: Обновление мировой иерархии:**  
   Вызов `update_hierarchy_system` синхронизирует мировые матрицы `global_transform`.

3. **Фаза 3: Детекция коллизий (Broadphase + Narrowphase):**
   - **Широкая фаза (`BroadphaseGrid`):** Алгоритм Sort-and-Sweep по оси X отсекает до 99.9% непересекающихся пар и пары статических препятствий за $O(N \log N)$.
   - **Узкая фаза (`check_collider_intersection`):** Точный SAT-солвер `Obb3D` формирует контактные манифолды с нормалью, глубиной и точкой контакта $\mathbf{p}$.

4. **Фаза 4: Итерации импульсного солвера (`velocity_iterations`):**
   - Расчет плеч сил: $\mathbf{r}_a = \mathbf{p} - \mathbf{c}_a$, $\mathbf{r}_b = \mathbf{p} - \mathbf{c}_b$.
   - Скорость в точке контакта: $\mathbf{v}_{\text{rel}} = (\mathbf{v}_b + \boldsymbol{\omega}_b \times \mathbf{r}_b) - (\mathbf{v}_a + \boldsymbol{\omega}_a \times \mathbf{r}_a)$.
   - Эффективная масса вдоль нормали:
     $$K_n = m_a^{-1} + m_b^{-1} + (\mathbf{r}_a \times \mathbf{n})^T I_a^{-1} (\mathbf{r}_a \times \mathbf{n}) + (\mathbf{r}_b \times \mathbf{n})^T I_b^{-1} (\mathbf{r}_b \times \mathbf{n})$$
   - Нормальный импульс упругости и тангенциальный импульс трения Кулона.

5. **Фаза 5: Итерации релаксации позиций (`position_iterations`):**  
   Расталкивание остаточного геометрического проникновения вдоль нормали с учетом флагов неподвижности.

---

## 3. Симуляция ткани и мягких тел (XPBD SoftBody)

Подсистема `SoftBody` реализует метод Extended Position-Based Dynamics (XPBD) для стабильной симуляции ткани без взрывов численной интеграции:

- **Частицы (`SoftParticle`):** Хранят текущую позицию, предыдущую позицию $\mathbf{x}_{\text{prev}}$, скорость и флаг фиксации (`is_fixed`).
- **Ограничения расстояния (`DistanceConstraint`):** Поддерживают структурные связи между вершинами сетки ткани с настраиваемой податливостью (compliance $\alpha$).
- **Подшаги (Sub-stepping):** В пределах одного кадра выполняется 4–8 микрошагов с пересчетом ограничений и столкновений с неподвижными препятствиями.
- **Синхронизация с мешем:** Позиции частиц напрямую обновляют вершины `Mesh3D`, обеспечивая рендеринг деформируемой ткани в реальном времени.
