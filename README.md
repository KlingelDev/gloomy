# Gloomy UI

A modern, GPU-accelerated immediate-mode UI library for Rust, built on top of `wgpu` and `winit`. Gloomy provides a declarative, RON-based approach to building beautiful user interfaces with a focus on performance and flexibility.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## ✨ Features

- **🎨 Theming System** - Comprehensive theme support with semantic colors, global styles, and live theme switching
- **📦 Declarative UI** - Define UIs using RON files or programmatically with a clean Rust API
- **⚡ GPU-Accelerated** - Hardware-accelerated rendering using `wgpu` with SDF-based primitives
- **🖼️ Rich Widgets** - Buttons, labels, inputs, checkboxes, sliders, dropdowns, and more
- **📐 Flexible Layouts** - Flexbox-style layout engine with Row, Column, and Grid support
- **🎯 Interactive** - Full mouse and keyboard input handling with focus management
- **📜 Scrolling** - Scrollable containers with automatic overflow handling
- **🎭 Advanced Styling** - Borders, shadows, gradients, corner radii (individual or uniform)
- **🔤 Text Rendering** - TTF font support with multi-font capability
- **🖼️ SVG Support** - Vector graphics rendering
- **📏 Dividers** - Visual separators for horizontal and vertical layouts
- **🎨 Custom Widgets** - Easy to extend with custom widget types

## 🚀 Quick Start

### Installation

Add Gloomy to your `Cargo.toml`:

```toml
[dependencies]
gloomy-app = "0.1.0"
gloomy-core = "0.1.0"
```

### Hello World

```rust
use gloomy_app::GloomyApp;
use gloomy_core::{Widget, layout::*, ui::*, Vec2};

fn main() -> anyhow::Result<()> {
    let mut ui = Widget::Container {
        background: Some((0.12, 0.12, 0.12, 1.0)),
        padding: 20.0,
        layout: Layout {
            direction: Direction::Column,
            spacing: 10.0,
            ..Default::default()
        },
        children: vec![
            Widget::label("Hello, Gloomy!"),
            Widget::Button {
                text: "Click Me".to_string(),
                action: "button_click".to_string(),
                background: (0.3, 0.6, 1.0, 1.0),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    GloomyApp::new()
        .on_draw(move |win, ctx| {
            let size = win.renderer.size();
            compute_layout(&mut ui, 0.0, 0.0, size.x, size.y);
            render_ui(&ui, &mut win.renderer, ctx.device, ctx.queue, None);
        })
        .run()
}
```

## 📚 Examples

Gloomy comes with numerous examples demonstrating different features:

### Running Examples

```bash
# Theme switching demo
cargo run --example theme_switcher

# Divider widget showcase
cargo run --example divider_demo

# Basic widgets
cargo run --example simple_starter

# Border styles
cargo run --example borders_showcase

# Complete widget showcase
cargo run --example widgets_ui

# Grid layouts
cargo run --example grid_ui

# Scrollable content
cargo run --example scroll_ui
```

See the [examples/README.md](examples/README.md) for a complete list and descriptions.

## 🎨 Theming

Gloomy includes a powerful theming system with semantic colors and global style defaults.

### Using Themes

```rust
use gloomy_core::{Theme, StyleContext};

// Use preset themes
let ctx = StyleContext::default(); // Dark theme + Modern style

// Or load from RON files
let theme = Theme::load("themes/dark.ron")?;
let ctx = StyleContext::new(theme, GlobalStyle::default());

// Switch themes at runtime
ctx.set_theme(Theme::light());
```

### Built-in Themes

- **Dark** - Modern dark theme (default)
- **Light** - Clean light theme
- **High Contrast** - Accessibility-focused high contrast theme

### Style Presets

- **Modern** - Smooth corners, subtle shadows
- **Classic** - Sharp edges, no shadows
- **Minimal** - Subtle, clean design

## 📐 Layout System

Gloomy uses a flexbox-inspired layout system:

```rust
Layout {
    direction: Direction::Column,  // or Row, Grid
    spacing: 16.0,
    align_items: Align::Center,
    justify_content: Justify::SpaceBetween,
    ..Default::default()
}
```

