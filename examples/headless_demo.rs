//! Headless rendering demo.
//!
//! Builds a UI, renders it offscreen without any window or
//! display server, and saves the result as a PNG.
//!
//! Usage: cargo run --example headless_demo

use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::widget::Widget;
use gloomy_core::style::{BoxStyle, ButtonStyle};
use gloomy_core::widget::WidgetBounds;
use gloomy_core::layout::{Direction, Layout};

fn build_demo_ui() -> Widget {
  Widget::Container {
    id: Some("root".to_string()),
    scrollable: false,
    bounds: WidgetBounds::default(),
    width: None,
    height: None,
    style: BoxStyle {
      background: Some((0.12, 0.12, 0.15, 1.0)),
      corner_radii: [0.0; 4],
      ..Default::default()
    },
    padding: 20.0,
    layout: Layout {
      direction: Direction::Column,
      spacing: 16.0,
      ..Default::default()
    },
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    children: vec![
      Widget::Label {
        text: "Headless Rendering Demo".to_string(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        size: 28.0,
        color: (1.0, 1.0, 1.0, 1.0),
        text_align: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        font: None,
      },
      Widget::Label {
        text: "Rendered without a window or display server."
          .to_string(),
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        size: 16.0,
        color: (0.7, 0.7, 0.7, 1.0),
        text_align: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        font: None,
      },
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle {
          background: Some((0.18, 0.18, 0.22, 1.0)),
          corner_radii: [8.0; 4],
          ..Default::default()
        },
        padding: 16.0,
        layout: Layout {
          direction: Direction::Row,
          spacing: 12.0,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::Button {
            text: "Save".to_string(),
            action: "save".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(120.0),
            height: Some(40.0),
            disabled: false,
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            font: None,
          },
          Widget::Button {
            text: "Cancel".to_string(),
            action: "cancel".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(120.0),
            height: Some(40.0),
            disabled: false,
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            font: None,
          },
        ],
        layout_cache: None,
        render_cache: Default::default(),
      },
      Widget::ProgressBar {
        value: 0.65,
        min: 0.0,
        max: 1.0,
        style: Default::default(),
        width: Some(400.0),
        height: Some(24.0),
        bounds: WidgetBounds::default(),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ],
    layout_cache: None,
    render_cache: Default::default(),
  }
}

fn main() -> anyhow::Result<()> {
  let mut ui = build_demo_ui();

  let mut renderer = HeadlessRenderer::new(HeadlessConfig {
    width: 800,
    height: 400,
    force_fallback_adapter: true,
    ..Default::default()
  })?;

  renderer.save_screenshot(
    &mut ui,
    "headless_demo.png",
    None,
    None,
  )?;

  println!("Saved headless_demo.png");
  Ok(())
}
