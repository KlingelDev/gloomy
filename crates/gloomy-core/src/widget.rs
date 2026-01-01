//! Widget types for declarative UI definitions.
//!
//! Widgets can be deserialized from RON files for declarative UI layouts.

use crate::layout::Layout;
use serde::{Deserialize, Serialize};

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
    background: Option<Color>,
    #[serde(default)]
    border: Option<Border>,
    #[serde(default)]
    corner_radius: f32,
    #[serde(default)]
    shadow: Option<Shadow>,
    #[serde(default)]
    gradient: Option<Gradient>,
    #[serde(default)]
    corner_radii: Option<[f32; 4]>,

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
  },

  /// Text label widget.
  Label {
    text: String,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(skip)]
    width: f32,
    #[serde(skip)]
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
    #[serde(default = "default_button_bg")]
    background: Color,
    #[serde(default = "default_button_hover")]
    hover_color: Color,
    #[serde(default = "default_button_active")]
    active_color: Color,
    #[serde(default)]
    border: Option<Border>,
    #[serde(default = "default_corner_radius")]
    corner_radius: f32,
    #[serde(default)]
    shadow: Option<Shadow>,
    #[serde(default)]
    gradient: Option<Gradient>,
    #[serde(default)]
    corner_radii: Option<[f32; 4]>,
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

fn default_font_size() -> f32 {
  16.0
}

fn default_color() -> Color {
  (1.0, 1.0, 1.0, 1.0)
}

fn default_spacer_size() -> f32 {
  10.0
}

fn default_button_bg() -> Color {
  (0.3, 0.3, 0.3, 1.0) // Flat dark gray
}

fn default_button_hover() -> Color {
  (0.4, 0.4, 0.4, 1.0) // Lighter gray
}

fn default_button_active() -> Color {
  (0.2, 0.2, 0.2, 1.0) // Darker gray
}

fn default_corner_radius() -> f32 {
  4.0
}

fn default_span_one() -> usize {
  1
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

fn default_slider_height() -> f32 {
  4.0
}

fn default_thumb_radius() -> f32 {
  8.0
}

fn default_active_color() -> Color {
  (0.6, 0.6, 0.6, 1.0) // Flat light gray
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
      background: None,
      border: None,
      corner_radius: 0.0,
      corner_radii: None,
      shadow: None,
      gradient: None,
      padding: 0.0,
      layout: Layout::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
      children: Vec::new(),
    }
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
      }
  }

  /// Returns the focusable ID of the widget if it is interactive.
  pub fn get_focusable_id(&self) -> Option<&str> {
      match self {
          Widget::Button { action, .. } => Some(action),
          Widget::TextInput { id, .. } => Some(id),
          Widget::Checkbox { id, .. } => Some(id),
          Widget::Slider { id, .. } => Some(id),
          Widget::ToggleSwitch { id, .. } => Some(id),
          Widget::RadioButton { group_id, value, .. } => Some(value), // Use value as ID for specific radio? Or group? Probably individual click target needs diff ID.
          Widget::Dropdown { id, .. } => Some(id),
          _ => None,
      }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Shadow {
    pub offset: (f32, f32),
    pub blur: f32,
    pub color: Color,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
}
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default, PartialEq)]
pub enum BorderStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Border {
    pub width: f32,
    pub color: Color,
    #[serde(default)]
    pub gradient: Option<Gradient>,
    #[serde(default)]
    pub style: BorderStyle,
    #[serde(default)]
    pub dash_len: f32,
    #[serde(default)]
    pub gap_len: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TextInputStyle {
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
    #[serde(default = "default_cursor_color")]
    pub cursor_color: Color,
    #[serde(default)]
    pub corner_radius: f32,
    #[serde(default)]
    pub font: Option<String>,
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
fn default_checkbox_checked() -> Color { (0.2, 0.2, 0.2, 1.0) } // Same as bg usually? Or accent?
fn default_thumb_color() -> Color { (1.0, 1.0, 1.0, 1.0) }


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