### Grid Layouts

```rust
Layout {
    direction: Direction::Grid { columns: 3 },
    template_columns: vec![1.0, 2.0, 1.0], // Proportional sizing
    spacing: 10.0,
    ..Default::default()
}
```

## 🎯 Widgets

### Available Widgets

- **Container** - Layout container with optional scrolling
- **Label** - Text display
- **Button** - Clickable button with hover/active states
- **TextInput** - Single-line text input
- **Checkbox** - Toggle checkbox
- **Slider** - Value slider
- **ToggleSwitch** - On/off switch
- **RadioButton** - Radio button groups
- **Dropdown** - Selection dropdown
- **ProgressBar** - Progress indicator
- **Image** - Raster image display
- **Icon** - Icon rendering
- **SVGImage** - Vector graphics
- **Divider** - Visual separator
- **DataGrid** - Feature-rich table with sorting, resizing, [Documentation](docs/datagrid.md)

### Widget Styling

All widgets support extensive styling options:

```rust
Widget::Button {
    background: (0.3, 0.6, 1.0, 1.0),
    hover_color: (0.4, 0.7, 1.0, 1.0),
    border: Some(Border {
        width: 2.0,
        color: (1.0, 1.0, 1.0, 0.3),
        ..Default::default()
    }),
    corner_radius: 8.0,
    shadow: Some(Shadow {
        offset: (0.0, 4.0),
        blur: 8.0,
        color: (0.0, 0.0, 0.0, 0.2),
    }),
    gradient: Some(Gradient {
        start: (0.3, 0.6, 1.0, 1.0),
        end: (0.2, 0.4, 0.8, 1.0),
    }),
    ..Default::default()
}
```

## 📂 Project Structure

```
gloomy/
├── crates/
│   ├── gloomy-core/       # Core UI rendering and widgets
│   ├── gloomy-app/        # Application framework and event loop
│   └── gloomy-designer/   # Visual UI designer (in development)
├── examples/              # Example applications
├── themes/                # Theme configuration files
├── styles/                # Style configuration files
└── docs/                  # Documentation
```

## 🏗️ Architecture

Gloomy is built on a modular architecture:

- **gloomy-core** - Core rendering, widgets, layout engine
  - `primitives` - SDF-based shape rendering
  - `text` - TTF text rendering via `wgpu-text`
  - `widget` - Widget definitions and types
  - `ui` - Rendering and interaction logic
  - `layout_engine` - Flexbox-style layout computation
  - `theme` - Theming system

- **gloomy-app** - Application framework
  - Window management via `winit`
  - Event loop and input handling
  - Callback-based API

## 🛠️ Development

### Building

```bash
# Build all crates
cargo build

# Build examples
cargo build --examples

# Run tests
cargo test

# Build documentation
cargo doc --open
```

### Code Style

Gloomy follows the Google Rust Style Guide:
- 79 character line limit
- Documentation comments above code
- Doxygen-compliant docstrings using `///`

### Unit Tests

Tests are located in the `tests/` directory:

```bash
cargo test
```

## 📖 Documentation

- [Examples README](examples/README.md) - Guide to all examples
- [Architecture](docs/architecture.md) - System architecture
- [Roadmap](docs/roadmap.md) - Future plans

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Guidelines

1. Follow the existing code style
2. Add tests for new features
3. Update documentation
4. Run `cargo fmt` and `cargo clippy` before committing

## 📄 License

- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

## 🙏 Acknowledgments

Built with:
- [wgpu](https://github.com/gfx-rs/wgpu) - Modern GPU API
- [winit](https://github.com/rust-windowing/winit) - Cross-platform windowing
- [wgpu-text](https://github.com/grovesNL/wgpu_text) - Text rendering
- [glam](https://github.com/bitshifter/glam-rs) - Math library
- [serde](https://serde.rs/) - Serialization
- [ron](https://github.com/ron-rs/ron) - RON format

---

**Status**: Active Development 🚧

Gloomy is under active development. The API may change between releases.
