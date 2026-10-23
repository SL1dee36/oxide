# Модуль `core` — Жизненный цикл приложения и контекст выполнения

Модуль `core` управляет жизненным циклом приложения, циклом событий операционной системы, контекстом выполнения кадра, таймингом, устройствами ввода и обработкой ошибок.

Исходный код: [`src/core/`](../../src/core/)

---

## 1. Разделение зон ответственности

| Сущность | Модуль | Зона ответственности |
| :--- | :--- | :--- |
| **`App` / `AppBuilder`** | `core::app` | Конфигурация параметров приложения (заголовок, размер), создание цикла событий `winit::event_loop::EventLoop`, запуск обработчика `AppHandler`. Не управляет напрямую графическими вызовами. |
| **`Context<'a>`** | `core::context` | Временный per-frame контекст, передаваемый в методы `AppHandler`. Предоставляет безопасный доступ к функциям OpenGL драйвера (`&glow::Context`), размерам области вывода, таймингу (`Time`) и вводу (`InputState`). Существует только в пределах обратного вызова кадра. |
| **`WindowContext`** | `window::window_context` | Владелец системных ресурсов окна (`winit::window::Window`), поверхности вывода (`glutin::surface::Surface`) и аппаратного контекста OpenGL (`PossiblyCurrentContext`). Отвечает за вызовы смены буферов (`swap_buffers`) и ресайз поверхности. |

---

## 2. Жизненный цикл приложения (`App`, `AppBuilder`, `AppHandler`)

### Структура `AppBuilder`
Предоставляет методы настройки перед входом в цикл событий:
- `AppBuilder::new() -> Self` — создание экземпляра со значениями по умолчанию (1280x720, "Oxide Runtime").
- `with_title(title: impl Into<String>) -> Self` — установка заголовка окна ОС.
- `with_size(width: u32, height: u32) -> Self` — установка логического размера окна в пикселях.
- `run<H: AppHandler + 'static>(self, handler: H) -> Result<(), OxideError>` — инициализация подсистемы окон, компиляция графического контекста и запуск блокирующего цикла событий.

### Трейт `AppHandler`
Интерфейс обратных вызовов, реализуемый пользовательской структурой:
```rust
use oxide::prelude::*;

pub struct SimulationHandler;

impl AppHandler for SimulationHandler {
    fn on_init(&mut self, ctx: &mut Context) {
        // Однократная инициализация после создания OpenGL контекста
        ctx.set_viewport(0, 0, ctx.width as i32, ctx.height as i32);
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        // Шаг вычислений логики симуляции
    }

    fn on_render(&mut self, ctx: &mut Context) {
        // Формирование команд отрисовки
        ctx.clear(Color::rgb(0.08, 0.10, 0.14));
    }
}
```

---

## 3. Контекст кадра (`Context`)

Структура `Context<'a>` агрегирует состояние текущего кадра:

```rust
pub struct Context<'a> {
    pub gl: &'a glow::Context,
    pub width: u32,
    pub height: u32,
    pub time: &'a Time,
    pub input: &'a InputState,
}
```

### Методы `Context`:
- `aspect_ratio(&self) -> f32` — вычисляет соотношение сторон области вывода (`width as f32 / height as f32`).
- `clear(&self, color: Color)` — очищает буфер цвета (`GL_COLOR_BUFFER_BIT`) и буфер глубины (`GL_DEPTH_BUFFER_BIT`).
- `set_viewport(&self, x: i32, y: i32, width: i32, height: i32)` — задает активный OpenGL viewport.
- `delta_time(&self) -> f32` — возвращает продолжительность предыдущего кадра в секундах ($\Delta t$).
- `elapsed(&self) -> f32` — возвращает общее время выполнения приложения с момента запуска.
- `fps(&self) -> f32` — возвращает усредненную частоту кадров за последний секундный интервал.

---

## 4. Расчет времени (`Time`)

Структура `Time` использует высокоточный системный таймер `std::time::Instant`:
- **Защита от спирали задержек:** Шаг $\Delta t$ ограничен сверху значением `0.1` с (`duration.as_secs_f32().min(0.1)`), что предотвращает скачки физики при перемещении или потере фокуса окна.
- **Подсчет FPS:** Скользящее усреднение по суммарному числу кадров за интервал $\ge 1.0$ секунды.
- **Счетчик кадров:** `frame_count() -> u64` отслеживает общее число отрисованных кадров.

---

## 5. Опрос устройств ввода (`InputState`)

Регистрирует состояния клавиатуры и мыши через `winit`:

### Клавиатура:
- `is_key_down(key: KeyCode) -> bool` — клавиша удерживается в нажатом состоянии.
- `is_key_just_pressed(key: KeyCode) -> bool` — клавиша была нажата в текущем кадре.
- `is_key_just_released(key: KeyCode) -> bool` — клавиша была отпущена в текущем кадре.

### Мышь:
- `is_mouse_button_down(button: MouseButton) -> bool` — кнопка мыши удерживается.
- `is_mouse_button_just_pressed(button: MouseButton) -> bool` — кнопка нажата в текущем кадре.
- `is_mouse_button_just_released(button: MouseButton) -> bool` — кнопка отпущена в текущем кадре.
- `mouse_position() -> Vec2` — текущие экранные координаты курсора в пикселях.
- `mouse_delta() -> Vec2` — смещение курсора относительно предыдущего кадра.
- `mouse_wheel_delta() -> Vec2` — вектор смещения колеса мыши за текущий кадр.

---

## 6. Представление цвета (`Color`)

Структура `Color` содержит 4 компонента `f32` в диапазоне `[0.0, 1.0]`:
```rust
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}
```
Константы: `Color::WHITE`, `Color::BLACK`, `Color::RED`, `Color::GREEN`, `Color::BLUE`, `Color::TRANSPARENT`.  
Методы преобразования: `to_array() -> [f32; 4]` и `to_vec4() -> glam::Vec4`.

---

## 7. Обработка ошибок (`OxideError`)

Все ошибки подсистем строго типизированы перечислением `OxideError` на базе `thiserror`:
- `WindowCreation(String)` — ошибка создания системного окна или цикла событий.
- `ContextError(String)` — ошибка компиляции, выбора конфигурации пикселей или переключения контекста OpenGL.
- `ShaderError(String)` — ошибка компиляции вершинного/фрагментного шейдера или линковки шейдерной программы.
- `TextureError(String)` — ошибка декодирования изображения или выделения текстуры в видеопамяти.
- `MeshError(String)` — ошибка парсинга геометрии (например, в парсере `tobj`).
- `Io(std::io::Error)` — ошибки ввода-вывода при чтении файлов с накопителя.
