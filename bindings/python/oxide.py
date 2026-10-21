import os
import sys
import ctypes
from ctypes import c_float, c_uint32, c_uint64, c_char_p, c_void_p, c_int32, c_bool, Structure, POINTER, CFUNCTYPE

def _find_library():
    names = ["oxide.dll", "liboxide.so", "liboxide.dylib"]
    search_dirs = [
        os.path.abspath(os.path.dirname(__file__)),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/debug")),
        os.path.abspath(os.path.join(os.path.dirname(__file__), "../../target/release")),
        os.path.abspath("target/debug"),
        os.path.abspath("target/release"),
    ]
    for d in search_dirs:
        for n in names:
            p = os.path.join(d, n)
            if os.path.exists(p):
                return p
    raise FileNotFoundError("Could not locate oxide dynamic library (oxide.dll / liboxide.so)")

_lib_path = _find_library()
_dll = ctypes.CDLL(_lib_path)

# C Structs
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

# Bind C Functions
_dll.oxide_app_create.argtypes = [c_char_p, c_uint32, c_uint32]
_dll.oxide_app_create.restype = c_void_p

_dll.oxide_app_destroy.argtypes = [c_void_p]
_dll.oxide_app_destroy.restype = None

_dll.oxide_app_run.argtypes = [c_void_p, OxideCallbacks]
_dll.oxide_app_run.restype = c_int32

_dll.oxide_context_clear.argtypes = [c_void_p, OxideColor]
_dll.oxide_context_clear.restype = None

_dll.oxide_context_get_fps.argtypes = [c_void_p]
_dll.oxide_context_get_fps.restype = c_float

_dll.oxide_context_get_delta_time.argtypes = [c_void_p]
_dll.oxide_context_get_delta_time.restype = c_float

_dll.oxide_context_get_aspect_ratio.argtypes = [c_void_p]
_dll.oxide_context_get_aspect_ratio.restype = c_float

_dll.oxide_scene_create.argtypes = []
_dll.oxide_scene_create.restype = c_void_p

_dll.oxide_scene_destroy.argtypes = [c_void_p]
_dll.oxide_scene_destroy.restype = None

_dll.oxide_scene_set_camera_perspective.argtypes = [
    c_void_p, c_float, c_float, c_float, c_float, OxideVec3, OxideVec3
]
_dll.oxide_scene_set_camera_perspective.restype = None

_dll.oxide_scene_spawn_cube_3d.argtypes = [c_void_p, c_float, OxideTransform]
_dll.oxide_scene_spawn_cube_3d.restype = c_uint64

_dll.oxide_scene_spawn_sphere_3d.argtypes = [c_void_p, c_float, OxideTransform]
_dll.oxide_scene_spawn_sphere_3d.restype = c_uint64

_dll.oxide_scene_spawn_surface_2d.argtypes = [c_void_p, OxideVec2, OxideColor, OxideTransform]
_dll.oxide_scene_spawn_surface_2d.restype = c_uint64

_dll.oxide_scene_spawn_text_2d.argtypes = [c_void_p, c_char_p, OxideColor, c_float, OxideTransform]
_dll.oxide_scene_spawn_text_2d.restype = c_uint64

_dll.oxide_scene_node_set_text.argtypes = [c_void_p, c_uint64, c_char_p]
_dll.oxide_scene_node_set_text.restype = None

_dll.oxide_scene_node_rotate_y.argtypes = [c_void_p, c_uint64, c_float]
_dll.oxide_scene_node_rotate_y.restype = None

_dll.oxide_scene_node_rotate_x.argtypes = [c_void_p, c_uint64, c_float]
_dll.oxide_scene_node_rotate_x.restype = None

_dll.oxide_scene_check_collision.argtypes = [c_void_p, c_uint64, c_uint64, POINTER(OxideContact)]
_dll.oxide_scene_check_collision.restype = c_bool

_dll.oxide_scene_resolve_penetration.argtypes = [c_void_p, c_uint64, c_uint64, POINTER(OxideContact)]
_dll.oxide_scene_resolve_penetration.restype = None

_dll.oxide_scene_update.argtypes = [c_void_p, c_void_p, c_float]
_dll.oxide_scene_update.restype = None

_dll.oxide_scene_render.argtypes = [c_void_p, c_void_p]
_dll.oxide_scene_render.restype = None


# Pythonic Wrappers
class Vec2:
    def __init__(self, x=0.0, y=0.0):
        self.x = float(x)
        self.y = float(y)

    def _to_c(self):
        return OxideVec2(self.x, self.y)

class Vec3:
    def __init__(self, x=0.0, y=0.0, z=0.0):
        self.x = float(x)
        self.y = float(y)
        self.z = float(z)

    def _to_c(self):
        return OxideVec3(self.x, self.y, self.z)

class Quat:
    def __init__(self, x=0.0, y=0.0, z=0.0, w=1.0):
        self.x = float(x)
        self.y = float(y)
        self.z = float(z)
        self.w = float(w)

    def _to_c(self):
        return OxideQuat(self.x, self.y, self.z, self.w)

class Color:
    def __init__(self, r=1.0, g=1.0, b=1.0, a=1.0):
        self.r = float(r)
        self.g = float(g)
        self.b = float(b)
        self.a = float(a)

    def _to_c(self):
        return OxideColor(self.r, self.g, self.b, self.a)

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

class Scene:
    def __init__(self):
        self._ptr = _dll.oxide_scene_create()

    def __del__(self):
        if hasattr(self, "_ptr") and self._ptr:
            _dll.oxide_scene_destroy(self._ptr)
            self._ptr = None

    def set_camera_perspective(self, fov_deg, aspect, near, far, eye, target):
        _dll.oxide_scene_set_camera_perspective(
            self._ptr, fov_deg, aspect, near, far, eye._to_c(), target._to_c()
        )

    def spawn_cube_3d(self, size, transform=None):
        t = transform or Transform()
        return _dll.oxide_scene_spawn_cube_3d(self._ptr, float(size), t._to_c())

    def spawn_sphere_3d(self, radius, transform=None):
        t = transform or Transform()
        return _dll.oxide_scene_spawn_sphere_3d(self._ptr, float(radius), t._to_c())

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
        _dll.oxide_scene_node_set_text(self._ptr, node_id, text.encode("utf-8"))

    def node_rotate_y(self, node_id, rad):
        _dll.oxide_scene_node_rotate_y(self._ptr, node_id, float(rad))

    def node_rotate_x(self, node_id, rad):
        _dll.oxide_scene_node_rotate_x(self._ptr, node_id, float(rad))

    def check_collision(self, node_a, node_b):
        contact = OxideContact()
        hit = _dll.oxide_scene_check_collision(self._ptr, node_a, node_b, ctypes.byref(contact))
        if hit:
            return contact
        return None

    def resolve_penetration(self, node_a, node_b, contact):
        _dll.oxide_scene_resolve_penetration(self._ptr, node_a, node_b, ctypes.byref(contact))

    def update(self, ctx, dt):
        _dll.oxide_scene_update(self._ptr, ctx._ptr, float(dt))

    def render(self, ctx):
        _dll.oxide_scene_render(self._ptr, ctx._ptr)

class App:
    def __init__(self, title="Oxide Python App", width=1280, height=720):
        self.title = title
        self.width = width
        self.height = height

    def run(self, handler):
        app_ptr = _dll.oxide_app_create(self.title.encode("utf-8"), self.width, self.height)

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
        return _dll.oxide_app_run(app_ptr, callbacks)
