# Gloomy Examples

This directory contains working examples demonstrating various features of the Gloomy UI library.

## Running Examples

All examples can be run using:
```bash
cargo run --example <example_name>
```

## Available Examples

### Basic Examples

- **`simple_starter`** - Simple counter app demonstrating basic button and label usage
  ```bash
  cargo run --example simple_starter
  ```

- **`hello_gloomy`** - Low-level API demonstration with manual container drawing
  ```bash
  cargo run --example hello_gloomy
  ```

### Widget Showcases

- **`basic_components`** - Interactive showcase of all basic widgets
  ```bash
  cargo run --example basic_components
  ```

- **`widgets_showcase`** - RON-based widget showcase (requires `examples/ui/widgets_demo.ron`)
  ```bash
  cargo run --example widgets_showcase
  ```

- **`widgets_ui`** - Another widget demonstration from RON file
  ```bash
  cargo run --example widgets_ui
  ```

### Layout & Styling

- **`grid_ui`** - Grid layout demonstration
  ```bash
  cargo run --example grid_ui
  ```

- **`borders_showcase`** - Border and styling examples
  ```bash
  cargo run --example borders_showcase
  ```

- **`visuals_demo`** - Shadows, gradients, and visual effects
  ```bash
  cargo run --example visuals_demo
  ```

- **`style_ui`** - Comprehensive styling demonstration
  ```bash
  cargo run --example style_ui
  ```

### Interactive Examples

- **`form_ui`** - Form with text inputs and validation
  ```bash
  cargo run --example form_ui
  ```

- **`scroll_ui`** - Scrollable containers
  ```bash
  cargo run --example scroll_ui
  ```

- **`table_ui`** - Table/grid data display
  ```bash
  cargo run --example table_ui
  ```

- **`simple_datagrid`** - Feature-rich DataGrid with sorting, resizing, scrolling
  ```bash
  cargo run --example simple_datagrid
  ```

### RON-based Examples

- **`ron_ui`** - Loading UI from RON files
  ```bash
  cargo run --example ron_ui
  ```

## Development Tips

### Building All Examples
```bash
cargo build --examples
```

### Running with Logging
```bash
RUST_LOG=debug cargo run --example simple_starter
```

### Creating a New Example

1. Create a new file in `examples/your_example.rs`
2. Follow the pattern from `simple_starter.rs`:
   - Use the callback-based API (`on_draw`, `on_mouse_input`, etc.)
   - Manage state with `Rc<RefCell<AppState>>`
   - Call `compute_layout` before rendering
   - Use `render_ui` to draw widgets

3. Build and test:
   ```bash
   cargo run --example your_example
   ```

## Common Patterns

### State Management
```rust
let state = Rc::new(RefCell::new(AppState {
    ui_root: create_ui(),
    interaction: InteractionState::default(),
    // your app state...
}));
```

### Click Handling
```rust
.on_mouse_input(move |win, elem_state, _btn| {
    let mut s = state.borrow_mut();
    if elem_state == ElementState::Pressed {
        if let Some(res) = hit_test(&s.ui_root, mouse_pos, ...) {
            s.handle_click(&res.action);
        }
    }
})
```

### Rendering
```rust
.on_draw(move |win, ctx| {
    let mut s = state.borrow_mut();
    compute_layout(&mut s.ui_root, width, height);
    render_ui(&s.ui_root, ..., Some(&s.interaction));
})
```

## Troubleshooting

**Example won't compile:**
- Make sure all examples are using the `font: None` field for Labels and Buttons
- Check that you're using the correct API (callback-based, not trait-based)

**Example window is blank:**
- Ensure `compute_layout` is called before `render_ui`
- Check that widget bounds are reasonable (not 0x0 or outside window)

**Clicks not working:**
- Verify `hit_test` is being called on mouse movement
- Check that `handle_interactions` is called in the draw loop
