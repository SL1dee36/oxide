//! C-compatible Foreign Function Interface (FFI) exports.
//!
//! Exposes an ANSI C99 compatible ABI (`extern "C"`) allowing C, C++, Python,
//! and other languages to drive windowing, scenes, collision detection, and rendering.
//!
//! # Ownership and Lifetime Model
//! - Heap allocations returned as opaque pointers (`*mut OxideApp`, `*mut OxideScene`) are managed
//!   via [`Box::into_raw`].
//! - Callers must release scenes with `oxide_scene_destroy`.
//! - Applications not executed via `oxide_app_run` must be released with `oxide_app_destroy`.
//!   Calling `oxide_app_run` consumes and deallocates the `OxideApp` instance upon event loop termination.
//! - `OxideContext` is a transient per-callback reference and must not be stored or used outside the callback scope.

use crate::core::app::{App, AppHandler};
use crate::core::color::Color;
use crate::core::context::Context;
use crate::spatial::camera::Camera;
use crate::spatial::node::{NodeId, NodeKind};
use crate::spatial::scene::Scene;
use crate::spatial::transform::Transform;
use crate::text::text2d::Text2D;
use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::collision::manifold::ContactManifold;
use glam::{Quat, Vec2, Vec3};
use std::ffi::{c_char, c_void, CStr};

/// C-compatible 2D vector.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideVec2 {
    pub x: f32,
    pub y: f32,
}

impl From<OxideVec2> for Vec2 {
    fn from(v: OxideVec2) -> Self {
        Vec2::new(v.x, v.y)
    }
}

impl From<Vec2> for OxideVec2 {
    fn from(v: Vec2) -> Self {
        Self { x: v.x, y: v.y }
    }
}

/// C-compatible 3D vector.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<OxideVec3> for Vec3 {
    fn from(v: OxideVec3) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

impl From<Vec3> for OxideVec3 {
    fn from(v: Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

/// C-compatible 4-component orientation quaternion.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideQuat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<OxideQuat> for Quat {
    fn from(q: OxideQuat) -> Self {
        Quat::from_xyzw(q.x, q.y, q.z, q.w)
    }
}

impl From<Quat> for OxideQuat {
    fn from(q: Quat) -> Self {
        Self { x: q.x, y: q.y, z: q.z, w: q.w }
    }
}

/// C-compatible 4-component RGBA color [0.0, 1.0].
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl From<OxideColor> for Color {
    fn from(c: OxideColor) -> Self {
        Color::rgba(c.r, c.g, c.b, c.a)
    }
}

impl From<Color> for OxideColor {
    fn from(c: Color) -> Self {
        Self { r: c.r, g: c.g, b: c.b, a: c.a }
    }
}

/// C-compatible 3D affine transformation structure.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideTransform {
    pub translation: OxideVec3,
    pub rotation: OxideQuat,
    pub scale: OxideVec3,
}

