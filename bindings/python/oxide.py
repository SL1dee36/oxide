import os
import sys
import math
import ctypes
from ctypes import (
    c_float,
    c_uint32,
    c_uint64,
    c_char_p,
    c_void_p,
    c_int32,
    c_bool,
    Structure,
    POINTER,
    CFUNCTYPE,
    byref,
)

def _find_library():
    names = ["oxide.dll", "liboxide.so", "liboxide.dylib"]
    search_dirs = [
        os.path.abspath(os.path.dirname(__file__)),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/release")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/debug")),
        os.path.abspath("target/release"),
        os.path.abspath("target/debug"),
    ]
    for d in search_dirs:
        for n in names:
            p = os.path.join(d, n)
            if os.path.exists(p):
                return p
    raise FileNotFoundError("Could not locate oxide dynamic library (oxide.dll / liboxide.so)")

_lib_path = _find_library()
_dll = ctypes.CDLL(_lib_path)

# ==========================================
# C ABI Structs
# ==========================================

class OxideVec2(Structure):
    _fields_ = [("x", c_float), ("y", c_float)]

class OxideVec3(Structure):
    _fields_ = [("x", c_float), ("y", c_float), ("z", c_float)]

class OxideQuat(Structure):
    _fields_ = [("x", c_float), ("y", c_float), ("z", c_float), ("w", c_float)]

class OxideColor(Structure):
    _fields_ = [("r", c_float), ("g", c_float), ("b", c_float), ("a", c_float)]

class OxideTransform(Structure):
    _fields_ = [
        ("translation", OxideVec3),
        ("rotation", OxideQuat),
        ("scale", OxideVec3),
    ]

class OxideContact(Structure):
    _fields_ = [
        ("normal", OxideVec3),
        ("depth", c_float),
        ("point", OxideVec3),
    ]

class OxideRaycastHit(Structure):
    _fields_ = [
        ("node_id", c_uint64),
        ("distance", c_float),
        ("point", OxideVec3),
        ("normal", OxideVec3),
    ]

# Callback types
OxideInitCallback = CFUNCTYPE(None, c_void_p, c_void_p)
OxideUpdateCallback = CFUNCTYPE(None, c_void_p, c_float, c_void_p)
OxideRenderCallback = CFUNCTYPE(None, c_void_p, c_void_p)

class OxideCallbacks(Structure):
    _fields_ = [
        ("on_init", OxideInitCallback),
        ("on_update", OxideUpdateCallback),
        ("on_render", OxideRenderCallback),
        ("user_data", c_void_p),
    ]

# ==========================================
# Bind C Functions
# ==========================================

# App
_dll.oxide_app_create.argtypes = [c_char_p, c_uint32, c_uint32]
_dll.oxide_app_create.restype = c_void_p

_dll.oxide_app_destroy.argtypes = [c_void_p]
_dll.oxide_app_destroy.restype = None

_dll.oxide_app_run.argtypes = [c_void_p, OxideCallbacks]
_dll.oxide_app_run.restype = c_int32

# Context
_dll.oxide_context_clear.argtypes = [c_void_p, OxideColor]
_dll.oxide_context_clear.restype = None

_dll.oxide_context_get_fps.argtypes = [c_void_p]
_dll.oxide_context_get_fps.restype = c_float

_dll.oxide_context_get_delta_time.argtypes = [c_void_p]
_dll.oxide_context_get_delta_time.restype = c_float

_dll.oxide_context_get_aspect_ratio.argtypes = [c_void_p]
_dll.oxide_context_get_aspect_ratio.restype = c_float

_dll.oxide_context_get_width.argtypes = [c_void_p]
_dll.oxide_context_get_width.restype = c_uint32

_dll.oxide_context_get_height.argtypes = [c_void_p]
_dll.oxide_context_get_height.restype = c_uint32

_dll.oxide_scene_despawn_node.argtypes = [c_void_p, c_uint64]
_dll.oxide_scene_despawn_node.restype = c_bool

