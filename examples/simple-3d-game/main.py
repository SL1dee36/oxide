#!/usr/bin/env python3
"""
Simple Interactive 3D simulation in Oxide

Features:
- Cursor recentering and capture (Press [Esc] to toggle/release mouse).
- Pick up Cube or Sphere by pressing [E] when within reach.
- Drop/put down held object gently by pressing [E].
- Throw held object forward with force by clicking Left Mouse Button [LMB].
- Inspect/rotate held object by holding Right Mouse Button [RMB] and dragging mouse.
  (While RMB is held, camera pitch/yaw remains stationary, and the object rotates).
- Realistic bounce, friction, rolling physics, and mutual collision resolution.
- Dynamic crosshair and interactive HUD status overlay.
"""

import os
import sys
import math
import random
import ctypes

# Ensure oxide runtime binding is importable
BINDINGS_PATH = os.path.abspath(os.path.join(os.path.dirname(__file__), "../../bindings/python"))
if BINDINGS_PATH not in sys.path:
    sys.path.insert(0, BINDINGS_PATH)

from oxide import (
    App,
    Scene,
    Context,
    Vec2,
    Vec3,
    Quat,
    Color,
    Transform,
    MouseButton,
)


class WindowsMouseController:
    """
    Provides infinite 360-degree mouse look on Windows by locking and recentering
    the cursor to the window center every frame.
    """
    def __init__(self, window_title):
        self.window_title = window_title
        self.user32 = getattr(ctypes, "windll", None) and getattr(ctypes.windll, "user32", None)
        self.hwnd = None
        self.grabbed = False
        self.just_grabbed = True

    def find_window(self):
        if not self.user32:
            return None
        hwnd = self.user32.FindWindowW(None, self.window_title)
        if not hwnd:
            hwnd = self.user32.GetForegroundWindow()
        self.hwnd = hwnd
        return hwnd

    def get_window_center(self):
        if not self.user32:
            return None, None
        if not self.hwnd:
            self.find_window()
        if not self.hwnd:
            return None, None

        class RECT(ctypes.Structure):
            _fields_ = [
                ("left", ctypes.c_long),
                ("top", ctypes.c_long),
                ("right", ctypes.c_long),
                ("bottom", ctypes.c_long),
            ]

        class POINT(ctypes.Structure):
            _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]

        rect = RECT()
        if not self.user32.GetClientRect(self.hwnd, ctypes.byref(rect)):
            return None, None

        w = rect.right - rect.left
        h = rect.bottom - rect.top
        if w <= 0 or h <= 0:
            return None, None

        pt = POINT(w // 2, h // 2)
        self.user32.ClientToScreen(self.hwnd, ctypes.byref(pt))
        return pt.x, pt.y

    def set_grabbed(self, grabbed: bool, ctx=None):
        self.grabbed = grabbed
        self.just_grabbed = True
        if ctx:
            ctx.set_cursor_visible(not grabbed)
            ctx.set_cursor_grab(grabbed)
        if grabbed and self.user32:
            cx, cy = self.get_window_center()
            if cx is not None:
                self.user32.SetCursorPos(cx, cy)

    def get_delta(self, fallback_delta: Vec2):
        if not self.grabbed or not self.user32:
            return Vec2(0.0, 0.0)

        # Ensure the game window is currently in focus
        foreground = self.user32.GetForegroundWindow()
        if self.hwnd and foreground != self.hwnd:
            return Vec2(0.0, 0.0)

        cx, cy = self.get_window_center()
        if cx is None:
            return fallback_delta

        class POINT(ctypes.Structure):
            _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]

        cur = POINT()
        if not self.user32.GetCursorPos(ctypes.byref(cur)):
            return fallback_delta

        if self.just_grabbed:
            self.just_grabbed = False
            self.user32.SetCursorPos(cx, cy)
            return Vec2(0.0, 0.0)

        dx = float(cur.x - cx)
        dy = float(cur.y - cy)

        # Re-center cursor to the client center every frame
        if dx != 0 or dy != 0:
            self.user32.SetCursorPos(cx, cy)

        return Vec2(dx, dy)