impl From<OxideTransform> for Transform {
    fn from(t: OxideTransform) -> Self {
        Transform {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

impl From<Transform> for OxideTransform {
    fn from(t: Transform) -> Self {
        Self {
            translation: t.translation.into(),
            rotation: t.rotation.into(),
            scale: t.scale.into(),
        }
    }
}

/// C-compatible contact manifold describing collision penetration and separation vector.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideContact {
    pub normal: OxideVec3,
    pub depth: f32,
    pub point: OxideVec3,
}

impl From<ContactManifold> for OxideContact {
    fn from(m: ContactManifold) -> Self {
        Self {
            normal: m.normal.into(),
            depth: m.depth,
            point: m.point.into(),
        }
    }
}

impl From<OxideContact> for ContactManifold {
    fn from(c: OxideContact) -> Self {
        ContactManifold::new(c.normal.into(), c.depth, c.point.into())
    }
}

/// Opaque wrapper around per-frame [`Context`].
pub struct OxideContext {
    ctx: *mut Context<'static>,
}

/// Initialization callback invoked once when OpenGL is ready.
pub type OxideInitFn = Option<extern "C" fn(ctx: *mut OxideContext, user_data: *mut c_void)>;
/// Update callback invoked every frame before rendering.
pub type OxideUpdateFn = Option<extern "C" fn(ctx: *mut OxideContext, dt: f32, user_data: *mut c_void)>;
/// Render callback invoked every frame to submit draw commands.
pub type OxideRenderFn = Option<extern "C" fn(ctx: *mut OxideContext, user_data: *mut c_void)>;

/// C-compatible callback table passed to [`oxide_app_run`].
#[repr(C)]
pub struct OxideCallbacks {
    pub on_init: OxideInitFn,
    pub on_update: OxideUpdateFn,
    pub on_render: OxideRenderFn,
    pub user_data: *mut c_void,
}

/// Opaque application configuration container allocated on heap.
pub struct OxideApp {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

/// Opaque scene graph container allocated on heap.
pub struct OxideScene {
    pub inner: Scene,
}

struct FfiAppHandler {
    callbacks: OxideCallbacks,
}

impl AppHandler for FfiAppHandler {
    fn on_init(&mut self, ctx: &mut Context) {
        if let Some(f) = self.callbacks.on_init {
            let mut ffi_ctx = OxideContext {
                ctx: unsafe { std::mem::transmute(ctx) },
            };
            f(&mut ffi_ctx, self.callbacks.user_data);
        }
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        if let Some(f) = self.callbacks.on_update {
            let mut ffi_ctx = OxideContext {
                ctx: unsafe { std::mem::transmute(ctx) },
            };
            f(&mut ffi_ctx, dt, self.callbacks.user_data);
        }
    }

    fn on_render(&mut self, ctx: &mut Context) {
        if let Some(f) = self.callbacks.on_render {
            let mut ffi_ctx = OxideContext {
                ctx: unsafe { std::mem::transmute(ctx) },
            };
            f(&mut ffi_ctx, self.callbacks.user_data);
        }
    }
}

// ==========================================
// App Lifecycle API
// ==========================================

/// Allocates an [`OxideApp`] on the heap with specified window title and resolution.
#[no_mangle]
pub unsafe extern "C" fn oxide_app_create(
    title: *const c_char,
    width: u32,
    height: u32,
) -> *mut OxideApp {
    let title_str = if title.is_null() {
        "Oxide Native App".to_string()
    } else {
        CStr::from_ptr(title).to_string_lossy().into_owned()
    };

    Box::into_raw(Box::new(OxideApp {
        title: title_str,
        width,
        height,
    }))
}

/// Releases an unexecuted [`OxideApp`] from the heap. Do not call if passed to [`oxide_app_run`].
#[no_mangle]
pub unsafe extern "C" fn oxide_app_destroy(app: *mut OxideApp) {
    if !app.is_null() {
        drop(Box::from_raw(app));
    }
}

/// Starts the window event loop. Takes ownership of `app` and deallocates it upon completion.
#[no_mangle]
pub unsafe extern "C" fn oxide_app_run(
    app: *mut OxideApp,
    callbacks: OxideCallbacks,
) -> i32 {
    if app.is_null() {
        return -1;
    }

    let app_box = Box::from_raw(app);
    let handler = FfiAppHandler { callbacks };

    let res = App::build()
        .with_title(app_box.title)
        .with_size(app_box.width, app_box.height)
        .run(handler);

    match res {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

// ==========================================
// Context API
// ==========================================

/// Clears color and depth buffer attachments with specified RGBA color.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_clear(ctx: *mut OxideContext, color: OxideColor) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.clear(color.into());
    }
}

/// Returns measured frames per second.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_fps(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.fps()
    } else {
        0.0
    }
}

/// Returns time elapsed during the previous frame in seconds ($\Delta t$).
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_delta_time(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.delta_time()
    } else {
        0.0
    }
}

/// Returns active viewport aspect ratio (`width / height`).
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_aspect_ratio(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.aspect_ratio()
    } else {
        1.0
    }
}

// ==========================================
// Scene Graph API
// ==========================================

/// Allocates an empty [`OxideScene`] on the heap. Must be released with [`oxide_scene_destroy`].
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_create() -> *mut OxideScene {
    Box::into_raw(Box::new(OxideScene {
        inner: Scene::new(),
    }))
}

/// Deallocates an [`OxideScene`] previously created via [`oxide_scene_create`].
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_destroy(scene: *mut OxideScene) {
    if !scene.is_null() {
        drop(Box::from_raw(scene));
    }
}

/// Configures the scene's 3D perspective camera with look-at target.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_set_camera_perspective(
    scene: *mut OxideScene,
    fov_deg: f32,
    aspect: f32,
    near: f32,
    far: f32,
    eye: OxideVec3,
    target: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let mut cam = Camera::perspective(fov_deg, aspect, near, far);
        cam.look_at(eye.into(), target.into(), Vec3::Y);
        s.inner.set_camera(cam);
    }
}

