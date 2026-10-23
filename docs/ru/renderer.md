# Модуль `renderer` — Графические конвейеры и ресурсы видеопамяти

Модуль `renderer` реализует низкоуровневые графические конвейеры OpenGL 4.5 Core Profile, компиляцию шейдерных программ GLSL, аллокацию буферов GPU и управление ресурсами текстур.

Исходный код: [`src/renderer/`](../../src/renderer/)

---

## 1. Разделение данных ОЗУ и ресурсов видеопамяти GPU

В архитектуре Oxide действует строгое разграничение между данными геометрии и изображений в оперативной памяти (модуль `assets`) и аллоцированными объектами видеопамяти (модуль `renderer`):

```mermaid
graph LR
    subgraph RAM [Оперативная память (ОЗУ) - assets]
        HostMesh["assets::Mesh3D<br/>(Vec<Vertex3D>, Vec<u32>)"]
        HostSurface["assets::Surface2D<br/>(Vec2 size, Color)"]
        RawImage["Декодированный буфер<br/>(Vec<u8> RGBA)"]
    end

    subgraph VRAM [Видеопамять GPU (VRAM) - renderer]
        VAO["renderer::VertexArray<br/>(glGenVertexArrays)"]
        VBO["renderer::VertexBuffer<br/>(GL_ARRAY_BUFFER)"]
        EBO["renderer::IndexBuffer<br/>(GL_ELEMENT_ARRAY_BUFFER)"]
        Texture["renderer::Texture2D<br/>(GL_TEXTURE_2D)"]
    end

    HostMesh -->|"ensure_gpu_buffers(gl)"| VAO & VBO & EBO
    RawImage -->|"Texture2D::new(gl, ...)"| Texture
```

---

## 2. Буферы данных видеопамяти (`buffer.rs`)

| Тип | Описание | Основные методы |
| :--- | :--- | :--- |
| `Vertex3D` | Структура вершины `#[repr(C)]`: `position: [f32; 3]`, `normal: [f32; 3]`, `uv: [f32; 2]`. Размер: 32 байта. | `new(pos, normal, uv)` |
| `VertexArray` | Инкапсулирует OpenGL VAO (`glGenVertexArrays`). Связывает формат атрибутов и привязку буферов. | `new(gl)`, `bind(gl)`, `unbind(gl)`, `destroy(gl)` |
| `VertexBuffer` | Инкапсулирует VBO (`GL_ARRAY_BUFFER`, `GL_STATIC_DRAW`). Хранит непрерывный массив вершин. | `new<T>(gl, data: &[T])`, `bind(gl)`, `unbind(gl)`, `count()`, `destroy(gl)` |
| `IndexBuffer` | Инкапсулирует EBO (`GL_ELEMENT_ARRAY_BUFFER`, `GL_STATIC_DRAW`). Хранит индексы треугольников `u32`. | `new(gl, indices: &[u32])`, `bind(gl)`, `unbind(gl)`, `count()`, `destroy(gl)` |

---

## 3. Шейдерные программы (`ShaderProgram`)

Класс `ShaderProgram` управляет компиляцией вершинных и фрагментных шейдеров GLSL 450 core и передачей uniform-переменных:

```rust
pub struct ShaderProgram { /* id: glow::Program */ }
```

### Методы передачи параметров:
- `set_mat4(gl, name: &str, matrix: &Mat4)` — передача матрицы 4x4 (`glUniformMatrix4fv`).
- `set_vec3(gl, name: &str, vec: Vec3)` — передача 3D-вектора (`glUniform3f`).
- `set_vec4(gl, name: &str, vec: Vec4)` — передача 4D-вектора/цвета (`glUniform4f`).
- `set_float(gl, name: &str, value: f32)` — передача скаляра `f32`.
- `set_int(gl, name: &str, value: i32)` — передача целочисленного значения (например, номера текстурного слота).

---

## 4. Текстуры GPU (`Texture2D`)

Структура `Texture2D` инкапсулирует объект `GL_TEXTURE_2D` с трилинейной фильтрацией и мипмапами:
- `Texture2D::new(gl, width, height, pixels: &[u8]) -> Result<Self, OxideError>` — выделяет текстуру, загружает RGBA8 пиксели и генерирует мипмапы (`glGenerateMipmap`).
- `Texture2D::white(gl) -> Result<Self, OxideError>` — создает белую текстуру 1x1 для отображения объектов без пользовательской карты текстуры.
- `bind(&self, gl, slot: u32)` — активирует текстурный юнит `GL_TEXTURE0 + slot` и привязывает текстуру.
- `destroy(&self, gl)` — освобождает ресурс в видеопамяти (`glDeleteTextures`).

---

## 5. Графический конвейер 3D (`Pipeline3D`)

`Pipeline3D` выполняет отрисовку полигональных сеток с перспективной проекцией, глубиной и направленным освещением:

```rust
pub struct Pipeline3D {
    pub light: Light3D,
}
```

### Модель освещения:
- **Направленный свет (`Light3D`):**
  $$\mathbf{I} = \mathbf{I}_{\text{ambient}} + \max(\mathbf{N} \cdot \mathbf{L}, 0.0) \cdot \mathbf{I}_{\text{light}}$$
  $$\mathbf{C}_{\text{final}} = \mathbf{C}_{\text{albedo}} \cdot \mathbf{I} + \mathbf{C}_{\text{emissive}}$$
- **Матрица нормалей:** Вычисляется в вершинном шейдере как `mat3(transpose(inverse(u_model)))` для предотвращения искажения нормалей при неравномерном масштабировании.

### Сигнатуры конвейера:
```rust
// 1. Активация конвейера, передача матриц вида/проекции и параметров света
pub fn begin(&self, gl: &glow::Context, view: &Mat4, projection: &Mat4);

// 2. Отрисовка полигонального меша
pub fn draw(
    &self,
    gl: &glow::Context,
    model: &Mat4,
    albedo: Color,
    emissive: Color,
    texture: Option<&Texture2D>,
    vao: &VertexArray,
    ebo: &IndexBuffer,
);

// 3. Завершение прохода
pub fn end(&self, gl: &glow::Context);
```

---

## 6. Графический конвейер 2D (`Pipeline2D`)

`Pipeline2D` предназначен для вывода плоских панелей и интерфейсных поверхностей:
- **Координатная система:** Ортографическая проекция с началом координат $(0, 0)$ в левом верхнем углу, оси: $X \in [0, \text{width}]$, $Y \in [0, \text{height}]$ направлена вниз.
- **Альфа-блендинг:** `gl.enable(GL_BLEND)` с функцией `gl.blend_func(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)`.
- **Единичный квад:** Вершины `[0,0]`, `[1,0]`, `[1,1]`, `[0,1]` хранятся в статическом VBO/EBO пайплайна и масштабируются через модельную матрицу.

```rust
pub fn begin(&mut self, gl: &glow::Context, screen_width: f32, screen_height: f32);
pub fn draw(&self, gl: &glow::Context, model: &Mat4, color: Color, texture: Option<&Texture2D>);
pub fn end(&self, gl: &glow::Context);
```
