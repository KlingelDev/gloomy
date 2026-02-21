//! Visual showcase renderer for Gloomy widget development.
//!
//! Renders a comprehensive, labeled widget showcase to PNG so
//! both humans and AI agents can inspect rendered output.
//! With `--live`, opens a windowed app for real-time viewing.
//!
//! Usage:
//!   cargo run --example visual_check
//!   cargo run --example visual_check -- buttons
//!   cargo run --example visual_check -- --live
//!   cargo run --example visual_check -- --live buttons

use gloomy_app::GloomyApp;
use gloomy_core::style::TextInputStyle;
use gloomy_core::widget::{
  CheckboxStyle, Color, ProgressBarStyle, SliderStyle,
  TextAlign, ToggleSwitchStyle,
};
use gloomy_core::{
  compute_layout, render_ui, Align, BoxStyle, ButtonStyle,
  Direction, InteractionState, Justify, KpiCardStyle,
  KpiTrend, Layout, TrendDirection, Vec2, Widget,
  WidgetBounds,
};
use gloomy_driver::diff::collect_widgets;
use gloomy_driver::screenshot::HeadlessRenderer;
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

const WIDTH: u32 = 1200;
const HEIGHT: u32 = 900;
const SCALE: f32 = 1.0;
const SECTION_H: f32 = 85.0;
const HEADER_H: f32 = 55.0;
const LABEL_W: f32 = 160.0;

fn snapshot_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("snapshots")
}

// -- Helpers ------------------------------------------------

/// Builds a Container widget with the given layout parameters.
fn ctn(
  dir: Direction,
  spacing: f32,
  pad: f32,
  cw: Option<f32>,
  ch: Option<f32>,
  cflex: f32,
  sty: BoxStyle,
  align: Align,
  kids: Vec<Widget>,
) -> Widget {
  let mut w = Widget::container();
  if let Widget::Container {
    style,
    padding,
    layout,
    width,
    height,
    flex,
    children,
    ..
  } = &mut w
  {
    *style = sty;
    *padding = pad;
    *layout = Layout {
      direction: dir,
      spacing,
      align_items: align,
      justify_content: Justify::Start,
      template_columns: Vec::new(),
    };
    *width = cw;
    *height = ch;
    *flex = cflex;
    *children = kids;
  }
  w
}

fn vbox(spacing: f32, kids: Vec<Widget>) -> Widget {
  ctn(
    Direction::Column,
    spacing,
    0.0,
    None,
    None,
    0.0,
    BoxStyle::default(),
    Align::Stretch,
    kids,
  )
}

/// Creates a Label with flex=1 so it stretches to fill
/// the parent container's available space. This ensures
/// the scissor rect is wide enough for the rendered text.
fn lbl(text: &str, size: f32, color: Color) -> Widget {
  Widget::Label {
    text: text.to_string(),
    x: 0.0,
    y: 0.0,
    width: 0.0,
    height: 0.0,
    size,
    color,
    text_align: TextAlign::Left,
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: None,
  }
}

/// One labelled section row with a title on the left.
fn section(title: &str, kids: Vec<Widget>) -> Widget {
  ctn(
    Direction::Row,
    10.0,
    8.0,
    None,
    Some(SECTION_H),
    0.0,
    BoxStyle::default(),
    Align::Center,
    vec![
      // Title column uses Stretch so the label gets the
      // full column width, preventing scissor clipping.
      ctn(
        Direction::Column,
        0.0,
        0.0,
        Some(LABEL_W),
        None,
        0.0,
        BoxStyle::default(),
        Align::Stretch,
        vec![lbl(title, 13.0, GRAY)],
      ),
      ctn(
        Direction::Row,
        12.0,
        0.0,
        None,
        None,
        1.0,
        BoxStyle::default(),
        Align::Center,
        kids,
      ),
    ],
  )
}

// -- Palette ------------------------------------------------

