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
use crate::ecs::entity::Entity;
use crate::ecs::world::World;
use crate::physics::{DistanceConstraint, RigidBody, RigidBodyType, SoftBody};
use crate::renderer::render_system::RenderSystem;
use crate::spatial::camera::Camera;
use crate::spatial::transform::Transform;
use crate::text::text2d::Text2D;
use crate::assets::mesh::Mesh3D;
use crate::assets::surface::Surface2D;
use crate::collision::collider::Collider;
use crate::collision::manifold::ContactManifold;
use crate::collision::ray::Ray3D;
use crate::core::input::{KeyCode, MouseButton};
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

/// C-compatible raycast intersection hit report.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OxideRaycastHit {
    pub node_id: u64,
    pub distance: f32,
    pub point: OxideVec3,
    pub normal: OxideVec3,
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
    pub world: World,
    pub render_system: RenderSystem,
}

struct FfiAppHandler {
    callbacks: OxideCallbacks,
}

impl AppHandler for FfiAppHandler {
    fn on_init(&mut self, ctx: &mut Context) {
        if let Some(f) = self.callbacks.on_init {
            let mut ffi_ctx = OxideContext {
                ctx: (ctx as *mut Context<'_>).cast::<Context<'static>>(),
            };
            f(&mut ffi_ctx, self.callbacks.user_data);
        }
    }

    fn on_update(&mut self, ctx: &mut Context, dt: f32) {
        if let Some(f) = self.callbacks.on_update {
            let mut ffi_ctx = OxideContext {
                ctx: (ctx as *mut Context<'_>).cast::<Context<'static>>(),
            };
            f(&mut ffi_ctx, dt, self.callbacks.user_data);
        }
    }

    fn on_render(&mut self, ctx: &mut Context) {
        if let Some(f) = self.callbacks.on_render {
            let mut ffi_ctx = OxideContext {
                ctx: (ctx as *mut Context<'_>).cast::<Context<'static>>(),
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

/// Returns active viewport width in pixels.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_width(ctx: *mut OxideContext) -> u32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.width
    } else {
        0
    }
}

/// Returns active viewport height in pixels.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_height(ctx: *mut OxideContext) -> u32 {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.height
    } else {
        0
    }
}

// ==========================================
// Context Input API
// ==========================================

fn parse_key_code(name: &str) -> Option<KeyCode> {
    match name.trim().to_uppercase().as_str() {
        "A" | "KEYA" => Some(KeyCode::KeyA),
        "B" | "KEYB" => Some(KeyCode::KeyB),
        "C" | "KEYC" => Some(KeyCode::KeyC),
        "D" | "KEYD" => Some(KeyCode::KeyD),
        "E" | "KEYE" => Some(KeyCode::KeyE),
        "F" | "KEYF" => Some(KeyCode::KeyF),
        "G" | "KEYG" => Some(KeyCode::KeyG),
        "H" | "KEYH" => Some(KeyCode::KeyH),
        "I" | "KEYI" => Some(KeyCode::KeyI),
        "J" | "KEYJ" => Some(KeyCode::KeyJ),
        "K" | "KEYK" => Some(KeyCode::KeyK),
        "L" | "KEYL" => Some(KeyCode::KeyL),
        "M" | "KEYM" => Some(KeyCode::KeyM),
        "N" | "KEYN" => Some(KeyCode::KeyN),
        "O" | "KEYO" => Some(KeyCode::KeyO),
        "P" | "KEYP" => Some(KeyCode::KeyP),
        "Q" | "KEYQ" => Some(KeyCode::KeyQ),
        "R" | "KEYR" => Some(KeyCode::KeyR),
        "S" | "KEYS" => Some(KeyCode::KeyS),
        "T" | "KEYT" => Some(KeyCode::KeyT),
        "U" | "KEYU" => Some(KeyCode::KeyU),
        "V" | "KEYV" => Some(KeyCode::KeyV),
        "W" | "KEYW" => Some(KeyCode::KeyW),
        "X" | "KEYX" => Some(KeyCode::KeyX),
        "Y" | "KEYY" => Some(KeyCode::KeyY),
        "Z" | "KEYZ" => Some(KeyCode::KeyZ),
        "0" | "DIGIT0" => Some(KeyCode::Digit0),
        "1" | "DIGIT1" => Some(KeyCode::Digit1),
        "2" | "DIGIT2" => Some(KeyCode::Digit2),
        "3" | "DIGIT3" => Some(KeyCode::Digit3),
        "4" | "DIGIT4" => Some(KeyCode::Digit4),
        "5" | "DIGIT5" => Some(KeyCode::Digit5),
        "6" | "DIGIT6" => Some(KeyCode::Digit6),
        "7" | "DIGIT7" => Some(KeyCode::Digit7),
        "8" | "DIGIT8" => Some(KeyCode::Digit8),
        "9" | "DIGIT9" => Some(KeyCode::Digit9),
        "SPACE" => Some(KeyCode::Space),
        "ESCAPE" | "ESC" => Some(KeyCode::Escape),
        "ENTER" | "RETURN" => Some(KeyCode::Enter),
        "TAB" => Some(KeyCode::Tab),
        "SHIFT" | "SHIFTLEFT" => Some(KeyCode::ShiftLeft),
        "SHIFTRIGHT" => Some(KeyCode::ShiftRight),
        "CONTROL" | "CONTROLLEFT" | "CTRL" => Some(KeyCode::ControlLeft),
        "CONTROLRIGHT" => Some(KeyCode::ControlRight),
        "ALT" | "ALTLEFT" => Some(KeyCode::AltLeft),
        "ALTRIGHT" => Some(KeyCode::AltRight),
        "UP" | "ARROWUP" => Some(KeyCode::ArrowUp),
        "DOWN" | "ARROWDOWN" => Some(KeyCode::ArrowDown),
        "LEFT" | "ARROWLEFT" => Some(KeyCode::ArrowLeft),
        "RIGHT" | "ARROWRIGHT" => Some(KeyCode::ArrowRight),
        _ => None,
    }
}

fn parse_mouse_button(button: u32) -> MouseButton {
    match button {
        0 => MouseButton::Left,
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        _ => MouseButton::Other(button as u16),
    }
}

/// Returns `true` if physical keyboard key specified by `key_name` is currently held down.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_key_down(
    ctx: *mut OxideContext,
    key_name: *const c_char,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !key_name.is_null() {
            if let Ok(name) = CStr::from_ptr(key_name).to_str() {
                if let Some(code) = parse_key_code(name) {
                    return c.input.is_key_down(code);
                }
            }
        }
    }
    false
}

/// Returns `true` if physical keyboard key transitioned to pressed state in the current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_key_just_pressed(
    ctx: *mut OxideContext,
    key_name: *const c_char,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !key_name.is_null() {
            if let Ok(name) = CStr::from_ptr(key_name).to_str() {
                if let Some(code) = parse_key_code(name) {
                    return c.input.is_key_just_pressed(code);
                }
            }
        }
    }
    false
}

