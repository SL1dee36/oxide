use crate::core::color::Color;
use glam::Vec2;

/// Planar 2D rectangular surface defined by pixel dimensions and fill color.
#[derive(Debug, Clone, PartialEq)]
pub struct Surface2D {
    /// Width and height of the surface quad in pixels.
    pub size: Vec2,
    /// Solid fill color.
    pub color: Color,
}

impl Surface2D {
    /// Creates a 2D surface with given pixel dimensions and default white fill color.
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            color: Color::WHITE,
        }
    }

    /// Creates a 2D surface with explicit dimensions and fill color.
    pub fn from_color(size: Vec2, color: Color) -> Self {
        Self { size, color }
    }
}
