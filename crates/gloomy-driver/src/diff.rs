//! Pixel-level image comparison with structured diff output.

use gloomy_core::widget::{Widget, WidgetBounds};
use image::RgbaImage;
use serde::Serialize;
use std::collections::VecDeque;

/// Configuration for image comparison.
#[derive(Debug, Clone)]
pub struct DiffConfig {
  /// Per-channel threshold below which differences are ignored.
  /// 0 means exact match required.
  pub threshold: u8,
  /// Grid cell size (pixels) for clustering diffs into regions.
  pub cell_size: u32,
}

impl Default for DiffConfig {
  fn default() -> Self {
    Self {
      threshold: 0,
      cell_size: 32,
    }
  }
}

/// Structured diff report suitable for JSON serialisation.
#[derive(Debug, Clone, Serialize)]
pub struct DiffReport {
  /// Whether the images match within the configured threshold.
  pub passed: bool,
  /// Fraction of pixels that differ (0.0 – 1.0).
  pub diff_ratio: f64,
  /// Absolute count of differing pixels.
  pub diff_pixels: u64,
  /// Total pixel count.
  pub total_pixels: u64,
  /// Maximum per-channel difference across all pixels.
  pub max_channel_diff: u8,
  /// Image width.
  pub width: u32,
  /// Image height.
  pub height: u32,
  /// Clustered diff regions.
  pub regions: Vec<DiffRegion>,
}

/// A contiguous region of pixel differences.
#[derive(Debug, Clone, Serialize)]
pub struct DiffRegion {
  /// Left edge (pixels).
  pub x: u32,
  /// Top edge (pixels).
  pub y: u32,
  /// Region width (pixels).
  pub width: u32,
  /// Region height (pixels).
  pub height: u32,
  /// Fraction of pixels within this region that differ.
  pub diff_ratio: f64,
  /// Absolute count of differing pixels in this region.
  pub diff_pixels: u64,
  /// Severity label: "low", "medium", or "high".
  pub severity: String,
  /// ID of the deepest widget overlapping this region, if any.
  pub widget_id: Option<String>,
}

