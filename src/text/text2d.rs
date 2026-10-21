use crate::core::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct Text2D {
    pub content: String,
    pub color: Color,
    pub scale: f32,
}

impl Text2D {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: Color::WHITE,
            scale: 1.0,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}
