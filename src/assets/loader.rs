use crate::assets::mesh::Mesh3D;
use crate::core::error::OxideError;
use crate::renderer::buffer::Vertex3D;
use crate::renderer::texture::Texture2D;
use std::io::Cursor;
use std::path::Path;
use tobj::LoadOptions;

/// Deserializes a Wavefront OBJ polygonal model from an in-memory byte slice.
pub fn load_obj_from_memory(bytes: &[u8]) -> Result<Mesh3D, OxideError> {
    let mut reader = Cursor::new(bytes);
    let options = LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };

    let (models, _) = tobj::load_obj_buf(&mut reader, &options, |_| {
        Err(tobj::LoadError::GenericFailure)
    })
    .map_err(|e| OxideError::MeshError(e.to_string()))?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for model in models {
        let mesh = model.mesh;
        let num_vertices = mesh.positions.len() / 3;

        let base_index = vertices.len() as u32;

        for i in 0..num_vertices {
            let px = mesh.positions[i * 3];
            let py = mesh.positions[i * 3 + 1];
            let pz = mesh.positions[i * 3 + 2];

            let (nx, ny, nz) = if mesh.normals.len() > i * 3 + 2 {
                (mesh.normals[i * 3], mesh.normals[i * 3 + 1], mesh.normals[i * 3 + 2])
            } else {
                (0.0, 1.0, 0.0)
            };

            let (u, v) = if mesh.texcoords.len() > i * 2 + 1 {
                (mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1])
            } else {
                (0.0, 0.0)
            };

            vertices.push(Vertex3D::new([px, py, pz], [nx, ny, nz], [u, v]));
        }

        for idx in mesh.indices {
            indices.push(base_index + idx);
        }
    }

    Ok(Mesh3D::new(vertices, indices))
}

/// Reads and parses a Wavefront OBJ polygonal file from disk.
pub fn load_obj_file(path: impl AsRef<Path>) -> Result<Mesh3D, OxideError> {
    let bytes = std::fs::read(path)?;
    load_obj_from_memory(&bytes)
}

/// Decodes an image from memory (PNG, JPEG) and uploads it into a GPU 2D texture.
pub fn load_texture_from_memory(gl: &glow::Context, bytes: &[u8]) -> Result<Texture2D, OxideError> {
    let img = image::load_from_memory(bytes)
        .map_err(|e| OxideError::TextureError(e.to_string()))?;
    let rgba = img.to_rgba8();
    let width = rgba.width();
    let height = rgba.height();
    Texture2D::new(gl, width, height, &rgba)
}

/// Reads an image file from disk and uploads it into a GPU 2D texture.
pub fn load_texture_file(gl: &glow::Context, path: impl AsRef<Path>) -> Result<Texture2D, OxideError> {
    let bytes = std::fs::read(path)?;
    load_texture_from_memory(gl, &bytes)
}
