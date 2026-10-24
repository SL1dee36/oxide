use oxide::prelude::*;
use oxide::renderer::buffer::{IndexBuffer, VertexBuffer};

#[test]
fn test_font_loading_from_file_or_fallback() {
    let font_res = Font::default_font();
    assert!(font_res.is_ok(), "Default or local font should load successfully");
}

#[test]
fn test_font_metrics_and_rasterization() {
    let font = Font::default_font().expect("Font should load");
    let (metrics_a, bitmap_a) = font.inner().rasterize('A', 32.0);
    assert!(metrics_a.width > 0);
    assert!(metrics_a.height > 0);
    assert!(!bitmap_a.is_empty());

    // Test Cyrillic character rasterization
    let (metrics_ya, bitmap_ya) = font.inner().rasterize('Я', 32.0);
    assert!(metrics_ya.width > 0);
    assert!(metrics_ya.height > 0);
    assert!(!bitmap_ya.is_empty());
}

#[test]
fn test_buffer_capacity_tracking() {
    let vbo = VertexBuffer::dummy_for_test(1024);
    assert_eq!(vbo.count(), 1024);
    assert_eq!(vbo.capacity(), 1024);

    let ebo = IndexBuffer::dummy_for_test(1536);
    assert_eq!(ebo.count(), 1536);
    assert_eq!(ebo.capacity(), 1536);
}