/// Compares two RGBA images and produces a structured diff report.
///
/// The optional `widget_tree` is used to map diff regions to
/// widget IDs for more actionable output.
pub fn compare_images(
  expected: &RgbaImage,
  actual: &RgbaImage,
  config: &DiffConfig,
  widget_tree: Option<&Widget>,
) -> DiffReport {
  let w = expected.width();
  let h = expected.height();
  assert_eq!(
    (w, h),
    (actual.width(), actual.height()),
    "image dimensions must match"
  );

  let total_pixels = (w as u64) * (h as u64);

  // Per-pixel diff mask.
  let mut diff_mask = vec![false; (w * h) as usize];
  let mut diff_pixels: u64 = 0;
  let mut max_channel_diff: u8 = 0;

  let exp_raw = expected.as_raw();
  let act_raw = actual.as_raw();
  for i in 0..((w * h) as usize) {
    let base = i * 4;
    let dr = exp_raw[base].abs_diff(act_raw[base]);
    let dg = exp_raw[base + 1].abs_diff(act_raw[base + 1]);
    let db = exp_raw[base + 2].abs_diff(act_raw[base + 2]);
    let da = exp_raw[base + 3].abs_diff(act_raw[base + 3]);
    let max_d = dr.max(dg).max(db).max(da);
    if max_d > config.threshold {
      diff_mask[i] = true;
      diff_pixels += 1;
    }
    max_channel_diff = max_channel_diff.max(max_d);
  }

  let passed = diff_pixels == 0;
  let diff_ratio = if total_pixels > 0 {
    diff_pixels as f64 / total_pixels as f64
  } else {
    0.0
  };

  // Grid clustering: count diffs per cell.
  let cs = config.cell_size.max(1);
  let cols = (w + cs - 1) / cs;
  let rows = (h + cs - 1) / cs;
  let grid_len = (cols * rows) as usize;
  let mut grid_counts = vec![0u32; grid_len];

  for py in 0..h {
    for px in 0..w {
      if diff_mask[(py * w + px) as usize] {
        let gc = px / cs;
        let gr = py / cs;
        grid_counts[(gr * cols + gc) as usize] += 1;
      }
    }
  }

  // Flood-fill adjacent nonzero cells into regions.
  let mut visited = vec![false; grid_len];
  let mut regions = Vec::new();

  for start in 0..grid_len {
    if grid_counts[start] == 0 || visited[start] {
      continue;
    }
    // BFS.
    let mut queue = VecDeque::new();
    queue.push_back(start);
    visited[start] = true;
    let mut min_gc = (start as u32) % cols;
    let mut max_gc = min_gc;
    let mut min_gr = (start as u32) / cols;
    let mut max_gr = min_gr;
    let mut region_diff: u64 = 0;

    while let Some(idx) = queue.pop_front() {
      let gc = (idx as u32) % cols;
      let gr = (idx as u32) / cols;
      region_diff += grid_counts[idx] as u64;
      min_gc = min_gc.min(gc);
      max_gc = max_gc.max(gc);
      min_gr = min_gr.min(gr);
      max_gr = max_gr.max(gr);

      // 4-connected neighbours.
      for (dc, dr) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
        let nc = gc as i32 + dc;
        let nr = gr as i32 + dr;
        if nc < 0 || nr < 0
          || nc >= cols as i32
          || nr >= rows as i32
        {
          continue;
        }
        let ni = (nr as u32 * cols + nc as u32) as usize;
        if !visited[ni] && grid_counts[ni] > 0 {
          visited[ni] = true;
          queue.push_back(ni);
        }
      }
    }

    let rx = min_gc * cs;
    let ry = min_gr * cs;
    let rw = ((max_gc + 1) * cs).min(w) - rx;
    let rh = ((max_gr + 1) * cs).min(h) - ry;
    let region_total = (rw as u64) * (rh as u64);
    let region_ratio = if region_total > 0 {
      region_diff as f64 / region_total as f64
    } else {
      0.0
    };

    let severity = if region_ratio < 0.05 {
      "low"
    } else if region_ratio < 0.25 {
      "medium"
    } else {
      "high"
    }
    .to_string();

    let widget_id = widget_tree.and_then(|tree| {
      let cx = rx as f32 + rw as f32 / 2.0;
      let cy = ry as f32 + rh as f32 / 2.0;
      find_deepest_widget(tree, cx, cy)
    });

    regions.push(DiffRegion {
      x: rx,
      y: ry,
      width: rw,
      height: rh,
      diff_ratio: region_ratio,
      diff_pixels: region_diff,
      severity,
      widget_id,
    });
  }

  DiffReport {
    passed,
    diff_ratio,
    diff_pixels,
    total_pixels,
    max_channel_diff,
    width: w,
    height: h,
    regions,
  }
}

/// Walks the widget tree and returns the ID of the deepest widget
/// whose bounds contain the point (px, py).
fn find_deepest_widget(
  widget: &Widget,
  px: f32,
  py: f32,
) -> Option<String> {
  let bounds = widget_bounds(widget);
  if !bounds_contain(&bounds, px, py) {
    return None;
  }

  // Check children first (depth-first, last match wins).
  if let Widget::Container { children, .. } = widget {
    for child in children.iter().rev() {
      if let Some(id) = find_deepest_widget(child, px, py) {
        return Some(id);
      }
    }
  }

  widget_id(widget).map(|s| s.to_string())
}

fn bounds_contain(b: &WidgetBounds, px: f32, py: f32) -> bool {
  px >= b.x
    && px < b.x + b.width
    && py >= b.y
    && py < b.y + b.height
}

fn widget_bounds(widget: &Widget) -> WidgetBounds {
  match widget {
    Widget::Container { bounds, .. }
    | Widget::Button { bounds, .. }
    | Widget::TextInput { bounds, .. }
    | Widget::NumberInput { bounds, .. }
    | Widget::DatePicker { bounds, .. }
    | Widget::Autocomplete { bounds, .. }
    | Widget::Checkbox { bounds, .. }
    | Widget::Slider { bounds, .. }
    | Widget::Dropdown { bounds, .. }
    | Widget::ToggleSwitch { bounds, .. }
    | Widget::ProgressBar { bounds, .. }
    | Widget::RadioButton { bounds, .. }
    | Widget::Divider { bounds, .. }
    | Widget::Scrollbar { bounds, .. }
    | Widget::DataGrid { bounds, .. }
    | Widget::KpiCard { bounds, .. }
    | Widget::ListView { bounds, .. }
    | Widget::Image { bounds, .. }
    | Widget::Icon { bounds, .. }
    | Widget::Tab { bounds, .. } => *bounds,
    Widget::Label { x, y, width, height, .. } => {
      WidgetBounds {
        x: *x,
        y: *y,
        width: *width,
        height: *height,
      }
    }
    Widget::Tree { .. } | Widget::Spacer { .. } => {
      WidgetBounds::default()
    }
  }
}

