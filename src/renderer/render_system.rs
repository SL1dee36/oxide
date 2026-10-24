use crate::core::context::Context;
use crate::ecs::World;
use crate::renderer::pipeline_2d::Pipeline2D;
use crate::renderer::pipeline_3d::Pipeline3D;
use crate::text::atlas::FontAtlas;
use crate::text::font::Font;
use crate::text::pipeline_text::PipelineText;
use glam::{Mat4, Vec2, Vec3};
use std::sync::Arc;

/// Decoupled graphics subsystem coordinating multi-pass rendering for an ECS [`World`].
///
/// Encapsulates GPU shaders, buffers, pipelines, and font atlases, keeping [`World`] completely
/// agnostic of OpenGL and graphics hardware resources.
pub struct RenderSystem {
    /// 3D forward lighting geometry pipeline.
    pub pipeline_3d: Option<Pipeline3D>,
    /// 2D planar quad screen-space pipeline.
    pub pipeline_2d: Option<Pipeline2D>,
    /// Persistent dynamic text typography pipeline.
    pub pipeline_text: Option<PipelineText>,
    /// Active shared font atlas for typography rendering.
    pub font_atlas: Option<Arc<FontAtlas>>,
}

impl RenderSystem {
    /// Instantiates an uninitialized render system. Graphics pipelines are lazily initialized upon first render.
    pub fn new() -> Self {
        Self {
            pipeline_3d: None,
            pipeline_2d: None,
            pipeline_text: None,
            font_atlas: None,
        }
    }

    /// Replaces the active shared typography font atlas.
    pub fn set_font_atlas(&mut self, atlas: Arc<FontAtlas>) {
        self.font_atlas = Some(atlas);
    }

    /// Returns a reference clone to the active shared font atlas if loaded.
    pub fn font_atlas(&self) -> Option<Arc<FontAtlas>> {
        self.font_atlas.clone()
    }