/// Returns `true` if physical keyboard key was released during the current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_key_just_released(
    ctx: *mut OxideContext,
    key_name: *const c_char,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !key_name.is_null() {
            if let Ok(name) = CStr::from_ptr(key_name).to_str() {
                if let Some(code) = parse_key_code(name) {
                    return c.input.is_key_just_released(code);
                }
            }
        }
    }
    false
}

/// Returns `true` if mouse button (0: Left, 1: Right, 2: Middle) is held down.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_mouse_button_down(
    ctx: *mut OxideContext,
    button: u32,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        return c.input.is_mouse_button_down(parse_mouse_button(button));
    }
    false
}

/// Returns `true` if mouse button transitioned to pressed state in current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_mouse_button_just_pressed(
    ctx: *mut OxideContext,
    button: u32,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        return c.input.is_mouse_button_just_pressed(parse_mouse_button(button));
    }
    false
}

/// Returns `true` if mouse button was released during current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_is_mouse_button_just_released(
    ctx: *mut OxideContext,
    button: u32,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        return c.input.is_mouse_button_just_released(parse_mouse_button(button));
    }
    false
}

/// Retrieves the current physical cursor position in window pixels.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_mouse_position(
    ctx: *mut OxideContext,
    out_pos: *mut OxideVec2,
) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !out_pos.is_null() {
            *out_pos = c.input.mouse_position().into();
        }
    }
}