_dll.oxide_context_is_key_down.argtypes = [c_void_p, c_char_p]
_dll.oxide_context_is_key_down.restype = c_bool

_dll.oxide_context_is_key_just_pressed.argtypes = [c_void_p, c_char_p]
_dll.oxide_context_is_key_just_pressed.restype = c_bool

_dll.oxide_context_is_key_just_released.argtypes = [c_void_p, c_char_p]
_dll.oxide_context_is_key_just_released.restype = c_bool

_dll.oxide_context_is_mouse_button_down.argtypes = [c_void_p, c_uint32]
_dll.oxide_context_is_mouse_button_down.restype = c_bool

_dll.oxide_context_is_mouse_button_just_pressed.argtypes = [c_void_p, c_uint32]
_dll.oxide_context_is_mouse_button_just_pressed.restype = c_bool

_dll.oxide_context_is_mouse_button_just_released.argtypes = [c_void_p, c_uint32]
_dll.oxide_context_is_mouse_button_just_released.restype = c_bool

_dll.oxide_context_get_mouse_position.argtypes = [c_void_p, POINTER(OxideVec2)]
_dll.oxide_context_get_mouse_position.restype = None

_dll.oxide_context_get_mouse_delta.argtypes = [c_void_p, POINTER(OxideVec2)]
_dll.oxide_context_get_mouse_delta.restype = None

_dll.oxide_context_get_mouse_wheel.argtypes = [c_void_p, POINTER(OxideVec2)]
_dll.oxide_context_get_mouse_wheel.restype = None

_dll.oxide_context_set_cursor_visible.argtypes = [c_void_p, c_bool]
_dll.oxide_context_set_cursor_visible.restype = None

_dll.oxide_context_set_cursor_grab.argtypes = [c_void_p, c_bool]
_dll.oxide_context_set_cursor_grab.restype = c_bool

# Scene
_dll.oxide_scene_create.argtypes = []
_dll.oxide_scene_create.restype = c_void_p

_dll.oxide_scene_destroy.argtypes = [c_void_p]
_dll.oxide_scene_destroy.restype = None

_dll.oxide_scene_set_camera_perspective.argtypes = [
    c_void_p, c_float, c_float, c_float, c_float, OxideVec3, OxideVec3
]
_dll.oxide_scene_set_camera_perspective.restype = None

_dll.oxide_scene_set_camera_look_at.argtypes = [
    c_void_p, OxideVec3, OxideVec3, OxideVec3
]
_dll.oxide_scene_set_camera_look_at.restype = None

_dll.oxide_scene_set_light.argtypes = [
    c_void_p, OxideVec3, OxideVec3, OxideVec3
]
_dll.oxide_scene_set_light.restype = None

_dll.oxide_scene_spawn_cube_3d.argtypes = [c_void_p, c_float, OxideTransform]
_dll.oxide_scene_spawn_cube_3d.restype = c_uint64

_dll.oxide_scene_spawn_cube_3d_colored.argtypes = [c_void_p, c_float, OxideColor, OxideTransform]
_dll.oxide_scene_spawn_cube_3d_colored.restype = c_uint64

_dll.oxide_scene_spawn_sphere_3d.argtypes = [c_void_p, c_float, OxideTransform]
_dll.oxide_scene_spawn_sphere_3d.restype = c_uint64

_dll.oxide_scene_spawn_sphere_3d_colored.argtypes = [c_void_p, c_float, OxideColor, OxideTransform]
_dll.oxide_scene_spawn_sphere_3d_colored.restype = c_uint64

_dll.oxide_scene_spawn_box_3d.argtypes = [
    c_void_p, c_float, c_float, c_float, OxideColor, OxideTransform, c_bool
]
_dll.oxide_scene_spawn_box_3d.restype = c_uint64

_dll.oxide_scene_spawn_surface_2d.argtypes = [c_void_p, OxideVec2, OxideColor, OxideTransform]
_dll.oxide_scene_spawn_surface_2d.restype = c_uint64

