use crate::core::error::OxideError;
use glow::HasContext;

pub struct Texture2D {
    id: glow::Texture,
    width: u32,
    height: u32,
}

impl Texture2D {
    pub fn new(
        gl: &glow::Context,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<Self, OxideError> {
        unsafe {
            let id = gl
                .create_texture()
                .map_err(|e| OxideError::TextureError(e.to_string()))?;

            gl.bind_texture(glow::TEXTURE_2D, Some(id));

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::REPEAT as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::REPEAT as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR_MIPMAP_LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
            );

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(pixels),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Self { id, width, height })
        }
    }

    pub fn single_color(gl: &glow::Context, color: [u8; 4]) -> Result<Self, OxideError> {
        Self::new(gl, 1, 1, &color)
    }

    pub fn white(gl: &glow::Context) -> Result<Self, OxideError> {
        Self::single_color(gl, [255, 255, 255, 255])
    }

    #[inline]
    pub fn bind(&self, gl: &glow::Context, slot: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
        }
    }

    #[inline]
    pub fn unbind(&self, gl: &glow::Context, slot: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.id);
        }
    }
}