/// Retrieves cursor movement delta vector in pixels accumulated during current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_mouse_delta(
    ctx: *mut OxideContext,
    out_delta: *mut OxideVec2,
) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !out_delta.is_null() {
            *out_delta = c.input.mouse_delta().into();
        }
    }
}

/// Retrieves scroll wheel delta accumulated during current frame.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_get_mouse_wheel(
    ctx: *mut OxideContext,
    out_wheel: *mut OxideVec2,
) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        if !out_wheel.is_null() {
            *out_wheel = c.input.mouse_wheel_delta().into();
        }
    }
}

/// Sets OS mouse cursor visibility.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_set_cursor_visible(
    ctx: *mut OxideContext,
    visible: bool,
) {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.set_cursor_visible(visible);
    }
}

/// Configures OS mouse cursor grab/confinement mode.
#[no_mangle]
pub unsafe extern "C" fn oxide_context_set_cursor_grab(
    ctx: *mut OxideContext,
    grab: bool,
) -> bool {
    if let Some(c) = ctx.as_mut().and_then(|x| x.ctx.as_mut()) {
        c.set_cursor_grab(grab)
    } else {
        false
    }
}

// ==========================================
// Scene Graph API
// ==========================================

/// Allocates an empty [`OxideScene`] on the heap. Must be released with [`oxide_scene_destroy`].
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_create() -> *mut OxideScene {
    Box::into_raw(Box::new(OxideScene {
        world: World::new(),
        render_system: RenderSystem::new(),
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
    _aspect: f32,
    near: f32,
    far: f32,
    eye: OxideVec3,
    target: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let mut cam = Camera::perspective(fov_deg, near, far);
        cam.look_at(eye.into(), target.into(), Vec3::Y);
        s.world.set_camera(cam);
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
        let id = s.world.spawn_mesh_3d(
            Mesh3D::cube(size),
            transform.into(),
            Collider::box_3d(Vec3::splat(size)),
        );
        id.to_ffi()
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
        let id = s.world.spawn_mesh_3d(
            Mesh3D::sphere(radius, 24, 24),
            transform.into(),
            Collider::sphere(radius),
        );
        id.to_ffi()
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
        let id = s.world.spawn_surface_2d(
            Surface2D::from_color(size.into(), color.into()),
            transform.into(),
            Collider::box_2d(size.into()),
        );
        id.to_ffi()
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

        let id = s.world.spawn_text_2d(
            Text2D::new(text_str)
                .with_color(color.into())
                .with_scale(scale),
            transform.into(),
        );
        id.to_ffi()
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

        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.text_2d_mut(entity) {
            t.content = text_str.to_string();
        }
    }
}

/// Configures directional lighting in the scene.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_set_light(
    scene: *mut OxideScene,
    direction: OxideVec3,
    color: OxideVec3,
    ambient: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let dir: Vec3 = direction.into();
        let col: Vec3 = color.into();
        let amb: Vec3 = ambient.into();
        s.world.set_light(crate::renderer::pipeline_3d::Light3D {
            direction: if dir.length_squared() > 1e-6 {
                dir.normalize()
            } else {
                Vec3::Y
            },
            color: col,
            ambient: amb,
        });
    }
}

/// Sets camera look-at orientation directly.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_set_camera_look_at(
    scene: *mut OxideScene,
    eye: OxideVec3,
    target: OxideVec3,
    up: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        s.world.camera_mut().look_at(eye.into(), target.into(), up.into());
    }
}

