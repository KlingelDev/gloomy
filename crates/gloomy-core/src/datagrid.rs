//! DataGrid widget for displaying tabular data.
//!
//! The DataGrid provides a high-performance table widget with support
//! for sorting, selection, column resizing, and virtual scrolling.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::widget::{WidgetBounds, TextAlign, Color};

/// Column width specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnWidth {
    /// Fixed pixel width
    Fixed(f32),
    /// Flexible weight (proportional sizing)
    Flex(f32),
    /// Auto-size to content (future)
    Auto,
}

impl Default for ColumnWidth {
    fn default() -> Self {
        ColumnWidth::Flex(1.0)
    }
}

/// Column definition for DataGrid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    /// Column header text
    pub header: String,

    /// Field name in data source
    pub field: String,

    /// Column width specification
    #[serde(default)]
    pub width: ColumnWidth,

    /// Text alignment in cells
    #[serde(default)]
    pub align: TextAlign,

    /// Whether this column can be sorted
    #[serde(default = "default_true")]
    pub sortable: bool,

    /// Whether this column can be resized
    #[serde(default = "default_true")]
    pub resizable: bool,

    /// Minimum width when resizing
    #[serde(default = "default_min_width")]
    pub min_width: f32,

    /// Maximum width when resizing
    #[serde(default)]
    pub max_width: Option<f32>,
}

impl ColumnDef {
    /// Creates a new column definition.
    pub fn new(header: impl Into<String>, field: impl Into<String>) -> Self {
        Self {
            header: header.into(),
            field: field.into(),
            width: ColumnWidth::default(),
            align: TextAlign::default(),
            sortable: true,
            resizable: true,
            min_width: default_min_width(),
            max_width: None,
        }
    }

    /// Sets the column width.
    pub fn width(mut self, width: ColumnWidth) -> Self {
        self.width = width;
        self
    }

    /// Sets the text alignment.
    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets whether the column is sortable.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }
}

/// Selection mode for DataGrid rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// No selection allowed
    None,
    /// Single row selection
    Single,
    /// Multiple row selection
    Multiple,
}

impl Default for SelectionMode {
    fn default() -> Self {
        SelectionMode::None
    }
}

/// Sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascending order (A-Z, 0-9)
    Ascending,
    /// Descending order (Z-A, 9-0)
    Descending,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

/// Style configuration for DataGrid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGridStyle {
    /// Header background color
    #[serde(default = "default_header_bg")]
    pub header_background: Color,

    /// Header text color
    #[serde(default = "default_header_text")]
    pub header_text_color: Color,

    /// Even row background
    #[serde(default = "default_row_bg")]
    pub row_background: Color,

    /// Odd row background (for striping)
    #[serde(default = "default_alt_row_bg")]
    pub alt_row_background: Color,

    /// Row text color
    #[serde(default = "default_row_text")]
    pub row_text_color: Color,

    /// Hover row background
    #[serde(default = "default_hover_bg")]
    pub hover_background: Color,

    /// Selected row background
    #[serde(default = "default_selected_bg")]
    pub selected_background: Color,

    /// Grid line color
    #[serde(default = "default_grid_line")]
    pub grid_line_color: Color,

    /// Grid line width
    #[serde(default = "default_grid_line_width")]
    pub grid_line_width: f32,

    /// Cell padding
    #[serde(default = "default_cell_padding")]
    pub cell_padding: f32,
}

impl Default for DataGridStyle {
    fn default() -> Self {
        Self {
            header_background: default_header_bg(),
            header_text_color: default_header_text(),
            row_background: default_row_bg(),
            alt_row_background: default_alt_row_bg(),
            row_text_color: default_row_text(),
            hover_background: default_hover_bg(),
            selected_background: default_selected_bg(),
            grid_line_color: default_grid_line(),
            grid_line_width: default_grid_line_width(),
            cell_padding: default_cell_padding(),
        }
    }
}

