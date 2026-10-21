use glam::Vec2;
use std::collections::HashSet;
pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct InputState {
    keys_down: HashSet<KeyCode>,
    keys_just_pressed: HashSet<KeyCode>,
    keys_just_released: HashSet<KeyCode>,
    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_just_pressed: HashSet<MouseButton>,
    mouse_buttons_just_released: HashSet<MouseButton>,
    mouse_position: Vec2,
    mouse_delta: Vec2,
    mouse_wheel_delta: Vec2,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset_frame_deltas(&mut self) {
        self.keys_just_pressed.clear();
        self.keys_just_released.clear();
        self.mouse_buttons_just_pressed.clear();
        self.mouse_buttons_just_released.clear();
        self.mouse_delta = Vec2::ZERO;
        self.mouse_wheel_delta = Vec2::ZERO;
    }

    pub fn on_key_event(&mut self, key: KeyCode, pressed: bool) {
        if pressed {
            if self.keys_down.insert(key) {
                self.keys_just_pressed.insert(key);
            }
        } else {
            if self.keys_down.remove(&key) {
                self.keys_just_released.insert(key);
            }
        }
    }

    pub fn on_mouse_button_event(&mut self, button: MouseButton, pressed: bool) {
        if pressed {
            if self.mouse_buttons_down.insert(button) {
                self.mouse_buttons_just_pressed.insert(button);
            }
        } else {
            if self.mouse_buttons_down.remove(&button) {
                self.mouse_buttons_just_released.insert(button);
            }
        }
    }

    pub fn on_cursor_moved(&mut self, new_pos: Vec2) {
        self.mouse_delta += new_pos - self.mouse_position;
        self.mouse_position = new_pos;
    }

    pub fn on_mouse_wheel(&mut self, delta: Vec2) {
        self.mouse_wheel_delta += delta;
    }

    #[inline]
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_down.contains(&key)
    }

    #[inline]
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.keys_just_pressed.contains(&key)
    }

    #[inline]
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.keys_just_released.contains(&key)
    }

    #[inline]
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.mouse_buttons_down.contains(&button)
    }

    #[inline]
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_pressed.contains(&button)
    }

    #[inline]
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons_just_released.contains(&button)
    }

    #[inline]
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    #[inline]
    pub fn mouse_delta(&self) -> Vec2 {
        self.mouse_delta
    }

    #[inline]
    pub fn mouse_wheel_delta(&self) -> Vec2 {
        self.mouse_wheel_delta
    }
}
