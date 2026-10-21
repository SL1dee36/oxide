# Модуль `assets` — Геометрия, текстуры и загрузка ресурсов

Модуль `assets` отвечает за представление трехмерной геометрии (`Mesh3D`), двумерных поверхностей (`Surface2D`), процедурную генерацию примитивов и парсинг файлов с диска.

Исходный код: [`src/assets/`](file:///e:/PRJCT/Oxide/src/assets/)

---

## 1. Трехмерная геометрия (`Mesh3D`)

`Mesh3D` хранит данные вершин и индексов в оперативной памяти и лениво загружает их в буферы видеопамяти (VAO/VBO/EBO):

```rust
pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}
```

### Встроенные процедурные примитивы:
- `Mesh3D::cube(size: f32)` — куб с корректными нормалями для каждой из 6 граней и UV-разверткой.
- `Mesh3D::plane(width: f32, depth: f32)` — горизонтальная плоскость с нормалью, направленной вверх `[0, 1, 0]`.
- `Mesh3D::sphere(radius: f32, rings: u32, sectors: u32)` — UV-сфера с настраиваемой детализацией сетки.

### Инициализация видеопамяти:
Метод `mesh.ensure_gpu_buffers(gl)` вызывается сценой автоматически перед первым рендером, создавая VAO, VBO и EBO.

---

## 2. Двумерная поверхность (`Surface2D`)

Используется для интерфейсных панелей, оверлеев и прямоугольных элементов:

```rust
let panel = Surface2D::from_color(
    Vec2::new(300.0, 100.0),
    Color::rgba(0.1, 0.15, 0.25, 0.9),
);
```

---

## 3. Загрузчики файлов (`loader.rs`)

Поддерживается загрузка геометрии и изображений из файловой системы и из срезов памяти:

### Загрузка 3D-мешей (.obj):
Используется парсер `tobj` с автоматической триангуляцией:
```rust
// Из файла
let mesh = load_obj_file("assets/models/teapot.obj")?;

// Из памяти
let mesh = load_obj_from_memory(OBJ_BYTE_DATA)?;
```

### Загрузка текстур (PNG, JPEG):
Используется библиотека `image`:
```rust
// Загрузка текстуры в видеопамять из файла
let texture = load_texture_file(ctx.gl, "assets/textures/diffuse.png")?;

// Из памяти
let texture = load_texture_from_memory(ctx.gl, PNG_BYTES)?;
```
