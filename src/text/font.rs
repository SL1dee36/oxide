use crate::core::error::OxideError;
use fontdue::{Font as FontdueFont, FontSettings};
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Font {
    inner: Arc<FontdueFont>,
}

impl Font {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OxideError> {
        let settings = FontSettings::default();
        let font = FontdueFont::from_bytes(bytes, settings)
            .map_err(|e| OxideError::TextureError(format!("Failed to parse font: {}", e)))?;
        Ok(Self {
            inner: Arc::new(font),
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, OxideError> {
        let bytes = std::fs::read(path.as_ref())?;
        Self::from_bytes(&bytes)
    }

    pub fn default_font() -> Result<Self, OxideError> {
        let candidate_paths = [
            "assets/fonts/arial.ttf",
            "C:\\Windows\\Fonts\\segoeui.ttf",
            "C:\\Windows\\Fonts\\arial.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
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

    #[inline]
    pub fn inner(&self) -> &FontdueFont {
        &self.inner
    }
}