const WHITE: Color = (1.0, 1.0, 1.0, 1.0);
const GRAY: Color = (0.55, 0.55, 0.6, 1.0);
const LIGHT: Color = (0.8, 0.8, 0.8, 1.0);
const DIM: Color = (0.6, 0.6, 0.6, 1.0);
// Linear-space color: sRGB ≈ pow(x, 1/2.2), so 0.01 → ~12%
// which gives a very dark charcoal background.
const BG: Color = (0.01, 0.01, 0.014, 1.0);

// -- Section builders ---------------------------------------

fn section_header() -> Widget {
  ctn(
    Direction::Row,
    0.0,
    10.0,
    None,
    Some(HEADER_H),
    0.0,
    BoxStyle::default(),
    Align::Stretch,
    vec![lbl("Gloomy Widget Showcase", 22.0, WHITE)],
  )
}

fn section_buttons() -> Widget {
  let default_btn = Widget::Button {
    text: "Default".to_string(),
    action: "btn_default".to_string(),
    bounds: WidgetBounds::default(),
    style: ButtonStyle::default(),
    width: Some(120.0),
    height: Some(36.0),
    disabled: false,
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: None,
  };
  let green = ButtonStyle {
    idle: BoxStyle::fill((0.1, 0.5, 0.2, 1.0))
      .with_radius(4.0),
    hover: BoxStyle::fill((0.15, 0.6, 0.25, 1.0))
      .with_radius(4.0),
    active: BoxStyle::fill((0.08, 0.4, 0.15, 1.0))
      .with_radius(4.0),
    disabled: BoxStyle::fill((0.1, 0.1, 0.1, 0.5))
      .with_radius(4.0),
    text_color: WHITE,
  };
  let green_btn = Widget::Button {
    text: "Success".to_string(),
    action: "btn_green".to_string(),
    bounds: WidgetBounds::default(),
    style: green,
    width: Some(120.0),
    height: Some(36.0),
    disabled: false,
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: None,
  };
  let disabled_btn = Widget::Button {
    text: "Disabled".to_string(),
    action: "btn_disabled".to_string(),
    bounds: WidgetBounds::default(),
    style: ButtonStyle::default(),
    width: Some(120.0),
    height: Some(36.0),
    disabled: true,
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: None,
  };
  section("Buttons", vec![default_btn, green_btn, disabled_btn])
}

