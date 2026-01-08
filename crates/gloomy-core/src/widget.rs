//! Widget types for declarative UI definitions.
//!
//! Widgets can be deserialized from RON files for declarative UI layouts.

use crate::layout::Layout;
use std::cell::RefCell;
use serde::{Deserialize, Serialize};
use crate::validation::ValidationRule;
use chrono::NaiveDate;
use crate::style::{BoxStyle, ButtonStyle, TextInputStyle, ListViewStyle, Shadow, Gradient, Border, BorderStyle};

/// RGBA color as tuple for serde.
pub type Color = (f32, f32, f32, f32);

/// Widget bounds for positioning.
#[derive(Debug, Clone, Deserialize, Serialize, Default, Copy)]
pub struct WidgetBounds {
  #[serde(default)]
  pub x: f32,
  #[serde(default)]
  pub y: f32,
  #[serde(default)]
  pub width: f32,
  #[serde(default)]
  pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize, Default)]
pub enum TextAlign {
  #[default]
  Left,
  Center,
  Right,
}

/// Orientation for dividers and layout direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Orientation {
  /// Horizontal orientation (left to right)
  Horizontal,
  /// Vertical orientation (top to bottom)
  Vertical,
}

impl Default for Orientation {
  fn default() -> Self {
    Self::Horizontal
  }
}

/// Architecture: Layout Caching
/// Stores inputs and results of the last layout calculation.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LayoutCache {
    /// Input width constraint
    pub input_width: f32,
    /// Input height constraint
    pub input_height: f32,
    /// Parent x position (invalidation trigger)
    pub parent_x: f32,
    /// Parent y position (invalidation trigger)
    pub parent_y: f32,
    /// Resulting bounds
    pub result_bounds: WidgetBounds,
    /// Whether the cache is valid
    pub valid: bool,
}

/// Architecture: Render Caching
/// Stores a snapshot of the rendering commands produced by a widget tree.
#[derive(Clone, Default)]
pub struct RenderCache {
    pub primitives: Option<crate::primitives::PrimitiveSnapshot>,
    pub text: Option<crate::text::TextSnapshot>,
    pub images: Option<crate::image_renderer::ImageSnapshot>,
    pub base_offset: glam::Vec2,
}

// Implement Debug manually if needed, or omit Debug for Snapshots if they are large/complex.
// Assuming Snapshots implement Debug (I added Debug to them or they are simple enough).
// ImageSnapshot needs Clone (I added it). Primitive/Text had Debug?
// I added Debug to PrimitiveSnapshot and TextSnapshot.
// ImageSnapshot I added Clone but maybe not Debug?
// Let's implement non-derived Debug for RenderCache to be safe.
impl std::fmt::Debug for RenderCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderCache")
         .field("base_offset", &self.base_offset)
         .finish_non_exhaustive()
    }
}

