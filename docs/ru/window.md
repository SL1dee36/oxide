# Модуль `window` — Оконная подсистема и контекст OpenGL

Модуль `window` отвечает за инициализацию окна ОС, настройку графического контекста OpenGL 3.3 Core Profile и привязку библиотеки `glow`.

Исходный код: [`src/window/`](file:///e:/PRJCT/Oxide/src/window/)

---

## 1. Архитектура интеграции

Для создания кроссплатформенного окна и контекста используется связка крейтов:
- **`winit 0.29`**: Управление окном, получение событий ввода ОС (мышь, клавиатура, ресайз, закрытие).
- **`glutin 0.31` + `glutin-winit 0.4`**: Создание аппаратного контекста OpenGL, выбор конфигурации буферов (RGBA8, 24-бит буфер глубины, 8-бит буфер трафарета).
- **`raw-window-handle 0.5`**: Низкоуровневые дескрипторы оконной системы для безопасной связки `glutin` и `winit`.
- **`glow 0.13`**: Безопасный интерфейс OpenGL с нулевыми накладными расходами.

---

## 2. Структура `WindowContext`

```rust
pub struct WindowContext {
    pub window: Window,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
    pub glow_gl: glow::Context,
}
```

### Поля:
- `window` — объект окна `winit::window::Window`.
- `gl_context` — активный контекст OpenGL (`glutin::context::PossiblyCurrentContext`).
- `gl_surface` — поверхность отрисовки (`glutin::surface::Surface<WindowSurface>`).
- `glow_gl` — инстанс контекста `glow::Context` с загруженными функциями драйвера видеокарты.

---

## 3. Процесс инициализации

При вызове `WindowContext::new(...)`:
1. Формируется `WindowBuilder` с требуемыми размерами и заголовком.
2. `DisplayBuilder` находит доступные аппаратные конфигурации дисплея с максимальным сглаживанием (MSAA) и поддержкой аппаратного ускорения.
3. Запрашивается контекст **OpenGL 3.3 Core Profile**.
4. Создается поверхность окна и контекст делается текущим (`make_current`).
5. Включается вертикальная синхронизация (`SwapInterval::Wait(1)`).
6. Через `glow::Context::from_loader_function` загружаются адреса всех функций OpenGL драйвера.

---

## 4. Методы управления

```rust
// Изменение размера буферов при ресайзе окна
pub fn resize(&self, width: u32, height: u32);

// Смена переднего и заднего буферов кадра (VSync)
pub fn swap_buffers(&self) -> Result<(), OxideError>;
```