_dll.oxide_scene_spawn_text_2d.argtypes = [c_void_p, c_char_p, OxideColor, c_float, OxideTransform]
_dll.oxide_scene_spawn_text_2d.restype = c_uint64

_dll.oxide_scene_node_set_text.argtypes = [c_void_p, c_uint64, c_char_p]
_dll.oxide_scene_node_set_text.restype = None

_dll.oxide_scene_node_get_transform.argtypes = [c_void_p, c_uint64, POINTER(OxideTransform)]
_dll.oxide_scene_node_get_transform.restype = c_bool

_dll.oxide_scene_node_set_transform.argtypes = [c_void_p, c_uint64, OxideTransform]
_dll.oxide_scene_node_set_transform.restype = c_bool

_dll.oxide_scene_node_get_translation.argtypes = [c_void_p, c_uint64, POINTER(OxideVec3)]
_dll.oxide_scene_node_get_translation.restype = c_bool

_dll.oxide_scene_node_set_translation.argtypes = [c_void_p, c_uint64, OxideVec3]
_dll.oxide_scene_node_set_translation.restype = c_bool

_dll.oxide_scene_node_get_rotation.argtypes = [c_void_p, c_uint64, POINTER(OxideQuat)]
_dll.oxide_scene_node_get_rotation.restype = c_bool

_dll.oxide_scene_node_set_rotation.argtypes = [c_void_p, c_uint64, OxideQuat]
_dll.oxide_scene_node_set_rotation.restype = c_bool

_dll.oxide_scene_node_rotate_y.argtypes = [c_void_p, c_uint64, c_float]
_dll.oxide_scene_node_rotate_y.restype = None

_dll.oxide_scene_node_rotate_x.argtypes = [c_void_p, c_uint64, c_float]
_dll.oxide_scene_node_rotate_x.restype = None

_dll.oxide_scene_node_rotate_z.argtypes = [c_void_p, c_uint64, c_float]
_dll.oxide_scene_node_rotate_z.restype = None

_dll.oxide_scene_node_set_scale.argtypes = [c_void_p, c_uint64, OxideVec3]
_dll.oxide_scene_node_set_scale.restype = c_bool

_dll.oxide_scene_node_set_color.argtypes = [c_void_p, c_uint64, OxideColor]
_dll.oxide_scene_node_set_color.restype = c_bool

_dll.oxide_scene_node_set_visible.argtypes = [c_void_p, c_uint64, c_bool]
_dll.oxide_scene_node_set_visible.restype = c_bool

_dll.oxide_scene_node_set_static.argtypes = [c_void_p, c_uint64, c_bool]
_dll.oxide_scene_node_set_static.restype = c_bool

_dll.oxide_scene_check_collision.argtypes = [c_void_p, c_uint64, c_uint64, POINTER(OxideContact)]
_dll.oxide_scene_check_collision.restype = c_bool

_dll.oxide_scene_resolve_penetration.argtypes = [c_void_p, c_uint64, c_uint64, POINTER(OxideContact)]
_dll.oxide_scene_resolve_penetration.restype = None

_dll.oxide_scene_raycast.argtypes = [c_void_p, OxideVec3, OxideVec3, POINTER(OxideRaycastHit)]
_dll.oxide_scene_raycast.restype = c_bool

_dll.oxide_scene_update.argtypes = [c_void_p, c_void_p, c_float]
_dll.oxide_scene_update.restype = None

_dll.oxide_scene_render.argtypes = [c_void_p, c_void_p]
_dll.oxide_scene_render.restype = None


# ==========================================
# Pythonic Object Wrappers
# ==========================================

class MouseButton:
    LEFT = 0
    RIGHT = 1
    MIDDLE = 2

