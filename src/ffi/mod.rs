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

pub struct OxideContext {
    ctx: *mut Context<'static>,
}

pub type OxideInitFn = Option<extern "C" fn(ctx: *mut OxideContext, user_data: *mut c_void)>;
pub type OxideUpdateFn = Option<extern "C" fn(ctx: *mut OxideContext, dt: f32, user_data: *mut c_void)>;
pub type OxideRenderFn = Option<extern "C" fn(ctx: *mut OxideContext, user_data: *mut c_void)>;

#[repr(C)]
pub struct OxideCallbacks {
    pub on_init: OxideInitFn,
    pub on_update: OxideUpdateFn,
    pub on_render: OxideRenderFn,
    pub user_data: *mut c_void,
}

pub struct OxideApp {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

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

// App API
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

#[no_mangle]
pub unsafe extern "C" fn oxide_app_destroy(app: *mut OxideApp) {
    if !app.is_null() {
        drop(Box::from_raw(app));
    }
}

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

// Context API
#[no_mangle]
pub unsafe extern "C" fn oxide_context_clear(ctx: *mut OxideContext, color: OxideColor) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.clear(color.into());
    }
}

#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_fps(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.fps()
    } else {
        0.0
    }
}

#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_delta_time(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.delta_time()
    } else {
        0.0
    }
}

#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_aspect_ratio(ctx: *mut OxideContext) -> f32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.aspect_ratio()
    } else {
        1.0
    }
}

// Scene API
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_create() -> *mut OxideScene {
    Box::into_raw(Box::new(OxideScene {
        inner: Scene::new(),
    }))
}

#[no_mangle]
pub unsafe extern "C" fn oxide_scene_destroy(scene: *mut OxideScene) {
    if !scene.is_null() {
        drop(Box::from_raw(scene));
    }
}

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

#[no_mangle]
pub unsafe extern "C" fn oxide_scene_render(
    scene: *mut OxideScene,
    ctx: *mut OxideContext,
) {
    if let (Some(s), Some(c)) = (scene.as_mut(), ctx.as_mut().and_then(|x| x.ctx.as_mut())) {
        s.inner.render(c);
    }
}