/// Spawns a 3D cube mesh with specific color tint.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_cube_3d_colored(
    scene: *mut OxideScene,
    size: f32,
    color: OxideColor,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let mesh = Mesh3D::cube(size).with_color(color.into());
        let id = s.world.spawn_mesh_3d(
            mesh,
            transform.into(),
            Collider::box_3d(Vec3::splat(size)),
        );
        id.to_ffi()
    } else {
        u64::MAX
    }
}

/// Spawns a 3D sphere mesh with specific color tint.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_sphere_3d_colored(
    scene: *mut OxideScene,
    radius: f32,
    color: OxideColor,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let mesh = Mesh3D::sphere(radius, 24, 24).with_color(color.into());
        let id = s.world.spawn_mesh_3d(mesh, transform.into(), Collider::sphere(radius));
        id.to_ffi()
    } else {
        u64::MAX
    }
}

/// Spawns a 3D box mesh node with dimensions `(width, height, depth)` and optional static flag.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_box_3d(
    scene: *mut OxideScene,
    width: f32,
    height: f32,
    depth: f32,
    color: OxideColor,
    transform: OxideTransform,
    is_static: bool,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let mesh = Mesh3D::box_mesh(width, height, depth).with_color(color.into());
        let id = if is_static {
            s.world.spawn_static_mesh_3d(
                mesh,
                transform.into(),
                Collider::box_3d(Vec3::new(width, height, depth)),
            )
        } else {
            s.world.spawn_mesh_3d(
                mesh,
                transform.into(),
                Collider::box_3d(Vec3::new(width, height, depth)),
            )
        };
        id.to_ffi()
    } else {
        u64::MAX
    }
}

/// Removes and despawns a node and all of its recursive children from the scene.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_despawn_node(scene: *mut OxideScene, node_id: u64) -> bool {
    if let Some(s) = scene.as_mut() {
        s.world.despawn(Entity::from_ffi(node_id))
    } else {
        false
    }
}

/// Retrieves the local transform of a spatial node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_get_transform(
    scene: *mut OxideScene,
    node_id: u64,
    out_transform: *mut OxideTransform,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform(entity) {
            if !out_transform.is_null() {
                *out_transform = (*t).into();
            }
            return true;
        }
    }
    false
}

/// Sets the local transform of a spatial node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_transform(
    scene: *mut OxideScene,
    node_id: u64,
    transform: OxideTransform,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if s.world.is_alive(entity) {
            s.world.set_transform(entity, transform.into());
            return true;
        }
    }
    false
}

/// Gets translation position vector of a node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_get_translation(
    scene: *mut OxideScene,
    node_id: u64,
    out_translation: *mut OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform(entity) {
            if !out_translation.is_null() {
                *out_translation = t.translation.into();
            }
            return true;
        }
    }
    false
}

/// Sets translation position vector of a node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_translation(
    scene: *mut OxideScene,
    node_id: u64,
    translation: OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.translation = translation.into();
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
            return true;
        }
    }
    false
}

/// Gets rotation quaternion of a node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_get_rotation(
    scene: *mut OxideScene,
    node_id: u64,
    out_rotation: *mut OxideQuat,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform(entity) {
            if !out_rotation.is_null() {
                *out_rotation = t.rotation.into();
            }
            return true;
        }
    }
    false
}

/// Sets rotation quaternion of a node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_rotation(
    scene: *mut OxideScene,
    node_id: u64,
    rotation: OxideQuat,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.rotation = rotation.into();
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
            return true;
        }
    }
    false
}

/// Rotates node around its local Y-axis by `rad` radians.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_rotate_y(
    scene: *mut OxideScene,
    node_id: u64,
    rad: f32,
) {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.rotate_y(rad);
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
        }
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
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.rotate_x(rad);
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
        }
    }
}

/// Rotates node around its local Z-axis by `rad` radians.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_rotate_z(
    scene: *mut OxideScene,
    node_id: u64,
    rad: f32,
) {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.rotate_z(rad);
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
        }
    }
}