class Vec2:
    def __init__(self, x=0.0, y=0.0):
        self.x = float(x)
        self.y = float(y)

    def _to_c(self):
        return OxideVec2(self.x, self.y)

    @classmethod
    def _from_c(cls, c_vec):
        return cls(c_vec.x, c_vec.y)

    def __repr__(self):
        return f"Vec2({self.x:.3f}, {self.y:.3f})"

    def __add__(self, other):
        return Vec2(self.x + other.x, self.y + other.y)

    def __sub__(self, other):
        return Vec2(self.x - other.x, self.y - other.y)

    def __mul__(self, scalar):
        return Vec2(self.x * scalar, self.y * scalar)

    def length(self):
        return math.hypot(self.x, self.y)


class Vec3:
    def __init__(self, x=0.0, y=0.0, z=0.0):
        self.x = float(x)
        self.y = float(y)
        self.z = float(z)

    def _to_c(self):
        return OxideVec3(self.x, self.y, self.z)

    @classmethod
    def _from_c(cls, c_vec):
        return cls(c_vec.x, c_vec.y, c_vec.z)

    def __repr__(self):
        return f"Vec3({self.x:.3f}, {self.y:.3f}, {self.z:.3f})"

    def __add__(self, other):
        return Vec3(self.x + other.x, self.y + other.y, self.z + other.z)

    def __sub__(self, other):
        return Vec3(self.x - other.x, self.y - other.y, self.z - other.z)

    def __neg__(self):
        return Vec3(-self.x, -self.y, -self.z)

    def __mul__(self, scalar):
        if isinstance(other := scalar, (int, float)):
            return Vec3(self.x * other, self.y * other, self.z * other)
        return Vec3(self.x * scalar.x, self.y * scalar.y, self.z * scalar.z)

    def __truediv__(self, scalar):
        inv = 1.0 / scalar
        return Vec3(self.x * inv, self.y * inv, self.z * inv)

    def dot(self, other):
        return self.x * other.x + self.y * other.y + self.z * other.z

    def cross(self, other):
        return Vec3(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )

    def length_squared(self):
        return self.x * self.x + self.y * self.y + self.z * self.z

    def length(self):
        return math.sqrt(self.length_squared())

    def normalize(self):
        l = self.length()
        if l > 1e-6:
            inv = 1.0 / l
            return Vec3(self.x * inv, self.y * inv, self.z * inv)
        return Vec3(0.0, 0.0, 0.0)

    def distance(self, other):
        return (self - other).length()


class Quat:
    def __init__(self, x=0.0, y=0.0, z=0.0, w=1.0):
        self.x = float(x)
        self.y = float(y)
        self.z = float(z)
        self.w = float(w)

    def _to_c(self):
        return OxideQuat(self.x, self.y, self.z, self.w)

    @classmethod
    def _from_c(cls, c_quat):
        return cls(c_quat.x, c_quat.y, c_quat.z, c_quat.w)

    @classmethod
    def from_axis_angle(cls, axis: Vec3, rad: float):
        half = rad * 0.5
        s = math.sin(half)
        a = axis.normalize()
        return cls(a.x * s, a.y * s, a.z * s, math.cos(half))

    def __repr__(self):
        return f"Quat({self.x:.3f}, {self.y:.3f}, {self.z:.3f}, {self.w:.3f})"

    def __mul__(self, other):
        # Quaternion multiplication (Hamilton product)
        w1, x1, y1, z1 = self.w, self.x, self.y, self.z
        w2, x2, y2, z2 = other.w, other.x, other.y, other.z
        return Quat(
            w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
            w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
            w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
            w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
        )


class Color:
    def __init__(self, r=1.0, g=1.0, b=1.0, a=1.0):
        self.r = float(r)
        self.g = float(g)
        self.b = float(b)
        self.a = float(a)

    def _to_c(self):
        return OxideColor(self.r, self.g, self.b, self.a)

    @classmethod
    def _from_c(cls, c_col):
        return cls(c_col.r, c_col.g, c_col.b, c_col.a)

    def __repr__(self):
        return f"Color({self.r:.2f}, {self.g:.2f}, {self.b:.2f}, {self.a:.2f})"


