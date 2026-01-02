# DataGrid Widget

The `DataGrid` is a high-performance, feature-rich table widget for the Gloomy UI framework. It supports displaying tabular data with capabilities for sorting, selection, column resizing, and virtual scrolling.

## Key Features

- **Virtual Scrolling**: Efficiently renders only the visible rows, capable of handling large datasets (thousands of rows) with minimal performance impact.
- **Sorting**: Interactive column sorting (Ascending/Descending) with visual indicators (▲/▼).
- **Column Resizing**: Interactive resizing of columns by dragging the separators between headers.
- **Selection**: Support for single row selection with visual highlighting.
- **Custom Styling**: Fully customizable colors for headers, rows, striping, selection, and grid lines.
- **Flexible Layout**: Supports mixed `Fixed` and `Flex` column widths.

## Usage

### 1. Define Data Source

Implement the `DataSource` trait or use the provided `VecDataSource`.

```rust
use gloomy_core::data_source::{VecDataSource, CellValue};

let columns = vec!["ID".to_string(), "Name".to_string()];
let rows = vec![
    vec![CellValue::Integer(1), CellValue::Text("Alice".to_string())],
    vec![CellValue::Integer(2), CellValue::Text("Bob".to_string())],
];

let data_source = VecDataSource::new(columns, rows);
```

### 2. Register Data Provider

Use `MapDataProvider` to make sources available to the UI.

```rust
use gloomy_core::data_source::MapDataProvider;

let mut provider = MapDataProvider::new();
provider.register("users", data_source);
```

### 3. Define Widget

Create the `DataGrid` widget definition.

```rust
use gloomy_core::widget::{Widget, WidgetBounds, TextAlign};
use gloomy_core::datagrid::{ColumnDef, ColumnWidth, DataGridStyle};

Widget::DataGrid {
    id: Some("main_grid".to_string()),
    bounds: WidgetBounds::default(),
    columns: vec![
        ColumnDef::new("ID", "ID").width(ColumnWidth::Fixed(50.0)),
        ColumnDef::new("Name", "Name").width(ColumnWidth::Flex(1.0)),
    ],
    data_source_id: Some("users".to_string()),
    header_height: 30.0,
    row_height: 25.0,
    striped: true,
    selection_mode: gloomy_core::datagrid::SelectionMode::Single,
    selected_rows: vec![],
    sort_column: None,    // Manage state externally
    sort_direction: None, // Manage state externally
    show_vertical_lines: true,
    show_horizontal_lines: true,
    style: DataGridStyle::default(),
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
}
```

## Interaction Handling

The `DataGrid` generates specific action strings during hit testing for interaction handling:

- **Row Click**: `"{widget_id}:row:{row_index}"`
- **Header Click**: `"{widget_id}:header:{col_index}"`
- **Header Resize**: `"{widget_id}:header_resize:{col_index}"`

### Example Event Loop

See `examples/simple_datagrid.rs` for a complete implementation of:
- Handling clicks to select rows.
- Handling header clicks to trigger `DataSource::sort`.
- Handling drag events to update `ColumnWidth`.
- Managing scroll state via `scroll_offsets`.

## Architecture details

- **Layout**: The grid calculates visible range based on `scroll_offset` and viewport height.
- **Rendering**: Primitives (rectangles, lines) and Text are batched. Text rendering uses `wgpu_text`.
- **State**: `DataGrid` is largely stateless; state (selection, sort, column widths) is passed in via the `Widget` enum fields, typically managed by the application loop.
