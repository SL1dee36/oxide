# Module `text` — Typography & Text Rendering

The `text` module delivers high-performance vector font rasterization (TTF/OTF), texture atlas packing, and batched text rendering with complete Unicode Cyrillic and Latin typography support.

Source files: [`src/text/`](file:///e:/PRJCT/Oxide/src/text/)

---

## 1. Font Loading (`Font`)

Powered by `fontdue` for zero-allocation glyph extraction:

```rust
// Load from file path
let custom_font = Font::from_file("assets/fonts/my_font.ttf")?;

// Load from in-memory byte slice (e.g. include_bytes!)
let mem_font = Font::from_bytes(FONT_BYTES)?;

// Fallback search across project assets and OS font paths
let default_font = Font::default_font()?;
```

---

## 2. The Glyph Texture Atlas (`FontAtlas`)

When calling `FontAtlas::new(gl, font, font_size)`:
1. Allocates a `1024x1024` texture canvas.
2. Rasterizes a wide character set:
   - Full ASCII printable range (`32..=126`).
   - Extended Cyrillic (`0x0400..=0x045F`: А-Я, а-я, Ё, ё).
   - Typographic symbols (`№`, `«`, `»`, `—`, `–`, `°`, `±`, `•`, `…`).
3. Packs glyphs using a shelf-packing algorithm.
4. Generates an RGBA8 `Texture2D` where the alpha channel encodes glyph subpixel coverage.

### Measuring Text Dimensions:
```rust
let dimensions: Vec2 = atlas.measure_text("Text string to measure width and height");
```

---

## 3. Text Descriptor (`Text2D`)

Declarative structure for UI and scene text:

```rust
let text = Text2D::new("Status: Active")
    .with_color(Color::rgb(0.2, 0.8, 1.0))
    .with_scale(1.2);
```

---

## 4. Text Render Pipeline (`PipelineText`)

- **Shader Model:** Computes fragment opacity from the atlas alpha channel: `o_color = vec4(u_color.rgb, u_color.a * texture(u_atlas, v_uv).a)`.
- **Dynamic Batching:** Combines all characters into a single vertex/index buffer (4 vertices and 6 indices per glyph), issuing a single `glDrawElements` call per text sequence.
- **Scene Integration:**
  ```rust
  // Assign custom font atlas to scene
  scene.set_font_atlas(Arc::new(atlas));

  // Spawn text node
  let node_id = scene.spawn_text_2d(text, Transform::from_xyz(40.0, 50.0, 0.0));
  ```