class Transform:
    def __init__(self, translation=None, rotation=None, scale=None):
        self.translation = translation or Vec3(0, 0, 0)
        self.rotation = rotation or Quat(0, 0, 0, 1)
        self.scale = scale or Vec3(1, 1, 1)

    @classmethod
    def from_xyz(cls, x, y, z):
        return cls(translation=Vec3(x, y, z))

    def _to_c(self):
        return OxideTransform(
            self.translation._to_c(),
            self.rotation._to_c(),
            self.scale._to_c(),
        )

    @classmethod
    def _from_c(cls, c_t):
        return cls(
            translation=Vec3._from_c(c_t.translation),
            rotation=Quat._from_c(c_t.rotation),
            scale=Vec3._from_c(c_t.scale),
        )

    def __repr__(self):
        return f"Transform(pos={self.translation}, rot={self.rotation}, scale={self.scale})"


class RaycastHit:
    def __init__(self, node_id, distance, point, normal):
        self.node_id = int(node_id)
        self.distance = float(distance)
        self.point = point
        self.normal = normal

    def __repr__(self):
        return f"RaycastHit(node={self.node_id}, dist={self.distance:.2f}, pt={self.point})"


class Context:
    def __init__(self, ptr):
        self._ptr = ptr

    def clear(self, color):
        _dll.oxide_context_clear(self._ptr, color._to_c())

    @property
    def fps(self):
        return _dll.oxide_context_get_fps(self._ptr)

    @property
    def delta_time(self):
        return _dll.oxide_context_get_delta_time(self._ptr)

    @property
    def aspect_ratio(self):
        return _dll.oxide_context_get_aspect_ratio(self._ptr)

    @property
    def width(self):
        return _dll.oxide_context_get_width(self._ptr)

    @property
    def height(self):
        return _dll.oxide_context_get_height(self._ptr)

    def is_key_down(self, key_name: str) -> bool:
        return _dll.oxide_context_is_key_down(self._ptr, key_name.encode("utf-8"))

    def is_key_just_pressed(self, key_name: str) -> bool:
        return _dll.oxide_context_is_key_just_pressed(self._ptr, key_name.encode("utf-8"))

    def is_key_just_released(self, key_name: str) -> bool:
        return _dll.oxide_context_is_key_just_released(self._ptr, key_name.encode("utf-8"))

    def is_mouse_button_down(self, button: int) -> bool:
        return _dll.oxide_context_is_mouse_button_down(self._ptr, button)

    def is_mouse_button_just_pressed(self, button: int) -> bool:
        return _dll.oxide_context_is_mouse_button_just_pressed(self._ptr, button)

    def is_mouse_button_just_released(self, button: int) -> bool:
        return _dll.oxide_context_is_mouse_button_just_released(self._ptr, button)

    @property
    def mouse_position(self) -> Vec2:
        out = OxideVec2()
        _dll.oxide_context_get_mouse_position(self._ptr, byref(out))
        return Vec2._from_c(out)

    @property
    def mouse_delta(self) -> Vec2:
        out = OxideVec2()
        _dll.oxide_context_get_mouse_delta(self._ptr, byref(out))
        return Vec2._from_c(out)

    @property
    def mouse_wheel(self) -> Vec2:
        out = OxideVec2()
        _dll.oxide_context_get_mouse_wheel(self._ptr, byref(out))
        return Vec2._from_c(out)

    def set_cursor_visible(self, visible: bool):
        _dll.oxide_context_set_cursor_visible(self._ptr, visible)

    def set_cursor_grab(self, grab: bool) -> bool:
        return _dll.oxide_context_set_cursor_grab(self._ptr, grab)