    /// Executes multi-pass rendering across all visual components in `world`.
    pub fn render_world(&mut self, ctx: &mut Context, world: &mut World) {
        // Drain and destroy any orphaned GPU resources from despawned entities
        for cleanup in world.pending_gpu_cleanup.drain(..) {
            cleanup.destroy(ctx.gl);
        }

        // Ensure 3D pipeline
        if self.pipeline_3d.is_none() {
            match Pipeline3D::new(ctx.gl) {
                Ok(p) => self.pipeline_3d = Some(p),
                Err(e) => log::error!("Failed to initialize Pipeline3D: {}", e),
            }
        }

        // Ensure 2D pipeline
        if self.pipeline_2d.is_none() {
            match Pipeline2D::new(ctx.gl) {
                Ok(p) => self.pipeline_2d = Some(p),
                Err(e) => log::error!("Failed to initialize Pipeline2D: {}", e),
            }
        }

        // Ensure text pipeline
        if self.pipeline_text.is_none() {
            match PipelineText::new(ctx.gl) {
                Ok(p) => self.pipeline_text = Some(p),
                Err(e) => log::error!("Failed to initialize PipelineText: {}", e),
            }
        }

        // Ensure default font atlas if not explicitly provided
        if self.font_atlas.is_none() {
            match Font::default_font() {
                Ok(font) => match FontAtlas::new(ctx.gl, font, 24.0) {
                    Ok(atlas) => self.font_atlas = Some(Arc::new(atlas)),
                    Err(e) => log::error!("Failed to initialize FontAtlas: {}", e),
                },
                Err(e) => log::error!("Failed to load default font: {}", e),
            }
        }

        let visibles = &world.visible;
        let global_transforms = &world.global_transform;
        let transforms = &world.transform;

        // Phase 1: 3D Render Pass
        if let Some(ref mut p3d) = self.pipeline_3d {
            p3d.light = world.light;
            let view = world.camera.view_matrix();
            let proj = world.camera.projection_matrix(ctx.aspect_ratio());
            p3d.begin(ctx.gl, &view, &proj);

            for (entity, mesh) in &mut world.mesh_3d {
                let is_visible = visibles.get(entity).map(|v| v.0).unwrap_or(true);
                if !is_visible {
                    continue;
                }

                let global = if let Some(gt) = global_transforms.get(entity) {
                    *gt
                } else if let Some(t) = transforms.get(entity) {
                    t.matrix()
                } else {
                    Mat4::IDENTITY
                };

                let _ = mesh.ensure_gpu_buffers(ctx.gl);
                // Dynamically re-upload vertices for soft bodies that deformed
                if world.soft_body.contains_key(entity) {
                    mesh.update_gpu_buffers(ctx.gl);
                }

                if let (Some(vao), Some(ebo)) = (&mesh.vao, &mesh.ebo) {
                    p3d.draw(
                        ctx.gl,
                        &global,
                        mesh.color,
                        mesh.emissive,
                        mesh.texture.as_deref(),
                        vao,
                        ebo,
                    );
                }
            }
            p3d.end(ctx.gl);
        }

        // Phase 2: 2D Render Pass (Overlay)
        if let Some(ref mut p2d) = self.pipeline_2d {
            p2d.begin(ctx.gl, ctx.width as f32, ctx.height as f32);

            for (entity, surface) in &world.surface_2d {
                let is_visible = visibles.get(entity).map(|v| v.0).unwrap_or(true);
                if !is_visible {
                    continue;
                }

                let global = if let Some(gt) = global_transforms.get(entity) {
                    *gt
                } else if let Some(t) = transforms.get(entity) {
                    t.matrix()
                } else {
                    Mat4::IDENTITY
                };

                let (scale, rot, trans) = global.to_scale_rotation_translation();
                let final_scale = Vec3::new(
                    scale.x * surface.size.x,
                    scale.y * surface.size.y,
                    1.0,
                );
                let model_2d = Mat4::from_scale_rotation_translation(
                    final_scale,
                    rot,
                    Vec3::new(trans.x, trans.y, 0.0),
                );
                p2d.draw(ctx.gl, &model_2d, surface.color, surface.texture.as_deref());
            }
            p2d.end(ctx.gl);
        }

        // Phase 3: Text Render Pass (Overlay)
        if let (Some(ref mut ptext), Some(ref atlas)) = (&mut self.pipeline_text, &self.font_atlas)
        {
            ptext.begin(ctx.gl, ctx.width as f32, ctx.height as f32);

            for (entity, text) in &world.text_2d {
                let is_visible = visibles.get(entity).map(|v| v.0).unwrap_or(true);
                if !is_visible {
                    continue;
                }

                let global = if let Some(gt) = global_transforms.get(entity) {
                    *gt
                } else if let Some(t) = transforms.get(entity) {
                    t.matrix()
                } else {
                    Mat4::IDENTITY
                };

                let (scale, _rot, trans) = global.to_scale_rotation_translation();
                let pos = Vec2::new(trans.x, trans.y);
                let final_scale = text.scale * scale.x;
                let _ =
                    ptext.draw_text(ctx.gl, atlas, &text.content, pos, text.color, final_scale);
            }
            ptext.end(ctx.gl);
        }
    }

    /// Releases GPU pipelines, shaders, buffers, texture atlas, and drains orphaned world GPU resources.
    pub fn destroy(&mut self, gl: &glow::Context, world: &mut World) {
        for cleanup in world.pending_gpu_cleanup.drain(..) {
            cleanup.destroy(gl);
        }
        self.destroy_pipelines(gl);
    }

    /// Releases GPU pipelines, shaders, buffers, and texture atlas without accessing a world.
    pub fn destroy_pipelines(&mut self, gl: &glow::Context) {
        if let Some(p3d) = self.pipeline_3d.take() {
            p3d.destroy(gl);
        }
        if let Some(p2d) = self.pipeline_2d.take() {
            p2d.destroy(gl);
        }
        if let Some(ptext) = self.pipeline_text.take() {
            ptext.destroy(gl);
        }
        if let Some(atlas) = self.font_atlas.take() {
            atlas.texture.destroy(gl);
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new()
    }
}
