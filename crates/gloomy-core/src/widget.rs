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
    border_width: f32,
    #[serde(default)]
    border_color: Option<Color>,
    #[serde(default)]
    corner_radius: f32,
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
    border_width: f32,
    #[serde(default)]
    border_color: Option<Color>,
    #[serde(default = "default_corner_radius")]
    corner_radius: f32,
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
    background: Option<Color>,
    #[serde(default)]
    border_width: f32,
    #[serde(default)]
    border_color: Option<Color>,
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
    #[serde(default = "default_checkbox_color")]
    color: Color,
    #[serde(default = "default_checkmark_color")]
    check_color: Color,
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
    #[serde(default = "default_slider_height")]
    track_height: f32,
    #[serde(default = "default_thumb_radius")]
    thumb_radius: f32,
    #[serde(default = "default_active_color")]
    active_color: Color,
    #[serde(default = "default_inactive_color")]
    inactive_color: Color,
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
      border_width: 0.0,
      border_color: None,
      corner_radius: 0.0,
      corner_radii: None,
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
      size: 16.0,
      color: (1.0, 1.0, 1.0, 1.0),
      text_align: TextAlign::Left,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
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
          Widget::Label { x, y, .. } => WidgetBounds { x: *x, y: *y, width: 0.0, height: 0.0 }, // Label has no explicit size usually
          Widget::Spacer { .. } => WidgetBounds::default(),
      }
  }

  /// Returns the focusable ID of the widget if it is interactive.
  pub fn get_focusable_id(&self) -> Option<&str> {
      match self {
          Widget::Button { action, .. } => Some(action),
          Widget::TextInput { id, .. } => Some(id),
          Widget::Checkbox { id, .. } => Some(id),
          Widget::Slider { id, .. } => Some(id),
          _ => None,
      }
  }
}
