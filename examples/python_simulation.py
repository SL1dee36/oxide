import sys
import os

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../bindings/python")))
from oxide import App, Scene, Vec3, Vec2, Color, Transform

class PythonSimulation:
    def __init__(self):
        self.scene = Scene()
        self.cube_node = None
        self.sphere_node = None
        self.panel_node = None
        self.title_node = None
        self.stats_node = None

    def on_init(self, ctx):
        self.scene.set_camera_perspective(
            fov_deg=60.0,
            aspect=ctx.aspect_ratio,
            near=0.1,
            far=100.0,
            eye=Vec3(0.0, 3.0, 8.0),
            target=Vec3(0.0, 0.0, 0.0),
        )

        self.cube_node = self.scene.spawn_cube_3d(1.5, Transform.from_xyz(-1.2, 0.0, 0.0))
        self.sphere_node = self.scene.spawn_sphere_3d(1.0, Transform.from_xyz(1.2, 0.0, 0.0))

        self.panel_node = self.scene.spawn_surface_2d(
            Vec2(340.0, 90.0),
            Color(0.1, 0.14, 0.22, 0.85),
            Transform.from_xyz(25.0, 25.0, 0.0),
        )

        self.title_node = self.scene.spawn_text_2d(
            "Oxide Python Runtime (C-ABI)",
            Color(0.3, 0.8, 1.0),
            scale=0.9,
            transform=Transform.from_xyz(40.0, 48.0, 0.0),
        )

        self.stats_node = self.scene.spawn_text_2d(
            "FPS: 0.0 | Status: Running",
            Color(0.3, 0.9, 0.4),
            scale=0.7,
            transform=Transform.from_xyz(40.0, 78.0, 0.0),
        )

    def on_update(self, ctx, dt):
        self.scene.node_rotate_y(self.cube_node, 1.2 * dt)
        self.scene.node_rotate_x(self.cube_node, 0.6 * dt)

        contact = self.scene.check_collision(self.cube_node, self.sphere_node)
        status = "Clear"
        if contact:
            self.scene.resolve_penetration(self.cube_node, self.sphere_node, contact)
            status = "Collision Resolved"

        self.scene.node_set_text(self.stats_node, f"FPS: {ctx.fps:.1f} | Status: {status}")
        self.scene.update(ctx, dt)

    def on_render(self, ctx):
        ctx.clear(Color(0.07, 0.08, 0.11))
        self.scene.render(ctx)

if __name__ == "__main__":
    app = App(title="Oxide - Python Native Workspace", width=1280, height=720)
    app.run(PythonSimulation())
