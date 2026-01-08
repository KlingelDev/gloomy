# Theming System

Gloomy features a robust, runtime-switchable theming system based on semantic colors.

## Concepts

### Semantic Palette
Instead of hardcoding colors (e.g., `(0.1, 0.1, 0.1)`), widgets refer to semantic names like `background`, `primary`, or `error`. This ensures consistency and makes theme switching effortless.

Key categories:
- **Base**: `background`, `surface` (cards/panels).
- **Brand**: `primary`, `secondary`, `accent`.
- **Text**: `text` (primary), `text_secondary` (muted), `text_disabled`.
- **Feedback**: `success` (green), `warning` (orange), `error` (red), `info` (blue).
- **UI**: `border`, `divider`, `hover`, `active`, `focus`.

### Theme Structure

```rust
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
}
```

## Built-in Themes

Gloomy comes with three standard themes:

1.  **Dark** (Default): High contrast dark mode.
2.  **Light**: Standard light mode.
3.  **High Contrast**: Pure black/white/primary for accessibility.

## Using Themes

### In Code

Access colors via the `Theme` struct:

```rust
let theme = Theme::dark();
let my_color = theme.colors.primary;
```

### In RON Files

Themes can be loaded from `.ron` files.

**Example `themes/custom.ron`:**
```ron
(
    name: "Dracula",
    colors: (
        background: (0.15, 0.16, 0.21, 1.0),
        surface: (0.26, 0.27, 0.35, 1.0),
        primary: (0.74, 0.47, 0.85, 1.0), // Purple
        // ... all other fields ...
    )
)
```

## Runtime Switching

To switch themes at runtime:
1.  Store the current `Theme` in your `AppState`.
2.  Pass the theme (or derived colors) to your widgets during the `view` function.
3.  Update the `Theme` in `AppState` when a user selects a new one.

See `examples/theme_switcher.rs` for a complete implementation.