fn section_inputs() -> Widget {
  let empty = Widget::TextInput {
    id: "input_empty".to_string(),
    value: String::new(),
    placeholder: String::new(),
    font_size: 0.0,
    text_align: TextAlign::Left,
    bounds: WidgetBounds::default(),
    validation: None,
    style: TextInputStyle::default(),
    width: 180.0,
    height: 32.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let placeholder = Widget::TextInput {
    id: "input_placeholder".to_string(),
    value: String::new(),
    placeholder: "Enter name...".to_string(),
    font_size: 0.0,
    text_align: TextAlign::Left,
    bounds: WidgetBounds::default(),
    validation: None,
    style: TextInputStyle::default(),
    width: 180.0,
    height: 32.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let with_val = Widget::TextInput {
    id: "input_value".to_string(),
    value: "Hello World".to_string(),
    placeholder: String::new(),
    font_size: 0.0,
    text_align: TextAlign::Left,
    bounds: WidgetBounds::default(),
    validation: None,
    style: TextInputStyle::default(),
    width: 180.0,
    height: 32.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  section("Text Inputs", vec![empty, placeholder, with_val])
}

fn checkbox(id: &str, checked: bool) -> Widget {
  let sty = CheckboxStyle {
    background: (0.15, 0.15, 0.2, 1.0),
    background_checked: (0.2, 0.5, 0.9, 1.0),
    checkmark_color: (1.0, 1.0, 1.0, 1.0),
    border: Some(gloomy_core::style::Border {
      color: (0.4, 0.4, 0.45, 1.0),
      width: 1.0,
      radius: [0.0; 4],
    }),
    corner_radius: 3.0,
  };
  Widget::Checkbox {
    id: id.to_string(),
    checked,
    size: 20.0,
    style: sty,
    bounds: WidgetBounds::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  }
}

fn check_pair(
  label: &str, widget: Widget,
) -> Widget {
  ctn(
    Direction::Row,
    8.0,
    0.0,
    None,
    None,
    1.0,
    BoxStyle::default(),
    Align::Center,
    vec![widget, lbl(label, 12.0, DIM)],
  )
}

fn section_checks() -> Widget {
  let cb_off = checkbox("cb_off", false);
  let cb_on = checkbox("cb_on", true);
  let tog_off = Widget::ToggleSwitch {
    id: "tog_off".to_string(),
    checked: false,
    style: ToggleSwitchStyle::default(),
    bounds: WidgetBounds::default(),
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let tog_on = Widget::ToggleSwitch {
    id: "tog_on".to_string(),
    checked: true,
    style: ToggleSwitchStyle::default(),
    bounds: WidgetBounds::default(),
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  section(
    "Checks & Toggles",
    vec![
      check_pair("Unchecked", cb_off),
      check_pair("Checked", cb_on),
      check_pair("Toggle Off", tog_off),
      check_pair("Toggle On", tog_on),
    ],
  )
}

fn section_sliders() -> Widget {
  let base = SliderStyle {
    track_color: (0.15, 0.15, 0.2, 1.0),
    active_track_color: (0.3, 0.5, 0.9, 1.0),
    thumb_color: (0.9, 0.9, 0.95, 1.0),
    thumb_border: None,
    track_height: 6.0,
    thumb_radius: 8.0,
  };
  let s40 = Widget::Slider {
    id: "slider_40".to_string(),
    value: 40.0,
    min: 0.0,
    max: 100.0,
    style: base.clone(),
    bounds: WidgetBounds::default(),
    width: 200.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let s90 = Widget::Slider {
    id: "slider_90".to_string(),
    value: 90.0,
    min: 0.0,
    max: 100.0,
    style: base.clone(),
    bounds: WidgetBounds::default(),
    width: 200.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let blue = SliderStyle {
    active_track_color: (0.2, 0.7, 0.4, 1.0),
    thumb_color: (0.2, 0.7, 0.4, 1.0),
    ..base
  };
  let s_blue = Widget::Slider {
    id: "slider_green".to_string(),
    value: 65.0,
    min: 0.0,
    max: 100.0,
    style: blue,
    bounds: WidgetBounds::default(),
    width: 200.0,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  section("Sliders", vec![s40, s90, s_blue])
}

fn section_labels() -> Widget {
  section(
    "Labels",
    vec![
      lbl("Large 22px", 22.0, WHITE),
      lbl("Medium 16px", 16.0, LIGHT),
      lbl("Small 11px", 11.0, DIM),
      lbl("Red", 14.0, (0.9, 0.2, 0.2, 1.0)),
      lbl("Green", 14.0, (0.2, 0.8, 0.3, 1.0)),
      lbl("Blue", 14.0, (0.3, 0.5, 0.9, 1.0)),
    ],
  )
}

fn section_progress() -> Widget {
  let p25 = Widget::ProgressBar {
    value: 25.0,
    min: 0.0,
    max: 100.0,
    style: ProgressBarStyle::default(),
    width: Some(200.0),
    height: Some(16.0),
    bounds: WidgetBounds::default(),
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let p50 = Widget::ProgressBar {
    value: 50.0,
    min: 0.0,
    max: 100.0,
    style: ProgressBarStyle {
      fill_color: Some((0.2, 0.7, 0.3, 1.0)),
      corner_radius: 4.0,
      ..Default::default()
    },
    width: Some(200.0),
    height: Some(16.0),
    bounds: WidgetBounds::default(),
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let p75 = Widget::ProgressBar {
    value: 75.0,
    min: 0.0,
    max: 100.0,
    style: ProgressBarStyle {
      fill_color: Some((0.9, 0.6, 0.1, 1.0)),
      corner_radius: 8.0,
      ..Default::default()
    },
    width: Some(200.0),
    height: Some(16.0),
    bounds: WidgetBounds::default(),
    layout: Layout::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  section(
    "Progress Bars",
    vec![
      vbox(2.0, vec![lbl("25%", 11.0, DIM), p25]),
      vbox(2.0, vec![lbl("50%", 11.0, DIM), p50]),
      vbox(2.0, vec![lbl("75%", 11.0, DIM), p75]),
    ],
  )
}

fn kpi_bounds() -> WidgetBounds {
  WidgetBounds {
    x: 0.0,
    y: 0.0,
    width: 200.0,
    height: 70.0,
  }
}

fn section_kpi() -> Widget {
  let sty = KpiCardStyle::default();
  let revenue = Widget::KpiCard {
    id: Some("kpi_revenue".to_string()),
    title: "Revenue".to_string(),
    value: "$12,450".to_string(),
    trend: Some(KpiTrend {
      direction: TrendDirection::Up,
      value: "+12%".to_string(),
    }),
    style: sty.clone(),
    bounds: kpi_bounds(),
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let users = Widget::KpiCard {
    id: Some("kpi_users".to_string()),
    title: "Users".to_string(),
    value: "3,842".to_string(),
    trend: Some(KpiTrend {
      direction: TrendDirection::Up,
      value: "+5%".to_string(),
    }),
    style: sty.clone(),
    bounds: kpi_bounds(),
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  let bounce = Widget::KpiCard {
    id: Some("kpi_bounce".to_string()),
    title: "Bounce Rate".to_string(),
    value: "24%".to_string(),
    trend: Some(KpiTrend {
      direction: TrendDirection::Down,
      value: "-3%".to_string(),
    }),
    style: sty,
    bounds: kpi_bounds(),
    flex: 1.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  };
  section("KPI Cards", vec![revenue, users, bounce])
}

fn section_containers() -> Widget {
  let plain = ctn(
    Direction::Column,
    4.0,
    8.0,
    Some(120.0),
    Some(60.0),
    0.0,
    BoxStyle::fill((0.15, 0.15, 0.2, 1.0)),
    Align::Start,
    vec![lbl("Plain", 11.0, LIGHT)],
  );
  let bordered = ctn(
    Direction::Column,
    4.0,
    8.0,
    Some(120.0),
    Some(60.0),
    0.0,
    BoxStyle::fill((0.12, 0.12, 0.17, 1.0))
      .with_border((0.4, 0.4, 0.5, 1.0), 1.0),
    Align::Start,
    vec![lbl("Bordered", 11.0, LIGHT)],
  );
  let rounded = ctn(
    Direction::Column,
    4.0,
    8.0,
    Some(120.0),
    Some(60.0),
    0.0,
    BoxStyle::fill((0.12, 0.12, 0.17, 1.0))
      .with_radius(12.0)
      .with_border((0.3, 0.5, 0.7, 1.0), 1.0),
    Align::Start,
    vec![lbl("Rounded", 11.0, LIGHT)],
  );
  let inner = ctn(
    Direction::Column,
    2.0,
    6.0,
    Some(90.0),
    Some(30.0),
    0.0,
    BoxStyle::fill((0.2, 0.2, 0.28, 1.0))
      .with_radius(4.0),
    Align::Start,
    vec![lbl("Inner", 10.0, LIGHT)],
  );
  let nested = ctn(
    Direction::Column,
    4.0,
    8.0,
    Some(140.0),
    Some(60.0),
    0.0,
    BoxStyle::fill((0.12, 0.12, 0.17, 1.0))
      .with_radius(8.0),
    Align::Start,
    vec![lbl("Nested", 11.0, LIGHT), inner],
  );
  section(
    "Containers",
    vec![plain, bordered, rounded, nested],
  )
}

// -- Showcase assembly --------------------------------------

fn build_sections() -> Vec<(&'static str, Widget)> {
  vec![
    ("buttons", section_buttons()),
    ("inputs", section_inputs()),
    ("checks", section_checks()),
    ("sliders", section_sliders()),
    ("labels", section_labels()),
    ("progress", section_progress()),
    ("kpi", section_kpi()),
    ("containers", section_containers()),
  ]
}

fn section_names() -> Vec<&'static str> {
  vec![
    "buttons",
    "inputs",
    "checks",
    "sliders",
    "labels",
    "progress",
    "kpi",
    "containers",
  ]
}

/// Filters sections by name and assembles the root widget.
fn build_root(filter: Option<&str>) -> Widget {
  let all = build_sections();
  let sections: Vec<Widget> = match filter {
    Some(f) => {
      let matched: Vec<Widget> = all
        .into_iter()
        .filter(|(name, _)| *name == f)
        .map(|(_, w)| w)
        .collect();
      if matched.is_empty() {
        eprintln!("Unknown section: {f}");
        eprintln!(
          "Available: {}",
          section_names().join(", ")
        );
        std::process::exit(2);
      }
      matched
    }
    None => all.into_iter().map(|(_, w)| w).collect(),
  };

  let mut children = vec![section_header()];
  children.extend(sections);

  ctn(
    Direction::Column,
    2.0,
    10.0,
    None,
    None,
    0.0,
    BoxStyle::fill(BG),
    Align::Stretch,
    children,
  )
}

/// Renders widgets headlessly to a PNG snapshot.
fn run_headless(filter: Option<&str>) {
  let mut root = build_root(filter);
  let pixel_h =
    if filter.is_some() { 200 } else { HEIGHT };

  let mut renderer =
    HeadlessRenderer::new(WIDTH, pixel_h, SCALE)
      .expect("failed to create HeadlessRenderer");
  let image = renderer
    .render_to_image(&mut root, None, None)
    .expect("render failed");

  let dir = snapshot_dir();
  std::fs::create_dir_all(&dir)
    .expect("create snapshot dir");
  let out = dir.join("showcase.png");
  image.save(&out).expect("save PNG");

  let abs = out.canonicalize().unwrap_or(out);
  let widgets = collect_widgets(&root);
  let report = serde_json::json!({
    "image": abs.display().to_string(),
    "width": image.width(),
    "height": image.height(),
    "widgets": widgets,
  });
  println!(
    "{}",
    serde_json::to_string_pretty(&report).unwrap()
  );
}

/// Opens a live window displaying the widget showcase.
fn run_live(filter: Option<&str>) {
  let mut root = build_root(filter);
  let interaction =
    Rc::new(RefCell::new(InteractionState::default()));

  let ix_draw = interaction.clone();
  let ix_cursor = interaction.clone();
  let ix_mouse = interaction.clone();

  GloomyApp::new()
    .with_title("Gloomy Widget Showcase")
    .with_size(WIDTH, HEIGHT)
    .on_cursor_move(move |_win, x, y| {
      ix_cursor.borrow_mut().mouse_pos =
        Vec2::new(x, y);
    })
    .on_mouse_input(move |_win, state, _btn| {
      ix_mouse.borrow_mut().is_pressed = state
        == winit::event::ElementState::Pressed;
    })
    .on_draw(move |window, ctx| {
      let scale = window.renderer.scale_factor;
      let size = window.window.inner_size();
      let w = size.width as f32 / scale;
      let h = size.height as f32 / scale;

      if let Widget::Container { bounds, .. } =
        &mut root
      {
        bounds.width = w;
        bounds.height = h;
      }

      compute_layout(&mut root, 0.0, 0.0, w, h);

      let ix = ix_draw.borrow().clone();
      render_ui(
        &root,
        &mut window.renderer,
        &ctx.device,
        &ctx.queue,
        Some(&ix),
        None,
      );
    })
    .run()
    .expect("event loop failed");
}

fn main() {
  let args: Vec<String> = std::env::args().collect();

  let mut live = false;
  let mut filter: Option<String> = None;
  for arg in &args[1..] {
    if arg == "--live" {
      live = true;
    } else {
      filter = Some(arg.clone());
    }
  }

  if live {
    run_live(filter.as_deref());
  } else {
    run_headless(filter.as_deref());
  }
}
