# Tab Widget

The `Tab` widget provides a container for displaying multiple pages of content, accessible via interactive tabs. It supports horizontal (top) and vertical (left/right) orientations.

## Usage

```rust
use gloomy_core::widget::{Widget, TabItem, TabStyle, Orientation};

let tabs = Widget::tab(
    "my_tabs",
    vec![
        TabItem { title: "Page 1".into(), content: Box::new(Widget::label("First Page")) },
        TabItem { title: "Page 2".into(), content: Box::new(Widget::label("Second Page")) },
    ],
    Orientation::Horizontal,
    TabStyle::default(),
);
```

## Configuration

### Orientation
Use the `Orientation` enum to control tab placement:
- `Orientation::Horizontal`: Tabs are displayed at the top.
- `Orientation::Vertical`: Tabs are displayed on the left side.

### Styling
Customize the appearance using `TabStyle`:

```rust
use gloomy_core::widget::TabStyle;
use gloomy_core::theme::Color;

let style = TabStyle {
    background: (0.1, 0.1, 0.1, 1.0),
    selected_color: (0.2, 0.2, 0.25, 1.0),
    unselected_color: (0.15, 0.15, 0.15, 1.0),
    text_color: (1.0, 1.0, 1.0, 1.0),
    border_color: (0.3, 0.3, 0.3, 1.0),
    border_width: 1.0,
    header_height: 32.0, // Height for horizontal or width for vertical tabs
};
```

## Interaction
The `Tab` widget handles click interactions on tab headers internally if `handle_interactions` is called on the widget tree. It updates its `selected` index transparently.

For application-level state management, you can monitor the clicked ID, which follows the format `"{id}:tab:{index}"` (e.g., `"my_tabs:tab:1"`), and update the selected index manually if needed (as shown in `examples/tab_widget_demo.rs`).
