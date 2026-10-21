# Модуль `core` — Ядро рантайма и жизненный цикл

Модуль `core` управляет жизненным циклом приложения, циклом обработки событий операционной системы, контекстом графического устройства, таймингом и вводом.

Исходный код: [`src/core/`](file:///e:/PRJCT/Oxide/src/core/)

---

## 1. Конфигурация и запуск приложения (`App`, `AppBuilder`)

### Описание
`App` предоставляет строитель `AppBuilder` для настройки параметров окна (заголовок, разрешение) и передачи структуры, реализующей трейт `AppHandler`.

```rust
use oxide::prelude::*;

struct SimulationWorkspace;

impl AppHandler for SimulationWorkspace {
    fn on_init(&mut self, ctx: &mut Context) {
        // Выполняется один раз при старте
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        // Шаг симуляции (dt в секундах)
    }

    fn on_render(&mut self, ctx: &mut Context) {
        // Отрисовка кадра
        ctx.clear(Color::rgb(0.08, 0.09, 0.12));
    }
}

fn main() -> Result<(), OxideError> {
    App::build()
        .with_title("Oxide Workspace")
        .with_size(1280, 720)
        .run(SimulationWorkspace)
}
```

### Методы `AppBuilder`
- `new() -> Self` — создание конфигуратора со значениями по умолчанию (1280x720, "Oxide Runtime").
- `with_title(title: impl Into<String>) -> Self` — установка заголовка окна.
- `with_size(width: u32, height: u32) -> Self` — установка логического размера окна в пикселях.
- `run<H: AppHandler + 'static>(self, handler: H) -> Result<(), OxideError>` — инициализация окна, GL-контекста и запуск бесконечного цикла событий.

---

## 2. Трейт жизненного цикла (`AppHandler`)

Интерфейс обратных вызовов для пользовательской логики:

```rust
pub trait AppHandler {
    fn on_init(&mut self, _ctx: &mut Context) {}
    fn on_update(&mut self, _ctx: &mut Context, _dt: f32) {}
    fn on_render(&mut self, _ctx: &mut Context) {}
}
```

- **`on_init`**: Вызывается сразу после создания контекста OpenGL до входа в основной цикл. Здесь создаются камеры, меши и загружаются шрифты.
- **`on_update`**: Вызывается перед отрисовкой каждого кадра. Передает актуальный `dt: f32` (delta time).
- **`on_render`**: Вызывается для проходов рендеринга. Должен вызывать `ctx.clear(...)` и `scene.render(ctx)`.

---

## 3. Контекст выполнения (`Context`)

`Context` передается по ссылке во все методы `AppHandler` и предоставляет безопасный доступ к графическому драйверу и периферии:

| Метод | Возвращаемое значение | Назначение |
| :--- | :--- | :--- |
| `ctx.clear(color: Color)` | `()` | Очищает буфер цвета и буфер глубины заданным цветом |
| `ctx.aspect_ratio()` | `f32` | Возвращает соотношение сторон экрана (`width / height`) |
| `ctx.set_viewport(x, y, w, h)` | `()` | Задает границы OpenGL viewport |
| `ctx.delta_time()` | `f32` | Текущий шаг времени между кадрами в секундах |
| `ctx.elapsed()` | `f32` | Суммарное время работы приложения с момента запуска |
| `ctx.fps()` | `f32` | Сглаженное значение кадров в секунду |

---

## 4. Управление временем (`Time`)

Структура `Time` использует высокоточный системный таймер `std::time::Instant`:
- Защита от спирали задержек (delta time ограничивается сверху значением `0.1с` при подвисании окна).
- Внутренний секундный аккумулятор для точного вычисления FPS.
- Счетчик суммарного количества кадров `frame_count()`.

---

## 5. Обработка ввода (`InputState`)

Доступ через `ctx.input`. Позволяет опрашивать состояние клавиатуры и мыши:

```rust
// Проверка клавиатуры
if ctx.input.is_key_down(KeyCode::Space) {
    // Клавиша зажата
}
if ctx.input.is_key_just_pressed(KeyCode::KeyR) {
    // Клавиша была нажата именно в текущем кадре
}

// Мышь
let mouse_pos = ctx.input.mouse_position(); // glam::Vec2 в пикселях
let mouse_delta = ctx.input.mouse_delta();   // смещение за кадр
if ctx.input.is_mouse_button_down(MouseButton::Left) {
    // Зажата левая кнопка
}
```

---

## 6. Цвет (`Color`)

Структура `Color` оперирует компонентами в диапазоне `[0.0, 1.0]`:

```rust
let col = Color::rgba(0.2, 0.6, 1.0, 0.85);
let white = Color::WHITE;
let transparent = Color::TRANSPARENT;

let array: [f32; 4] = col.to_array();
let vec4: glam::Vec4 = col.to_vec4();
```
