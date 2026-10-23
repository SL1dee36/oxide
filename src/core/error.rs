use thiserror::Error;

/// Error enumeration representing subsystem failures across the engine.
#[derive(Error, Debug)]
pub enum OxideError {
    /// Window or display backend creation failure.
    #[error("Window initialization failed: {0}")]
    WindowCreation(String),

    /// Graphics context creation, activation, or buffer swap error.
    #[error("OpenGL context error: {0}")]
    ContextError(String),

    /// Shader compilation or program linking error.
    #[error("Shader compilation error: {0}")]
    ShaderError(String),

    /// Image parsing or texture upload failure.
    #[error("Texture loading failed: {0}")]
    TextureError(String),

    /// Mesh parsing or geometry layout validation error.
    #[error("Mesh parsing error: {0}")]
    MeshError(String),

    /// Filesystem or generic I/O error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
