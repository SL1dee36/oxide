use thiserror::Error;

#[derive(Error, Debug)]
pub enum OxideError {
    #[error("Window initialization failed: {0}")]
    WindowCreation(String),

    #[error("OpenGL context error: {0}")]
    ContextError(String),

    #[error("Shader compilation error: {0}")]
    ShaderError(String),

    #[error("Texture loading failed: {0}")]
    TextureError(String),

    #[error("Mesh parsing error: {0}")]
    MeshError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
