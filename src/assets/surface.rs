use crate::core::color::Color;
use glam::Vec2;

#[derive(Debug, Clone, PartialEq)]
pub struct Surface2D {
    pub size: Vec2,
    pub color: Color,
}

impl Surface2D {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            color: Color::WHITE,
        }
    }

    pub fn from_color(size: Vec2, color: Color) -> Self {
        Self { size, color }
    }
}
