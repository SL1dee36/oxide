use crate::core::error::OxideError;
use crate::renderer::texture::Texture2D;
use crate::text::font::Font;
use glam::Vec2;
use std::collections::HashMap;

/// Pre-rasterized glyph metrics and atlas UV texture coordinates.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// Upper-left UV coordinate in texture atlas.
    pub uv_min: Vec2,
    /// Bottom-right UV coordinate in texture atlas.
    pub uv_max: Vec2,
    /// Physical pixel dimensions `(width, height)` of the rasterized glyph.
    pub size: Vec2,
    /// Baseline offset vector `(xmin, ymin)`.
    pub offset: Vec2,
    /// Horizontal cursor advance distance in pixels.
    pub advance_x: f32,
}

/// Dynamic 1024x1024 glyph texture atlas and typographic metric table.
pub struct FontAtlas {
    /// Associated source font.
    pub font: Font,
    /// Base rasterization font size in pixels.
    pub font_size: f32,
    /// GPU texture containing packed glyph alpha masks.
    pub texture: Texture2D,
    /// Lookup table mapping Unicode characters to glyph atlas metrics.
    pub glyphs: HashMap<char, Glyph>,
    /// Default line advance height in pixels.
    pub line_height: f32,
}

impl FontAtlas {
    /// Rasterizes ASCII, Cyrillic, and punctuation ranges at `font_size` and packs them into a 1024x1024 GPU texture.
    pub fn new(gl: &glow::Context, font: Font, font_size: f32) -> Result<Self, OxideError> {
        let atlas_width = 1024u32;
        let atlas_height = 1024u32;
        let mut pixels = vec![0u8; (atlas_width * atlas_height * 4) as usize];

        let mut glyphs = HashMap::new();
        let mut current_x = 1u32;
        let mut current_y = 1u32;
        let mut row_height = 0u32;

        let mut chars_to_pack: Vec<char> = Vec::new();
        // ASCII printable
        chars_to_pack.extend((32u8..=126u8).map(|b| b as char));
        // Cyrillic uppercase & lowercase + special letters
        chars_to_pack.extend((0x0400u32..=0x045Fu32).filter_map(std::char::from_u32));
        // Common typography
        chars_to_pack.extend(['№', '«', '»', '—', '–', '°', '±', '•', '…', '’', '”', '“']);

        for c in chars_to_pack {
            let (metrics, bitmap) = font.inner().rasterize(c, font_size);
            let w = metrics.width as u32;
            let h = metrics.height as u32;

            if current_x + w + 1 >= atlas_width {
                current_x = 1;
                current_y += row_height + 1;
                row_height = 0;
            }

            if current_y + h + 1 >= atlas_height {
                break;
            }

            for row in 0..h {
                for col in 0..w {
                    let src_idx = (row * w + col) as usize;
                    let alpha = bitmap[src_idx];
                    let dst_x = current_x + col;
                    let dst_y = current_y + row;
                    let dst_idx = ((dst_y * atlas_width + dst_x) * 4) as usize;

                    pixels[dst_idx] = 255;
                    pixels[dst_idx + 1] = 255;
                    pixels[dst_idx + 2] = 255;
                    pixels[dst_idx + 3] = alpha;
                }
            }

            let uv_min = Vec2::new(
                current_x as f32 / atlas_width as f32,
                current_y as f32 / atlas_height as f32,
            );
            let uv_max = Vec2::new(
                (current_x + w) as f32 / atlas_width as f32,
                (current_y + h) as f32 / atlas_height as f32,
            );

            glyphs.insert(
                c,
                Glyph {
                    uv_min,
                    uv_max,
                    size: Vec2::new(w as f32, h as f32),
                    offset: Vec2::new(metrics.xmin as f32, metrics.ymin as f32),
                    advance_x: metrics.advance_width,
                },
            );

            current_x += w + 1;
            row_height = row_height.max(h);
        }

        let texture = Texture2D::new(gl, atlas_width, atlas_height, &pixels)?;

        let line_height = font
            .inner()
            .horizontal_line_metrics(font_size)
            .map(|m| m.new_line_size)
            .unwrap_or(font_size * 1.2);

        Ok(Self {
            font,
            font_size,
            texture,
            glyphs,
            line_height,
        })
    }

    /// Measures the pixel bounding box `(width, height)` of a multi-line text string.
    pub fn measure_text(&self, text: &str) -> Vec2 {
        let mut width = 0.0f32;
        let mut max_width = 0.0f32;
        let mut height = self.line_height;

        for c in text.chars() {
            if c == '\n' {
                max_width = max_width.max(width);
                width = 0.0;
                height += self.line_height;
                continue;
            }

            if let Some(glyph) = self.glyphs.get(&c) {
                width += glyph.advance_x;
            } else {
                width += self.font_size * 0.6;
            }
        }

        Vec2::new(max_width.max(width), height)
    }
}
