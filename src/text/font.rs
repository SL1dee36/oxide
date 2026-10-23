use crate::core::error::OxideError;
use fontdue::{Font as FontdueFont, FontSettings};
use std::path::Path;
use std::sync::Arc;

/// Parsed vector font handle wrapping a [`fontdue::Font`] instance in an [`Arc`].
#[derive(Clone)]
pub struct Font {
    inner: Arc<FontdueFont>,
}

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

    /// Attempts to locate and load a default TrueType font, checking project assets then standard OS paths.
    pub fn default_font() -> Result<Self, OxideError> {
        let candidate_paths = [
            "assets/fonts/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\DejaVuSans.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
        ];

        for path in candidate_paths {
            if Path::new(path).exists() {
                if let Ok(font) = Self::from_file(path) {
                    return Ok(font);
                }
            }
        }

        Err(OxideError::TextureError(
            "No compatible system or local font found".to_string(),
        ))
    }

    /// Returns a direct reference to the internal [`fontdue::Font`] structure.
    #[inline]
    pub fn inner(&self) -> &FontdueFont {
        &self.inner
    }
}
