# Widget Overview

Gloomy provides a comprehensive set of widgets for building data-driven UIs.

## Layout & Containers

- **[Container](container.md)**: The fundamental building block. Supports Flexbox and Grid layouts, padding, margins, borders, and shadows.
- **Divider**: Visual separator (horizontal or vertical).
- **Spacer**: Empty space for layout adjustments.
- **Scrollbar**: Interactive scrollbar for containers (typically managed automatically).
- **Tab**: Tabbed container for switching views. [Read more](tab.md).

## Data Display

- **Label**: Basic text display.
- **[DataGrid](datagrid.md)**: High-performance table for tabular data with sorting, resizing, and virtual scrolling.
- **[Tree](tree.md)**: Hierarchical data display with expandable nodes.
- **ListView**: Simple list of items.
- **KpiCard**: Specialized card for analytics dashboards showing key performance indicators and trends.
- **Image**: Display images from file paths.
- **Icon**: Display vector icons (if supported/loaded).

## Input & Interaction

- **Button**: Clickable button.
- **TextInput**: Single-line text entry.
- **NumberInput**: Numeric entry with optional spinners.
- **Autocomplete**: Text input with a dropdown of suggestions.
- **DatePicker**: Date selection with a calendar popup.
- **Checkbox**: Boolean toggle (box).
- **ToggleSwitch**: Boolean toggle (switch).
- **RadioButton**: Mutually exclusive selection.
- **Dropdown**: Select one option from a list.
- **Slider**: Select a value from a continuous range.
- **ProgressBar**: Visual indicator of progress.

## Common Properties

All widgets share common layout properties defined in `Widget`:
- `id`: Optional unique identifier for interaction and finding.
- `bounds`: Calculated position and size (managed by layout engine).
- `flex`: Flex grow/shrink factor.
- `grid_col` / `grid_row`: Explicit placement in Grid layout.
- `col_span` / `row_span`: Spanning multiple cells in Grid layout.
