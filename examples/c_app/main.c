#include "oxide.h"
#include <stdio.h>

typedef struct {
    OxideScene* scene;
    uint64_t cube;
    uint64_t sphere;
    uint64_t text;
} UserApp;

void on_init(OxideContext* ctx, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    app->scene = oxide_scene_create();

    OxideVec3 eye = {0.0f, 3.0f, 8.0f};
    OxideVec3 target = {0.0f, 0.0f, 0.0f};
    oxide_scene_set_camera_perspective(app->scene, 60.0f, oxide_context_get_aspect_ratio(ctx), 0.1f, 100.0f, eye, target);

    OxideTransform t_cube = { {-1.2f, 0.0f, 0.0f}, {0.0f, 0.0f, 0.0f, 1.0f}, {1.0f, 1.0f, 1.0f} };
    app->cube = oxide_scene_spawn_cube_3d(app->scene, 1.5f, t_cube);

    OxideTransform t_sphere = { {1.2f, 0.0f, 0.0f}, {0.0f, 0.0f, 0.0f, 1.0f}, {1.0f, 1.0f, 1.0f} };
    app->sphere = oxide_scene_spawn_sphere_3d(app->scene, 1.0f, t_sphere);

    OxideTransform t_text = { {40.0f, 50.0f, 0.0f}, {0.0f, 0.0f, 0.0f, 1.0f}, {1.0f, 1.0f, 1.0f} };
    OxideColor text_col = {0.3f, 0.8f, 1.0f, 1.0f};
    app->text = oxide_scene_spawn_text_2d(app->scene, "Oxide Native C99 Application", text_col, 0.9f, t_text);
}

void on_update(OxideContext* ctx, float dt, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    oxide_scene_node_rotate_y(app->scene, app->cube, 1.2f * dt);

    OxideContact contact;
    if (oxide_scene_check_collision(app->scene, app->cube, app->sphere, &contact)) {
        oxide_scene_resolve_penetration(app->scene, app->cube, app->sphere, &contact);
    }

    oxide_scene_update(app->scene, ctx, dt);
}

void on_render(OxideContext* ctx, void* user_data) {
    UserApp* app = (UserApp*)user_data;
    OxideColor clear_col = {0.07f, 0.08f, 0.11f, 1.0f};
    oxide_context_clear(ctx, clear_col);
    oxide_scene_render(app->scene, ctx);
}

int main(void) {
    UserApp user_app = {0};
    OxideApp* app = oxide_app_create("Oxide - Pure C App", 1280, 720);

    OxideCallbacks callbacks = {
        .on_init = on_init,
        .on_update = on_update,
        .on_render = on_render,
        .user_data = &user_app
    };

    int result = oxide_app_run(app, callbacks);
    oxide_scene_destroy(user_app.scene);
    return result;
}