/// A UI widget that can be rendered.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Widget {
  /// Container widget that can hold children.
  Container {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    scrollable: bool,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    style: BoxStyle,

    #[serde(default)]
    padding: f32,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
    #[serde(default)]
    children: Vec<Widget>,

    /// Internal cache for high-performance layout skipping.
    #[serde(skip)]
    layout_cache: Option<Box<LayoutCache>>,
    /// Internal cache for high-performance rendering.
    #[serde(skip)]
    render_cache: RefCell<Option<Box<RenderCache>>>,
  },

  /// Tab widget for switching between pages.
  Tab {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    tabs: Vec<TabItem>,
    #[serde(default)]
    selected: usize,
    #[serde(default)]
    orientation: Orientation,
    #[serde(default)]
    style: TabStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
    #[serde(default)]
    layout_cache: Option<Box<LayoutCache>>,
    #[serde(skip)]
    render_cache: RefCell<Option<Box<RenderCache>>>,
  },

  /// Text label widget.
  Label {
    text: String,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    width: f32,
    #[serde(default)]
    height: f32,
    #[serde(default = "default_font_size")]
    size: f32,
    #[serde(default = "default_color")]
    color: Color,
    #[serde(default)]
    text_align: TextAlign,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
    #[serde(default)]
    font: Option<String>,
  },

  /// Interactive button widget.
  Button {
    text: String,
    action: String,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    style: ButtonStyle,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    disabled: bool,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
    #[serde(default)]
    font: Option<String>,
  },

  /// List/Menu widget.
  ListView {
      #[serde(default)]
      id: String,
      items: Vec<String>,
      #[serde(default)]
      selected_index: Option<usize>,
      #[serde(default)]
      style: ListViewStyle,

      #[serde(default)]
      bounds: WidgetBounds,
      #[serde(default)]
      width: Option<f32>,
      #[serde(default)]
      height: Option<f32>,
      #[serde(default)]
      layout: Layout,
      #[serde(default)]
      flex: f32,
      #[serde(default)]
      grid_col: Option<usize>,
      #[serde(default)]
      grid_row: Option<usize>,
      #[serde(default = "default_span_one")]
      col_span: usize,
      #[serde(default = "default_span_one")]
      row_span: usize,

      #[serde(skip)]
      scroll_offset: f32,
  },

  /// Tree Widget
  Tree {
      #[serde(default)]
      id: Option<String>,
      #[serde(default)]
      bounds: WidgetBounds,
      
      // Data
      #[serde(default)]
      root_nodes: Vec<crate::tree::TreeNode>,
      
      // State
      #[serde(default)]
      selected_id: Option<String>,
      #[serde(default)]
      expanded_ids: std::collections::HashSet<String>,
      
      // Style
      #[serde(default)]
      style: crate::tree::TreeStyle,
      
      // Layout
      #[serde(default)]
      flex: f32,
      #[serde(default)]
      grid_col: Option<usize>,
      #[serde(default)]
      grid_row: Option<usize>,
      #[serde(default = "default_span_one")]
      col_span: usize,
      #[serde(default = "default_span_one")]
      row_span: usize,
  },

  /// Toggle switch widget.
  ToggleSwitch {
    id: String,
    checked: bool,
    #[serde(default)]
    style: ToggleSwitchStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Progress bar widget.
  ProgressBar {
    value: f32,
    #[serde(default)]
    min: f32,
    #[serde(default)]
    max: f32,
    #[serde(default)]
    style: ProgressBarStyle,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Radio button widget.
  RadioButton {
    group_id: String,
    value: String,
    selected: bool,
    label: String,
    #[serde(default)]
    style: RadioButtonStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Dropdown widget.
  Dropdown {
    id: String,
    options: Vec<String>,
    #[serde(default)]
    selected_index: Option<usize>,
    #[serde(default)]
    expanded: bool,
    #[serde(default)]
    style: DropdownStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    width: Option<f32>,
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    layout: Layout,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Horizontal spacer.
  Spacer {
    #[serde(default = "default_spacer_size")]
    size: f32,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Visual divider for separating content.
  Divider {
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    orientation: Orientation,
    #[serde(default = "default_divider_thickness")]
    thickness: f32,
    #[serde(default = "default_divider_color")]
    color: Color,
    #[serde(default = "default_divider_margin")]
    margin: f32,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Scrollbar for indicating scroll position.
  Scrollbar {
    #[serde(default)]
    bounds: WidgetBounds,
    content_size: f32,
    viewport_size: f32,
    scroll_offset: f32,
    #[serde(default)]
    orientation: Orientation,
    #[serde(default)]
    style: ScrollbarStyle,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Data grid for displaying tabular data.
  DataGrid {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    bounds: WidgetBounds,
    columns: Vec<crate::datagrid::ColumnDef>,
    #[serde(default)]
    data_source_id: Option<String>,
    #[serde(default)]
    header_height: f32,
    #[serde(default)]
    row_height: f32,
    #[serde(default)]
    striped: bool,
    #[serde(default)]
    selection_mode: crate::datagrid::SelectionMode,
    #[serde(default)]
    show_vertical_lines: bool,
    #[serde(default)]
    show_horizontal_lines: bool,
    #[serde(default)]
    selected_rows: Vec<usize>,
    #[serde(default)]
    sort_column: Option<usize>,
    #[serde(default)]
    sort_direction: Option<crate::data_source::SortDirection>,
    #[serde(default)]
    style: crate::datagrid::DataGridStyle,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// KPI Card for analytics.
  KpiCard {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    title: String,
    #[serde(default)]
    value: String,
    #[serde(default)]
    trend: Option<crate::kpi::KpiTrend>,
    #[serde(default)]
    style: crate::kpi::KpiCardStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Text input field.
  TextInput {
    #[serde(default)]
    value: String,
    #[serde(default)]
    placeholder: String,
    id: String,
    #[serde(default)]
    font_size: f32,
    #[serde(default)]
    text_align: TextAlign,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    validation: Option<Vec<ValidationRule>>,
    #[serde(default)]
    style: TextInputStyle,
    #[serde(default)]
    width: f32,
    #[serde(default)]
    height: f32,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Numeric input with optional spinner buttons.
  NumberInput {
    id: String,
    #[serde(default)]
    value: f64,
    #[serde(default)]
    min: Option<f64>,
    #[serde(default)]
    max: Option<f64>,
    #[serde(default = "default_step")]
    step: f64,
    #[serde(default)]
    precision: usize,
    #[serde(default = "default_true")]
    show_spinner: bool,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    validation: Option<Vec<ValidationRule>>,
    #[serde(default)]
    style: NumberInputStyle,
    #[serde(default)]
    width: f32,
    #[serde(default)]
    height: f32,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Autocomplete dropdown input.
  Autocomplete {
      id: String,
      #[serde(default)]
      value: String,
      #[serde(default)]
      placeholder: String,
      #[serde(default)]
      suggestions: Vec<String>,
      #[serde(default = "default_max_visible")]
      max_visible: usize,
      #[serde(default)]
      bounds: WidgetBounds,
      #[serde(default)]
      style: AutocompleteStyle,
      #[serde(default)]
      validation: Option<Vec<ValidationRule>>,
      #[serde(default)]
      width: f32,
      #[serde(default)]
      height: f32,
      #[serde(default)]
      flex: f32,
      #[serde(default)]
      grid_col: Option<usize>,
      #[serde(default)]
      grid_row: Option<usize>,
      #[serde(default = "default_span_one")]
      col_span: usize,
      #[serde(default = "default_span_one")]
      row_span: usize,
  },

  /// Date picker widget with calendar dropdown.
  DatePicker {
      id: String,
      #[serde(default)]
      value: Option<NaiveDate>,
      #[serde(default)]
      placeholder: String,
      #[serde(default)]
      min_date: Option<NaiveDate>,
      #[serde(default)]
      max_date: Option<NaiveDate>,
      #[serde(default = "default_date_format")]
      format: String,
      #[serde(default)]
      bounds: WidgetBounds,
      #[serde(default)]
      style: DatePickerStyle,
      #[serde(default)]
      validation: Option<Vec<ValidationRule>>,
      #[serde(default)]
      width: f32,
      #[serde(default)]
      height: f32,
      #[serde(default)]
      flex: f32,
      #[serde(default)]
      grid_col: Option<usize>,
      #[serde(default)]
      grid_row: Option<usize>,
      #[serde(default = "default_span_one")]
      col_span: usize,
      #[serde(default = "default_span_one")]
      row_span: usize,
  },

  /// Checkbox toggle.
  Checkbox {
    id: String,
    #[serde(default)]
    checked: bool,
    #[serde(default = "default_checkbox_size")]
    size: f32,
    #[serde(default)]
    style: CheckboxStyle,
    #[serde(default)]
    bounds: WidgetBounds, // positioning
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },

  /// Slider range input.
  Slider {
    id: String,
    #[serde(default)]
    value: f32,
    #[serde(default = "default_slider_min")]
    min: f32,
    #[serde(default = "default_slider_max")]
    max: f32,
    #[serde(default)]
    style: SliderStyle,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    width: f32, // explicit width or flex
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },
  /// Image widget.
  Image {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    path: String,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    width: f32,
    #[serde(default)]
    height: f32,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },
  Icon {
    id: String,
    icon_name: String,
    size: f32,
    #[serde(default)]
    color: Option<Color>,
    #[serde(default)]
    bounds: WidgetBounds,
    #[serde(default)]
    flex: f32,
    #[serde(default)]
    grid_col: Option<usize>,
    #[serde(default)]
    grid_row: Option<usize>,
    #[serde(default = "default_span_one")]
    col_span: usize,
    #[serde(default = "default_span_one")]
    row_span: usize,
  },
}

/// Item representing a single tab.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabItem {
    /// Title shown on the tab header.
    pub title: String,
    /// Content widget displayed when the tab is selected.
    pub content: Box<Widget>,
}

/// Styling options for the Tab widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStyle {
    /// Background color of the tab bar.
    #[serde(default = "default_tab_background")]
    pub background: Color,
    /// Color of the selected tab header.
    #[serde(default = "default_tab_selected")]
    pub selected_color: Color,
    /// Color of unselected tab headers.
    #[serde(default = "default_tab_unselected")]
    pub unselected_color: Color,
    /// Optional border around the tab bar.
    pub border: Option<Border>,
    /// Optional shadow for the tab bar.
    pub shadow: Option<Shadow>,
}

fn default_tab_background() -> Color { (0.15, 0.15, 0.18, 1.0) }
fn default_tab_selected() -> Color { (0.3, 0.6, 1.0, 1.0) }
fn default_tab_unselected() -> Color { (0.5, 0.5, 0.5, 1.0) }

impl Default for TabStyle {
    fn default() -> Self {
        TabStyle {
            background: default_tab_background(),
            selected_color: default_tab_selected(),
            unselected_color: default_tab_unselected(),
            border: None,
            shadow: None,
        }
    }
}

impl Widget {
    /// Convenience constructor for a Tab widget.
    pub fn tab(
        id: impl Into<String>,
        tabs: Vec<TabItem>,
        orientation: Orientation,
        style: TabStyle,
    ) -> Self {
        Widget::Tab {
            id: Some(id.into()),
            tabs,
            selected: 0,
            orientation,
            style,
            bounds: WidgetBounds::default(),
            width: None,
            height: None,
            layout: Layout::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout_cache: None,
            render_cache: RefCell::new(None),
        }
    }
}

fn default_font_size() -> f32 {
  16.0
}

fn default_color() -> Color {
  (1.0, 1.0, 1.0, 1.0)
}

fn default_spacer_size() -> f32 {
  10.0
}

fn default_divider_thickness() -> f32 {
  1.0
}

fn default_divider_color() -> Color {
  (0.25, 0.25, 0.25, 1.0)
}

fn default_divider_margin() -> f32 {
  8.0
}

fn default_checkbox_size() -> f32 {
  20.0
}

fn default_checkbox_color() -> Color {
  (0.2, 0.2, 0.2, 1.0) // Flat dark surface
}

fn default_checkmark_color() -> Color {
  (0.9, 0.9, 0.9, 1.0) // White-ish check
}

fn default_slider_min() -> f32 {
  0.0
}

fn default_slider_max() -> f32 {
  1.0
}

fn default_active_color() -> Color {
  (0.6, 0.6, 0.6, 1.0) // Flat light gray
}

fn default_span_one() -> usize {
  1
}

fn default_inactive_color() -> Color {
  (0.2, 0.2, 0.2, 1.0) // Flat dark surface
}

impl Widget {
  /// Creates a new container widget.
  pub fn container() -> Self {
    Widget::Container {
      id: None,
      scrollable: false,
      bounds: WidgetBounds::default(),
      width: None,
      height: None,
      style: BoxStyle::default(),
      padding: 0.0,
      layout: Layout::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
      children: Vec::new(),
      layout_cache: None,
      render_cache: RefCell::new(None),
    }
  }

  /// Explicitly invalidates the layout cache for this widget and its subtree.
  /// Should be called whenever the widget structure or style changes.
  pub fn mark_dirty(&mut self) {
      if let Widget::Container { layout_cache, render_cache, children, .. } = self {
          *layout_cache = None;
          *render_cache.borrow_mut() = None;
          for child in children {
              child.mark_dirty();
          }
      }
      // For other widgets (leaves), there is no cache to clear, 
      // but if we add caching to leaf nodes later, we'd clear it here.
  }

  /// Creates a new label widget.
  pub fn label(text: impl Into<String>) -> Self {
    Widget::Label {
      text: text.into(),
      x: 0.0,
      y: 0.0,
      width: 0.0,
      height: 0.0,
      size: 16.0,
      color: (1.0, 1.0, 1.0, 1.0),
      text_align: TextAlign::Left,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
      font: None,
    }
  }

  /// Returns the bounds of the widget.
  pub fn bounds(&self) -> WidgetBounds {
      match self {
          Widget::Container { bounds, .. } => *bounds,
          Widget::Button { bounds, .. } => *bounds,
          Widget::TextInput { bounds, .. } => *bounds,
          Widget::NumberInput { bounds, .. } => *bounds,
          Widget::Autocomplete { bounds, .. } => *bounds,
          Widget::DatePicker { bounds, .. } => *bounds,
          Widget::Checkbox { bounds, .. } => *bounds,
          Widget::Slider { bounds, .. } => *bounds,
          Widget::Image { bounds, .. } => *bounds,
          Widget::Icon { bounds, .. } => *bounds,
          Widget::Label { x, y, width, height, .. } => WidgetBounds { x: *x, y: *y, width: *width, height: *height },
          Widget::ToggleSwitch { bounds, .. } => *bounds,
          Widget::ProgressBar { bounds, .. } => *bounds,
          Widget::RadioButton { bounds, .. } => *bounds,
          Widget::Dropdown { bounds, .. } => *bounds,
          Widget::Spacer { .. } => WidgetBounds::default(),
          Widget::Divider { bounds, .. } => *bounds,
          Widget::Scrollbar { bounds, .. } => *bounds,
          Widget::DataGrid { bounds, .. } => *bounds,
          Widget::Tree { bounds, .. } => *bounds,
          Widget::KpiCard { bounds, .. } => *bounds,
          Widget::ListView { bounds, .. } => *bounds,
          Widget::Tab { bounds, .. } => *bounds,
      }
  }

  /// Returns the focusable ID of the widget if it is interactive.
  pub fn get_focusable_id(&self) -> Option<&str> {
      match self {
          Widget::Button { action, .. } => Some(action),
          Widget::TextInput { id, .. } => Some(id),
          Widget::NumberInput { id, .. } => Some(id),
          Widget::Autocomplete { id, .. } => Some(id),
          Widget::DatePicker { id, .. } => Some(id),
          Widget::Checkbox { id, .. } => Some(id),
          Widget::Slider { id, .. } => Some(id),
          Widget::ToggleSwitch { id, .. } => Some(id),
          Widget::RadioButton { group_id, value, .. } => Some(value), // Use value as ID for specific radio? Or group? Probably individual click target needs diff ID.
          Widget::Dropdown { id, .. } => Some(id),
          Widget::ListView { id, .. } => Some(id), 
          Widget::Tab { id, .. } => id.as_deref(),
          _ => None,
      }
  }

  /// Validates the widget's current value against its rules.
  pub fn validate(&self) -> Vec<String> {
      let mut errors = Vec::new();
      match self {
          Widget::TextInput { value, validation: Some(rules), .. } => {
              for rule in rules {
                  if let Err(e) = rule.validate(value) {
                      errors.push(e);
                  }
              }
          }
          Widget::NumberInput { value, validation: Some(rules), .. } => {
              let val_str = value.to_string();
              for rule in rules {
                  if let Err(e) = rule.validate(&val_str) {
                      errors.push(e);
                  }
              }
          }
          Widget::Autocomplete { value, validation: Some(rules), .. } => {
              for rule in rules {
                  if let Err(e) = rule.validate(value) {
                      errors.push(e);
                  }
              }
          }
          Widget::DatePicker { value, min_date, max_date, validation, .. } => {
              if let Some(val) = value {
                  if let Some(min) = min_date {
                      if *val < *min { errors.push(format!("Date must be after {}", min)); }
                  }
                  if let Some(max) = max_date {
                      if *val > *max { errors.push(format!("Date must be before {}", max)); }
                  }
              }
              if let Some(rules) = validation {
                  let val_str = value.map(|d| d.format("%Y-%m-%d").to_string()).unwrap_or_default();
                  for rule in rules {
                       if let Err(e) = rule.validate(&val_str) {
                           errors.push(e);
                       }
                  }
              }
          }
          _ => {}
      }
      errors
  }
}

// Structs moved to style.rs

/// Style for NumberInput widget.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NumberInputStyle {
    #[serde(default)]
    pub background: Option<Color>,
    #[serde(default)]
    pub background_focused: Option<Color>,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub border_focused: Option<Border>,
    #[serde(default = "default_text_color")]
    pub text_color: Color,
    #[serde(default = "default_spinner_color")]
    pub spinner_color: Color,
    #[serde(default = "default_spinner_hover_color")]
    pub spinner_hover_color: Color,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub font: Option<String>,
}

impl Default for NumberInputStyle {
    fn default() -> Self {
        Self {
            background: Some((0.15, 0.15, 0.18, 1.0)),
            background_focused: Some((0.18, 0.18, 0.22, 1.0)),
            border: Some(Border {
                width: 1.0,
                color: (0.3, 0.3, 0.35, 1.0),
                ..Default::default()
            }),
            border_focused: Some(Border {
                width: 1.0,
                color: (0.4, 0.6, 1.0, 1.0),
                ..Default::default()
            }),
            text_color: (0.9, 0.9, 0.9, 1.0),
            spinner_color: (0.5, 0.5, 0.55, 1.0),
            spinner_hover_color: (0.7, 0.7, 0.75, 1.0),
            corner_radius: 4.0,
            font: None,
        }
    }
}

/// Style for Autocomplete widget.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AutocompleteStyle {
    #[serde(default)]
    pub background: Option<Color>,
    #[serde(default)]
    pub background_focused: Option<Color>,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub border_focused: Option<Border>,
    #[serde(default = "default_text_color")]
    pub text_color: Color,
    #[serde(default = "default_cursor_color")]
    pub cursor_color: Color,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub font: Option<String>,
    #[serde(default)]
    pub dropdown_background: Option<Color>,
    #[serde(default)]
    pub dropdown_border: Option<Border>,
    #[serde(default = "default_text_color")]
    pub dropdown_text_color: Color,
    #[serde(default = "default_highlight_color")]
    pub dropdown_highlight_color: Color,
}

impl Default for AutocompleteStyle {
    fn default() -> Self {
        Self {
            background: Some((0.15, 0.15, 0.18, 1.0)),
            background_focused: Some((0.18, 0.18, 0.22, 1.0)),
            border: Some(Border {
                width: 1.0,
                color: (0.3, 0.3, 0.35, 1.0),
                ..Default::default()
            }),
            border_focused: Some(Border {
                width: 1.0,
                color: (0.4, 0.6, 1.0, 1.0),
                ..Default::default()
            }),
            text_color: (0.9, 0.9, 0.9, 1.0),
            cursor_color: (0.8, 0.8, 0.8, 1.0),
            corner_radius: 4.0,
            font: None,
            dropdown_background: Some((0.12, 0.12, 0.15, 1.0)),
            dropdown_border: Some(Border {
                width: 1.0,
                color: (0.25, 0.25, 0.3, 1.0),
                ..Default::default()
            }),
            dropdown_text_color: (0.9, 0.9, 0.9, 1.0),
            dropdown_highlight_color: (0.25, 0.35, 0.5, 1.0),
        }
    }
}

/// Style for DatePicker widget.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatePickerStyle {
    #[serde(default)]
    pub background: Option<Color>,
    #[serde(default)]
    pub background_focused: Option<Color>,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub border_focused: Option<Border>,
    #[serde(default = "default_text_color")]
    pub text_color: Color,
    #[serde(default = "default_placeholder_color")]
    pub placeholder_color: Color,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub font: Option<String>,
    // Calendar Popup Styles
    #[serde(default)]
    pub calendar_background: Option<Color>,
    #[serde(default)]
    pub calendar_border: Option<Border>,
    #[serde(default = "default_text_color")]
    pub day_text_color: Color,
    #[serde(default = "default_highlight_color")]
    pub selected_day_color: Color,
    #[serde(default = "default_today_color")]
    pub today_color: Color,
    #[serde(default = "default_day_hover_color")]
    pub day_hover_color: Color,
    #[serde(default = "default_month_header_color")]
    pub month_header_color: Color,
}

impl Default for DatePickerStyle {
    fn default() -> Self {
        Self {
            background: Some((0.15, 0.15, 0.18, 1.0)),
            background_focused: Some((0.18, 0.18, 0.22, 1.0)),
            border: Some(Border {
                width: 1.0,
                color: (0.3, 0.3, 0.35, 1.0),
                ..Default::default()
            }),
            border_focused: Some(Border {
                width: 1.0,
                color: (0.4, 0.6, 1.0, 1.0),
                ..Default::default()
            }),
            text_color: (0.9, 0.9, 0.9, 1.0),
            placeholder_color: default_placeholder_color(),
            corner_radius: 4.0,
            font: None,
            calendar_background: Some((0.12, 0.12, 0.15, 1.0)),
            calendar_border: Some(Border {
                width: 1.0,
                color: (0.25, 0.25, 0.3, 1.0),
                ..Default::default()
            }),
            day_text_color: (0.9, 0.9, 0.9, 1.0),
            selected_day_color: (0.25, 0.5, 0.8, 1.0),
            today_color: default_today_color(),
            day_hover_color: default_day_hover_color(),
            month_header_color: default_month_header_color(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CheckboxStyle {
    #[serde(default = "default_checkbox_color")]
    pub background: Color,
    #[serde(default = "default_checkbox_checked")]
    pub background_checked: Color, // Was check_color? No check_color is typically the tick.
    #[serde(default = "default_checkmark_color")]
    pub checkmark_color: Color,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub corner_radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SliderStyle {
    #[serde(default = "default_inactive_color")]
    pub track_color: Color,
    #[serde(default = "default_active_color")]
    pub active_track_color: Color,
    #[serde(default = "default_thumb_color")]
    pub thumb_color: Color,
    #[serde(default)]
    pub thumb_border: Option<Border>,
    #[serde(default)]
    pub track_height: f32,
    #[serde(default)]
    pub thumb_radius: f32,
}

// Helpers for defaults
fn default_text_color() -> Color { (0.9, 0.9, 0.95, 1.0) }
fn default_placeholder_color() -> Color { (0.5, 0.5, 0.6, 1.0) }
fn default_cursor_color() -> Color { (0.8, 0.8, 0.8, 1.0) }
fn default_checkbox_checked() -> Color { (0.2, 0.2, 0.2, 1.0) }
fn default_thumb_color() -> Color { (1.0, 1.0, 1.0, 1.0) }
fn default_step() -> f64 { 1.0 }
fn default_true() -> bool { true }
fn default_spinner_color() -> Color { (0.5, 0.5, 0.55, 1.0) }
fn default_spinner_hover_color() -> Color { (0.7, 0.7, 0.75, 1.0) }
fn default_max_visible() -> usize { 5 }
fn default_highlight_color() -> Color { (0.25, 0.35, 0.5, 1.0) }
fn default_date_format() -> String { "%Y-%m-%d".to_string() }
fn default_today_color() -> Color { (0.3, 0.5, 0.3, 1.0) }
fn default_day_hover_color() -> Color { (0.2, 0.2, 0.25, 1.0) }
fn default_month_header_color() -> Color { (0.8, 0.8, 0.85, 1.0) }


#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ToggleSwitchStyle {
    #[serde(default)]
    pub track_color_on: Option<Color>,
    #[serde(default)]
    pub track_color_off: Option<Color>,
    #[serde(default)]
    pub thumb_color: Option<Color>,
    #[serde(default)]
    pub thumb_radius: f32,
    #[serde(default)]
    pub track_height: f32,
    #[serde(default)]
    pub width: f32,
}

/// Style configuration for scrollbars.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ScrollbarStyle {
    pub track_color: Color,
    pub thumb_color: Color,
    pub thumb_hover_color: Color,
    pub width: f32,
    pub corner_radius: f32,
}

impl Default for ScrollbarStyle {
    fn default() -> Self {
        Self {
            track_color: (0.1, 0.1, 0.1, 1.0),
            thumb_color: (0.3, 0.3, 0.3, 1.0),
            thumb_hover_color: (0.4, 0.4, 0.4, 1.0),
            width: 12.0,
            corner_radius: 6.0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProgressBarStyle {
    #[serde(default)]
    pub background_color: Option<Color>,
    #[serde(default)]
    pub fill_color: Option<Color>,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub corner_radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RadioButtonStyle {
    #[serde(default)]
    pub outer_color: Option<Color>,
    #[serde(default)]
    pub inner_color: Option<Color>,
    #[serde(default)]
    pub size: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DropdownStyle {
    #[serde(default)]
    pub background: Option<Color>,
    #[serde(default)]
    pub border: Option<Border>,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub text_color: Option<Color>,
    #[serde(default)]
    pub font: Option<String>,
}
