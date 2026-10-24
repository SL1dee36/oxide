#ifndef OXIDE_H
#define OXIDE_H

#include <stdint.h>
#include <stdbool.h>

#define OXIDE_INVALID_NODE_ID UINT64_MAX

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

typedef struct {
    uint64_t node_id;
    float distance;
    OxideVec3 point;
    OxideVec3 normal;
} OxideRaycastHit;

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
uint32_t oxide_context_get_width(OxideContext* ctx);
uint32_t oxide_context_get_height(OxideContext* ctx);

// Context Input & Cursor API
bool oxide_context_is_key_down(OxideContext* ctx, const char* key_name);
bool oxide_context_is_key_just_pressed(OxideContext* ctx, const char* key_name);
bool oxide_context_is_key_just_released(OxideContext* ctx, const char* key_name);
bool oxide_context_is_mouse_button_down(OxideContext* ctx, uint32_t button);
bool oxide_context_is_mouse_button_just_pressed(OxideContext* ctx, uint32_t button);
bool oxide_context_is_mouse_button_just_released(OxideContext* ctx, uint32_t button);
void oxide_context_get_mouse_position(OxideContext* ctx, OxideVec2* out_pos);
void oxide_context_get_mouse_delta(OxideContext* ctx, OxideVec2* out_delta);
void oxide_context_get_mouse_wheel(OxideContext* ctx, OxideVec2* out_wheel);
void oxide_context_set_cursor_visible(OxideContext* ctx, bool visible);
bool oxide_context_set_cursor_grab(OxideContext* ctx, bool grab);

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
void oxide_scene_set_camera_look_at(OxideScene* scene, OxideVec3 eye, OxideVec3 target, OxideVec3 up);
void oxide_scene_set_light(OxideScene* scene, OxideVec3 direction, OxideVec3 color, OxideVec3 ambient);

// Node Spawning
uint64_t oxide_scene_spawn_cube_3d(OxideScene* scene, float size, OxideTransform transform);
uint64_t oxide_scene_spawn_cube_3d_colored(OxideScene* scene, float size, OxideColor color, OxideTransform transform);
uint64_t oxide_scene_spawn_sphere_3d(OxideScene* scene, float radius, OxideTransform transform);
uint64_t oxide_scene_spawn_sphere_3d_colored(OxideScene* scene, float radius, OxideColor color, OxideTransform transform);
uint64_t oxide_scene_spawn_box_3d(OxideScene* scene, float width, float height, float depth, OxideColor color, OxideTransform transform, bool is_static);
uint64_t oxide_scene_spawn_surface_2d(OxideScene* scene, OxideVec2 size, OxideColor color, OxideTransform transform);
uint64_t oxide_scene_spawn_text_2d(OxideScene* scene, const char* text, OxideColor color, float scale, OxideTransform transform);

// Node Manipulation
bool oxide_scene_despawn_node(OxideScene* scene, uint64_t node_id);
bool oxide_scene_node_get_transform(OxideScene* scene, uint64_t node_id, OxideTransform* out_transform);
bool oxide_scene_node_set_transform(OxideScene* scene, uint64_t node_id, OxideTransform transform);
bool oxide_scene_node_get_translation(OxideScene* scene, uint64_t node_id, OxideVec3* out_translation);
bool oxide_scene_node_set_translation(OxideScene* scene, uint64_t node_id, OxideVec3 translation);
bool oxide_scene_node_get_rotation(OxideScene* scene, uint64_t node_id, OxideQuat* out_rotation);
bool oxide_scene_node_set_rotation(OxideScene* scene, uint64_t node_id, OxideQuat rotation);
void oxide_scene_node_rotate_y(OxideScene* scene, uint64_t node_id, float rad);
void oxide_scene_node_rotate_x(OxideScene* scene, uint64_t node_id, float rad);
void oxide_scene_node_rotate_z(OxideScene* scene, uint64_t node_id, float rad);
bool oxide_scene_node_set_scale(OxideScene* scene, uint64_t node_id, OxideVec3 scale);
bool oxide_scene_node_set_color(OxideScene* scene, uint64_t node_id, OxideColor color);
bool oxide_scene_node_set_visible(OxideScene* scene, uint64_t node_id, bool visible);
bool oxide_scene_node_set_static(OxideScene* scene, uint64_t node_id, bool is_static);
void oxide_scene_node_set_text(OxideScene* scene, uint64_t node_id, const char* text);

