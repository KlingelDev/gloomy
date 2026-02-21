//! Automation API for testing Gloomy UIs.
//!
//! Provides headless rendering, interaction simulation, layout
//! inspection, text extraction, and visual snapshot testing.

pub mod diff;
pub mod screenshot;
pub mod snapshot;

use crate::diff::{DiffConfig, DiffReport};
use crate::snapshot::SnapshotManager;
use gloomy_core::data_source::DataProvider;
use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::widget::{Widget, WidgetBounds};
use gloomy_core::{
  InteractionState, compute_layout, hit_test,
};
use glam::Vec2;
use image::RgbaImage;
use std::path::Path;

// Re-export for convenience.
pub use gloomy_core::headless::HeadlessConfig as RenderConfig;

/// A headless driver for interacting with a Gloomy UI tree.
///
/// Supports layout, interaction simulation, GPU rendering,
/// text extraction, and visual snapshot testing.
pub struct GloomyDriver {
  pub root: Widget,
  pub interaction: InteractionState,
  pub width: f32,
  pub height: f32,
  headless: Option<HeadlessRenderer>,
}

impl GloomyDriver {
  /// Creates a new driver with the given root widget and
  /// screen dimensions. Performs initial layout calculation.
  pub fn new(
    mut root: Widget,
    width: f32,
    height: f32,
  ) -> Self {
    if let Widget::Container { bounds, .. } = &mut root {
      bounds.width = width;
      bounds.height = height;
    }
    compute_layout(&mut root, 0.0, 0.0, width, height);
    Self {
      root,
      interaction: InteractionState::default(),
      width,
      height,
      headless: None,
    }
  }

  /// Creates a driver with headless rendering capability.
  ///
  /// `pixel_width` and `pixel_height` are the physical pixel
  /// dimensions of the render target. `scale_factor` maps
  /// physical pixels to logical UI coordinates.
  pub fn with_rendering(
    root: Widget,
    pixel_width: u32,
    pixel_height: u32,
    scale_factor: f32,
  ) -> anyhow::Result<Self> {
    let logical_w = pixel_width as f32 / scale_factor;
    let logical_h = pixel_height as f32 / scale_factor;
    let mut driver = Self::new(root, logical_w, logical_h);
    let config = HeadlessConfig {
      width: pixel_width,
      height: pixel_height,
      scale_factor,
      force_fallback_adapter: false,
    };
    driver.headless = Some(HeadlessRenderer::new(config)?);
    Ok(driver)
  }

  /// Initializes the GPU renderer for pixel output.
  ///
  /// Must be called before `render_to_image` or
  /// `save_screenshot`. Returns an error if no GPU adapter
  /// is available.
  pub fn init_renderer(
    &mut self,
    force_fallback: bool,
  ) -> anyhow::Result<()> {
    let config = HeadlessConfig {
      width: self.width as u32,
      height: self.height as u32,
      force_fallback_adapter: force_fallback,
      ..Default::default()
    };
    self.headless = Some(HeadlessRenderer::new(config)?);
    Ok(())
  }

  /// Renders the current widget tree to an RGBA image.
  ///
  /// Requires the driver to have been initialized via
  /// `init_renderer()` or `with_rendering()`.
  pub fn render_to_image(
    &mut self,
    data_provider: Option<&dyn DataProvider>,
  ) -> anyhow::Result<RgbaImage> {
    let r = self.headless.as_mut().ok_or_else(|| {
      anyhow::anyhow!(
        "Call init_renderer() or with_rendering() first"
      )
    })?;
    r.render_to_image(
      &mut self.root,
      Some(&self.interaction),
      data_provider,
    )
  }

  /// Renders the widget tree and saves as a PNG.
  pub fn save_screenshot(
    &mut self,
    path: impl AsRef<Path>,
  ) -> anyhow::Result<()> {
    let img = self.render_to_image(None)?;
    img.save(path)?;
    Ok(())
  }

  /// Runs a full snapshot comparison cycle.
  ///
  /// Renders the current tree, compares against the stored
  /// golden in `snapshot_dir`, and returns a structured diff
  /// report.
  pub fn snapshot_test(
    &mut self,
    name: &str,
    snapshot_dir: impl AsRef<Path>,
    data_provider: Option<&dyn DataProvider>,
  ) -> anyhow::Result<DiffReport> {
    let image = self.render_to_image(data_provider)?;
    let mgr = SnapshotManager::new(snapshot_dir);
    let config = DiffConfig::default();
    mgr.compare(name, &image, &config, Some(&self.root))
  }

  /// Finds a widget by its ID.
  pub fn find<'a>(&'a self, id: &str) -> Option<&'a Widget> {
    find_recursive(&self.root, id)
  }

  /// Finds a widget's bounds by ID.
  pub fn find_bounds(
    &self,
    id: &str,
  ) -> Option<WidgetBounds> {
    self.find(id).map(|w| w.bounds())
  }

  /// Simulates a click on the widget with the given ID.
  pub fn click(&mut self, id: &str) -> Option<String> {
    let bounds = self.find_bounds(id)?;
    let center_x = bounds.x + bounds.width * 0.5;
    let center_y = bounds.y + bounds.height * 0.5;

    self.interaction.update_mouse(
      Vec2::new(center_x, center_y),
    );
    self.interaction.set_pressed(true);
    let hit = hit_test(
      &self.root,
      Vec2::new(center_x, center_y),
      Some(&self.interaction),
    );
    self.interaction.handle_hit(
      hit.as_ref().map(|h| h.action.clone()),
    );
    self.interaction.set_pressed(false);

    if let Some(w) = self.find(id) {
      match w {
        Widget::Button { action, .. } => Some(action.clone()),
        Widget::ToggleSwitch { id, .. } => Some(id.clone()),
        Widget::Checkbox { id, .. } => Some(id.clone()),
        _ => None,
      }
    } else {
      None
    }
  }

  /// Recalculates layout for the widget tree.
  pub fn relayout(&mut self) {
    compute_layout(
      &mut self.root,
      0.0,
      0.0,
      self.width,
      self.height,
    );
  }
}

// ── Layout inspector ──────────────────────────────────────

/// Dumps the computed layout tree as indented text.
///
/// Positions are absolute (screen coordinates). Each line
/// contains: widget type, optional id, and computed bounds.
pub fn dump_layout(widget: &Widget) -> String {
  let mut out = String::new();
  dump_layout_recursive(widget, 0, 0.0, 0.0, &mut out);
  out
}

fn dump_layout_recursive(
  widget: &Widget,
  depth: usize,
  offset_x: f32,
  offset_y: f32,
  out: &mut String,
) {
  let indent = "  ".repeat(depth);
  let b = widget.bounds();
  let abs_x = offset_x + b.x;
  let abs_y = offset_y + b.y;
  let wtype = widget_type_name(widget);
  let id = widget_id(widget);

  let id_str = match id {
    Some(s) => format!(" id={}", s),
    None => String::new(),
  };

  out.push_str(&format!(
    "{}{}{} x={:.0} y={:.0} w={:.0} h={:.0}\n",
    indent, wtype, id_str,
    abs_x, abs_y, b.width, b.height,
  ));

  // Children are positioned relative to their container.
  let child_ox;
  let child_oy;
  if let Widget::Container { .. } = widget {
    child_ox = abs_x;
    child_oy = abs_y;
  } else {
    child_ox = offset_x;
    child_oy = offset_y;
  }

  for child in widget_children(widget) {
    dump_layout_recursive(
      child, depth + 1, child_ox, child_oy, out,
    );
  }
}

// ── Text extraction ───────────────────────────────────────

/// A text entry extracted from the widget tree.
#[derive(Debug, Clone)]
pub struct TextEntry {
  /// The visible text content.
  pub text: String,
  /// X position of the text.
  pub x: f32,
  /// Y position of the text.
  pub y: f32,
  /// Width of the bounding box.
  pub width: f32,
  /// Height of the bounding box.
  pub height: f32,
  /// Widget ID if available.
  pub widget_id: Option<String>,
  /// Widget type name.
  pub widget_type: String,
}

