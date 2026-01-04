# Gloomy Architecture

Gloomy is a GPU-accelerated, keyboard-centric UI library designed for high-performance rendering of flat 3D UI elements. It leverages `wgpu` for rendering and `winit` for window management, providing a modular and declarative approach to building user interfaces.

## High-Level Structure

The project is organized as a Cargo workspace with three primary crates, fostering a clear separation of concerns:

### 1. `gloomy-core`
The foundation of the library, containing all rendering logic, primitives, and state management.
- **Rendering Primitives**: SDF (Signed Distance Field) based rendering for resolution-independent shapes (rectangles, circles, lines).
- **Widget System**: A rich set of widgets (`Container`, `Label`, `Button`, `DataGrid`, `KpiCard`) defined via the `Widget` enum.
- **Layout Engine**: A Flexbox-inspired and Grid-based layout system (`Layout`, `Align`, `Justify`, `compute_layout`).
- **Styling**: Comprehensive styling support including borders, shadows, gradients, and themes (`Style`, `Theme`).
- **Text Rendering**: High-quality text rendering powered by `wgpu_text`.
- **Data Binding**: interfaces for data-driven components (`DataSource`, `DataGrid`).

### 2. `gloomy-app`
The application shell and runtime layer.
- **Window Management**: Wraps `winit` to handle window creation, lifecycle, and input events.
- **App Lifecycle**: Provides a builder-pattern `GloomyApp` struct to configure callbacks (`on_draw`, `on_update`,Input handlers).
- **Runtime**: Manages the main event loop and bridges OS events to the `core` library.

### 3. `gloomy-designer`
A visual tool for designing Gloomy UIs.
- Allows for visual layout and editing of UI components.
- Likely produces serialization formats (e.g., RON) compatible with `core`'s data loading.

## Rendering Pipeline

Gloomy uses a retained-mode style API for defining the UI tree, but processes it in an immediate-mode fashion during the render pass.

1.  **Tree Construction**: The user's application constructs a `Widget` tree in the `on_draw` callback. This tree describes the structure, style, and content of the UI.
2.  **Layout Calculation**: `gloomy_core::compute_layout` traverses the widget tree to calculate the position and size (`WidgetBounds`) of each element based on constraints and flex/grid rules.
3.  **Primitive Generation**: The `render_ui` function flattens the widget tree into a list of drawing primitives (instances).
4.  **GPU Upload**: Instance data (positions, colors, sizes, SDF parameters) is uploaded to GPU buffers.
5.  **Shading**: specialized shaders (`primitives.wgsl`) use SDF math to render shapes with anti-aliasing, rounded corners, and soft shadows in a single pass per primitive type.

## Key Concepts

### Widgets & Composition
Everything is a `Widget`. Composition is achieved by nesting widgets within `Container` widgets. The `Widget` enum handles dispatching for layout and rendering. This avoids complex inheritance hierarchies.

### Layout System
The layout engine supports:
- **Flexbox**: Row/Column direction, alignment, justification, and `flex` growing/shrinking.
- **Grid**: Explicit column/row placement and spanning.
- **Padding & Spacing**: usage of standard box model concepts.

### Interactivity
Input handling is centralized in `gloomy-app` and propagated via `InteractionState`. The `hit_test` function in `core` associates mouse/cursor positions with specific widgets to handle hover and click states.

### Data Flow
- **State**: Check `gloomy-app` callbacks manage application state (`AppState`).
- **UI**: The UI is a function of this state.
- **Events**: Input events mutate the state, triggering a new UI generation/render cycle.


## High-Performance Data Strategy (Excel-like Workloads)

For applications handling millions of rows (e.g., large data grids), Gloomy employs a specific architecture to avoid performance pitfalls:

1.  **Columnar Storage**: Data sources should store data in column-major format (e.g., `Vec<Column>`) rather than row-major. This allows for SIMD-friendly bulk updates (e.g., "multiply column A by 1.5").
2.  **Atomic Swaps (Double Buffering)**:
    -   The UI renders from an immutable snapshot (`Arc<DataSource>`).
    -   Updates happen on a background thread, cloning/mutating the data structure.
    -   The main thread atomically calls `swap()` to replace the pointer.
3.  **Versioning**: Data sources implement a `version()` method. The UI only re-fetches data if the version has changed.
4.  **View Virtualization**: The `DataGrid` widget uses the `DataSource` to render *only* the visible rows, ensuring rendering cost is O(Visible) rather than O(Total).


## Dynamic Font Loading

To support user-provided TTF fonts at runtime, Gloomy must work around the immutability of the underlying `wgpu_text` brush:

1.  **Limitation**: The `TextBrush` cannot accept new fonts after creation.
2.  **Strategy**: "Stop-the-world" Rebuild.
    -   When `add_font(bytes)` is called, the renderer parses the new font.
    -   it appends it to the internal list of `FontArc` instances.
    -   It **destroys** the old `TextBrush` and **builds a new one** with the updated font list.
3.  **Performance**: This is an expensive operation (re-allocating texture atlases), so it should only occur on specific user actions (e.g., "Import Font"), but subsequent rendering remains performant.

## Dependencies

- **`wgpu`**: Cross-platform GPU API.
- **`winit`**: Window creation and event loop.
- **`glam`**: Vector math types (`Vec2`, `Vec4`).
- **`wgpu_text`**: Text rasterization and caching.
- **`ron`**: Rusty Object Notation for serialization/deserialization of UI layouts.