// Collision & Raycasting
bool oxide_scene_check_collision(OxideScene* scene, uint64_t node_a, uint64_t node_b, OxideContact* out_contact);
void oxide_scene_resolve_penetration(OxideScene* scene, uint64_t node_a, uint64_t node_b, const OxideContact* contact);
bool oxide_scene_raycast(OxideScene* scene, OxideVec3 origin, OxideVec3 direction, OxideRaycastHit* out_hit);

// Scene Lifecycle
bool oxide_scene_update(OxideScene* scene, OxideContext* ctx, float dt);
bool oxide_scene_render(OxideScene* scene, OxideContext* ctx);

// Physics Simulation
bool oxide_scene_step_physics(OxideScene* scene, float dt);
uint64_t oxide_scene_spawn_rigid_body_box(OxideScene* scene, OxideVec3 size, OxideTransform transform, float mass, float restitution, float friction, bool is_static);
uint64_t oxide_scene_spawn_rigid_body_sphere(OxideScene* scene, float radius, OxideTransform transform, float mass, float restitution, float friction, bool is_static);
bool oxide_scene_add_rigid_body(OxideScene* scene, uint64_t node_id, float mass, float restitution, float friction, uint32_t body_type);
void oxide_scene_rigid_body_apply_force(OxideScene* scene, uint64_t node_id, OxideVec3 force);
void oxide_scene_rigid_body_apply_torque(OxideScene* scene, uint64_t node_id, OxideVec3 torque);
void oxide_scene_rigid_body_apply_impulse(OxideScene* scene, uint64_t node_id, OxideVec3 impulse);
bool oxide_scene_rigid_body_set_velocity(OxideScene* scene, uint64_t node_id, OxideVec3 velocity);
bool oxide_scene_rigid_body_get_velocity(OxideScene* scene, uint64_t node_id, OxideVec3* out_velocity);
bool oxide_scene_rigid_body_set_angular_velocity(OxideScene* scene, uint64_t node_id, OxideVec3 ang_vel);
bool oxide_scene_rigid_body_get_angular_velocity(OxideScene* scene, uint64_t node_id, OxideVec3* out_ang_vel);
bool oxide_scene_rigid_body_get_mass(OxideScene* scene, uint64_t node_id, float* out_mass);
uint64_t oxide_scene_spawn_soft_body_cloth(OxideScene* scene, float width, float height, uint32_t segments_x, uint32_t segments_y, float total_mass, float stiffness, OxideTransform transform);
bool oxide_scene_node_add_soft_body_cloth(OxideScene* scene, uint64_t node_id, float width, float height, uint32_t segments_x, uint32_t segments_y, float total_mass, float stiffness);
bool oxide_scene_soft_body_set_particle_fixed(OxideScene* scene, uint64_t node_id, uint32_t particle_idx, bool fixed);
bool oxide_scene_soft_body_get_particle_count(OxideScene* scene, uint64_t node_id, uint32_t* out_count);
bool oxide_scene_soft_body_get_particle_position(OxideScene* scene, uint64_t node_id, uint32_t particle_idx, OxideVec3* out_pos);
bool oxide_scene_soft_body_set_particle_position(OxideScene* scene, uint64_t node_id, uint32_t particle_idx, OxideVec3 pos);
bool oxide_scene_soft_body_add_constraint(OxideScene* scene, uint64_t node_id, uint32_t particle_a, uint32_t particle_b, float rest_length, float stiffness);
bool oxide_scene_soft_body_set_compliance(OxideScene* scene, uint64_t node_id, float compliance);
bool oxide_scene_soft_body_set_substeps(OxideScene* scene, uint64_t node_id, uint32_t substeps);

#ifdef __cplusplus
}
#endif

#endif // OXIDE_H

