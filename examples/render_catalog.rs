//! Widget catalog generator.
//!
//! Renders every widget type in its idle state and saves
//! golden PNGs under `catalog/`.
//!
//! Usage: cargo run --example render_catalog

use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::layout::{Direction, Layout};
use gloomy_core::style::{
  BoxStyle, ButtonStyle, TextInputStyle,
};
use gloomy_core::widget::{
  Widget, WidgetBounds, CheckboxStyle, SliderStyle,
  ToggleSwitchStyle, ProgressBarStyle, RadioButtonStyle,
  DropdownStyle,
};

const WIDTH: u32 = 400;
const HEIGHT: u32 = 120;

fn wrap(child: Widget) -> Widget {
  Widget::Container {
    id: None,
    scrollable: false,
    bounds: WidgetBounds::default(),
    width: None,
    height: None,
    style: BoxStyle {
      background: Some((0.12, 0.12, 0.15, 1.0)),
      ..Default::default()
    },
    padding: 16.0,
    layout: Layout {
      direction: Direction::Column,
      spacing: 8.0,
      align_items: gloomy_core::layout::Align::Stretch,
      ..Default::default()
    },
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    children: vec![child],
    layout_cache: None,
    render_cache: Default::default(),
  }
}

fn catalog_entries() -> Vec<(&'static str, Widget)> {
  vec![
    (
      "label",
      Widget::label("Hello, Gloomy!"),
    ),
    (
      "button",
      Widget::Button {
        text: "Click Me".to_string(),
        action: "click".to_string(),
        bounds: WidgetBounds::default(),
        style: ButtonStyle::default(),
        width: Some(160.0),
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
    ),
    (
      "text_input",
      Widget::TextInput {
        value: "Sample text".to_string(),
        placeholder: "Type here...".to_string(),
        id: "input1".to_string(),
        font_size: 14.0,
        text_align: Default::default(),
        bounds: WidgetBounds::default(),
        validation: None,
        style: TextInputStyle::default(),
        width: 200.0,
        height: 36.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "checkbox",
      Widget::Checkbox {
        id: "cb1".to_string(),
        checked: true,
        size: 20.0,
        style: CheckboxStyle::default(),
        bounds: WidgetBounds::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "toggle_switch",
      Widget::ToggleSwitch {
        id: "toggle1".to_string(),
        checked: true,
        style: ToggleSwitchStyle::default(),
        bounds: WidgetBounds::default(),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "slider",
      Widget::Slider {
        id: "slider1".to_string(),
        value: 0.6,
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
      },
    ),
    (
      "progress_bar",
      Widget::ProgressBar {
        value: 0.7,
        min: 0.0,
        max: 1.0,
        style: ProgressBarStyle::default(),
        width: Some(300.0),
        height: Some(24.0),
        bounds: WidgetBounds::default(),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "radio_button",
      Widget::RadioButton {
        group_id: "rg1".to_string(),
        value: "opt1".to_string(),
        selected: true,
        label: "Option A".to_string(),
        style: RadioButtonStyle::default(),
        bounds: WidgetBounds::default(),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "dropdown",
      Widget::Dropdown {
        id: "dd1".to_string(),
        options: vec![
          "Alpha".to_string(),
          "Beta".to_string(),
          "Gamma".to_string(),
        ],
        selected_index: Some(0),
        expanded: false,
        style: DropdownStyle::default(),
        bounds: WidgetBounds::default(),
        width: Some(160.0),
        height: Some(36.0),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "divider",
      Widget::Divider {
        bounds: WidgetBounds::default(),
        orientation: Default::default(),
        thickness: 1.0,
        color: (0.3, 0.3, 0.3, 1.0),
        margin: 8.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    (
      "spacer",
      Widget::Spacer {
        size: 20.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
  ]
}

fn main() -> anyhow::Result<()> {
  std::fs::create_dir_all("catalog")?;

  let mut renderer =
    HeadlessRenderer::new(HeadlessConfig {
      width: WIDTH,
      height: HEIGHT,
      force_fallback_adapter: true,
      ..Default::default()
    })?;

  for (name, widget) in catalog_entries() {
    let mut wrapped = wrap(widget);
    let path = format!("catalog/{}.png", name);
    renderer.save_screenshot(
      &mut wrapped,
      &path,
      None,
      None,
    )?;
    println!("  {}", path);
  }

  println!("Catalog complete: {} widgets", catalog_entries().len());
  Ok(())
}
