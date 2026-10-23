use crate::core::color::Color;

/// 2D typography element descriptor specifying text content, font color, and uniform scale.
#[derive(Debug, Clone, PartialEq)]
pub struct Text2D {
    /// UTF-8 string content to render.
    pub content: String,
    /// RGBA text color tint.
    pub color: Color,
    /// Font size scaling multiplier.
    pub scale: f32,
}

impl Text2D {
    /// Constructs a text element with default white color and scale 1.0.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            color: Color::WHITE,
            scale: 1.0,
        }
    }

    /// Sets the text tint color.
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the text scaling multiplier.
    pub fn with_scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }
}