class Scene:
    def __init__(self):
        self._ptr = _dll.oxide_scene_create()

    def destroy(self):
        """Release underlying OxideScene allocated in Rust heap."""
        if hasattr(self, "_ptr") and self._ptr:
            _dll.oxide_scene_destroy(self._ptr)
            self._ptr = None

    def despawn_node(self, node_id: int) -> bool:
        """Removes and despawns a node and all of its recursive children from the scene."""
        return _dll.oxide_scene_despawn_node(self._ptr, node_id)

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()

    def __del__(self):
        self.destroy()

    def set_camera_perspective(self, fov_deg, aspect, near, far, eye: Vec3, target: Vec3):
        _dll.oxide_scene_set_camera_perspective(
            self._ptr, fov_deg, aspect, near, far, eye._to_c(), target._to_c()
        )

    def set_camera_look_at(self, eye: Vec3, target: Vec3, up: Vec3 = None):
        u = up or Vec3(0, 1, 0)
        _dll.oxide_scene_set_camera_look_at(self._ptr, eye._to_c(), target._to_c(), u._to_c())

    def set_light(self, direction: Vec3, color: Vec3 = None, ambient: Vec3 = None):
        col = color or Vec3(1.0, 0.98, 0.95)
        amb = ambient or Vec3(0.25, 0.25, 0.28)
        _dll.oxide_scene_set_light(self._ptr, direction._to_c(), col._to_c(), amb._to_c())

    def spawn_cube_3d(self, size, transform=None, color=None):
        t = transform or Transform()
        if color is not None:
            return _dll.oxide_scene_spawn_cube_3d_colored(self._ptr, float(size), color._to_c(), t._to_c())
        return _dll.oxide_scene_spawn_cube_3d(self._ptr, float(size), t._to_c())

    def spawn_sphere_3d(self, radius, transform=None, color=None):
        t = transform or Transform()
        if color is not None:
            return _dll.oxide_scene_spawn_sphere_3d_colored(self._ptr, float(radius), color._to_c(), t._to_c())
        return _dll.oxide_scene_spawn_sphere_3d(self._ptr, float(radius), t._to_c())

    def spawn_box_3d(self, width, height, depth, transform=None, color=None, is_static=False):
        t = transform or Transform()
        c = color or Color(1.0, 1.0, 1.0, 1.0)
        return _dll.oxide_scene_spawn_box_3d(
            self._ptr, float(width), float(height), float(depth), c._to_c(), t._to_c(), bool(is_static)
        )

    def spawn_surface_2d(self, size, color=None, transform=None):
        c = color or Color()
        t = transform or Transform()
        return _dll.oxide_scene_spawn_surface_2d(self._ptr, size._to_c(), c._to_c(), t._to_c())

    def spawn_text_2d(self, text, color=None, scale=1.0, transform=None):
        c = color or Color()
        t = transform or Transform()
        return _dll.oxide_scene_spawn_text_2d(
            self._ptr, text.encode("utf-8"), c._to_c(), float(scale), t._to_c()
        )

    def node_set_text(self, node_id, text):
        if node_id is not None:
            _dll.oxide_scene_node_set_text(self._ptr, int(node_id), str(text).encode("utf-8"))

    def node_get_transform(self, node_id) -> Transform:
        out = OxideTransform()
        if _dll.oxide_scene_node_get_transform(self._ptr, node_id, byref(out)):
            return Transform._from_c(out)
        return Transform()

    def node_set_transform(self, node_id, transform: Transform):
        return _dll.oxide_scene_node_set_transform(self._ptr, node_id, transform._to_c())

    def node_get_translation(self, node_id) -> Vec3:
        out = OxideVec3()
        if _dll.oxide_scene_node_get_translation(self._ptr, node_id, byref(out)):
            return Vec3._from_c(out)
        return Vec3()

    def node_set_translation(self, node_id, translation: Vec3):
        return _dll.oxide_scene_node_set_translation(self._ptr, node_id, translation._to_c())

    def node_get_rotation(self, node_id) -> Quat:
        out = OxideQuat()
        if _dll.oxide_scene_node_get_rotation(self._ptr, node_id, byref(out)):
            return Quat._from_c(out)
        return Quat()

    def node_set_rotation(self, node_id, rotation: Quat):
        return _dll.oxide_scene_node_set_rotation(self._ptr, node_id, rotation._to_c())

    def node_rotate_y(self, node_id, rad):
        _dll.oxide_scene_node_rotate_y(self._ptr, node_id, float(rad))

    def node_rotate_x(self, node_id, rad):
        _dll.oxide_scene_node_rotate_x(self._ptr, node_id, float(rad))

    def node_rotate_z(self, node_id, rad):
        _dll.oxide_scene_node_rotate_z(self._ptr, node_id, float(rad))

    def node_set_scale(self, node_id, scale: Vec3):
        return _dll.oxide_scene_node_set_scale(self._ptr, node_id, scale._to_c())

    def node_set_color(self, node_id, color: Color):
        return _dll.oxide_scene_node_set_color(self._ptr, node_id, color._to_c())

    def node_set_visible(self, node_id, visible: bool):
        return _dll.oxide_scene_node_set_visible(self._ptr, node_id, bool(visible))

    def node_set_static(self, node_id, is_static: bool):
        return _dll.oxide_scene_node_set_static(self._ptr, node_id, bool(is_static))

    def check_collision(self, node_a, node_b):
        contact = OxideContact()
        hit = _dll.oxide_scene_check_collision(self._ptr, node_a, node_b, byref(contact))
        if hit:
            return contact
        return None

    def resolve_penetration(self, node_a, node_b, contact):
        _dll.oxide_scene_resolve_penetration(self._ptr, node_a, node_b, byref(contact))

    def raycast(self, origin: Vec3, direction: Vec3):
        out = OxideRaycastHit()
        hit = _dll.oxide_scene_raycast(self._ptr, origin._to_c(), direction._to_c(), byref(out))
        if hit:
            return RaycastHit(
                node_id=out.node_id,
                distance=out.distance,
                point=Vec3._from_c(out.point),
                normal=Vec3._from_c(out.normal),
            )
        return None

    def update(self, ctx, dt):
        _dll.oxide_scene_update(self._ptr, ctx._ptr, float(dt))

    def render(self, ctx):
        _dll.oxide_scene_render(self._ptr, ctx._ptr)