/// Sets node scale vector.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_scale(
    scene: *mut OxideScene,
    node_id: u64,
    scale: OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(t) = s.world.transform_mut(entity) {
            t.scale = scale.into();
            let matrix = t.matrix();
            if s.world.parent.get(entity).is_none() {
                s.world.global_transform.insert(entity, matrix);
            }
            return true;
        }
    }
    false
}

/// Sets the base color tint of an existing node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_color(
    scene: *mut OxideScene,
    node_id: u64,
    color: OxideColor,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        let mut updated = false;
        if let Some(m) = s.world.mesh_3d_mut(entity) {
            m.color = color.into();
            updated = true;
        }
        if let Some(surf) = s.world.surface_2d_mut(entity) {
            surf.color = color.into();
            updated = true;
        }
        if let Some(txt) = s.world.text_2d_mut(entity) {
            txt.color = color.into();
            updated = true;
        }
        return updated;
    }
    false
}

/// Sets node visibility flag.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_visible(
    scene: *mut OxideScene,
    node_id: u64,
    visible: bool,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if s.world.is_alive(entity) {
            s.world.set_visible(entity, visible);
            return true;
        }
    }
    false
}

/// Sets node static obstacle flag.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_set_static(
    scene: *mut OxideScene,
    node_id: u64,
    is_static: bool,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if s.world.is_alive(entity) {
            s.world.set_static(entity, is_static);
            return true;
        }
    }
    false
}

/// Casts a ray through the scene, finding the closest intersecting node.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_raycast(
    scene: *mut OxideScene,
    origin: OxideVec3,
    direction: OxideVec3,
    out_hit: *mut OxideRaycastHit,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let dir: Vec3 = direction.into();
        if dir.length_squared() > 1e-6 {
            let ray = Ray3D::new(origin.into(), dir);
            s.world.update_transforms();
            if let Some((hit_entity, hit)) = s.world.raycast(&ray) {
                if !out_hit.is_null() {
                    *out_hit = OxideRaycastHit {
                        node_id: hit_entity.to_ffi(),
                        distance: hit.distance,
                        point: hit.point.into(),
                        normal: hit.normal.into(),
                    };
                }
                return true;
            }
        }
    }
    false
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
        let a = Entity::from_ffi(node_a);
        let b = Entity::from_ffi(node_b);
        if let Some(contact) = s.world.check_intersection(a, b) {
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
            let a = Entity::from_ffi(node_a);
            let b = Entity::from_ffi(node_b);
            let manifold: ContactManifold = (*c).into();
            s.world.resolve_penetration(a, b, &manifold);
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
        s.world.update(c, dt);
    }
}

/// Renders the scene graph (3D passes, 2D planar passes, and typography overlay).
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_render(
    scene: *mut OxideScene,
    ctx: *mut OxideContext,
) {
    if let (Some(s), Some(c)) = (scene.as_mut(), ctx.as_mut().and_then(|x| x.ctx.as_mut())) {
        s.render_system.render_world(c, &mut s.world);
    }
}

// ---------------------------------------------------------------------------
// Physics Simulation C ABI
// ---------------------------------------------------------------------------

/// Advances the physics simulation for all dynamic rigid bodies and deformable soft bodies in the scene.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_step_physics(
    scene: *mut OxideScene,
    dt: f32,
) {
    if let Some(s) = scene.as_mut() {
        s.world.step_physics(dt);
    }
}

/// Spawns a box mesh node with an attached rigid body and collider.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_rigid_body_box(
    scene: *mut OxideScene,
    size: OxideVec3,
    transform: OxideTransform,
    mass: f32,
    restitution: f32,
    friction: f32,
    is_static: bool,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let size_v = Vec3::new(size.x, size.y, size.z);
        let mesh = Mesh3D::box_mesh(size.x, size.y, size.z);
        let collider = Collider::box_3d(size_v);
        let body_type = if is_static {
            RigidBodyType::Static
        } else {
            RigidBodyType::Dynamic
        };
        let rb = RigidBody::new(mass, body_type)
            .with_restitution(restitution)
            .with_friction(friction);
        let entity = s.world.spawn_rigid_body_with_mesh(mesh, transform.into(), collider, rb);
        return entity.to_ffi();
    }
    0
}

