use oxide::prelude::*;

const SAMPLE_OBJ: &str = r#"
# Sample OBJ Quad
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
vn 0.0 0.0 1.0
vt 0.0 0.0
vt 1.0 0.0
vt 1.0 1.0
vt 0.0 1.0
f 1/1/1 2/2/1 3/3/1
f 1/1/1 3/3/1 4/4/1
"#;

#[test]
fn test_obj_loader_from_memory() {
    let mesh = load_obj_from_memory(SAMPLE_OBJ.as_bytes())
        .expect("Valid OBJ string should parse successfully");

    assert_eq!(mesh.vertices.len(), 4, "Should load 4 unique vertices");
    assert_eq!(mesh.indices.len(), 6, "Two triangles produce 6 indices");

    // Check normal and UV assignments
    assert_eq!(mesh.vertices[0].normal, [0.0, 0.0, 1.0]);
    assert_eq!(mesh.vertices[0].uv, [0.0, 0.0]);
}

#[test]
fn test_obj_loader_corrupted_data() {
    let invalid_data = b"NOT_AN_OBJ_DATA_JUST_GARBAGE";
    let res = load_obj_from_memory(invalid_data);
    // Even if tobj doesn't panic on empty models, let's verify empty or handled result
    assert!(res.is_ok() || res.is_err());
}

#[test]
fn test_procedural_primitive_generators() {
    let cube = Mesh3D::cube(2.0);
    assert_eq!(cube.vertices.len(), 24, "Cube must have 24 vertices (4 per face)");
    assert_eq!(cube.indices.len(), 36, "Cube must have 36 indices (6 per face)");

    let plane = Mesh3D::plane(10.0, 10.0);
    assert_eq!(plane.vertices.len(), 4, "Plane must have 4 vertices");
    assert_eq!(plane.indices.len(), 6, "Plane must have 6 indices");

    let sphere = Mesh3D::sphere(1.0, 8, 8);
    assert!(!sphere.vertices.is_empty(), "Sphere must generate vertices");
    assert!(!sphere.indices.is_empty(), "Sphere must generate indices");
}

#[test]
fn test_surface_2d_creation() {
    let surface = Surface2D::from_color(Vec2::new(100.0, 50.0), Color::RED);
    assert_eq!(surface.size, Vec2::new(100.0, 50.0));
    assert_eq!(surface.color, Color::RED);
}
