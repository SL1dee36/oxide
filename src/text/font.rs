use crate::core::error::OxideError;
use fontdue::{Font as FontdueFont, FontSettings};
use std::path::Path;
use std::sync::Arc;

/// Parsed vector font handle wrapping a [`fontdue::Font`] instance in an [`Arc`].
#[derive(Clone)]
pub struct Font {
    inner: Arc<FontdueFont>,
}

const DEFAULT_FALLBACK_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/DejaVuSans.ttf");

impl Font {
    /// Parses font metadata and glyph outlines from an in-memory byte slice.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OxideError> {
        let settings = FontSettings::default();
        let font = FontdueFont::from_bytes(bytes, settings)
            .map_err(|e| OxideError::TextureError(format!("Failed to parse font: {}", e)))?;
        Ok(Self {
            inner: Arc::new(font),
        })
    }

    /// Reads and parses a TTF/OTF font file from the filesystem.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, OxideError> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::from_bytes(&bytes)
    }

    /// Attempts to locate and load a default TrueType font, checking project assets then standard OS paths,
    /// falling back to embedded font data if no filesystem font is found.
    pub fn default_font() -> Result<Self, OxideError> {
        let candidate_paths = [
            "assets/fonts/DejaVuSans.ttf",
            "../assets/fonts/DejaVuSans.ttf",
            "../../assets/fonts/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/System/Library/Fonts/SFNS.ttf",
            "/Library/Fonts/Arial.ttf",
        ];

        for path in candidate_paths {
            if Path::new(path).exists() {
                if let Ok(font) = Self::from_file(path) {
                    return Ok(font);
                }
            }
        }

        Self::from_bytes(DEFAULT_FALLBACK_FONT_BYTES)
    }

    /// Returns a direct reference to the internal [`fontdue::Font`] structure.
    #[inline]
    pub fn inner(&self) -> &FontdueFont {
        &self.inner
    }
}