/// Spawns a spherical mesh node with an attached rigid body and collider.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_rigid_body_sphere(
    scene: *mut OxideScene,
    radius: f32,
    transform: OxideTransform,
    mass: f32,
    restitution: f32,
    friction: f32,
    is_static: bool,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let mesh = Mesh3D::sphere(radius, 16, 16);
        let collider = Collider::sphere(radius);
        let body_type = if is_static {
            RigidBodyType::Static
        } else {
            RigidBodyType::Dynamic
        };
        let rb = RigidBody::new(mass, body_type)
            .with_restitution(restitution)
            .with_friction(friction);
        let entity = s.world.spawn_rigid_body_with_mesh(mesh, transform.into(), collider, rb);
        return entity.to_ffi();
    }
    0
}

/// Attaches a [`RigidBody`] component to an existing entity node.
///
/// `body_type`: 0 = Dynamic, 1 = Static, 2 = Kinematic.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_add_rigid_body(
    scene: *mut OxideScene,
    node_id: u64,
    mass: f32,
    restitution: f32,
    friction: f32,
    body_type: u32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if s.world.is_alive(entity) {
            let rb_type = match body_type {
                1 => RigidBodyType::Static,
                2 => RigidBodyType::Kinematic,
                _ => RigidBodyType::Dynamic,
            };
            let rb = RigidBody::new(mass, rb_type)
                .with_restitution(restitution)
                .with_friction(friction);
            s.world.rigid_body.insert(entity, rb);
            return true;
        }
    }
    false
}

/// Applies a persistent spatial force to a rigid body's force accumulator.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_apply_force(
    scene: *mut OxideScene,
    node_id: u64,
    force: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get_mut(entity) {
            rb.apply_force(force.into());
        }
    }
}

/// Applies a rotational torque to a rigid body's torque accumulator.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_apply_torque(
    scene: *mut OxideScene,
    node_id: u64,
    torque: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get_mut(entity) {
            rb.apply_torque(torque.into());
        }
    }
}

/// Applies an instantaneous linear velocity impulse to a rigid body.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_apply_impulse(
    scene: *mut OxideScene,
    node_id: u64,
    impulse: OxideVec3,
) {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get_mut(entity) {
            rb.apply_impulse(impulse.into());
        }
    }
}

/// Overwrites a rigid body's linear velocity.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_set_velocity(
    scene: *mut OxideScene,
    node_id: u64,
    velocity: OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get_mut(entity) {
            rb.velocity = velocity.into();
            return true;
        }
    }
    false
}

/// Retrieves a rigid body's current linear velocity vector.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_get_velocity(
    scene: *mut OxideScene,
    node_id: u64,
    out_velocity: *mut OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get(entity) {
            if !out_velocity.is_null() {
                *out_velocity = rb.velocity.into();
            }
            return true;
        }
    }
    false
}

/// Overwrites a rigid body's angular velocity.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_set_angular_velocity(
    scene: *mut OxideScene,
    node_id: u64,
    ang_vel: OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get_mut(entity) {
            rb.angular_velocity = ang_vel.into();
            return true;
        }
    }
    false
}

/// Retrieves a rigid body's current angular velocity vector.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_get_angular_velocity(
    scene: *mut OxideScene,
    node_id: u64,
    out_ang_vel: *mut OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get(entity) {
            if !out_ang_vel.is_null() {
                *out_ang_vel = rb.angular_velocity.into();
            }
            return true;
        }
    }
    false
}

/// Retrieves a rigid body's mass in kilograms.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_rigid_body_get_mass(
    scene: *mut OxideScene,
    node_id: u64,
    out_mass: *mut f32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(rb) = s.world.rigid_body.get(entity) {
            if !out_mass.is_null() {
                *out_mass = rb.mass;
            }
            return true;
        }
    }
    false
}

