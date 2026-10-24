# Модуль `assets` — Геометрия в ОЗУ и загрузка ресурсов

Модуль `assets` отвечает за хранение геометрических сеток в оперативной памяти (`Mesh3D`), описание плоских двумерных элементов (`Surface2D`), процедурную генерацию примитивов и парсинг файлов форматов Wavefront OBJ и растровых изображений.

Исходный код: [`src/assets/`](../../src/assets/)

---

## 1. Структура полигонального меша (`Mesh3D`)

`Mesh3D` хранит геометрию в системной оперативной памяти хоста. Буферы видеопамяти (`vao`, `vbo`, `ebo`) инициализируются лениво:

```rust
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub color: Color,
    pub emissive: Color,
    pub texture: Option<Arc<Texture2D>>,
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}
```

### Процедурные генераторы примитивов:
- `Mesh3D::cube(size: f32) -> Self` — генерирует параллелепипед из 24 вершин и 36 индексов с независимыми нормалями для каждой из 6 граней и UV-координатами `[0.0, 1.0]`.
- `Mesh3D::plane(width: f32, depth: f32) -> Self` — генерирует горизонтальный четырехугольник в плоскости XZ (нормаль `[0.0, 1.0, 0.0]`) из 4 вершин и 6 индексов.
- `Mesh3D::sphere(radius: f32, rings: u32, sectors: u32) -> Self` — генерирует параметрическую UV-сферу с заданным числом параллелей (`rings`) и меридианов (`sectors`).

### Выделение буферов в видеопамяти и управление ресурсами:
```rust
pub fn ensure_gpu_buffers(&mut self, gl: &glow::Context) -> Result<(), OxideError>;
```
Проверяет наличие `self.vao`. Если буферы еще не аллоцированы, создает `VertexArray`, `VertexBuffer` и `IndexBuffer`, настраивает атрибуты вершин через `Pipeline3D::setup_vertex_attributes` и сохраняет дескрипторы в структуре.

#### Безопасная очистка видеопамяти (VRAM cleanup):
При удалении сущности меша через `world.despawn(entity)` или `world.despawn_recursive(entity)` буферы `vao`, `vbo`, `ebo` не теряются в VRAM. Они перемещаются в очередь `world.pending_gpu_cleanup` (`PendingGpuMeshCleanup`). В начале каждого кадра `RenderSystem::render_world` очищает очередь, корректно вызывая `destroy(gl)` для всех освобождённых буферов.

---

## 2. Двумерная поверхность (`Surface2D`)

Структура `Surface2D` описывает параметры плоского прямоугольного элемента (размер в пикселях, цвет и опциональную текстуру) без аллокации собственных полигональных сеток, так как `Pipeline2D` переиспользует общий единичный квад:

```rust
pub struct Surface2D {
    pub size: glam::Vec2,
    pub color: Color,
    pub texture: Option<Arc<Texture2D>>,
}
```

Конструкторы:
- `Surface2D::new(size: Vec2) -> Self`
- `Surface2D::from_color(size: Vec2, color: Color) -> Self`

---

## 3. Функции загрузки ресурсов (`loader.rs`)

Подсистема загрузки обеспечивает десериализацию как из файлов на диске, так и напрямую из срезов памяти (например, для встраивания ресурсов через `include_bytes!`):

```rust
// Загрузка 3D-модели OBJ из оперативной памяти
pub fn load_obj_from_memory(bytes: &[u8]) -> Result<Mesh3D, OxideError>;

// Загрузка 3D-модели OBJ с диска
pub fn load_obj_file(path: impl AsRef<Path>) -> Result<Mesh3D, OxideError>;

// Декодирование изображения (PNG/JPEG) и загрузка в текстуру GPU из памяти
pub fn load_texture_from_memory(gl: &glow::Context, bytes: &[u8]) -> Result<Texture2D, OxideError>;

// Чтение изображения с диска и загрузка в текстуру GPU
pub fn load_texture_file(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Texture2D, OxideError>;
```

---

## 4. Этапы передачи ресурсов в конвейеры рендеринга

### Передача 3D-меша в `Pipeline3D`:
1. Загрузка файла модели в ОЗУ и текстуры на GPU:
   ```rust
   let mut mesh = load_obj_file("assets/models/component.obj")?;
   let texture = Arc::new(load_texture_file(ctx.gl, "assets/textures/diffuse.png")?);
   mesh.texture = Some(texture);
   ```
2. Аллокация буферов GPU при наличии контекста:
   ```rust
   mesh.ensure_gpu_buffers(ctx.gl)?;
   ```
3. Отрисовка через конвейер:
   ```rust
   if let (Some(vao), Some(ebo)) = (&mesh.vao, &mesh.ebo) {
       pipeline_3d.draw(ctx.gl, &model_matrix, mesh.color, mesh.emissive, mesh.texture.as_deref(), vao, ebo);
   }
   ```
   *(При рендеринге сцены `RenderSystem::render_world` автоматически выполняет `ensure_gpu_buffers` и передаёт `mesh.texture.as_deref()` в `pipeline_3d.draw`).*

### Передача 2D-поверхности в `Pipeline2D`:
1. Создание описания поверхности (с опциональной текстурой):
   ```rust
   let mut surface = Surface2D::from_color(Vec2::new(300.0, 80.0), Color::rgba(0.1, 0.15, 0.25, 0.9));
   surface.texture = Some(my_texture);
   ```
2. Формирование матрицы масштабирования и переноса:
   ```rust
   let model_2d = Mat4::from_scale_rotation_translation(
       Vec3::new(surface.size.x, surface.size.y, 1.0),
       Quat::IDENTITY,
       Vec3::new(20.0, 20.0, 0.0),
   );
   ```
3. Отрисовка:
   ```rust
   pipeline_2d.draw(ctx.gl, &model_2d, surface.color, surface.texture.as_deref());
   ```
