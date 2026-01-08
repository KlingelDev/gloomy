# Tree Widget

The `Tree` widget displays hierarchical data in an expandable/collapsible format. It supports custom styling, selection, and arbitrary depth.

## Usage

```rust
use gloomy_core::widget::{Widget, WidgetBounds};
use gloomy_core::tree::{TreeNode, TreeStyle};

let root_nodes = vec![
    TreeNode::new("node1", "Root Node 1")
        .with_child(TreeNode::new("child1", "Child 1"))
        .with_child(TreeNode::new("child2", "Child 2")),
    TreeNode::new("node2", "Root Node 2"),
];

Widget::Tree {
    id: Some("my_tree".to_string()),
    bounds: WidgetBounds::default(),
    root_nodes,
    selected_id: None, // Manage state externally
    expanded_ids: std::collections::HashSet::new(), // Manage state externally
    style: TreeStyle::default(),
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
}
```

## Features

- **Recursive Structure**: Nodes can have any number of children.
- **Selection**: Track the selected node via `selected_id`.
- **Expansion**: Track expanded nodes via `expanded_ids`.
- **Rich Text**: Node labels support Rich Text formatting (e.g., `<color="#FF0000">Red</color> Node`).
- **Icons**: Supports custom icons for leaf and folder nodes (via style or rich text).

## Interaction

The tree generates the following interaction IDs:
- **Toggle Expansion**: `"{tree_id}:toggle:{node_id}"`
- **Select Node**: `"{tree_id}:select:{node_id}"`

Your application's event loop should handle these actions by updating the `expanded_ids` set or `selected_id` field in the next frame.

## Styling

Customize the tree appearance with `TreeStyle`:

```rust
pub struct TreeStyle {
    pub background: Option<Color>,
    pub text_color: Color,
    pub selected_background: Color,
    pub selected_text_color: Color,
    pub hover_background: Color,
    pub indent_size: f32,
    pub row_height: f32,
    pub font: Option<String>,
    // ...
}
```
