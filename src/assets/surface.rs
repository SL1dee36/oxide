use crate::core::color::Color;
use crate::renderer::texture::Texture2D;
use glam::Vec2;
use std::sync::Arc;

/// Planar 2D rectangular surface defined by pixel dimensions, fill color, and optional texture.
#[derive(Debug, Clone)]
pub struct Surface2D {
    /// Width and height of the surface quad in pixels.
    pub size: Vec2,
    /// Solid fill color.
    pub color: Color,
    /// Optional diffuse 2D texture map.
    pub texture: Option<Arc<Texture2D>>,
}

impl Surface2D {
    /// Creates a 2D surface with given pixel dimensions and default white fill color.
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            color: Color::WHITE,
            texture: None,
        }
    }

    /// Creates a 2D surface with explicit dimensions and fill color.
    pub fn from_color(size: Vec2, color: Color) -> Self {
        Self {
            size,
            color,
            texture: None,
        }
    }

    /// Associates a diffuse 2D texture map with the surface.
    pub fn with_texture(mut self, texture: Arc<Texture2D>) -> Self {
        self.texture = Some(texture);
        self
    }
}

impl PartialEq for Surface2D {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size
            && self.color == other.color
            && match (&self.texture, &other.texture) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