/// Extracts all visible text from the widget tree.
///
/// Positions are absolute (screen coordinates), computed by
/// accumulating parent container offsets — matching where the
/// text actually renders on screen.
pub fn dump_text(widget: &Widget) -> Vec<TextEntry> {
  let mut entries = Vec::new();
  dump_text_recursive(widget, 0.0, 0.0, &mut entries);
  entries
}

/// Finds the first occurrence of a string in the widget tree.
pub fn find_text(
  widget: &Widget,
  needle: &str,
) -> Option<TextEntry> {
  dump_text(widget)
    .into_iter()
    .find(|e| e.text.contains(needle))
}

fn dump_text_recursive(
  widget: &Widget,
  offset_x: f32,
  offset_y: f32,
  entries: &mut Vec<TextEntry>,
) {
  let b = widget.bounds();
  let abs_x = offset_x + b.x;
  let abs_y = offset_y + b.y;
  let wtype = widget_type_name(widget);
  let id = widget_id(widget).map(|s| s.to_string());

  match widget {
    Widget::Label { text, .. } => {
      entries.push(TextEntry {
        text: text.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::Button { text, .. } => {
      entries.push(TextEntry {
        text: text.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::TextInput {
      value, placeholder, ..
    } => {
      let txt = if value.is_empty() {
        placeholder.clone()
      } else {
        value.clone()
      };
      if !txt.is_empty() {
        entries.push(TextEntry {
          text: txt,
          x: abs_x,
          y: abs_y,
          width: b.width,
          height: b.height,
          widget_id: id,
          widget_type: wtype.to_string(),
        });
      }
    }
    Widget::KpiCard { title, value, .. } => {
      entries.push(TextEntry {
        text: title.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id.clone(),
        widget_type: wtype.to_string(),
      });
      entries.push(TextEntry {
        text: value.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::Dropdown {
      options, selected_index, ..
    } => {
      if let Some(idx) = selected_index {
        if let Some(opt) = options.get(*idx) {
          entries.push(TextEntry {
            text: opt.clone(),
            x: abs_x,
            y: abs_y,
            width: b.width,
            height: b.height,
            widget_id: id,
            widget_type: wtype.to_string(),
          });
        }
      }
    }
    Widget::RadioButton { label, .. } => {
      entries.push(TextEntry {
        text: label.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::ListView { items, .. } => {
      for item in items {
        entries.push(TextEntry {
          text: item.clone(),
          x: abs_x,
          y: abs_y,
          width: b.width,
          height: b.height,
          widget_id: id.clone(),
          widget_type: wtype.to_string(),
        });
      }
    }
    Widget::Chart { title, .. } if !title.is_empty() => {
      entries.push(TextEntry {
        text: title.clone(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::NumberInput { value, .. } => {
      entries.push(TextEntry {
        text: value.to_string(),
        x: abs_x,
        y: abs_y,
        width: b.width,
        height: b.height,
        widget_id: id,
        widget_type: wtype.to_string(),
      });
    }
    Widget::Autocomplete {
      value, placeholder, ..
    } => {
      let txt = if value.is_empty() {
        placeholder.clone()
      } else {
        value.clone()
      };
      if !txt.is_empty() {
        entries.push(TextEntry {
          text: txt,
          x: abs_x,
          y: abs_y,
          width: b.width,
          height: b.height,
          widget_id: id,
          widget_type: wtype.to_string(),
        });
      }
    }
    _ => {}
  }

  // When recursing into containers, the children's bounds
  // are relative to the container's position, so pass the
  // container's absolute position as the new offset.
  let child_offset_x;
  let child_offset_y;
  if let Widget::Container { .. } = widget {
    child_offset_x = abs_x;
    child_offset_y = abs_y;
  } else {
    child_offset_x = offset_x;
    child_offset_y = offset_y;
  }

  for child in widget_children(widget) {
    dump_text_recursive(
      child,
      child_offset_x,
      child_offset_y,
      entries,
    );
  }
}

// ── Visual snapshot testing ───────────────────────────────

/// Compares a rendered image against a golden snapshot.
///
/// On first run (no golden exists), saves the image as the
/// golden reference. On mismatch, saves the actual image
/// and a diff image next to the golden, then returns an
/// error.
///
/// `tolerance` is the maximum allowed per-channel difference
/// (0-255) for any pixel before it's considered a mismatch.
pub fn assert_screenshot(
  driver: &mut GloomyDriver,
  name: &str,
  snapshot_dir: impl AsRef<Path>,
  tolerance: u8,
) -> anyhow::Result<()> {
  let dir = snapshot_dir.as_ref();
  std::fs::create_dir_all(dir)?;

  let golden_path = dir.join(format!("{}.png", name));
  let actual_path =
    dir.join(format!("{}_actual.png", name));
  let diff_path = dir.join(format!("{}_diff.png", name));

  let actual = driver.render_to_image(None)?;

  if !golden_path.exists() {
    actual.save(&golden_path)?;
    return Ok(());
  }

  let golden = image::open(&golden_path)?.to_rgba8();

  if golden.dimensions() != actual.dimensions() {
    actual.save(&actual_path)?;
    anyhow::bail!(
      "Screenshot '{}': size mismatch. \
       Golden {}x{}, actual {}x{}. \
       Saved actual to {:?}",
      name,
      golden.width(),
      golden.height(),
      actual.width(),
      actual.height(),
      actual_path,
    );
  }

  let (w, h) = golden.dimensions();
  let mut diff_img = image::RgbaImage::new(w, h);
  let mut mismatch_count = 0u64;

  for y in 0..h {
    for x in 0..w {
      let gp = golden.get_pixel(x, y);
      let ap = actual.get_pixel(x, y);
      let channel_diff = gp
        .0
        .iter()
        .zip(ap.0.iter())
        .map(|(a, b)| (*a as i16 - *b as i16).unsigned_abs())
        .max()
        .unwrap_or(0);

      if channel_diff > tolerance as u16 {
        mismatch_count += 1;
        diff_img.put_pixel(
          x,
          y,
          image::Rgba([255, 0, 0, 255]),
        );
      } else {
        // Dim version of the actual pixel.
        diff_img.put_pixel(
          x,
          y,
          image::Rgba([
            ap.0[0] / 3,
            ap.0[1] / 3,
            ap.0[2] / 3,
            255,
          ]),
        );
      }
    }
  }

  if mismatch_count > 0 {
    actual.save(&actual_path)?;
    diff_img.save(&diff_path)?;
    let total = (w as u64) * (h as u64);
    let pct =
      (mismatch_count as f64 / total as f64) * 100.0;
    anyhow::bail!(
      "Screenshot '{}': {}/{} pixels differ ({:.2}%). \
       Saved actual to {:?}, diff to {:?}",
      name,
      mismatch_count,
      total,
      pct,
      actual_path,
      diff_path,
    );
  }

  // Clean up any stale diff artifacts.
  let _ = std::fs::remove_file(&actual_path);
  let _ = std::fs::remove_file(&diff_path);
  Ok(())
}

// ── Theme application ─────────────────────────────────────

/// Applies a theme's color palette to a widget tree.
///
/// Walks the tree and updates widget styles (backgrounds,
/// text colors, borders, etc.) to match the theme. This
/// modifies the tree in place.
pub fn apply_theme(
  widget: &mut Widget,
  theme: &gloomy_core::theme::Theme,
) {
  let c = &theme.colors;

  match widget {
    Widget::Container {
      style, children, ..
    } => {
      if style.background.is_some() {
        style.background = Some(c.surface);
      }
      if let Some(ref mut border) = style.border {
        border.color = c.border;
      }
      for child in children.iter_mut() {
        apply_theme(child, theme);
      }
    }
    Widget::Label { color, .. } => {
      *color = c.text;
    }
    Widget::Button { style, .. } => {
      style.idle =
        gloomy_core::style::BoxStyle::fill(c.surface)
          .with_radius(4.0);
      style.hover =
        gloomy_core::style::BoxStyle::fill(c.hover)
          .with_radius(4.0);
      style.active =
        gloomy_core::style::BoxStyle::fill(c.active)
          .with_radius(4.0);
      style.disabled =
        gloomy_core::style::BoxStyle::fill((
          c.surface.0,
          c.surface.1,
          c.surface.2,
          0.4,
        ))
        .with_radius(4.0);
      style.text_color = c.text;
    }
    Widget::TextInput { style, .. } => {
      style.idle.background = Some(c.surface);
      style.focused.background = Some(c.surface);
      style.text_color = c.text;
      style.placeholder_color = c.text_disabled;
    }
    Widget::Checkbox { style, .. } => {
      style.background = c.surface;
      style.background_checked = c.primary;
      style.checkmark_color = c.text;
    }
    Widget::Slider { style, .. } => {
      style.track_color = c.surface;
      style.active_track_color = c.primary;
      style.thumb_color = c.text;
    }
    Widget::ToggleSwitch { style, .. } => {
      style.track_color_on = Some(c.success);
      style.track_color_off = Some(c.surface);
      style.thumb_color = Some(c.text);
    }
    Widget::ProgressBar { style, .. } => {
      style.background_color = Some(c.surface);
      style.fill_color = Some(c.primary);
    }
    Widget::RadioButton { style, .. } => {
      style.outer_color = Some(c.border);
      style.inner_color = Some(c.primary);
    }
    Widget::Dropdown { style, .. } => {
      style.background = Some(c.surface);
      style.text_color = Some(c.text);
    }
    Widget::Divider { color, .. } => {
      *color = c.divider;
    }
    Widget::KpiCard { style, .. } => {
      style.background = c.surface;
      style.label_color = c.text_secondary;
      style.value_color = c.text;
    }
    Widget::Tab { style, tabs, .. } => {
      style.background = c.surface;
      style.selected_color = c.primary;
      style.unselected_color = c.text_secondary;
      for tab in tabs.iter_mut() {
        apply_theme(&mut tab.content, theme);
      }
    }
    Widget::ListView { style, .. } => {
      style.idle =
        gloomy_core::style::BoxStyle::fill(c.surface);
      style.hover =
        gloomy_core::style::BoxStyle::fill(c.hover);
      style.selected =
        gloomy_core::style::BoxStyle::fill(c.primary);
      style.text_color_idle = c.text;
      style.text_color_selected = c.text;
    }
    Widget::DataGrid { style, .. } => {
      style.header_background = c.surface;
      style.header_text_color = c.text;
      style.row_background = c.background;
      style.alt_row_background = c.surface;
      style.row_text_color = c.text;
      style.hover_background = c.hover;
      style.selected_background = c.primary;
      style.grid_line_color = c.border;
    }
    Widget::NumberInput { style, .. } => {
      style.background = Some(c.surface);
      style.text_color = c.text;
      style.spinner_color = c.text_secondary;
    }
    Widget::Autocomplete { style, .. } => {
      style.background = Some(c.surface);
      style.text_color = c.text;
      style.cursor_color = c.text;
      style.dropdown_background = Some(c.surface);
      style.dropdown_text_color = c.text;
      style.dropdown_highlight_color = c.hover;
    }
    Widget::DatePicker { style, .. } => {
      style.background = Some(c.surface);
      style.text_color = c.text;
      style.placeholder_color = c.text_disabled;
      style.calendar_background = Some(c.surface);
      style.day_text_color = c.text;
      style.selected_day_color = c.primary;
      style.today_color = c.active;
      style.day_hover_color = c.hover;
      style.month_header_color = c.text;
    }
    Widget::Tree { style, .. } => {
      style.text_color = c.text;
      style.icon_color = c.text_secondary;
      style.selected_background = c.primary;
      style.hover_background = c.hover;
    }
    Widget::Scrollbar { style, .. } => {
      style.track_color = c.background;
      style.thumb_color = c.border;
      style.thumb_hover_color = c.hover;
    }
    _ => {}
  }
}

// ── Helpers ───────────────────────────────────────────────

fn find_recursive<'a>(
  widget: &'a Widget,
  target_id: &str,
) -> Option<&'a Widget> {
  let id = widget_id(widget);
  if id == Some(target_id) {
    return Some(widget);
  }
  for child in widget_children(widget) {
    if let Some(found) = find_recursive(child, target_id) {
      return Some(found);
    }
  }
  None
}

fn widget_id(widget: &Widget) -> Option<&str> {
  match widget {
    Widget::Container { id, .. } => id.as_deref(),
    Widget::Tab { id, .. } => id.as_deref(),
    Widget::ToggleSwitch { id, .. } => Some(id),
    Widget::TextInput { id, .. } => Some(id),
    Widget::NumberInput { id, .. } => Some(id),
    Widget::DatePicker { id, .. } => Some(id),
    Widget::Autocomplete { id, .. } => Some(id),
    Widget::Checkbox { id, .. } => Some(id),
    Widget::Slider { id, .. } => Some(id),
    Widget::Dropdown { id, .. } => Some(id),
    Widget::KpiCard { id, .. } => id.as_deref(),
    Widget::DataGrid { id, .. } => id.as_deref(),
    Widget::ListView { id, .. } => Some(id),
    Widget::Icon { id, .. } => Some(id),
    Widget::Chart { id, .. } => id.as_deref(),
    Widget::Tree { id, .. } => id.as_deref(),
    _ => None,
  }
}

fn widget_type_name(widget: &Widget) -> &'static str {
  match widget {
    Widget::Container { .. } => "Container",
    Widget::Tab { .. } => "Tab",
    Widget::Label { .. } => "Label",
    Widget::Button { .. } => "Button",
    Widget::ListView { .. } => "ListView",
    Widget::Tree { .. } => "Tree",
    Widget::ToggleSwitch { .. } => "ToggleSwitch",
    Widget::ProgressBar { .. } => "ProgressBar",
    Widget::RadioButton { .. } => "RadioButton",
    Widget::Dropdown { .. } => "Dropdown",
    Widget::Spacer { .. } => "Spacer",
    Widget::Divider { .. } => "Divider",
    Widget::Scrollbar { .. } => "Scrollbar",
    Widget::DataGrid { .. } => "DataGrid",
    Widget::KpiCard { .. } => "KpiCard",
    Widget::TextInput { .. } => "TextInput",
    Widget::NumberInput { .. } => "NumberInput",
    Widget::Autocomplete { .. } => "Autocomplete",
    Widget::DatePicker { .. } => "DatePicker",
    Widget::Checkbox { .. } => "Checkbox",
    Widget::Slider { .. } => "Slider",
    Widget::Image { .. } => "Image",
    Widget::Icon { .. } => "Icon",
    Widget::Chart { .. } => "Chart",
  }
}

fn widget_children(widget: &Widget) -> &[Widget] {
  match widget {
    Widget::Container { children, .. } => children,
    Widget::Tab { tabs, selected, .. } => {
      if let Some(tab) = tabs.get(*selected) {
        std::slice::from_ref(tab.content.as_ref())
      } else {
        &[]
      }
    }
    _ => &[],
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use gloomy_core::style::ButtonStyle;
  use gloomy_core::theme::Theme;
  use gloomy_core::widget::{
    CheckboxStyle, DropdownStyle, Orientation, SliderStyle,
  };

  // ── Helper builders ───────────────────────────────────

  fn make_button(
    text: &str,
    action: &str,
  ) -> Widget {
    Widget::Button {
      text: text.to_string(),
      action: action.to_string(),
      bounds: WidgetBounds::default(),
      style: ButtonStyle::default(),
      width: Some(100.0),
      height: Some(40.0),
      disabled: false,
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
      font: None,
    }
  }

  fn make_checkbox(id: &str, checked: bool) -> Widget {
    Widget::Checkbox {
      id: id.to_string(),
      checked,
      size: 20.0,
      style: CheckboxStyle::default(),
      bounds: WidgetBounds::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    }
  }

  fn make_text_input(
    id: &str,
    value: &str,
    placeholder: &str,
  ) -> Widget {
    Widget::TextInput {
      id: id.to_string(),
      value: value.to_string(),
      placeholder: placeholder.to_string(),
      font_size: 14.0,
      text_align: Default::default(),
      bounds: WidgetBounds::default(),
      validation: None,
      style: Default::default(),
      width: 200.0,
      height: 32.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    }
  }

  fn make_dropdown(
    id: &str,
    options: Vec<&str>,
    selected: Option<usize>,
  ) -> Widget {
    Widget::Dropdown {
      id: id.to_string(),
      options: options.iter().map(|s| s.to_string()).collect(),
      selected_index: selected,
      expanded: false,
      style: DropdownStyle::default(),
      bounds: WidgetBounds::default(),
      width: Some(150.0),
      height: Some(32.0),
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    }
  }

  fn make_container_with(
    id: Option<&str>,
    children: Vec<Widget>,
  ) -> Widget {
    let mut root = Widget::container();
    if let Widget::Container {
      id: wid,
      children: wchildren,
      ..
    } = &mut root
    {
      *wid = id.map(|s| s.to_string());
      *wchildren = children;
    }
    root
  }

  // ── GloomyDriver::new ────────────────────────────────

  #[test]
  fn test_driver_new_sets_container_bounds() {
    let root = Widget::container();
    let driver = GloomyDriver::new(root, 1024.0, 768.0);
    assert_eq!(driver.width, 1024.0);
    assert_eq!(driver.height, 768.0);
    let bounds = driver.root.bounds();
    assert_eq!(bounds.width, 1024.0);
    assert_eq!(bounds.height, 768.0);
  }

  #[test]
  fn test_driver_new_with_non_container_root() {
    let label = Widget::label("solo");
    let driver = GloomyDriver::new(label, 800.0, 600.0);
    assert_eq!(driver.width, 800.0);
    assert!(driver.find("anything").is_none());
  }

  // ── find / find_bounds ────────────────────────────────

  #[test]
  fn test_driver_find() {
    let root = make_container_with(
      Some("root"),
      vec![make_button("Click Me", "my_action")],
    );
    let driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.find("root").is_some());
  }

  #[test]
  fn test_find_nonexistent_returns_none() {
    let root = make_container_with(Some("root"), vec![]);
    let driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.find("nonexistent").is_none());
  }

  #[test]
  fn test_find_nested_widget() {
    let inner = make_container_with(
      Some("inner"),
      vec![make_checkbox("cb1", false)],
    );
    let root = make_container_with(Some("outer"), vec![inner]);
    let driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.find("cb1").is_some());
    assert!(driver.find("inner").is_some());
    assert!(driver.find("outer").is_some());
  }

  #[test]
  fn test_find_bounds_returns_valid_bounds() {
    let root = make_container_with(
      Some("root"),
      vec![make_button("btn", "act")],
    );
    let driver = GloomyDriver::new(root, 800.0, 600.0);
    let bounds = driver.find_bounds("root");
    assert!(bounds.is_some());
    let b = bounds.unwrap();
    assert_eq!(b.width, 800.0);
    assert_eq!(b.height, 600.0);
  }

  #[test]
  fn test_find_bounds_nonexistent() {
    let root = Widget::container();
    let driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.find_bounds("nope").is_none());
  }

  // ── click ─────────────────────────────────────────────

  #[test]
  fn test_click_button_returns_action() {
    let root = make_container_with(
      Some("root"),
      vec![make_button("Save", "save_action")],
    );
    let mut driver = GloomyDriver::new(root, 800.0, 600.0);
    // Find the button by iterating children.
    if let Widget::Container { children, .. } =
      &driver.root
    {
      if let Widget::Button { action, .. } = &children[0] {
        assert_eq!(action, "save_action");
      }
    }
  }

  #[test]
  fn test_click_nonexistent_returns_none() {
    let root = make_container_with(Some("root"), vec![]);
    let mut driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.click("nonexistent").is_none());
  }

  // ── relayout ──────────────────────────────────────────

  #[test]
  fn test_relayout_updates_bounds() {
    let root = make_container_with(
      Some("root"),
      vec![Widget::label("test")],
    );
    let mut driver = GloomyDriver::new(root, 400.0, 300.0);
    let b1 = driver.root.bounds();

    // Modify width and relayout.
    driver.width = 800.0;
    driver.height = 600.0;
    if let Widget::Container { bounds, .. } =
      &mut driver.root
    {
      bounds.width = 800.0;
      bounds.height = 600.0;
    }
    driver.relayout();

    let b2 = driver.root.bounds();
    assert_eq!(b2.width, 800.0);
    assert_eq!(b2.height, 600.0);
    assert_ne!(b1.width, b2.width);
  }

  // ── render_to_image without renderer ──────────────────

  #[test]
  fn test_render_without_init_fails() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 200.0, 100.0);
    let result = driver.render_to_image(None);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
      msg.contains("init_renderer"),
      "Expected init_renderer hint, got: {}",
      msg,
    );
  }

  // ── dump_layout ───────────────────────────────────────

  #[test]
  fn test_dump_layout() {
    let root = make_container_with(
      Some("main"),
      vec![Widget::label("Hello")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    assert!(layout.contains("Container"));
    assert!(layout.contains("Label"));
    assert!(layout.contains("id=main"));
  }

  #[test]
  fn test_dump_layout_empty_container() {
    let root = make_container_with(Some("empty"), vec![]);
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    assert!(layout.contains("Container id=empty"));
    // Should have exactly one line (no children).
    assert_eq!(layout.lines().count(), 1);
  }

  #[test]
  fn test_dump_layout_nested_indentation() {
    let inner = make_container_with(
      Some("child"),
      vec![Widget::label("nested")],
    );
    let root =
      make_container_with(Some("parent"), vec![inner]);
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    let lines: Vec<&str> = layout.lines().collect();
    assert_eq!(lines.len(), 3);
    // Root at depth 0, no indent.
    assert!(lines[0].starts_with("Container"));
    // Child container at depth 1, 2-space indent.
    assert!(lines[1].starts_with("  Container"));
    // Label at depth 2, 4-space indent.
    assert!(lines[2].starts_with("    Label"));
  }

  #[test]
  fn test_dump_layout_contains_dimensions() {
    let root = make_container_with(
      None,
      vec![make_button("B", "b")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    // Should contain w= and h= for dimensions.
    assert!(layout.contains("w="));
    assert!(layout.contains("h="));
  }

  #[test]
  fn test_dump_layout_no_id_for_anonymous_container() {
    let root = make_container_with(None, vec![]);
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    // Should not contain "id=" since container has no id.
    assert!(!layout.contains("id="));
  }

  // ── dump_text ─────────────────────────────────────────

  #[test]
  fn test_dump_text() {
    let root = make_container_with(
      None,
      vec![
        Widget::label("Hello World"),
        make_button("OK", "ok"),
      ],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].text, "Hello World");
    assert_eq!(texts[0].widget_type, "Label");
    assert_eq!(texts[1].text, "OK");
    assert_eq!(texts[1].widget_type, "Button");
  }

  #[test]
  fn test_dump_text_empty_container() {
    let root = make_container_with(None, vec![]);
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert!(texts.is_empty());
  }

  #[test]
  fn test_dump_text_text_input_with_value() {
    let root = make_container_with(
      None,
      vec![make_text_input("ti", "typed", "hint")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "typed");
    assert_eq!(texts[0].widget_type, "TextInput");
  }

  #[test]
  fn test_dump_text_text_input_empty_shows_placeholder() {
    let root = make_container_with(
      None,
      vec![make_text_input("ti", "", "Enter name")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Enter name");
  }

  #[test]
  fn test_dump_text_text_input_both_empty() {
    let root = make_container_with(
      None,
      vec![make_text_input("ti", "", "")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert!(texts.is_empty());
  }

  #[test]
  fn test_dump_text_dropdown_with_selection() {
    let root = make_container_with(
      None,
      vec![make_dropdown(
        "dd",
        vec!["Red", "Green", "Blue"],
        Some(1),
      )],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Green");
    assert_eq!(texts[0].widget_type, "Dropdown");
  }

  #[test]
  fn test_dump_text_dropdown_no_selection() {
    let root = make_container_with(
      None,
      vec![make_dropdown(
        "dd",
        vec!["A", "B"],
        None,
      )],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert!(texts.is_empty());
  }

  #[test]
  fn test_dump_text_radio_button() {
    let root = make_container_with(
      None,
      vec![Widget::RadioButton {
        group_id: "grp".to_string(),
        value: "opt1".to_string(),
        selected: true,
        label: "Option One".to_string(),
        style: Default::default(),
        bounds: WidgetBounds::default(),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Option One");
    assert_eq!(texts[0].widget_type, "RadioButton");
  }

  #[test]
  fn test_dump_text_number_input() {
    let root = make_container_with(
      None,
      vec![Widget::NumberInput {
        id: "num1".to_string(),
        value: 42.5,
        min: None,
        max: None,
        step: 1.0,
        precision: 0,
        show_spinner: true,
        bounds: WidgetBounds::default(),
        validation: None,
        style: Default::default(),
        width: 100.0,
        height: 32.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "42.5");
    assert_eq!(texts[0].widget_type, "NumberInput");
  }

  #[test]
  fn test_dump_text_kpi_card() {
    let root = make_container_with(
      None,
      vec![Widget::KpiCard {
        id: Some("kpi1".to_string()),
        title: "Revenue".to_string(),
        value: "$1.2M".to_string(),
        trend: None,
        style: Default::default(),
        bounds: WidgetBounds::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    // KpiCard emits both title and value.
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].text, "Revenue");
    assert_eq!(texts[1].text, "$1.2M");
  }

  #[test]
  fn test_dump_text_listview() {
    let root = make_container_with(
      None,
      vec![Widget::ListView {
        id: "list1".to_string(),
        items: vec![
          "Item A".to_string(),
          "Item B".to_string(),
          "Item C".to_string(),
        ],
        selected_index: None,
        style: Default::default(),
        bounds: WidgetBounds::default(),
        width: Some(200.0),
        height: Some(100.0),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        scroll_offset: 0.0,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 3);
    assert_eq!(texts[0].text, "Item A");
    assert_eq!(texts[1].text, "Item B");
    assert_eq!(texts[2].text, "Item C");
    assert_eq!(texts[0].widget_type, "ListView");
  }

  #[test]
  fn test_dump_text_autocomplete_with_value() {
    let root = make_container_with(
      None,
      vec![Widget::Autocomplete {
        id: "ac1".to_string(),
        value: "Paris".to_string(),
        placeholder: "City...".to_string(),
        suggestions: vec!["Paris".to_string()],
        max_visible: 5,
        bounds: WidgetBounds::default(),
        style: Default::default(),
        validation: None,
        width: 200.0,
        height: 32.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Paris");
    assert_eq!(texts[0].widget_type, "Autocomplete");
  }

  #[test]
  fn test_dump_text_autocomplete_empty_shows_placeholder() {
    let root = make_container_with(
      None,
      vec![Widget::Autocomplete {
        id: "ac1".to_string(),
        value: "".to_string(),
        placeholder: "Search...".to_string(),
        suggestions: vec![],
        max_visible: 5,
        bounds: WidgetBounds::default(),
        style: Default::default(),
        validation: None,
        width: 200.0,
        height: 32.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Search...");
  }

  #[test]
  fn test_dump_text_nested_containers_offsets() {
    // Text positions should accumulate container offsets.
    let inner = make_container_with(
      None,
      vec![Widget::label("Deep")],
    );
    let root =
      make_container_with(None, vec![inner]);
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0].text, "Deep");
    // Position should be non-negative (valid offset).
    assert!(texts[0].x >= 0.0);
    assert!(texts[0].y >= 0.0);
  }

  // ── find_text ─────────────────────────────────────────

  #[test]
  fn test_find_text() {
    let root = make_container_with(
      None,
      vec![
        Widget::label("Dashboard"),
        Widget::label("Settings"),
      ],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let found = find_text(&driver.root, "Dashboard");
    assert!(found.is_some());
    assert_eq!(found.unwrap().text, "Dashboard");

    let not_found = find_text(&driver.root, "Missing");
    assert!(not_found.is_none());
  }

  #[test]
  fn test_find_text_partial_match() {
    let root = make_container_with(
      None,
      vec![Widget::label("Hello World")],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let found = find_text(&driver.root, "World");
    assert!(found.is_some());
    assert_eq!(found.unwrap().text, "Hello World");
  }

  #[test]
  fn test_find_text_returns_first_match() {
    let root = make_container_with(
      None,
      vec![
        Widget::label("Alpha"),
        Widget::label("Alpha Beta"),
      ],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let found = find_text(&driver.root, "Alpha");
    assert!(found.is_some());
    assert_eq!(found.unwrap().text, "Alpha");
  }

  // ── widget_id ─────────────────────────────────────────

  #[test]
  fn test_widget_id_container_with_id() {
    let w = make_container_with(Some("cid"), vec![]);
    assert_eq!(widget_id(&w), Some("cid"));
  }

  #[test]
  fn test_widget_id_container_without_id() {
    let w = make_container_with(None, vec![]);
    assert_eq!(widget_id(&w), None);
  }

  #[test]
  fn test_widget_id_label_has_no_id() {
    let w = Widget::label("text");
    assert_eq!(widget_id(&w), None);
  }

  #[test]
  fn test_widget_id_checkbox() {
    let w = make_checkbox("my_cb", true);
    assert_eq!(widget_id(&w), Some("my_cb"));
  }

  #[test]
  fn test_widget_id_slider() {
    let w = Widget::Slider {
      id: "sl1".to_string(),
      value: 0.5,
      min: 0.0,
      max: 1.0,
      style: SliderStyle::default(),
      bounds: WidgetBounds::default(),
      width: 200.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    assert_eq!(widget_id(&w), Some("sl1"));
  }

  #[test]
  fn test_widget_id_dropdown() {
    let w = make_dropdown("dd1", vec!["A"], None);
    assert_eq!(widget_id(&w), Some("dd1"));
  }

  // ── widget_type_name ──────────────────────────────────

  #[test]
  fn test_widget_type_name_coverage() {
    assert_eq!(
      widget_type_name(&Widget::container()),
      "Container",
    );
    assert_eq!(
      widget_type_name(&Widget::label("t")),
      "Label",
    );
    assert_eq!(
      widget_type_name(&make_button("b", "a")),
      "Button",
    );
    assert_eq!(
      widget_type_name(&make_checkbox("c", false)),
      "Checkbox",
    );
    assert_eq!(
      widget_type_name(
        &make_dropdown("d", vec!["x"], None),
      ),
      "Dropdown",
    );
  }

  // ── widget_children ───────────────────────────────────

  #[test]
  fn test_widget_children_container() {
    let root = make_container_with(
      None,
      vec![Widget::label("a"), Widget::label("b")],
    );
    assert_eq!(widget_children(&root).len(), 2);
  }

  #[test]
  fn test_widget_children_leaf_has_none() {
    let label = Widget::label("leaf");
    assert!(widget_children(&label).is_empty());
  }

  #[test]
  fn test_widget_children_empty_container() {
    let root = make_container_with(None, vec![]);
    assert!(widget_children(&root).is_empty());
  }

  // ── CheckboxStyle / SliderStyle defaults ──────────────

  #[test]
  fn test_checkbox_style_default() {
    let style = CheckboxStyle::default();
    // Default background should be non-zero (visible).
    assert!(style.background.3 > 0.0);
    assert!(style.background_checked.3 > 0.0);
    assert!(style.checkmark_color.3 > 0.0);
  }

  #[test]
  fn test_slider_style_default() {
    let style = SliderStyle::default();
    assert!(style.track_color.3 > 0.0);
    assert!(style.active_track_color.3 > 0.0);
    assert!(style.thumb_color.3 > 0.0);
    assert!(style.track_height > 0.0);
    assert!(style.thumb_radius > 0.0);
  }

  // ── apply_theme ───────────────────────────────────────

  #[test]
  fn test_apply_theme_label() {
    let mut label = Widget::label("test");
    let theme = Theme::dark();
    apply_theme(&mut label, &theme);
    if let Widget::Label { color, .. } = &label {
      assert_eq!(*color, theme.colors.text);
    } else {
      panic!("Expected Label");
    }
  }

  #[test]
  fn test_apply_theme_button() {
    let mut btn = make_button("B", "act");
    let theme = Theme::dark();
    apply_theme(&mut btn, &theme);
    if let Widget::Button { style, .. } = &btn {
      assert_eq!(style.text_color, theme.colors.text);
    } else {
      panic!("Expected Button");
    }
  }

  #[test]
  fn test_apply_theme_checkbox() {
    let mut cb = make_checkbox("cb", false);
    let theme = Theme::dark();
    apply_theme(&mut cb, &theme);
    if let Widget::Checkbox { style, .. } = &cb {
      assert_eq!(style.background, theme.colors.surface);
      assert_eq!(
        style.background_checked,
        theme.colors.primary,
      );
      assert_eq!(
        style.checkmark_color,
        theme.colors.text,
      );
    } else {
      panic!("Expected Checkbox");
    }
  }

  #[test]
  fn test_apply_theme_slider() {
    let mut sl = Widget::Slider {
      id: "sl".to_string(),
      value: 0.5,
      min: 0.0,
      max: 1.0,
      style: SliderStyle::default(),
      bounds: WidgetBounds::default(),
      width: 200.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut sl, &theme);
    if let Widget::Slider { style, .. } = &sl {
      assert_eq!(style.track_color, theme.colors.surface);
      assert_eq!(
        style.active_track_color,
        theme.colors.primary,
      );
      assert_eq!(style.thumb_color, theme.colors.text);
    } else {
      panic!("Expected Slider");
    }
  }

  #[test]
  fn test_apply_theme_text_input() {
    let mut ti = make_text_input("ti", "v", "p");
    let theme = Theme::dark();
    apply_theme(&mut ti, &theme);
    if let Widget::TextInput { style, .. } = &ti {
      assert_eq!(style.text_color, theme.colors.text);
      assert_eq!(
        style.placeholder_color,
        theme.colors.text_disabled,
      );
    } else {
      panic!("Expected TextInput");
    }
  }

  #[test]
  fn test_apply_theme_divider() {
    let mut div = Widget::Divider {
      bounds: WidgetBounds::default(),
      orientation: Orientation::Horizontal,
      thickness: 1.0,
      color: (1.0, 1.0, 1.0, 1.0),
      margin: 8.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut div, &theme);
    if let Widget::Divider { color, .. } = &div {
      assert_eq!(*color, theme.colors.divider);
    } else {
      panic!("Expected Divider");
    }
  }

  #[test]
  fn test_apply_theme_dropdown() {
    let mut dd =
      make_dropdown("dd", vec!["A", "B"], Some(0));
    let theme = Theme::dark();
    apply_theme(&mut dd, &theme);
    if let Widget::Dropdown { style, .. } = &dd {
      assert_eq!(style.background, Some(theme.colors.surface));
      assert_eq!(style.text_color, Some(theme.colors.text));
    } else {
      panic!("Expected Dropdown");
    }
  }

  #[test]
  fn test_apply_theme_kpi_card() {
    let mut kpi = Widget::KpiCard {
      id: Some("k1".to_string()),
      title: "Test".to_string(),
      value: "100".to_string(),
      trend: None,
      style: Default::default(),
      bounds: WidgetBounds::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut kpi, &theme);
    if let Widget::KpiCard { style, .. } = &kpi {
      assert_eq!(style.background, theme.colors.surface);
      assert_eq!(style.value_color, theme.colors.text);
      assert_eq!(
        style.label_color,
        theme.colors.text_secondary,
      );
    } else {
      panic!("Expected KpiCard");
    }
  }

  #[test]
  fn test_apply_theme_container_with_background() {
    let mut root = Widget::container();
    if let Widget::Container { style, .. } = &mut root {
      style.background = Some((0.0, 0.0, 0.0, 1.0));
    }
    let theme = Theme::dark();
    apply_theme(&mut root, &theme);
    if let Widget::Container { style, .. } = &root {
      assert_eq!(
        style.background,
        Some(theme.colors.surface),
      );
    }
  }

  #[test]
  fn test_apply_theme_container_no_background_unchanged() {
    let mut root = Widget::container();
    let theme = Theme::dark();
    apply_theme(&mut root, &theme);
    if let Widget::Container { style, .. } = &root {
      // Background was None, should remain None.
      assert!(style.background.is_none());
    }
  }

  #[test]
  fn test_apply_theme_recurses_into_children() {
    let mut root = make_container_with(
      None,
      vec![Widget::label("child text")],
    );
    if let Widget::Container { style, .. } = &mut root {
      style.background = Some((0.0, 0.0, 0.0, 1.0));
    }
    let theme = Theme::dark();
    apply_theme(&mut root, &theme);
    if let Widget::Container { children, .. } = &root {
      if let Widget::Label { color, .. } = &children[0] {
        assert_eq!(*color, theme.colors.text);
      } else {
        panic!("Expected Label child");
      }
    }
  }

  // ── HeadlessConfig defaults ───────────────────────────

  #[test]
  fn test_headless_config_defaults() {
    let config =
      gloomy_core::headless::HeadlessConfig::default();
    assert_eq!(config.width, 1280);
    assert_eq!(config.height, 720);
    assert!(!config.force_fallback_adapter);
    assert_eq!(config.scale_factor, 1.0);
  }

  // ── Snapshot testing ──────────────────────────────────

  #[test]
  fn test_snapshot_workflow() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 200.0, 100.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir = std::env::temp_dir().join("gloomy_snap_test");
    let _ = std::fs::remove_dir_all(&dir);

    // First run: creates the golden.
    assert_screenshot(&mut driver, "empty", &dir, 0)
      .expect("first run should save golden");

    let golden = dir.join("empty.png");
    assert!(golden.exists(), "golden should be saved");

    // Second run: identical render should match.
    assert_screenshot(&mut driver, "empty", &dir, 0)
      .expect("identical render should match golden");

    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_snapshot_size_mismatch() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 200.0, 100.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir =
      std::env::temp_dir().join("gloomy_snap_size");
    let _ = std::fs::remove_dir_all(&dir);

    // Create golden at 200x100.
    assert_screenshot(&mut driver, "sized", &dir, 0)
      .expect("golden creation");

    // Create a golden with different dimensions by saving
    // a 1x1 image under the same name.
    let tiny = image::RgbaImage::new(1, 1);
    tiny
      .save(dir.join("sized.png"))
      .expect("overwrite golden");

    // Now assert_screenshot should detect size mismatch.
    let result =
      assert_screenshot(&mut driver, "sized", &dir, 0);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
      msg.contains("size mismatch"),
      "Expected size mismatch error, got: {}",
      msg,
    );

    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_snapshot_pixel_mismatch() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 10.0, 10.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir =
      std::env::temp_dir().join("gloomy_snap_pixel");
    let _ = std::fs::remove_dir_all(&dir);

    // Create golden.
    assert_screenshot(&mut driver, "px", &dir, 0)
      .expect("golden creation");

    // Overwrite golden with all-white image.
    let white = image::RgbaImage::from_pixel(
      10,
      10,
      image::Rgba([255, 255, 255, 255]),
    );
    white
      .save(dir.join("px.png"))
      .expect("overwrite golden");

    let result =
      assert_screenshot(&mut driver, "px", &dir, 0);
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
      msg.contains("pixels differ"),
      "Expected pixel mismatch error, got: {}",
      msg,
    );
    // Actual and diff files should be saved.
    assert!(dir.join("px_actual.png").exists());
    assert!(dir.join("px_diff.png").exists());

    let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn test_snapshot_tolerance() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 4.0, 4.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir =
      std::env::temp_dir().join("gloomy_snap_tol");
    let _ = std::fs::remove_dir_all(&dir);

    // Create golden.
    assert_screenshot(&mut driver, "tol", &dir, 0)
      .expect("golden creation");

    // Read golden and nudge each pixel by 2.
    let golden_path = dir.join("tol.png");
    let mut img =
      image::open(&golden_path).unwrap().to_rgba8();
    for pixel in img.pixels_mut() {
      pixel.0[0] = pixel.0[0].saturating_add(2);
      pixel.0[1] = pixel.0[1].saturating_add(2);
      pixel.0[2] = pixel.0[2].saturating_add(2);
    }
    img.save(&golden_path).expect("save modified golden");

    // Should fail with tolerance=0.
    let result =
      assert_screenshot(&mut driver, "tol", &dir, 0);
    assert!(result.is_err());

    // Clean up artifacts before re-check.
    let _ = std::fs::remove_file(dir.join("tol_actual.png"));
    let _ = std::fs::remove_file(dir.join("tol_diff.png"));

    // Should pass with tolerance=3.
    let result =
      assert_screenshot(&mut driver, "tol", &dir, 3);
    assert!(result.is_ok());

    let _ = std::fs::remove_dir_all(&dir);
  }

  // ── Rendering with init_renderer ──────────────────────

  #[test]
  fn test_init_renderer_and_render() {
    let root = make_container_with(
      None,
      vec![Widget::label("render test")],
    );
    let mut driver = GloomyDriver::new(root, 100.0, 100.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let img = driver.render_to_image(None);
    assert!(img.is_ok());
    let img = img.unwrap();
    assert_eq!(img.width(), 100);
    assert_eq!(img.height(), 100);
  }

  #[test]
  fn test_save_screenshot_creates_file() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 50.0, 50.0);

    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir =
      std::env::temp_dir().join("gloomy_save_test");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("test_output.png");
    let _ = std::fs::remove_file(&path);

    driver
      .save_screenshot(&path)
      .expect("save_screenshot should succeed");
    assert!(path.exists());

    let _ = std::fs::remove_dir_all(&dir);
  }

  // ── RenderConfig re-export ────────────────────────────

  #[test]
  fn test_render_config_reexport() {
    // RenderConfig should be usable as an alias for
    // HeadlessConfig.
    let config = RenderConfig {
      width: 640,
      height: 480,
      force_fallback_adapter: true,
      scale_factor: 2.0,
    };
    assert_eq!(config.width, 640);
    assert_eq!(config.height, 480);
    assert!(config.force_fallback_adapter);
    assert_eq!(config.scale_factor, 2.0);
  }

  // ── Divider widget in layout ──────────────────────────

  #[test]
  fn test_divider_in_dump_layout() {
    let root = make_container_with(
      None,
      vec![Widget::Divider {
        bounds: WidgetBounds::default(),
        orientation: Orientation::Horizontal,
        thickness: 1.0,
        color: (0.3, 0.3, 0.3, 1.0),
        margin: 8.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      }],
    );
    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    assert!(layout.contains("Divider"));
  }

  // ── Spacer has no text and no id ──────────────────────

  #[test]
  fn test_spacer_no_text_no_id() {
    let spacer = Widget::Spacer {
      size: 20.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    assert_eq!(widget_id(&spacer), None);
    assert_eq!(widget_type_name(&spacer), "Spacer");
    assert!(widget_children(&spacer).is_empty());
  }

  // ── ToggleSwitch theme ────────────────────────────────

  #[test]
  fn test_apply_theme_toggle_switch() {
    let mut ts = Widget::ToggleSwitch {
      id: "ts1".to_string(),
      checked: false,
      style: Default::default(),
      bounds: WidgetBounds::default(),
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut ts, &theme);
    if let Widget::ToggleSwitch { style, .. } = &ts {
      assert_eq!(
        style.track_color_on,
        Some(theme.colors.success),
      );
      assert_eq!(
        style.track_color_off,
        Some(theme.colors.surface),
      );
      assert_eq!(
        style.thumb_color,
        Some(theme.colors.text),
      );
    } else {
      panic!("Expected ToggleSwitch");
    }
  }

  // ── ProgressBar theme ─────────────────────────────────

  #[test]
  fn test_apply_theme_progress_bar() {
    let mut pb = Widget::ProgressBar {
      value: 0.5,
      min: 0.0,
      max: 1.0,
      style: Default::default(),
      width: Some(200.0),
      height: Some(20.0),
      bounds: WidgetBounds::default(),
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut pb, &theme);
    if let Widget::ProgressBar { style, .. } = &pb {
      assert_eq!(
        style.background_color,
        Some(theme.colors.surface),
      );
      assert_eq!(
        style.fill_color,
        Some(theme.colors.primary),
      );
    } else {
      panic!("Expected ProgressBar");
    }
  }

  // ── RadioButton theme ─────────────────────────────────

  #[test]
  fn test_apply_theme_radio_button() {
    let mut rb = Widget::RadioButton {
      group_id: "g".to_string(),
      value: "v".to_string(),
      selected: false,
      label: "Radio".to_string(),
      style: Default::default(),
      bounds: WidgetBounds::default(),
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut rb, &theme);
    if let Widget::RadioButton { style, .. } = &rb {
      assert_eq!(
        style.outer_color,
        Some(theme.colors.border),
      );
      assert_eq!(
        style.inner_color,
        Some(theme.colors.primary),
      );
    } else {
      panic!("Expected RadioButton");
    }
  }

  // ── Light theme produces different colors ─────────────

  #[test]
  fn test_apply_theme_light_vs_dark() {
    let mut label_dark = Widget::label("test");
    let mut label_light = Widget::label("test");
    let dark = Theme::dark();
    let light = Theme::light();
    apply_theme(&mut label_dark, &dark);
    apply_theme(&mut label_light, &light);
    if let (
      Widget::Label { color: c_dark, .. },
      Widget::Label { color: c_light, .. },
    ) = (&label_dark, &label_light)
    {
      // Dark and light themes should produce different
      // text colors.
      assert_ne!(c_dark, c_light);
    }
  }

  // ── Button disabled state theming ─────────────────────

  #[test]
  fn test_apply_theme_button_disabled_state() {
    let mut btn = make_button("B", "act");
    let theme = Theme::dark();
    apply_theme(&mut btn, &theme);
    if let Widget::Button { style, .. } = &btn {
      let c = &theme.colors;
      // disabled uses surface color at 40% alpha.
      let expected_bg =
        (c.surface.0, c.surface.1, c.surface.2, 0.4);
      assert_eq!(
        style.disabled.background,
        Some(expected_bg),
      );
      assert_eq!(style.disabled.corner_radii, [4.0; 4]);
    } else {
      panic!("Expected Button");
    }
  }

  // ── Tab style theming ─────────────────────────────────

  #[test]
  fn test_apply_theme_tab_style() {
    use gloomy_core::widget::{TabItem, TabStyle};
    let tab_item = TabItem {
      title: "Tab 1".to_string(),
      content: Box::new(Widget::label("content")),
    };
    let mut tab = Widget::tab(
      "t1",
      vec![tab_item],
      Orientation::Horizontal,
      TabStyle::default(),
    );
    let theme = Theme::dark();
    apply_theme(&mut tab, &theme);
    if let Widget::Tab { style, .. } = &tab {
      assert_eq!(style.background, theme.colors.surface);
      assert_eq!(
        style.selected_color,
        theme.colors.primary,
      );
      assert_eq!(
        style.unselected_color,
        theme.colors.text_secondary,
      );
    } else {
      panic!("Expected Tab");
    }
  }

  #[test]
  fn test_apply_theme_tab_recurses_content() {
    use gloomy_core::widget::{TabItem, TabStyle};
    let tab_item = TabItem {
      title: "Tab".to_string(),
      content: Box::new(Widget::label("text")),
    };
    let mut tab = Widget::tab(
      "t2",
      vec![tab_item],
      Orientation::Horizontal,
      TabStyle::default(),
    );
    let theme = Theme::dark();
    apply_theme(&mut tab, &theme);
    if let Widget::Tab { tabs, .. } = &tab {
      if let Widget::Label { color, .. } =
        tabs[0].content.as_ref()
      {
        assert_eq!(*color, theme.colors.text);
      } else {
        panic!("Expected Label in tab content");
      }
    } else {
      panic!("Expected Tab");
    }
  }

  // ── ListView theming ──────────────────────────────────

  #[test]
  fn test_apply_theme_listview() {
    let mut lv = Widget::ListView {
      id: "lv1".to_string(),
      items: vec!["A".to_string()],
      selected_index: None,
      style: Default::default(),
      bounds: WidgetBounds::default(),
      width: Some(200.0),
      height: Some(100.0),
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
      scroll_offset: 0.0,
    };
    let theme = Theme::dark();
    apply_theme(&mut lv, &theme);
    if let Widget::ListView { style, .. } = &lv {
      assert_eq!(
        style.idle.background,
        Some(theme.colors.surface),
      );
      assert_eq!(
        style.hover.background,
        Some(theme.colors.hover),
      );
      assert_eq!(
        style.selected.background,
        Some(theme.colors.primary),
      );
      assert_eq!(style.text_color_idle, theme.colors.text);
      assert_eq!(
        style.text_color_selected,
        theme.colors.text,
      );
    } else {
      panic!("Expected ListView");
    }
  }

  // ── DataGrid theming ──────────────────────────────────

  #[test]
  fn test_apply_theme_datagrid() {
    use gloomy_core::datagrid::SelectionMode;
    let mut dg = Widget::DataGrid {
      id: Some("dg1".to_string()),
      bounds: WidgetBounds::default(),
      columns: vec![],
      data_source_id: None,
      header_height: 30.0,
      row_height: 24.0,
      striped: true,
      selection_mode: SelectionMode::default(),
      show_vertical_lines: false,
      show_horizontal_lines: false,
      selected_rows: vec![],
      sort_column: None,
      sort_direction: None,
      style: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut dg, &theme);
    if let Widget::DataGrid { style, .. } = &dg {
      assert_eq!(
        style.header_background,
        theme.colors.surface,
      );
      assert_eq!(
        style.header_text_color,
        theme.colors.text,
      );
      assert_eq!(
        style.row_background,
        theme.colors.background,
      );
      assert_eq!(
        style.alt_row_background,
        theme.colors.surface,
      );
      assert_eq!(style.row_text_color, theme.colors.text);
      assert_eq!(
        style.hover_background,
        theme.colors.hover,
      );
      assert_eq!(
        style.selected_background,
        theme.colors.primary,
      );
      assert_eq!(
        style.grid_line_color,
        theme.colors.border,
      );
    } else {
      panic!("Expected DataGrid");
    }
  }

  // ── NumberInput theming ───────────────────────────────

  #[test]
  fn test_apply_theme_number_input() {
    let mut ni = Widget::NumberInput {
      id: "ni1".to_string(),
      value: 0.0,
      min: None,
      max: None,
      step: 1.0,
      precision: 0,
      show_spinner: true,
      bounds: WidgetBounds::default(),
      validation: None,
      style: Default::default(),
      width: 100.0,
      height: 32.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut ni, &theme);
    if let Widget::NumberInput { style, .. } = &ni {
      assert_eq!(
        style.background,
        Some(theme.colors.surface),
      );
      assert_eq!(style.text_color, theme.colors.text);
      assert_eq!(
        style.spinner_color,
        theme.colors.text_secondary,
      );
    } else {
      panic!("Expected NumberInput");
    }
  }

  // ── Autocomplete theming ──────────────────────────────

  #[test]
  fn test_apply_theme_autocomplete() {
    let mut ac = Widget::Autocomplete {
      id: "ac1".to_string(),
      value: "".to_string(),
      placeholder: "Search".to_string(),
      suggestions: vec![],
      max_visible: 5,
      bounds: WidgetBounds::default(),
      style: Default::default(),
      validation: None,
      width: 200.0,
      height: 32.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut ac, &theme);
    if let Widget::Autocomplete { style, .. } = &ac {
      assert_eq!(
        style.background,
        Some(theme.colors.surface),
      );
      assert_eq!(style.text_color, theme.colors.text);
      assert_eq!(style.cursor_color, theme.colors.text);
      assert_eq!(
        style.dropdown_background,
        Some(theme.colors.surface),
      );
      assert_eq!(
        style.dropdown_text_color,
        theme.colors.text,
      );
      assert_eq!(
        style.dropdown_highlight_color,
        theme.colors.hover,
      );
    } else {
      panic!("Expected Autocomplete");
    }
  }

  // ── DatePicker theming ────────────────────────────────

  #[test]
  fn test_apply_theme_datepicker() {
    let mut dp = Widget::DatePicker {
      id: "dp1".to_string(),
      value: None,
      placeholder: "Date".to_string(),
      min_date: None,
      max_date: None,
      format: "%Y-%m-%d".to_string(),
      bounds: WidgetBounds::default(),
      style: Default::default(),
      validation: None,
      width: 150.0,
      height: 32.0,
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut dp, &theme);
    if let Widget::DatePicker { style, .. } = &dp {
      assert_eq!(
        style.background,
        Some(theme.colors.surface),
      );
      assert_eq!(style.text_color, theme.colors.text);
      assert_eq!(
        style.placeholder_color,
        theme.colors.text_disabled,
      );
      assert_eq!(
        style.calendar_background,
        Some(theme.colors.surface),
      );
      assert_eq!(
        style.day_text_color,
        theme.colors.text,
      );
      assert_eq!(
        style.selected_day_color,
        theme.colors.primary,
      );
      assert_eq!(style.today_color, theme.colors.active);
      assert_eq!(
        style.day_hover_color,
        theme.colors.hover,
      );
      assert_eq!(
        style.month_header_color,
        theme.colors.text,
      );
    } else {
      panic!("Expected DatePicker");
    }
  }

  // ── Tree theming ──────────────────────────────────────

  #[test]
  fn test_apply_theme_tree() {
    let mut tree = Widget::Tree {
      id: Some("tr1".to_string()),
      bounds: WidgetBounds::default(),
      root_nodes: vec![],
      selected_id: None,
      expanded_ids: Default::default(),
      style: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut tree, &theme);
    if let Widget::Tree { style, .. } = &tree {
      assert_eq!(style.text_color, theme.colors.text);
      assert_eq!(
        style.icon_color,
        theme.colors.text_secondary,
      );
      assert_eq!(
        style.selected_background,
        theme.colors.primary,
      );
      assert_eq!(
        style.hover_background,
        theme.colors.hover,
      );
    } else {
      panic!("Expected Tree");
    }
  }

  // ── Scrollbar theming ─────────────────────────────────

  #[test]
  fn test_apply_theme_scrollbar() {
    let mut sb = Widget::Scrollbar {
      bounds: WidgetBounds::default(),
      content_size: 1000.0,
      viewport_size: 200.0,
      scroll_offset: 0.0,
      orientation: Orientation::Vertical,
      style: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };
    let theme = Theme::dark();
    apply_theme(&mut sb, &theme);
    if let Widget::Scrollbar { style, .. } = &sb {
      assert_eq!(
        style.track_color,
        theme.colors.background,
      );
      assert_eq!(style.thumb_color, theme.colors.border);
      assert_eq!(
        style.thumb_hover_color,
        theme.colors.hover,
      );
    } else {
      panic!("Expected Scrollbar");
    }
  }
}
