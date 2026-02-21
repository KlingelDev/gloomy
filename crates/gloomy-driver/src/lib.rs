//! Automation API for testing Gloomy UIs.
//!
//! Provides headless rendering, interaction simulation, layout
//! inspection, text extraction, and visual snapshot testing.

use gloomy_core::data_source::DataProvider;
use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::widget::{Widget, WidgetBounds};
use gloomy_core::{
  InteractionState, compute_layout, hit_test,
};
use glam::Vec2;
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
  renderer: Option<HeadlessRenderer>,
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
      renderer: None,
    }
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
    self.renderer = Some(HeadlessRenderer::new(config)?);
    Ok(())
  }

  /// Renders the current widget tree to an RGBA image.
  pub fn render_to_image(
    &mut self,
  ) -> anyhow::Result<image::RgbaImage> {
    let r = self.renderer.as_mut().ok_or_else(|| {
      anyhow::anyhow!("Call init_renderer() first")
    })?;
    r.render_to_image(
      &mut self.root,
      Some(&self.interaction),
      None,
    )
  }

  /// Renders with a data provider.
  pub fn render_to_image_with_data(
    &mut self,
    data_provider: &dyn DataProvider,
  ) -> anyhow::Result<image::RgbaImage> {
    let r = self.renderer.as_mut().ok_or_else(|| {
      anyhow::anyhow!("Call init_renderer() first")
    })?;
    r.render_to_image(
      &mut self.root,
      Some(&self.interaction),
      Some(data_provider),
    )
  }

  /// Renders the widget tree and saves as a PNG.
  pub fn save_screenshot(
    &mut self,
    path: impl AsRef<Path>,
  ) -> anyhow::Result<()> {
    let img = self.render_to_image()?;
    img.save(path)?;
    Ok(())
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

  let actual = driver.render_to_image()?;

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
    let pct = (mismatch_count as f64 / total as f64) * 100.0;
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
      style.idle = gloomy_core::style::BoxStyle::fill(c.surface)
        .with_radius(4.0);
      style.hover =
        gloomy_core::style::BoxStyle::fill(c.hover)
          .with_radius(4.0);
      style.active =
        gloomy_core::style::BoxStyle::fill(c.active)
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
    Widget::Tab { tabs, .. } => {
      for tab in tabs.iter_mut() {
        apply_theme(&mut tab.content, theme);
      }
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

  #[test]
  fn test_driver_find() {
    let mut root = Widget::container();
    if let Widget::Container { id, children, .. } =
      &mut root
    {
      *id = Some("root".to_string());
      *children = vec![Widget::Button {
        text: "Click Me".to_string(),
        action: "my_action".to_string(),
        bounds: WidgetBounds {
          x: 0.0,
          y: 0.0,
          width: 100.0,
          height: 50.0,
        },
        style: ButtonStyle::default(),
        width: Some(100.0),
        height: Some(50.0),
        disabled: false,
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        font: None,
      }];
    }

    let driver = GloomyDriver::new(root, 800.0, 600.0);
    assert!(driver.find("root").is_some());
  }

  #[test]
  fn test_dump_layout() {
    let mut root = Widget::container();
    if let Widget::Container {
      id, children, ..
    } = &mut root
    {
      *id = Some("main".to_string());
      *children = vec![Widget::label("Hello")];
    }

    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let layout = dump_layout(&driver.root);
    assert!(layout.contains("Container"));
    assert!(layout.contains("Label"));
    assert!(layout.contains("id=main"));
  }

  #[test]
  fn test_dump_text() {
    let mut root = Widget::container();
    if let Widget::Container { children, .. } = &mut root {
      *children = vec![
        Widget::label("Hello World"),
        Widget::Button {
          text: "OK".to_string(),
          action: "ok".to_string(),
          bounds: WidgetBounds::default(),
          style: ButtonStyle::default(),
          width: None,
          height: None,
          disabled: false,
          layout: Default::default(),
          flex: 0.0,
          grid_col: None,
          grid_row: None,
          col_span: 1,
          row_span: 1,
          font: None,
        },
      ];
    }

    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let texts = dump_text(&driver.root);
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[0].text, "Hello World");
    assert_eq!(texts[0].widget_type, "Label");
    assert_eq!(texts[1].text, "OK");
    assert_eq!(texts[1].widget_type, "Button");
  }

  #[test]
  fn test_find_text() {
    let mut root = Widget::container();
    if let Widget::Container { children, .. } = &mut root {
      *children = vec![
        Widget::label("Dashboard"),
        Widget::label("Settings"),
      ];
    }

    let driver = GloomyDriver::new(root, 400.0, 300.0);
    let found = find_text(&driver.root, "Dashboard");
    assert!(found.is_some());
    assert_eq!(found.unwrap().text, "Dashboard");

    let not_found = find_text(&driver.root, "Missing");
    assert!(not_found.is_none());
  }

  #[test]
  fn test_snapshot_workflow() {
    let root = Widget::container();
    let mut driver = GloomyDriver::new(root, 200.0, 100.0);

    // init_renderer may fail if no GPU/software adapter is
    // available; skip the test in that case.
    if driver.init_renderer(true).is_err() {
      eprintln!("Skipping: no GPU adapter available");
      return;
    }

    let dir = std::env::temp_dir().join("gloomy_snap_test");
    let _ = std::fs::remove_dir_all(&dir);

    // First run: creates the golden.
    assert_screenshot(
      &mut driver, "empty", &dir, 0,
    )
    .expect("first run should save golden");

    let golden = dir.join("empty.png");
    assert!(golden.exists(), "golden should be saved");

    // Second run: identical render should match.
    assert_screenshot(
      &mut driver, "empty", &dir, 0,
    )
    .expect("identical render should match golden");

    let _ = std::fs::remove_dir_all(&dir);
  }
}
