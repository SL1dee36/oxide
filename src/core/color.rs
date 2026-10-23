/// RGBA color representation using 32-bit floating point components in range [0.0, 1.0].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red channel intensity [0.0, 1.0].
    pub r: f32,
    /// Green channel intensity [0.0, 1.0].
    pub g: f32,
    /// Blue channel intensity [0.0, 1.0].
    pub b: f32,
    /// Alpha opacity channel [0.0, 1.0].
    pub a: f32,
}

impl Color {
    /// Opaque white color: (1.0, 1.0, 1.0, 1.0).
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);
    /// Opaque black color: (0.0, 0.0, 0.0, 1.0).
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);
    /// Opaque pure red color: (1.0, 0.0, 0.0, 1.0).
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);
    /// Opaque pure green color: (0.0, 1.0, 0.0, 1.0).
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);
    /// Opaque pure blue color: (0.0, 0.0, 1.0, 1.0).
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);
    /// Fully transparent black color: (0.0, 0.0, 0.0, 0.0).
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Constructs an opaque RGB color with alpha initialized to 1.0.
    #[inline]
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Constructs a color with explicit RGBA channels.
    #[inline]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Converts channels to a fixed-size 4-element array `[r, g, b, a]`.
    #[inline]
    pub fn to_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Converts channels to a `glam::Vec4`.
    #[inline]
    pub fn to_vec4(self) -> glam::Vec4 {
        glam::Vec4::new(self.r, self.g, self.b, self.a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::WHITE
    }
}
