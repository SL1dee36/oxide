#!/usr/bin/env python3
"""
Simple Interactive 3D simulation in Oxide powered by native engine physics.

Features:
- Cross-platform cursor capture using native Context API ([Esc] to toggle/release mouse).
- Native 6-DOF RigidBody simulation (gravity, friction, restitution, inertia tensors).
- Pick up Cube or Sphere by pressing [E] when within reach.
- Drop/put down held object gently by pressing [E].
- Throw held object forward with force by clicking Left Mouse Button [LMB].
- Inspect/rotate held object by holding Right Mouse Button [RMB] and dragging mouse.
- Dynamic crosshair and interactive HUD status overlay.
"""

import os
import sys
import math
import random

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


class PhysicsObject:
    """Manages physical state for a 3D scene node backed by Oxide's native RigidBody physics."""

    def __init__(self, node_id, name, kind, size_or_radius, color):
        self.node_id = int(node_id)
        self.name = name
        self.kind = kind  # "cube" or "sphere"
        self.size_or_radius = float(size_or_radius)
        self.color = color
        self.is_held = False
        self.held_rotation = Quat(0.0, 0.0, 0.0, 1.0)


class Simple3DGame:
    def __init__(self, window_title="Oxide - 3D Interactive World (Native Physics)"):
        self.scene = Scene()
        self.window_title = window_title
        self.screen_width = 1280
        self.screen_height = 720

        # Cross-platform cursor state
        self.cursor_grabbed = False

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
        # 1. Grab mouse cursor for infinite 360 FPS view
        self.cursor_grabbed = True
        ctx.set_cursor_visible(False)
        ctx.set_cursor_grab(True)

        # Configure Viewport & 3D Perspective Camera
        self.update_camera(ctx)

        # Directional sunlight + ambient illumination
        sun_direction = Vec3(0.5, 1.0, 0.7).normalize()
        self.scene.set_light(
            direction=sun_direction,
            color=Vec3(1.0, 0.98, 0.92),
            ambient=Vec3(0.32, 0.35, 0.42),
        )

        # 2. Flat World Platform (56x56 units, static rigid body floor)
        floor_w, floor_h, floor_d = 56.0, 0.6, 56.0
        self.floor_node = self.scene.spawn_rigid_body_box(
            Vec3(floor_w, floor_h, floor_d),
            transform=Transform.from_xyz(0.0, -floor_h * 0.5, 0.0),
            mass=0.0,
            restitution=0.4,
            friction=0.8,
            is_static=True,
        )
        self.scene.node_set_color(self.floor_node, Color(0.20, 0.24, 0.30, 1.0))

        # Perimeter curbs (static rigid body obstacles)
        curb_color = Color(0.35, 0.40, 0.50, 1.0)
        ch = 0.8
        for curb_size, curb_pos in [
            (Vec3(floor_w, ch, 0.8), Vec3(0.0, ch * 0.5, -28.0)),
            (Vec3(floor_w, ch, 0.8), Vec3(0.0, ch * 0.5, 28.0)),
            (Vec3(0.8, ch, floor_d), Vec3(-28.0, ch * 0.5, 0.0)),
            (Vec3(0.8, ch, floor_d), Vec3(28.0, ch * 0.5, 0.0)),
        ]:
            node = self.scene.spawn_rigid_body_box(
                curb_size,
                transform=Transform.from_xyz(curb_pos.x, curb_pos.y, curb_pos.z),
                mass=0.0,
                restitution=0.5,
                friction=0.8,
                is_static=True,
            )
            self.scene.node_set_color(node, curb_color)
            self.boundary_nodes.append(node)

        # 3. Interactive Native Physics Objects: Cube and Sphere
        # Cube: dynamic box of size 1.0, mass 2.0 kg
        cube_size = 1.0
        cube_pos = Vec3(-2.2, cube_size * 0.5 + 0.1, -3.0)
        cube_color = Color(1.0, 0.46, 0.18, 1.0)
        c_node = self.scene.spawn_rigid_body_box(
            Vec3(cube_size, cube_size, cube_size),
            transform=Transform.from_xyz(cube_pos.x, cube_pos.y, cube_pos.z),
            mass=2.0,
            restitution=0.42,
            friction=0.65,
            is_static=False,
        )
        self.scene.node_set_color(c_node, cube_color)
        self.cube = PhysicsObject(c_node, "Cube", "cube", cube_size, cube_color)

        # Sphere: dynamic sphere of radius 0.6, mass 1.5 kg
        sphere_radius = 0.6
        sphere_pos = Vec3(2.2, sphere_radius + 0.1, -3.0)
        sphere_color = Color(0.18, 0.88, 0.62, 1.0)
        s_node = self.scene.spawn_rigid_body_sphere(
            sphere_radius,
            transform=Transform.from_xyz(sphere_pos.x, sphere_pos.y, sphere_pos.z),
            mass=1.5,
            restitution=0.68,
            friction=0.55,
            is_static=False,
        )
        self.scene.node_set_color(s_node, sphere_color)
        self.sphere = PhysicsObject(s_node, "Sphere", "sphere", sphere_radius, sphere_color)

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
            Vec2(520.0, 120.0),
            color=panel_color,
            transform=Transform.from_xyz(24.0, 24.0, 0.0),
        )

        self.title_node = self.scene.spawn_text_2d(
            "Oxide 3D - Native Physics Engine",
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
            Vec2(680.0, 42.0),
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
        # 0. Cursor Capture & Mouse Delta Handling
        # ----------------------------------------------------
        # Press [Escape] to release or toggle cursor grab
        if ctx.is_key_just_pressed("Escape"):
            self.cursor_grabbed = not self.cursor_grabbed
            ctx.set_cursor_visible(not self.cursor_grabbed)
            ctx.set_cursor_grab(self.cursor_grabbed)

        # Click left mouse button to re-grab cursor if currently free
        if not self.cursor_grabbed:
            if ctx.is_mouse_button_just_pressed(MouseButton.LEFT):
                self.cursor_grabbed = True
                ctx.set_cursor_visible(False)
                ctx.set_cursor_grab(True)
            m_delta = Vec2(0.0, 0.0)
        else:
            m_delta = ctx.mouse_delta

        # ----------------------------------------------------
        # 1. Mouse Interaction: Camera Look vs Object Rotation
        # ----------------------------------------------------
        if self.held_object is not None and rmb_down:
            # Camera remains stationary; held object rotates in player's hands
            rot_sensitivity = 0.007
            dx = m_delta.x * rot_sensitivity
            dy = m_delta.y * rot_sensitivity

            # Rotate around camera UP and RIGHT axes
            q_yaw = Quat.from_axis_angle(up, dx)
            q_pitch = Quat.from_axis_angle(right, dy)
            self.held_object.held_rotation = q_pitch * (q_yaw * self.held_object.held_rotation)
            self.scene.node_set_rotation(self.held_object.node_id, self.held_object.held_rotation)
        elif self.cursor_grabbed:
            look_sensitivity = 0.0022
            self.yaw += m_delta.x * look_sensitivity
            self.pitch -= m_delta.y * look_sensitivity

            max_pitch = math.radians(88.0)
            self.pitch = max(-max_pitch, min(max_pitch, self.pitch))

        # ----------------------------------------------------
        # 2. Player Movement (WASD + Jump)
        # ----------------------------------------------------
        move_dir = Vec3(0.0, 0.0, 0.0)
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
            if ray_hit.node_id == self.cube.node_id and ray_hit.distance <= 5.0:
                self.aimed_object = self.cube
            elif ray_hit.node_id == self.sphere.node_id and ray_hit.distance <= 5.0:
                self.aimed_object = self.sphere

        # Proximity fallback if ray missed slightly
        if self.aimed_object is None and self.held_object is None:
            best_dist = 3.6
            for candidate in [self.cube, self.sphere]:
                cand_pos = self.scene.node_get_translation(candidate.node_id)
                to_obj = cand_pos - self.player_pos
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
                # Put down / release gently into the physics world
                self.scene.rigid_body_set_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))
                self.scene.rigid_body_set_angular_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))
                self.held_object.is_held = False
                self.held_object = None
            elif self.aimed_object is not None:
                # Pick up: zero out velocity and capture initial rotation
                self.held_object = self.aimed_object
                self.held_object.is_held = True
                self.held_object.held_rotation = self.scene.node_get_rotation(self.held_object.node_id)
                self.scene.rigid_body_set_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))
                self.scene.rigid_body_set_angular_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))

        # ----------------------------------------------------
        # 5. Throw with Force ([LMB] Click)
        # ----------------------------------------------------
        if ctx.is_mouse_button_just_pressed(MouseButton.LEFT) and self.held_object is not None and self.cursor_grabbed:
            throw_speed = 20.0
            throw_vec = forward * throw_speed + Vec3(0.0, 2.5, 0.0)
            self.scene.rigid_body_set_velocity(self.held_object.node_id, throw_vec)
            self.scene.rigid_body_set_angular_velocity(
                self.held_object.node_id,
                Vec3(
                    random.uniform(-6.0, 6.0),
                    random.uniform(-6.0, 6.0),
                    random.uniform(-6.0, 6.0),
                ),
            )
            self.held_object.is_held = False
            self.held_object = None

        # ----------------------------------------------------
        # 6. Held Object Position Constraint
        # ----------------------------------------------------
        if self.held_object is not None:
            hold_dist = 2.2
            hold_target = self.player_pos + forward * hold_dist
            self.scene.node_set_translation(self.held_object.node_id, hold_target)
            self.scene.node_set_rotation(self.held_object.node_id, self.held_object.held_rotation)
            self.scene.rigid_body_set_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))
            self.scene.rigid_body_set_angular_velocity(self.held_object.node_id, Vec3(0.0, 0.0, 0.0))

        # ----------------------------------------------------
        # 7. Update UI & Scene (Native Physics Simulation Step)
        # ----------------------------------------------------
        self.update_camera(ctx)

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

        # Native physics step & scene hierarchy update
        self.scene.update(ctx, dt)

    def on_render(self, ctx: Context):
        ctx.clear(Color(0.08, 0.10, 0.16, 1.0))
        self.scene.render(ctx)


if __name__ == "__main__":
    title = "Oxide - 3D Interactive World (Native Physics)"
    print("=" * 60)
    print(" Oxide 3D Game Starting (Native Physics)...")
    print(" Controls:")
    print("   [W, A, S, D] : Walk")
    print("   [Shift]      : Sprint")
    print("   [Space]      : Jump")
    print("   [Mouse]      : Full 360-Degree Look")
    print("   [Escape]     : Toggle Cursor Lock")
    print("   [E]          : Pick up / Put down target object")
    print("   [LMB]        : Throw held object with force")
    print("   [RMB (Hold)] : Inspect / Rotate held object with mouse")
    print("=" * 60)

    app = App(title=title, width=1280, height=720)
    app.run(Simple3DGame(window_title=title))