/// Spawns a 3D cube mesh with matching box collider, returning its numeric node identifier.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_cube_3d(
    scene: *mut OxideScene,
    size: f32,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let id = s.inner.spawn_mesh_3d(
            Mesh3D::cube(size),
            transform.into(),
            Collider::box_3d(Vec3::splat(size)),
        );
        id.0 as u64
    } else {
        u64::MAX
    }
}

/// Spawns a 3D UV sphere mesh with matching sphere collider, returning its numeric node identifier.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_sphere_3d(
    scene: *mut OxideScene,
    radius: f32,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let id = s.inner.spawn_mesh_3d(
            Mesh3D::sphere(radius, 24, 24),
            transform.into(),
            Collider::sphere(radius),
        );
        id.0 as u64
    } else {
        u64::MAX
    }
}

/// Spawns a 2D rectangular surface in screen coordinates with box collider, returning its node identifier.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_surface_2d(
    scene: *mut OxideScene,
    size: OxideVec2,
    color: OxideColor,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let id = s.inner.spawn_surface_2d(
            Surface2D::from_color(size.into(), color.into()),
            transform.into(),
            Collider::box_2d(size.into()),
        );
        id.0 as u64
    } else {
        u64::MAX
    }
}

/// Spawns a 2D typography text element, returning its numeric node identifier.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_text_2d(
    scene: *mut OxideScene,
    text: *const c_char,
    color: OxideColor,
    scale: f32,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let text_str = if text.is_null() {
            ""
        } else {
            CStr::from_ptr(text).to_str().unwrap_or("")
        };

        let id = s.inner.spawn_text_2d(
            Text2D::new(text_str)
                .with_color(color.into())
                .with_scale(scale),
            transform.into(),
        );
        id.0 as u64
    } else {
        u64::MAX
    }
}

/// Updates the string content of an existing 2D text node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_text(
    scene: *mut OxideScene,
    node_id: u64,
    text: *const c_char,
) {
    if let Some(s) = scene.as_mut() {
        let text_str = if text.is_null() {
            ""
        } else {
            CStr::from_ptr(text).to_str().unwrap_or("")
        };

        if let NodeKind::Text2D(ref mut t) = s.inner.node_mut(NodeId(node_id as usize)).kind {
            t.content = text_str.to_string();
        }
    }
}

/// Rotates node around its local Y-axis by `rad` radians.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_rotate_y(
    scene: *mut OxideScene,
    node_id: u64,
    rad: f32,
) {
    if let Some(s) = scene.as_mut() {
        s.inner.node_mut(NodeId(node_id as usize)).transform.rotate_y(rad);
    }
}

/// Rotates node around its local X-axis by `rad` radians.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_rotate_x(
    scene: *mut OxideScene,
    node_id: u64,
    rad: f32,
) {
    if let Some(s) = scene.as_mut() {
        s.inner.node_mut(NodeId(node_id as usize)).transform.rotate_x(rad);
    }
}

/// Evaluates geometric intersection between colliders of `node_a` and `node_b`.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_check_collision(
    scene: *mut OxideScene,
    node_a: u64,
    node_b: u64,
    out_contact: *mut OxideContact,
) -> bool {
    if let Some(s) = scene.as_mut() {
        if let Some(contact) = s.inner.check_intersection(NodeId(node_a as usize), NodeId(node_b as usize)) {
            if !out_contact.is_null() {
                *out_contact = contact.into();
            }
            return true;
        }
    }
    false
}

/// Kinematically separates two nodes along the contact normal to eliminate penetration.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_resolve_penetration(
    scene: *mut OxideScene,
    node_a: u64,
    node_b: u64,
    contact: *const OxideContact,
) {
    if let Some(s) = scene.as_mut() {
        if let Some(c) = contact.as_ref() {
            let manifold: ContactManifold = (*c).into();
            s.inner.resolve_penetration(NodeId(node_a as usize), NodeId(node_b as usize), &manifold);
        }
    }
}

/// Advances scene node behaviors and recalculates transform hierarchies.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_update(
    scene: *mut OxideScene,
    ctx: *mut OxideContext,
    dt: f32,
) {
    if let (Some(s), Some(c)) = (scene.as_mut(), ctx.as_mut().and_then(|x| x.ctx.as_mut())) {
        s.inner.update(c, dt);
    }
}

/// Renders the scene graph (3D passes, 2D planar passes, and typography overlay).
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_render(
    scene: *mut OxideScene,
    ctx: *mut OxideContext,
) {
    if let (Some(s), Some(c)) = (scene.as_mut(), ctx.as_mut().and_then(|x| x.ctx.as_mut())) {
        s.inner.render(c);
    }
}
