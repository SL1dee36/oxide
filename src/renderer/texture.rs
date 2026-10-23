use crate::core::error::OxideError;
use glow::HasContext;

/// 2D texture allocation resident in GPU memory (`GL_TEXTURE_2D`).
pub struct Texture2D {
    id: glow::Texture,
    width: u32,
    height: u32,
}

impl Texture2D {
    /// Creates a 2D RGBA texture, uploads pixel data, sets repeat/linear filtering, and generates mipmaps.
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
                glow::PixelUnpackData::Slice(Some(pixels)),
            );

            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Self { id, width, height })
        }
    }

    /// Allocates a 1x1 pixel texture with a uniform RGBA color.
    pub fn single_color(gl: &glow::Context, color: [u8; 4]) -> Result<Self, OxideError> {
        Self::new(gl, 1, 1, &color)
    }

    /// Allocates a 1x1 solid opaque white texture (255, 255, 255, 255).
    pub fn white(gl: &glow::Context) -> Result<Self, OxideError> {
        Self::single_color(gl, [255, 255, 255, 255])
    }

    /// Binds texture to the specified texture unit slot (`GL_TEXTURE0 + slot`).
    #[inline]
    pub fn bind(&self, gl: &glow::Context, slot: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, Some(self.id));
        }
    }

    /// Unbinds texture from the specified texture unit slot.
    #[inline]
    pub fn unbind(&self, gl: &glow::Context, slot: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    /// Width of the texture image in pixels.
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height of the texture image in pixels.
    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Deallocates the texture object from GPU memory.
    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_texture(self.id);
        }
    }
}
