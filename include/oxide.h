#ifndef OXIDE_H
#define OXIDE_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Math Structures
typedef struct {
    float x;
    float y;
} OxideVec2;

typedef struct {
    float x;
    float y;
    float z;
} OxideVec3;

typedef struct {
    float x;
    float y;
    float z;
    float w;
} OxideQuat;

typedef struct {
    float r;
    float g;
    float b;
    float a;
} OxideColor;

typedef struct {
    OxideVec3 translation;
    OxideQuat rotation;
    OxideVec3 scale;
} OxideTransform;

typedef struct {
    OxideVec3 normal;
    float depth;
    OxideVec3 point;
} OxideContact;

// Opaque Engine Types
typedef struct OxideApp OxideApp;
typedef struct OxideContext OxideContext;
typedef struct OxideScene OxideScene;

// Callbacks
typedef void (*OxideInitFn)(OxideContext* ctx, void* user_data);
typedef void (*OxideUpdateFn)(OxideContext* ctx, float dt, void* user_data);
typedef void (*OxideRenderFn)(OxideContext* ctx, void* user_data);

typedef struct {
    OxideInitFn on_init;
    OxideUpdateFn on_update;
    OxideRenderFn on_render;
    void* user_data;
} OxideCallbacks;

// Application Lifecycle API
OxideApp* oxide_app_create(const char* title, uint32_t width, uint32_t height);
void oxide_app_destroy(OxideApp* app);
int32_t oxide_app_run(OxideApp* app, OxideCallbacks callbacks);

// Graphics & Execution Context API
void oxide_context_clear(OxideContext* ctx, OxideColor color);
float oxide_context_get_fps(OxideContext* ctx);
float oxide_context_get_delta_time(OxideContext* ctx);
float oxide_context_get_aspect_ratio(OxideContext* ctx);

// Scene & Node Graph API
OxideScene* oxide_scene_create(void);
void oxide_scene_destroy(OxideScene* scene);

void oxide_scene_set_camera_perspective(
    OxideScene* scene,
    float fov_deg,
    float aspect,
    float near_z,
    float far_z,
    OxideVec3 eye,
    OxideVec3 target
);

uint64_t oxide_scene_spawn_cube_3d(OxideScene* scene, float size, OxideTransform transform);
uint64_t oxide_scene_spawn_sphere_3d(OxideScene* scene, float radius, OxideTransform transform);
uint64_t oxide_scene_spawn_surface_2d(OxideScene* scene, OxideVec2 size, OxideColor color, OxideTransform transform);
uint64_t oxide_scene_spawn_text_2d(OxideScene* scene, const char* text, OxideColor color, float scale, OxideTransform transform);

void oxide_scene_node_set_text(OxideScene* scene, uint64_t node_id, const char* text);
void oxide_scene_node_rotate_y(OxideScene* scene, uint64_t node_id, float rad);
void oxide_scene_node_rotate_x(OxideScene* scene, uint64_t node_id, float rad);

bool oxide_scene_check_collision(OxideScene* scene, uint64_t node_a, uint64_t node_b, OxideContact* out_contact);
void oxide_scene_resolve_penetration(OxideScene* scene, uint64_t node_a, uint64_t node_b, const OxideContact* contact);

void oxide_scene_update(OxideScene* scene, OxideContext* ctx, float dt);
void oxide_scene_render(OxideScene* scene, OxideContext* ctx);

#ifdef __cplusplus
}
#endif

#endif // OXIDE_H