class App:
    def __init__(self, title="Oxide Python App", width=1280, height=720):
        self.title = title
        self.width = width
        self.height = height
        self._ptr = None

    def destroy(self):
        """Release underlying OxideApp if not already consumed by run()."""
        if self._ptr:
            _dll.oxide_app_destroy(self._ptr)
            self._ptr = None

    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        self.destroy()

    def __del__(self):
        self.destroy()

    def run(self, handler):
        """
        Initialize and run the OS event/frame loop.
        Ownership of the underlying OxideApp pointer is transferred to oxide_app_run.
        """
        app_ptr = _dll.oxide_app_create(self.title.encode("utf-8"), self.width, self.height)
        self._ptr = app_ptr

        try:
            def _init(ctx_ptr, user_data):
                if hasattr(handler, "on_init"):
                    handler.on_init(Context(ctx_ptr))

            def _update(ctx_ptr, dt, user_data):
                if hasattr(handler, "on_update"):
                    handler.on_update(Context(ctx_ptr), dt)

            def _render(ctx_ptr, user_data):
                if hasattr(handler, "on_render"):
                    handler.on_render(Context(ctx_ptr))

            c_init = OxideInitCallback(_init)
            c_update = OxideUpdateCallback(_update)
            c_render = OxideRenderCallback(_render)

            callbacks = OxideCallbacks(c_init, c_update, c_render, None)

            # oxide_app_run consumes and deallocates app_ptr internally
            self._ptr = None
            return _dll.oxide_app_run(app_ptr, callbacks)
        except Exception:
            if self._ptr:
                _dll.oxide_app_destroy(self._ptr)
                self._ptr = None
            raise
