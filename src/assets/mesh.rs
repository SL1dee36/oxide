use crate::core::color::Color;
use crate::core::error::OxideError;
use crate::renderer::buffer::{IndexBuffer, Vertex3D, VertexArray, VertexBuffer};
use crate::renderer::pipeline_3d::Pipeline3D;

pub struct Mesh3D {
    pub vertices: Vec<Vertex3D>,
    pub indices: Vec<u32>,
    pub color: Color,
    pub emissive: Color,
    pub vao: Option<VertexArray>,
    pub vbo: Option<VertexBuffer>,
    pub ebo: Option<IndexBuffer>,
}

impl Mesh3D {
    pub fn new(vertices: Vec<Vertex3D>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
            color: Color::WHITE,
            emissive: Color::rgba(0.0, 0.0, 0.0, 0.0),
            vao: None,
            vbo: None,
            ebo: None,
        }
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_emissive(mut self, emissive: Color) -> Self {
        self.emissive = emissive;
        self
    }

    pub fn cube(size: f32) -> Self {
        let h = size * 0.5;
        let p = [
            [-h, -h,  h], [ h, -h,  h], [ h,  h,  h], [-h,  h,  h], // Front
            [ h, -h, -h], [-h, -h, -h], [-h,  h, -h], [ h,  h, -h], // Back
            [-h,  h,  h], [ h,  h,  h], [ h,  h, -h], [-h,  h, -h], // Top
            [-h, -h, -h], [ h, -h, -h], [ h, -h,  h], [-h, -h,  h], // Bottom
            [ h, -h,  h], [ h, -h, -h], [ h,  h, -h], [ h,  h,  h], // Right
            [-h, -h, -h], [-h, -h,  h], [-h,  h,  h], [-h,  h, -h], // Left
        ];

        let normals = [
            [ 0.0,  0.0,  1.0], // Front
            [ 0.0,  0.0, -1.0], // Back
            [ 0.0,  1.0,  0.0], // Top
            [ 0.0, -1.0,  0.0], // Bottom
            [ 1.0,  0.0,  0.0], // Right
            [-1.0,  0.0,  0.0], // Left
        ];

        let uvs = [
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ];

        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);

        for face in 0..6 {
            let norm = normals[face];
            for v in 0..4 {
                vertices.push(Vertex3D::new(p[face * 4 + v], norm, uvs[v]));
            }
            let base = (face * 4) as u32;
            indices.extend_from_slice(&[
                base, base + 1, base + 2,
                base + 2, base + 3, base,
            ]);
        }

        Self::new(vertices, indices)
    }

    pub fn plane(width: f32, depth: f32) -> Self {
        let hw = width * 0.5;
        let hd = depth * 0.5;
        let norm = [0.0, 1.0, 0.0];

        let vertices = vec![
            Vertex3D::new([-hw, 0.0,  hd], norm, [0.0, 0.0]),
            Vertex3D::new([ hw, 0.0,  hd], norm, [1.0, 0.0]),
            Vertex3D::new([ hw, 0.0, -hd], norm, [1.0, 1.0]),
            Vertex3D::new([-hw, 0.0, -hd], norm, [0.0, 1.0]),
        ];

        let indices = vec![0, 1, 2, 2, 3, 0];
        Self::new(vertices, indices)
    }

    pub fn sphere(radius: f32, rings: u32, sectors: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let r_step = std::f32::consts::PI / rings as f32;
        let s_step = (std::f32::consts::PI * 2.0) / sectors as f32;

        for r in 0..=rings {
            let phi = std::f32::consts::PI * 0.5 - r as f32 * r_step;
            let xy = radius * phi.cos();
            let z = radius * phi.sin();

            for s in 0..=sectors {
                let theta = s as f32 * s_step;
                let x = xy * theta.cos();
                let y = z;
                let z_pos = xy * theta.sin();

                let norm = [x / radius, y / radius, z_pos / radius];
                let uv = [s as f32 / sectors as f32, r as f32 / rings as f32];

                vertices.push(Vertex3D::new([x, y, z_pos], norm, uv));
            }
        }

        for r in 0..rings {
            let mut k1 = r * (sectors + 1);
            let mut k2 = k1 + sectors + 1;

            for _s in 0..sectors {
                if r != 0 {
                    indices.push(k1);
                    indices.push(k2);
                    indices.push(k1 + 1);
                }
                if r != (rings - 1) {
                    indices.push(k1 + 1);
                    indices.push(k2);
                    indices.push(k2 + 1);
                }
                k1 += 1;
                k2 += 1;
            }
        }

        Self::new(vertices, indices)
    }

    pub fn ensure_gpu_buffers(&mut self, gl: &glow::Context) -> Result<(), OxideError> {
        if self.vao.is_some() {
            return Ok(());
        }

        let vao = VertexArray::new(gl)?;
        let vbo = VertexBuffer::new(gl, &self.vertices)?;
        let ebo = IndexBuffer::new(gl, &self.indices)?;

        vao.bind(gl);
        vbo.bind(gl);
        ebo.bind(gl);

        Pipeline3D::setup_vertex_attributes(gl);

        vao.unbind(gl);
        vbo.unbind(gl);
        ebo.unbind(gl);

        self.vao = Some(vao);
        self.vbo = Some(vbo);
        self.ebo = Some(ebo);

        Ok(())
    }
}
