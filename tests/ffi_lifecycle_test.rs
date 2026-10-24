use oxide::ffi::*;
use std::ffi::CString;

#[test]
fn test_ffi_scene_lifecycle_and_nodes() {
    unsafe {
        let scene = oxide_scene_create();
        assert!(!scene.is_null(), "oxide_scene_create must return non-null pointer");

        let eye = OxideVec3 { x: 0.0, y: 5.0, z: 10.0 };
        let target = OxideVec3 { x: 0.0, y: 0.0, z: 0.0 };
        oxide_scene_set_camera_perspective(scene, 60.0, 16.0 / 9.0, 0.1, 1000.0, eye, target);

        let t1 = OxideTransform {
            translation: OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
        };
        let cube_id = oxide_scene_spawn_cube_3d(scene, 2.0, t1);
        assert_ne!(cube_id, u64::MAX, "Spawning cube must return valid NodeId");

        let t2 = OxideTransform {
            translation: OxideVec3 { x: 1.5, y: 0.0, z: 0.0 },
            rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
        };
        let sphere_id = oxide_scene_spawn_sphere_3d(scene, 1.0, t2);
        assert_ne!(sphere_id, u64::MAX, "Spawning sphere must return valid NodeId");

        let mut contact = OxideContact {
            normal: OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
            depth: 0.0,
            point: OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
        };
        let intersected = oxide_scene_check_collision(scene, cube_id, sphere_id, &mut contact);
        assert!(intersected, "Cube and sphere should intersect given their positions and sizes");
        assert!(contact.depth > 0.0, "Contact depth must be positive");

        oxide_scene_resolve_penetration(scene, cube_id, sphere_id, &contact);

        oxide_scene_node_rotate_y(scene, cube_id, 0.5);
        oxide_scene_node_rotate_x(scene, cube_id, 0.25);

        let c_text = CString::new("FFI Test String").unwrap();
        let text_id = oxide_scene_spawn_text_2d(
            scene,
            c_text.as_ptr(),
            OxideColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            1.0,
            t1,
        );
        assert_ne!(text_id, u64::MAX);

        let c_updated = CString::new("Updated String").unwrap();
        oxide_scene_node_set_text(scene, text_id, c_updated.as_ptr());

        oxide_scene_destroy(scene);
    }
}

#[test]
fn test_ffi_app_allocation_and_destruction() {
    unsafe {
        let title = CString::new("FFI Test App").unwrap();
        let app = oxide_app_create(title.as_ptr(), 800, 600);
        assert!(!app.is_null(), "oxide_app_create must return valid pointer");

        oxide_app_destroy(app);
    }
}

#[test]
fn test_ffi_error_handling_and_boundary_safety() {
    unsafe {
        assert_eq!(OXIDE_INVALID_NODE_ID, u64::MAX);

        let t = OxideTransform {
            translation: OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
            rotation: OxideQuat { x: 0.0, y: 0.0, z: 0.0, w: 1.0 },
            scale: OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
        };

        // All spawn functions on null scene must return OXIDE_INVALID_NODE_ID
        assert_eq!(oxide_scene_spawn_cube_3d(std::ptr::null_mut(), 1.0, t), OXIDE_INVALID_NODE_ID);
        assert_eq!(oxide_scene_spawn_sphere_3d(std::ptr::null_mut(), 1.0, t), OXIDE_INVALID_NODE_ID);
        assert_eq!(oxide_scene_spawn_rigid_body_box(
            std::ptr::null_mut(),
            OxideVec3 { x: 1.0, y: 1.0, z: 1.0 },
            t,
            1.0, 0.5, 0.5, false
        ), OXIDE_INVALID_NODE_ID);
        assert_eq!(oxide_scene_spawn_rigid_body_sphere(
            std::ptr::null_mut(),
            1.0,
            t,
            1.0, 0.5, 0.5, false
        ), OXIDE_INVALID_NODE_ID);
        assert_eq!(oxide_scene_spawn_soft_body_cloth(
            std::ptr::null_mut(),
            1.0, 1.0, 2, 2, 1.0, 0.5, t
        ), OXIDE_INVALID_NODE_ID);

        // Critical entry points must return false/-1 on null pointers safely
        assert!(!oxide_scene_step_physics(std::ptr::null_mut(), 0.016));
        assert!(!oxide_scene_update(std::ptr::null_mut(), std::ptr::null_mut(), 0.016));
        assert!(!oxide_scene_render(std::ptr::null_mut(), std::ptr::null_mut()));
        assert!(!oxide_scene_raycast(
            std::ptr::null_mut(),
            OxideVec3 { x: 0.0, y: 0.0, z: 0.0 },
            OxideVec3 { x: 0.0, y: 0.0, z: 1.0 },
            std::ptr::null_mut()
        ));

        let callbacks = OxideCallbacks {
            on_init: None,
            on_update: None,
            on_render: None,
            user_data: std::ptr::null_mut(),
        };
        assert_eq!(oxide_app_run(std::ptr::null_mut(), callbacks), -1);
    }
}

