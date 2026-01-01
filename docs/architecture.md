# Gloomy Architecture

Gloomy is a keyboard-centric wgpu-based UI library for rendering flat 3D 
UI elements, optimized for displaying large datasets.

## Crate Structure

```
gloomy/
├── Cargo.toml              # Workspace manifest
├── crates/
│   ├── gloomy-core/        # Core rendering primitives
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── renderer.rs     # GPU context orchestration
│   │       ├── primitives.rs   # Instanced SDF shapes
│   │       └── shaders/
│   │           └── primitives.wgsl
│   └── gloomy-app/         # Window management
│       └── src/
│           ├── lib.rs
│           ├── app.rs          # Event loop, multi-window
│           └── window.rs       # wgpu surface management
└── examples/
    └── hello_gloomy.rs     # Basic demo
```

## Rendering Pipeline

1. **Primitive Batching**: Shapes are queued via `draw_rect()`, `draw_circle()`,
   `draw_line()` calls, stored in an instance buffer.

2. **GPU Upload**: `prepare()` uploads instance data to GPU.

3. **SDF Rendering**: Fragment shader uses signed distance functions for 
   anti-aliased shapes at any resolution.

## Key Design Patterns

- **Instanced Drawing**: Single draw call for all primitives of each type
- **SDF Shapes**: Resolution-independent crisp edges
- **Minimal API**: Builder pattern for app configuration

## Dependencies

| Crate    | Version | Purpose            |
|----------|---------|---------------------|
| wgpu     | 0.20    | GPU abstraction     |
| winit    | 0.29    | Window management   |
| glam     | 0.25    | Math (Vec2, Vec4)   |
| bytemuck | 1.14    | Safe transmutes     |

## Future Work

- Text rendering (wgpu_text integration)
- Widget system (buttons, tables, text input)
- Layout engine (flexbox-style)
- Multi-window spawn API