class PhysicsObject:
    """Manages physical state (velocity, gravity, bounce, rotation) for a 3D scene node."""

    def __init__(self, node_id, name, kind, size_or_radius, color, initial_pos):
        self.node_id = int(node_id)
        self.name = name
        self.kind = kind  # "cube" or "sphere"
        self.size_or_radius = float(size_or_radius)
        self.color = color
        self.position = initial_pos
        self.velocity = Vec3(0.0, 0.0, 0.0)
        self.angular_velocity = Vec3(0.0, 0.0, 0.0)
        self.is_held = False
        self.rotation = Quat(0.0, 0.0, 0.0, 1.0)

    @property
    def bottom_offset(self):
        return self.size_or_radius if self.kind == "sphere" else (self.size_or_radius * 0.5)

    def update_physics(self, dt, floor_y=0.0):
        if self.is_held:
            return

        # Gravity
        gravity = Vec3(0.0, -19.6, 0.0)
        self.velocity = self.velocity + gravity * dt
        self.position = self.position + self.velocity * dt

        # Floor collision
        bottom = self.position.y - self.bottom_offset
        if bottom < floor_y:
            self.position.y = floor_y + self.bottom_offset
            restitution = 0.68 if self.kind == "sphere" else 0.40
            friction = 0.94 if self.kind == "sphere" else 0.86

            self.velocity.y = -self.velocity.y * restitution
            self.velocity.x *= friction
            self.velocity.z *= friction
            self.angular_velocity = self.angular_velocity * 0.85

            if abs(self.velocity.y) < 0.25:
                self.velocity.y = 0.0

            # Rolling rotation for sphere on ground
            if self.kind == "sphere":
                h_speed = math.hypot(self.velocity.x, self.velocity.z)
                if h_speed > 0.05:
                    roll_rate = h_speed / self.size_or_radius
                    roll_axis = Vec3(-self.velocity.z, 0.0, self.velocity.x).normalize()
                    delta_q = Quat.from_axis_angle(roll_axis, roll_rate * dt)
                    self.rotation = delta_q * self.rotation

        # Platform bounds rebound (keep objects inside play area [-28, 28])
        bound = 27.5
        for axis in ["x", "z"]:
            val = getattr(self.position, axis)
            if abs(val) > bound:
                sign = 1.0 if val > 0 else -1.0
                setattr(self.position, axis, sign * bound)
                vel_val = getattr(self.velocity, axis)
                setattr(self.velocity, axis, -vel_val * 0.6)

        # Free flight spin damping
        if abs(self.angular_velocity.length_squared()) > 1e-4:
            spin_mag = self.angular_velocity.length()
            spin_axis = self.angular_velocity.normalize()
            delta_q = Quat.from_axis_angle(spin_axis, spin_mag * dt)
            self.rotation = delta_q * self.rotation
            self.angular_velocity = self.angular_velocity * (1.0 - 0.4 * dt)