fn widget_id(widget: &Widget) -> Option<&str> {
  match widget {
    Widget::Container { id, .. } => id.as_deref(),
    Widget::ToggleSwitch { id, .. }
    | Widget::TextInput { id, .. }
    | Widget::NumberInput { id, .. }
    | Widget::DatePicker { id, .. }
    | Widget::Autocomplete { id, .. }
    | Widget::Checkbox { id, .. }
    | Widget::Slider { id, .. }
    | Widget::Dropdown { id, .. }
    | Widget::Icon { id, .. } => Some(id),
    Widget::KpiCard { id, .. }
    | Widget::DataGrid { id, .. } => id.as_deref(),
    Widget::Button { action, .. } => Some(action.as_str()),
    Widget::Label { text, .. } => Some(text.as_str()),
    Widget::RadioButton { value, .. } => Some(value.as_str()),
    Widget::ListView { id, .. } => Some(id.as_str()),
    _ => None,
  }
}

/// Describes a widget in the tree with its type and bounds.
#[derive(Debug, Clone, Serialize)]
pub struct WidgetInfo {
  /// Widget variant name (e.g. "Button", "Container").
  pub widget_type: String,
  /// Widget ID, if present.
  pub id: Option<String>,
  /// Left edge (logical pixels).
  pub x: f32,
  /// Top edge (logical pixels).
  pub y: f32,
  /// Width (logical pixels).
  pub width: f32,
  /// Height (logical pixels).
  pub height: f32,
  /// Child widgets (Container children only).
  pub children: Vec<WidgetInfo>,
}