/// Spawns a deformable cloth soft body with spring constraints and triangular mesh.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_spawn_soft_body_cloth(
    scene: *mut OxideScene,
    width: f32,
    height: f32,
    segments_x: u32,
    segments_y: u32,
    total_mass: f32,
    stiffness: f32,
    transform: OxideTransform,
) -> u64 {
    if let Some(s) = scene.as_mut() {
        let entity = s.world.spawn_soft_body_cloth(
            width,
            height,
            segments_x as usize,
            segments_y as usize,
            total_mass,
            stiffness,
            transform.into(),
        );
        return entity.to_ffi();
    }
    0
}

/// Attaches a cloth soft body simulation component to an existing mesh entity.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_node_add_soft_body_cloth(
    scene: *mut OxideScene,
    node_id: u64,
    width: f32,
    height: f32,
    segments_x: u32,
    segments_y: u32,
    total_mass: f32,
    stiffness: f32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if s.world.is_alive(entity) {
            let (sb, _mesh) = SoftBody::cloth(width, height, segments_x as usize, segments_y as usize, total_mass, stiffness);
            s.world.soft_body.insert(entity, sb);
            return true;
        }
    }
    false
}

/// Marks or unmarks a specific soft body particle as immobile (pinned).
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_set_particle_fixed(
    scene: *mut OxideScene,
    node_id: u64,
    particle_idx: u32,
    fixed: bool,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get_mut(entity) {
            sb.set_particle_fixed(particle_idx as usize, fixed);
            return true;
        }
    }
    false
}

/// Retrieves the total number of particles in a soft body component.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_get_particle_count(
    scene: *mut OxideScene,
    node_id: u64,
    out_count: *mut u32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get(entity) {
            if !out_count.is_null() {
                *out_count = sb.particles.len() as u32;
            }
            return true;
        }
    }
    false
}

/// Retrieves the current world position of a specific soft body particle.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_get_particle_position(
    scene: *mut OxideScene,
    node_id: u64,
    particle_idx: u32,
    out_pos: *mut OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get(entity) {
            if let Some(p) = sb.particles.get(particle_idx as usize) {
                if !out_pos.is_null() {
                    *out_pos = p.position.into();
                }
                return true;
            }
        }
    }
    false
}

/// Sets the world position of a specific soft body particle.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_set_particle_position(
    scene: *mut OxideScene,
    node_id: u64,
    particle_idx: u32,
    pos: OxideVec3,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get_mut(entity) {
            if let Some(p) = sb.particles.get_mut(particle_idx as usize) {
                p.position = pos.into();
                p.old_position = p.position;
                p.predicted_position = p.position;
                return true;
            }
        }
    }
    false
}

/// Adds a distance constraint connecting two particles in a soft body.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_add_constraint(
    scene: *mut OxideScene,
    node_id: u64,
    particle_a: u32,
    particle_b: u32,
    rest_length: f32,
    stiffness: f32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get_mut(entity) {
            sb.add_constraint(DistanceConstraint::new(
                particle_a as usize,
                particle_b as usize,
                rest_length,
                stiffness,
            ));
            return true;
        }
    }
    false
}

/// Sets the XPBD compliance parameter $\alpha$ (elasticity) for a soft body.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_set_compliance(
    scene: *mut OxideScene,
    node_id: u64,
    compliance: f32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get_mut(entity) {
            sb.compliance = compliance.max(0.0);
            return true;
        }
    }
    false
}

/// Configures the number of PBD solver sub-steps per physics update for a soft body.
#[no_mangle]
pub unsafe extern "C" fn oxide_scene_soft_body_set_substeps(
    scene: *mut OxideScene,
    node_id: u64,
    substeps: u32,
) -> bool {
    if let Some(s) = scene.as_mut() {
        let entity = Entity::from_ffi(node_id);
        if let Some(sb) = s.world.soft_body.get_mut(entity) {
            sb.substeps = substeps.max(1);
            return true;
        }
    }
    false
}