class Simple3DGame:
    def __init__(self, window_title="Oxide - 3D Interactive World (Cube & Sphere)"):
        self.scene = Scene()
        self.window_title = window_title
        self.screen_width = 1280
        self.screen_height = 720

        # Mouse recentering controller for infinite 360-degree rotation
        self.mouse_ctrl = WindowsMouseController(self.window_title)

        # Camera & Player state
        self.player_pos = Vec3(0.0, 1.7, 3.5)
        self.player_vel_y = 0.0
        self.is_grounded = True
        self.yaw = 0.0    # Radians (0 looking towards -Z)
        self.pitch = 0.0  # Radians

        # Objects
        self.cube = None
        self.sphere = None
        self.floor_node = None
        self.boundary_nodes = []

        # Interaction state
        self.held_object = None  # None, self.cube, or self.sphere
        self.aimed_object = None

        # UI Overlay nodes
        self.crosshair_node = None
        self.title_node = None
        self.status_node = None
        self.instructions_node = None
        self.target_hint_node = None

    def on_init(self, ctx: Context):
        # 1. Grab and center mouse cursor for infinite 360 FPS view
        self.mouse_ctrl.set_grabbed(True, ctx)

        # Configure Viewport & 3D Perspective Camera
        self.update_camera(ctx)

        # Directional sunlight + ambient illumination
        sun_direction = Vec3(0.5, 1.0, 0.7).normalize()
        self.scene.set_light(
            direction=sun_direction,
            color=Vec3(1.0, 0.98, 0.92),
            ambient=Vec3(0.32, 0.35, 0.42),
        )

        # 2. Flat World Platform (56x56 units, static obstacle)
        floor_w, floor_h, floor_d = 56.0, 0.6, 56.0
        self.floor_node = self.scene.spawn_box_3d(
            floor_w,
            floor_h,
            floor_d,
            transform=Transform.from_xyz(0.0, -floor_h * 0.5, 0.0),
            color=Color(0.20, 0.24, 0.30, 1.0),
            is_static=True,
        )

        # Perimeter curbs
        curb_color = Color(0.35, 0.40, 0.50, 1.0)
        ch = 0.8
        self.scene.spawn_box_3d(floor_w, ch, 0.8, transform=Transform.from_xyz(0.0, ch * 0.5, -28.0), color=curb_color, is_static=True)
        self.scene.spawn_box_3d(floor_w, ch, 0.8, transform=Transform.from_xyz(0.0, ch * 0.5, 28.0), color=curb_color, is_static=True)
        self.scene.spawn_box_3d(0.8, ch, floor_d, transform=Transform.from_xyz(-28.0, ch * 0.5, 0.0), color=curb_color, is_static=True)
        self.scene.spawn_box_3d(0.8, ch, floor_d, transform=Transform.from_xyz(28.0, ch * 0.5, 0.0), color=curb_color, is_static=True)

        # 3. Interactive Objects: Cube and Sphere
        # Cube: size 1.0, warm amber/coral
        cube_size = 1.0
        cube_pos = Vec3(-2.2, cube_size * 0.5, -3.0)
        cube_color = Color(1.0, 0.46, 0.18, 1.0)
        c_node = self.scene.spawn_cube_3d(
            cube_size,
            transform=Transform.from_xyz(cube_pos.x, cube_pos.y, cube_pos.z),
            color=cube_color,
        )
        self.cube = PhysicsObject(c_node, "Cube", "cube", cube_size, cube_color, cube_pos)

        # Sphere: radius 0.6, radiant neon emerald
        sphere_radius = 0.6
        sphere_pos = Vec3(2.2, sphere_radius, -3.0)
        sphere_color = Color(0.18, 0.88, 0.62, 1.0)
        s_node = self.scene.spawn_sphere_3d(
            sphere_radius,
            transform=Transform.from_xyz(sphere_pos.x, sphere_pos.y, sphere_pos.z),
            color=sphere_color,
        )
        self.sphere = PhysicsObject(s_node, "Sphere", "sphere", sphere_radius, sphere_color, sphere_pos)

        # 4. HUD UI Elements
        sw = float(self.screen_width)
        sh = float(self.screen_height)

        # Crosshair in screen center
        cx = sw * 0.5
        cy = sh * 0.5
        self.crosshair_node = self.scene.spawn_text_2d(
            "+",
            color=Color(1.0, 1.0, 1.0, 0.85),
            scale=1.2,
            transform=Transform.from_xyz(cx - 7.0, cy - 14.0, 0.0),
        )

        # Top Information Cards
        panel_color = Color(0.10, 0.13, 0.20, 0.88)
        self.scene.spawn_surface_2d(
            Vec2(500.0, 120.0),
            color=panel_color,
            transform=Transform.from_xyz(24.0, 24.0, 0.0),
        )

        self.title_node = self.scene.spawn_text_2d(
            "Oxide 3D - Interactive Demo",
            color=Color(0.35, 0.85, 1.0),
            scale=0.82,
            transform=Transform.from_xyz(38.0, 46.0, 0.0),
        )

        self.status_node = self.scene.spawn_text_2d(
            "Held: None | FPS: 0.0",
            color=Color(0.9, 0.9, 0.9),
            scale=0.68,
            transform=Transform.from_xyz(38.0, 74.0, 0.0),
        )

        self.target_hint_node = self.scene.spawn_text_2d(
            "Aim: [None]",
            color=Color(0.4, 0.9, 0.4),
            scale=0.65,
            transform=Transform.from_xyz(38.0, 102.0, 0.0),
        )

        # Bottom Instructions Panel
        self.scene.spawn_surface_2d(
            Vec2(660.0, 42.0),
            color=Color(0.08, 0.10, 0.16, 0.85),
            transform=Transform.from_xyz(24.0, sh - 66.0, 0.0),
        )

        self.instructions_node = self.scene.spawn_text_2d(
            "[E] Pick/Drop | [LMB] Throw | [Hold RMB + Mouse] Rotate | [WASD] Move | [Esc] Mouse",
            color=Color(0.85, 0.88, 0.92),
            scale=0.58,
            transform=Transform.from_xyz(36.0, sh - 40.0, 0.0),
        )

    def get_camera_vectors(self):
        cos_p = math.cos(self.pitch)
        sin_p = math.sin(self.pitch)
        cos_y = math.cos(self.yaw)
        sin_y = math.sin(self.yaw)

        # Standard non-inverted coordinate orientation:
        # pitch > 0 looks UP (+Y)
        # yaw > 0 turns RIGHT (+X)
        forward = Vec3(sin_y * cos_p, sin_p, -cos_y * cos_p).normalize()
        right = Vec3(cos_y, 0.0, sin_y).normalize()
        up = Vec3(-sin_y * sin_p, cos_p, cos_y * sin_p).normalize()
        return forward, right, up

    def update_camera(self, ctx: Context):
        forward, _, _ = self.get_camera_vectors()
        target = self.player_pos + forward
        self.scene.set_camera_perspective(
            fov_deg=65.0,
            aspect=ctx.aspect_ratio,
            near=0.1,
            far=200.0,
            eye=self.player_pos,
            target=target,
        )

    def on_update(self, ctx: Context, dt: float):
        if dt <= 0.0:
            return

        dt = min(dt, 0.05)
        forward, right, up = self.get_camera_vectors()
        rmb_down = ctx.is_mouse_button_down(MouseButton.RIGHT)

        # ----------------------------------------------------
        # 0. Cursor Capture & Infinite 360-Degree Mouse Delta
        # ----------------------------------------------------
        # Press [Escape] to release or toggle cursor grab
        if ctx.is_key_just_pressed("Escape"):
            self.mouse_ctrl.set_grabbed(not self.mouse_ctrl.grabbed, ctx)

        # Click left mouse button to re-grab cursor if currently free
        if not self.mouse_ctrl.grabbed:
            if ctx.is_mouse_button_just_pressed(MouseButton.LEFT):
                self.mouse_ctrl.set_grabbed(True, ctx)
            m_delta = Vec2(0.0, 0.0)
        else:
            m_delta = self.mouse_ctrl.get_delta(ctx.mouse_delta)

        # ----------------------------------------------------
        # 1. Mouse Interaction: Camera Look vs Object Rotation
        # ----------------------------------------------------
        # SPEC: "правой кнопкой (зажав) можно двигать мышкой
        # (камера не двигается, а объект крутится)."
        if self.held_object is not None and rmb_down:
            # Camera DOES NOT move; held object rotates in hand
            rot_sensitivity = 0.007
            dx = m_delta.x * rot_sensitivity
            dy = m_delta.y * rot_sensitivity

            # Rotate around camera UP and RIGHT axes
            q_yaw = Quat.from_axis_angle(up, dx)
            q_pitch = Quat.from_axis_angle(right, dy)
            self.held_object.rotation = q_pitch * (q_yaw * self.held_object.rotation)
        elif self.mouse_ctrl.grabbed:
            # Standard, natural mouse look (infinite 360 rotation):
            # Move mouse RIGHT -> view turns RIGHT
            # Move mouse LEFT  -> view turns LEFT
            # Move mouse UP    -> view looks UP
            # Move mouse DOWN  -> view looks DOWN
            look_sensitivity = 0.0022
            self.yaw += m_delta.x * look_sensitivity
            self.pitch -= m_delta.y * look_sensitivity

            max_pitch = math.radians(88.0)
            self.pitch = max(-max_pitch, min(max_pitch, self.pitch))

        # ----------------------------------------------------
        # 2. Player Movement (WASD + Jump)
        # ----------------------------------------------------
        move_dir = Vec3(0.0, 0.0, 0.0)
        # Horizontal forward and right vectors aligned with camera view
        horiz_forward = Vec3(math.sin(self.yaw), 0.0, -math.cos(self.yaw)).normalize()
        horiz_right = Vec3(math.cos(self.yaw), 0.0, math.sin(self.yaw)).normalize()

        if ctx.is_key_down("W"):
            move_dir = move_dir + horiz_forward
        if ctx.is_key_down("S"):
            move_dir = move_dir - horiz_forward
        if ctx.is_key_down("A"):
            move_dir = move_dir - horiz_right
        if ctx.is_key_down("D"):
            move_dir = move_dir + horiz_right

        speed = 8.5 if (ctx.is_key_down("Shift") or ctx.is_key_down("ShiftLeft")) else 4.6
        if move_dir.length_squared() > 1e-4:
            self.player_pos = self.player_pos + move_dir.normalize() * (speed * dt)

        # Jump & Vertical physics
        if ctx.is_key_just_pressed("Space") and self.is_grounded:
            self.player_vel_y = 6.2
            self.is_grounded = False

        if not self.is_grounded:
            self.player_vel_y -= 19.6 * dt
            self.player_pos.y += self.player_vel_y * dt
            if self.player_pos.y <= 1.7:
                self.player_pos.y = 1.7
                self.player_vel_y = 0.0
                self.is_grounded = True

        # Keep player on platform
        self.player_pos.x = max(-26.5, min(26.5, self.player_pos.x))
        self.player_pos.z = max(-26.5, min(26.5, self.player_pos.z))

        # ----------------------------------------------------
        # 3. Raycast & Aim Detection
        # ----------------------------------------------------
        ray_hit = self.scene.raycast(self.player_pos, forward)
        self.aimed_object = None

        if ray_hit:
            if ray_hit.node_id == self.cube.node_id and ray_hit.distance <= 4.8:
                self.aimed_object = self.cube
            elif ray_hit.node_id == self.sphere.node_id and ray_hit.distance <= 4.8:
                self.aimed_object = self.sphere

        # Proximity fallback if ray missed slightly
        if self.aimed_object is None and self.held_object is None:
            best_dist = 3.6
            for candidate in [self.cube, self.sphere]:
                to_obj = candidate.position - self.player_pos
                dist = to_obj.length()
                if dist < best_dist:
                    cos_angle = forward.dot(to_obj.normalize())
                    if cos_angle > 0.65:
                        self.aimed_object = candidate
                        best_dist = dist

        # ----------------------------------------------------
        # 4. Pick Up / Put Down ([E] Key)
        # ----------------------------------------------------
        if ctx.is_key_just_pressed("E"):
            if self.held_object is not None:
                # SPEC: "E - положить" (Put down / release gently)
                self.held_object.velocity = Vec3(0.0, 0.0, 0.0)
                self.held_object.angular_velocity = Vec3(0.0, 0.0, 0.0)
                self.held_object.is_held = False
                self.held_object = None
            elif self.aimed_object is not None:
                # SPEC: "подобрать на Е объект"
                self.held_object = self.aimed_object
                self.held_object.is_held = True
                self.held_object.velocity = Vec3(0.0, 0.0, 0.0)
                self.held_object.angular_velocity = Vec3(0.0, 0.0, 0.0)

        # ----------------------------------------------------
        # 5. Throw with Force ([LMB] Click)
        # ----------------------------------------------------
        # SPEC: "на левый клик мыши кинуть с силой"
        if ctx.is_mouse_button_just_pressed(MouseButton.LEFT) and self.held_object is not None and self.mouse_ctrl.grabbed:
            throw_speed = 22.0
            throw_vec = forward * throw_speed + Vec3(0.0, 2.5, 0.0)
            self.held_object.velocity = throw_vec
            self.held_object.angular_velocity = Vec3(
                random.uniform(-8.0, 8.0),
                random.uniform(-8.0, 8.0),
                random.uniform(-8.0, 8.0),
            )
            self.held_object.is_held = False
            self.held_object = None

        # ----------------------------------------------------
        # 6. Held Object Position Constraint
        # ----------------------------------------------------
        if self.held_object is not None:
            hold_dist = 2.1
            hold_target = self.player_pos + forward * hold_dist
            self.held_object.position = hold_target
            self.scene.node_set_translation(self.held_object.node_id, hold_target)
            self.scene.node_set_rotation(self.held_object.node_id, self.held_object.rotation)

        # ----------------------------------------------------
        # 7. Physical Simulation for Free Objects
        # ----------------------------------------------------
        for obj in [self.cube, self.sphere]:
            if not obj.is_held:
                obj.update_physics(dt, floor_y=0.0)
                self.scene.node_set_translation(obj.node_id, obj.position)
                self.scene.node_set_rotation(obj.node_id, obj.rotation)

        # ----------------------------------------------------
        # 8. Mutual Collision Resolution (Cube vs Sphere)
        # ----------------------------------------------------
        contact = self.scene.check_collision(self.cube.node_id, self.sphere.node_id)
        if contact:
            self.scene.resolve_penetration(self.cube.node_id, self.sphere.node_id, contact)
            self.cube.position = self.scene.node_get_translation(self.cube.node_id)
            self.sphere.position = self.scene.node_get_translation(self.sphere.node_id)

            # Elastic impulse collision response
            normal = Vec3(contact.normal.x, contact.normal.y, contact.normal.z).normalize()
            rel_vel = self.cube.velocity - self.sphere.velocity
            vel_along_norm = rel_vel.dot(normal)

            if vel_along_norm < 0.0:
                impulse_mag = -(1.0 + 0.6) * vel_along_norm * 0.5
                impulse = normal * impulse_mag
                if not self.cube.is_held:
                    self.cube.velocity = self.cube.velocity + impulse
                if not self.sphere.is_held:
                    self.sphere.velocity = self.sphere.velocity - impulse

        # ----------------------------------------------------
        # 9. Update UI & Scene
        # ----------------------------------------------------
        self.update_camera(ctx)

        # Update HUD text safely
        if self.status_node is not None:
            held_name = self.held_object.name if self.held_object else "None"
            fps_text = f"Held: {held_name} | FPS: {ctx.fps:.1f}"
            self.scene.node_set_text(self.status_node, fps_text)

        if self.target_hint_node is not None:
            if self.held_object is not None:
                self.scene.node_set_text(
                    self.target_hint_node,
                    f"Holding {self.held_object.name} -> [LMB] Throw | [E] Drop | [RMB] Rotate",
                )
            elif self.aimed_object is not None:
                self.scene.node_set_text(
                    self.target_hint_node,
                    f"Target: {self.aimed_object.name} -> Press [E] to Pick Up",
                )
            else:
                self.scene.node_set_text(self.target_hint_node, "Aim at Cube or Sphere to Pick Up")

        self.scene.update(ctx, dt)

    def on_render(self, ctx: Context):
        ctx.clear(Color(0.08, 0.10, 0.16, 1.0))
        self.scene.render(ctx)


if __name__ == "__main__":
    title = "Oxide - 3D Interactive World (Cube & Sphere)"
    print("=" * 60)
    print(" Oxide 3D Game Starting...")
    print(" Controls:")
    print("   [W, A, S, D] : Walk")
    print("   [Shift]      : Sprint")
    print("   [Space]      : Jump")
    print("   [Mouse]      : Full 360-Degree Look (Non-inverted)")
    print("   [Escape]     : Toggle Cursor Lock")
    print("   [E]          : Pick up / Put down target object")
    print("   [LMB]        : Throw held object with force")
    print("   [RMB (Hold)] : Inspect / Rotate held object with mouse")
    print("=" * 60)

    app = App(title=title, width=1280, height=720)
    app.run(Simple3DGame(window_title=title))