/// Recursively collects widget info from a tree.
pub fn collect_widgets(widget: &Widget) -> WidgetInfo {
  let bounds = widget_bounds(widget);
  let id = widget_id(widget).map(|s| s.to_string());
  let wtype = widget_type_name(widget).to_string();
  let children = match widget {
    Widget::Container { children, .. } => {
      children.iter().map(collect_widgets).collect()
    }
    _ => Vec::new(),
  };
  WidgetInfo {
    widget_type: wtype,
    id,
    x: bounds.x,
    y: bounds.y,
    width: bounds.width,
    height: bounds.height,
    children,
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
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn solid_image(w: u32, h: u32, rgba: [u8; 4]) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for p in img.pixels_mut() {
      *p = image::Rgba(rgba);
    }
    img
  }

  #[test]
  fn identical_images_pass() {
    let a = solid_image(64, 64, [128, 0, 0, 255]);
    let b = solid_image(64, 64, [128, 0, 0, 255]);
    let report =
      compare_images(&a, &b, &DiffConfig::default(), None);
    assert!(report.passed);
    assert_eq!(report.diff_pixels, 0);
    assert_eq!(report.diff_ratio, 0.0);
    assert!(report.regions.is_empty());
  }

  #[test]
  fn single_pixel_diff_detected() {
    let a = solid_image(64, 64, [0, 0, 0, 255]);
    let mut b = a.clone();
    b.put_pixel(10, 10, image::Rgba([255, 0, 0, 255]));
    let report =
      compare_images(&a, &b, &DiffConfig::default(), None);
    assert!(!report.passed);
    assert_eq!(report.diff_pixels, 1);
    assert_eq!(report.regions.len(), 1);
    // Pixel (10, 10) falls in cell (0, 0).
    assert_eq!(report.regions[0].x, 0);
    assert_eq!(report.regions[0].y, 0);
  }

  #[test]
  fn two_separate_blobs_produce_two_regions() {
    let a = solid_image(128, 128, [0, 0, 0, 255]);
    let mut b = a.clone();
    // Blob 1: top-left corner.
    for y in 0..5 {
      for x in 0..5 {
        b.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
      }
    }
    // Blob 2: bottom-right corner (well separated).
    for y in 100..105 {
      for x in 100..105 {
        b.put_pixel(x, y, image::Rgba([0, 255, 0, 255]));
      }
    }
    let report =
      compare_images(&a, &b, &DiffConfig::default(), None);
    assert!(!report.passed);
    assert_eq!(report.regions.len(), 2);
  }

  #[test]
  fn threshold_filters_small_diffs() {
    let a = solid_image(32, 32, [100, 100, 100, 255]);
    let b = solid_image(32, 32, [103, 100, 100, 255]);
    let strict =
      compare_images(&a, &b, &DiffConfig::default(), None);
    assert!(!strict.passed);

    let lenient = compare_images(
      &a,
      &b,
      &DiffConfig { threshold: 5, ..Default::default() },
      None,
    );
    assert!(lenient.passed);
  }

  #[test]
  fn widget_id_mapped_to_region() {
    let mut root = Widget::container();
    if let Widget::Container {
      id, children, bounds, ..
    } = &mut root
    {
      *id = Some("root".to_string());
      bounds.width = 64.0;
      bounds.height = 64.0;
      *children = vec![Widget::Button {
        text: "btn".to_string(),
        action: "act".to_string(),
        bounds: WidgetBounds {
          x: 0.0,
          y: 0.0,
          width: 64.0,
          height: 32.0,
        },
        style: Default::default(),
        width: Some(64.0),
        height: Some(32.0),
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

    let a = solid_image(64, 64, [0, 0, 0, 255]);
    let mut b = a.clone();
    // Diff inside the button area.
    b.put_pixel(5, 5, image::Rgba([255, 0, 0, 255]));

    let report = compare_images(
      &a,
      &b,
      &DiffConfig::default(),
      Some(&root),
    );
    assert_eq!(report.regions.len(), 1);
    // The deepest widget at (5, 5) is the button; widget_id
    // returns the button's action field.
    assert_eq!(
      report.regions[0].widget_id.as_deref(),
      Some("act")
    );
  }

  // 4×4 image, single cell_size=4 cell → region covers 16 pixels.
  // 1 diff pixel = 6.25% → "medium"; 4 diff pixels = 25% → "high".
  #[test]
  fn severity_labels_medium_and_high() {
    let a = solid_image(4, 4, [0, 0, 0, 255]);
    let cfg = DiffConfig { cell_size: 4, ..Default::default() };

    let mut b = a.clone();
    b.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
    let r = compare_images(&a, &b, &cfg, None);
    assert_eq!(r.regions[0].severity, "medium");

    let mut b = a.clone();
    for x in 0..4 {
      b.put_pixel(x, 0, image::Rgba([255, 0, 0, 255]));
    }
    let r = compare_images(&a, &b, &cfg, None);
    assert_eq!(r.regions[0].severity, "high");
  }

  // Sum of per-region diff_pixels must equal the global diff_pixels.
  #[test]
  fn region_diff_pixels_sum_equals_total() {
    let a = solid_image(128, 128, [0, 0, 0, 255]);
    let mut b = a.clone();
    for y in 0..5 {
      for x in 0..5 {
        b.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
      }
    }
    for y in 100..110 {
      for x in 100..110 {
        b.put_pixel(x, y, image::Rgba([0, 255, 0, 255]));
      }
    }
    let r = compare_images(&a, &b, &DiffConfig::default(), None);
    let sum: u64 = r.regions.iter().map(|rg| rg.diff_pixels).sum();
    assert_eq!(sum, r.diff_pixels);
  }

  #[test]
  fn collect_widgets_captures_tree() {
    let mut root = Widget::container();
    if let Widget::Container {
      id, children, bounds, ..
    } = &mut root
    {
      *id = Some("root".to_string());
      bounds.width = 100.0;
      bounds.height = 80.0;
      *children = vec![Widget::Button {
        text: "btn".to_string(),
        action: "act".to_string(),
        bounds: WidgetBounds {
          x: 0.0,
          y: 0.0,
          width: 50.0,
          height: 30.0,
        },
        style: Default::default(),
        width: Some(50.0),
        height: Some(30.0),
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

    let info = collect_widgets(&root);
    assert_eq!(info.widget_type, "Container");
    assert_eq!(info.id.as_deref(), Some("root"));
    assert_eq!(info.width, 100.0);
    assert_eq!(info.height, 80.0);
    assert_eq!(info.children.len(), 1);

    let child = &info.children[0];
    assert_eq!(child.widget_type, "Button");
    // widget_id() returns the action field for Button.
    assert_eq!(child.id.as_deref(), Some("act"));
    assert_eq!(child.children.len(), 0);
  }

  // Nested containers: the innermost widget with an id should win.
  #[test]
  fn find_deepest_widget_prefers_deepest_named() {
    let toggle = Widget::ToggleSwitch {
      id: "toggle".to_string(),
      checked: false,
      style: Default::default(),
      bounds: WidgetBounds {
        x: 30.0,
        y: 30.0,
        width: 20.0,
        height: 20.0,
      },
      layout: Default::default(),
      flex: 0.0,
      grid_col: None,
      grid_row: None,
      col_span: 1,
      row_span: 1,
    };

    let mut inner = Widget::container();
    if let Widget::Container { id, children, bounds, .. } =
      &mut inner
    {
      *id = Some("inner".to_string());
      bounds.x = 20.0;
      bounds.y = 20.0;
      bounds.width = 60.0;
      bounds.height = 60.0;
      *children = vec![toggle];
    }

    let mut outer = Widget::container();
    if let Widget::Container { id, children, bounds, .. } =
      &mut outer
    {
      *id = Some("outer".to_string());
      bounds.width = 100.0;
      bounds.height = 100.0;
      *children = vec![inner];
    }

    let a = solid_image(100, 100, [0, 0, 0, 255]);
    let mut b = a.clone();
    // Diff at (35, 35): inside toggle bounds.
    b.put_pixel(35, 35, image::Rgba([255, 0, 0, 255]));

    // cell_size=32 → diff pixel at (35,35) lands in cell (1,1),
    // region x=32 y=32 w=32 h=32, center (48,48).
    // (48,48) is inside toggle bounds [30..50, 30..50], so
    // find_deepest_widget returns "toggle" over "inner".
    let r = compare_images(
      &a,
      &b,
      &DiffConfig { cell_size: 32, ..Default::default() },
      Some(&outer),
    );
    assert_eq!(r.regions.len(), 1);
    assert_eq!(
      r.regions[0].widget_id.as_deref(),
      Some("toggle")
    );
  }

  // widget_id() for the newly-wired variants: Label → text,
  // RadioButton → value, ListView → id.
  #[test]
  fn widget_id_mapped_for_label_radiobutton_listview() {
    let mut root = Widget::container();
    if let Widget::Container {
      id, children, bounds, ..
    } = &mut root
    {
      *id = Some("root".to_string());
      bounds.width = 200.0;
      bounds.height = 200.0;
      *children = vec![
        Widget::Label {
          text: "hello".to_string(),
          x: 0.0,
          y: 0.0,
          width: 60.0,
          height: 20.0,
          size: 14.0,
          color: (0.0, 0.0, 0.0, 1.0),
          text_align: Default::default(),
          flex: 0.0,
          grid_col: None,
          grid_row: None,
          col_span: 1,
          row_span: 1,
          font: None,
        },
        Widget::RadioButton {
          group_id: "grp".to_string(),
          value: "opt_a".to_string(),
          selected: false,
          label: "Option A".to_string(),
          style: Default::default(),
          bounds: WidgetBounds {
            x: 0.0,
            y: 30.0,
            width: 60.0,
            height: 20.0,
          },
          layout: Default::default(),
          flex: 0.0,
          grid_col: None,
          grid_row: None,
          col_span: 1,
          row_span: 1,
        },
        Widget::ListView {
          id: "list1".to_string(),
          items: vec!["a".to_string()],
          selected_index: None,
          style: Default::default(),
          bounds: WidgetBounds {
            x: 0.0,
            y: 60.0,
            width: 60.0,
            height: 40.0,
          },
          width: None,
          height: None,
          layout: Default::default(),
          flex: 0.0,
          grid_col: None,
          grid_row: None,
          col_span: 1,
          row_span: 1,
          scroll_offset: 0.0,
        },
      ];
    }

    let info = collect_widgets(&root);
    assert_eq!(info.children.len(), 3);
    // Label: id = text content.
    assert_eq!(info.children[0].widget_type, "Label");
    assert_eq!(info.children[0].id.as_deref(), Some("hello"));
    // RadioButton: id = value field.
    assert_eq!(info.children[1].widget_type, "RadioButton");
    assert_eq!(info.children[1].id.as_deref(), Some("opt_a"));
    // ListView: id = id field.
    assert_eq!(info.children[2].widget_type, "ListView");
    assert_eq!(info.children[2].id.as_deref(), Some("list1"));
  }

  // cell_size=0 is clamped to 1 internally; must not panic.
  #[test]
  fn cell_size_zero_does_not_panic() {
    let a = solid_image(32, 32, [0, 0, 0, 255]);
    let mut b = a.clone();
    b.put_pixel(5, 5, image::Rgba([255, 0, 0, 255]));
    let r = compare_images(
      &a,
      &b,
      &DiffConfig { cell_size: 0, ..Default::default() },
      None,
    );
    assert!(!r.passed);
    assert_eq!(r.diff_pixels, 1);
  }
}