/// DataGrid widget structure.
///
/// Internal state fields (scroll_offset, selected_rows, etc.) are not
/// serialized and are managed at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGrid {
    /// Widget bounds (position and size)
    #[serde(default)]
    pub bounds: WidgetBounds,

    /// Column definitions
    pub columns: Vec<ColumnDef>,

    /// Data source identifier (for runtime binding)
    #[serde(default)]
    pub data_source_id: Option<String>,

    /// Header row height in pixels
    #[serde(default = "default_header_height")]
    pub header_height: f32,

    /// Data row height in pixels
    #[serde(default = "default_row_height")]
    pub row_height: f32,

    /// Show alternating row colors
    #[serde(default = "default_true")]
    pub striped: bool,

    /// Row selection mode
    #[serde(default)]
    pub selection_mode: SelectionMode,

    /// Show vertical grid lines between columns
    #[serde(default = "default_true")]
    pub show_vertical_lines: bool,

    /// Show horizontal grid lines between rows
    #[serde(default = "default_true")]
    pub show_horizontal_lines: bool,

    /// Style configuration
    #[serde(default)]
    pub style: DataGridStyle,

    // --- Runtime state (not serialized) ---
    /// Current vertical scroll offset in pixels
    #[serde(skip)]
    pub scroll_offset: f32,

    /// Currently sorted column index
    #[serde(skip)]
    pub sort_column: Option<usize>,

    /// Current sort direction
    #[serde(skip)]
    pub sort_direction: SortDirection,

    /// Set of selected row indices
    #[serde(skip)]
    pub selected_rows: HashSet<usize>,

    /// Currently hovered row index
    #[serde(skip)]
    pub hovered_row: Option<usize>,

    /// Calculated column widths (computed during layout)
    #[serde(skip)]
    pub(crate) calculated_column_widths: Vec<f32>,

    // --- Layout fields ---
    /// Flex weight for layout
    #[serde(default)]
    pub flex: f32,

    /// Grid column position
    #[serde(default)]
    pub grid_col: Option<usize>,

    /// Grid row position
    #[serde(default)]
    pub grid_row: Option<usize>,

    /// Column span in grid layout
    #[serde(default = "default_span_one")]
    pub col_span: usize,

    /// Row span in grid layout
    #[serde(default = "default_span_one")]
    pub row_span: usize,
}

impl DataGrid {
    /// Creates a new DataGrid with the given columns.
    pub fn new(columns: Vec<ColumnDef>) -> Self {
        Self {
            bounds: WidgetBounds::default(),
            columns,
            data_source_id: None,
            header_height: default_header_height(),
            row_height: default_row_height(),
            striped: true,
            selection_mode: SelectionMode::default(),
            show_vertical_lines: true,
            show_horizontal_lines: true,
            style: DataGridStyle::default(),
            scroll_offset: 0.0,
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            selected_rows: HashSet::new(),
            hovered_row: None,
            calculated_column_widths: Vec::new(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
        }
    }
}

// --- Default value functions ---

fn default_true() -> bool {
    true
}

fn default_span_one() -> usize {
    1
}

fn default_min_width() -> f32 {
    50.0
}

fn default_header_height() -> f32 {
    40.0
}

fn default_row_height() -> f32 {
    32.0
}

fn default_header_bg() -> Color {
    (0.2, 0.2, 0.2, 1.0)
}

fn default_header_text() -> Color {
    (0.9, 0.9, 0.9, 1.0)
}

fn default_row_bg() -> Color {
    (0.12, 0.12, 0.12, 1.0)
}

fn default_alt_row_bg() -> Color {
    (0.15, 0.15, 0.15, 1.0)
}

fn default_row_text() -> Color {
    (0.85, 0.85, 0.85, 1.0)
}

fn default_hover_bg() -> Color {
    (0.25, 0.25, 0.3, 1.0)
}

fn default_selected_bg() -> Color {
    (0.3, 0.4, 0.6, 1.0)
}

fn default_grid_line() -> Color {
    (0.3, 0.3, 0.3, 1.0)
}

fn default_grid_line_width() -> f32 {
    1.0
}

fn default_cell_padding() -> f32 {
    8.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_def_builder() {
        let col = ColumnDef::new("Name", "name")
            .width(ColumnWidth::Fixed(200.0))
            .align(TextAlign::Left)
            .sortable(true);

        assert_eq!(col.header, "Name");
        assert_eq!(col.field, "name");
        assert!(matches!(col.width, ColumnWidth::Fixed(200.0)));
        assert!(col.sortable);
    }

    #[test]
    fn test_datagrid_creation() {
        let columns = vec![
            ColumnDef::new("ID", "id"),
            ColumnDef::new("Name", "name"),
        ];

        let grid = DataGrid::new(columns);

        assert_eq!(grid.columns.len(), 2);
        assert_eq!(grid.header_height, 40.0);
        assert_eq!(grid.row_height, 32.0);
        assert!(grid.striped);
    }
}
