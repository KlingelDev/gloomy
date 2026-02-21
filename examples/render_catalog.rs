//! Widget catalog generator.
//!
//! Renders every widget type in multiple states and saves
//! golden PNGs under `catalog/`.
//!
//! Usage: cargo run --example render_catalog

use gloomy_core::data_source::{
  CellValue, MapDataProvider, VecDataSource,
};
use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::layout::{Direction, Layout};
use gloomy_core::style::{
  BoxStyle, ButtonStyle, ListViewStyle, TextInputStyle,
};
use gloomy_core::widget::{
  AutocompleteStyle, CheckboxStyle, DatePickerStyle,
  DropdownStyle, NumberInputStyle, Orientation,
  ProgressBarStyle, RadioButtonStyle, ScrollbarStyle,
  SliderStyle, TabItem, TabStyle, ToggleSwitchStyle, Widget,
  WidgetBounds,
};
use gloomy_core::datagrid::{ColumnDef, DataGridStyle};
use gloomy_core::kpi::{KpiCardStyle, KpiTrend, TrendDirection};
use gloomy_core::tree::{TreeNode, TreeStyle};
use std::cell::RefCell;
use std::collections::HashSet;

const WIDTH: u32 = 400;
const HEIGHT: u32 = 200;

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
      justify_content: gloomy_core::layout::Justify::Center,
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

fn build_data_provider() -> MapDataProvider {
  let mut src = VecDataSource::new(
    vec![
      "Name".to_string(),
      "Role".to_string(),
      "Score".to_string(),
    ],
    vec![
      vec![
        CellValue::Text("Alice".into()),
        CellValue::Text("Engineer".into()),
        CellValue::Number(95.0),
      ],
      vec![
        CellValue::Text("Bob".into()),
        CellValue::Text("Designer".into()),
        CellValue::Number(88.0),
      ],
      vec![
        CellValue::Text("Carol".into()),
        CellValue::Text("Manager".into()),
        CellValue::Number(72.0),
      ],
    ],
  );
  let _ = &mut src; // suppress unused_mut if needed
  let mut provider = MapDataProvider::new();
  provider.register("catalog_grid", src);
  provider
}

fn col_def(name: &str) -> ColumnDef {
  ColumnDef {
    header: name.to_string(),
    field: name.to_string(),
    width: Default::default(),
    align: Default::default(),
    sortable: true,
    resizable: true,
    min_width: 50.0,
    max_width: None,
  }
}

fn make_button(
  text: &str,
  disabled: bool,
) -> Widget {
  Widget::Button {
    text: text.to_string(),
    action: "click".to_string(),
    bounds: WidgetBounds::default(),
    style: ButtonStyle::default(),
    width: Some(160.0),
    height: Some(40.0),
    disabled,
    layout: Default::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: None,
  }
}

fn make_checkbox(checked: bool) -> Widget {
  Widget::Checkbox {
    id: "cb1".to_string(),
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

fn make_toggle(checked: bool) -> Widget {
  Widget::ToggleSwitch {
    id: "toggle1".to_string(),
    checked,
    style: ToggleSwitchStyle::default(),
    bounds: WidgetBounds::default(),
    layout: Default::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  }
}

fn make_radio(selected: bool) -> Widget {
  Widget::RadioButton {
    group_id: "rg1".to_string(),
    value: "opt1".to_string(),
    selected,
    label: "Option A".to_string(),
    style: RadioButtonStyle::default(),
    bounds: WidgetBounds::default(),
    layout: Default::default(),
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  }
}

fn make_dropdown(expanded: bool) -> Widget {
  Widget::Dropdown {
    id: "dd1".to_string(),
    options: vec![
      "Alpha".to_string(),
      "Beta".to_string(),
      "Gamma".to_string(),
    ],
    selected_index: Some(0),
    expanded,
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
  }
}

fn make_slider(value: f32) -> Widget {
  Widget::Slider {
    id: "slider1".to_string(),
    value,
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
  }
}

fn make_progress(value: f32) -> Widget {
  Widget::ProgressBar {
    value,
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
  }
}

fn make_divider(orientation: Orientation) -> Widget {
  let flex = match orientation {
    Orientation::Vertical => 1.0,
    _ => 0.0,
  };
  Widget::Divider {
    bounds: WidgetBounds::default(),
    orientation,
    thickness: 1.0,
    color: (0.3, 0.3, 0.3, 1.0),
    margin: 8.0,
    flex,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
  }
}

fn catalog_entries() -> Vec<(&'static str, Widget)> {
  vec![
    // ── Label (1 state) ──────────────────────────────
    ("label", Widget::label("Hello, Gloomy!")),
    // ── TextInput (1 state) ──────────────────────────
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
    // ── Button (2 states) ────────────────────────────
    ("button_idle", make_button("Click Me", false)),
    ("button_disabled", make_button("Disabled", true)),
    // ── Checkbox (2 states) ──────────────────────────
    ("checkbox_checked", make_checkbox(true)),
    ("checkbox_unchecked", make_checkbox(false)),
    // ── ToggleSwitch (2 states) ──────────────────────
    ("toggle_switch_on", make_toggle(true)),
    ("toggle_switch_off", make_toggle(false)),
    // ── RadioButton (2 states) ───────────────────────
    ("radio_selected", make_radio(true)),
    ("radio_unselected", make_radio(false)),
    // ── Dropdown (2 states) ──────────────────────────
    ("dropdown_collapsed", make_dropdown(false)),
    ("dropdown_expanded", make_dropdown(true)),
    // ── Slider (3 states) ────────────────────────────
    ("slider_0", make_slider(0.0)),
    ("slider_50", make_slider(0.5)),
    ("slider_100", make_slider(1.0)),
    // ── ProgressBar (3 states) ───────────────────────
    ("progress_0", make_progress(0.0)),
    ("progress_50", make_progress(0.5)),
    ("progress_100", make_progress(1.0)),
    // ── Divider (2 states) ──────────────────────────
    ("divider_h", make_divider(Orientation::Horizontal)),
    ("divider_v", make_divider(Orientation::Vertical)),
    // ── Spacer (1 state) ─────────────────────────────
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
    // ── KpiCard ──────────────────────────────────────
    (
      "kpi_card",
      Widget::KpiCard {
        id: Some("kpi1".to_string()),
        title: "Revenue".to_string(),
        value: "$1.2M".to_string(),
        trend: Some(KpiTrend {
          direction: TrendDirection::Up,
          value: "+12%".to_string(),
        }),
        style: KpiCardStyle::default(),
        bounds: WidgetBounds {
          height: 100.0,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── Tab ──────────────────────────────────────────
    (
      "tab",
      Widget::Tab {
        id: Some("tab1".to_string()),
        tabs: vec![
          TabItem {
            title: "Overview".to_string(),
            content: Box::new(Widget::label("Tab 1")),
          },
          TabItem {
            title: "Details".to_string(),
            content: Box::new(Widget::label("Tab 2")),
          },
        ],
        selected: 0,
        orientation: Orientation::Horizontal,
        style: TabStyle::default(),
        bounds: WidgetBounds {
          height: 120.0,
          ..Default::default()
        },
        width: Some(300.0),
        height: Some(120.0),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        layout_cache: None,
        render_cache: RefCell::new(None),
      },
    ),
    // ── Tree ─────────────────────────────────────────
    (
      "tree",
      Widget::Tree {
        id: Some("tree1".to_string()),
        bounds: WidgetBounds::default(),
        root_nodes: vec![TreeNode {
          id: "root".to_string(),
          label: "Documents".to_string(),
          icon: None,
          children: vec![
            TreeNode {
              id: "child1".to_string(),
              label: "report.pdf".to_string(),
              icon: None,
              children: vec![],
              leaf: true,
            },
            TreeNode {
              id: "child2".to_string(),
              label: "notes.txt".to_string(),
              icon: None,
              children: vec![],
              leaf: true,
            },
          ],
          leaf: false,
        }],
        selected_id: None,
        expanded_ids: {
          let mut s = HashSet::new();
          s.insert("root".to_string());
          s
        },
        style: TreeStyle::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── ListView ─────────────────────────────────────
    (
      "list_view",
      Widget::ListView {
        id: "lv1".to_string(),
        items: vec![
          "Item One".to_string(),
          "Item Two".to_string(),
          "Item Three".to_string(),
        ],
        selected_index: Some(1),
        style: ListViewStyle::default(),
        bounds: WidgetBounds::default(),
        width: Some(200.0),
        height: Some(160.0),
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        scroll_offset: 0.0,
      },
    ),
    // ── NumberInput ───────────────────────────────────
    (
      "number_input",
      Widget::NumberInput {
        id: "ni1".to_string(),
        value: 42.0,
        min: Some(0.0),
        max: Some(100.0),
        step: 1.0,
        precision: 0,
        show_spinner: true,
        bounds: WidgetBounds::default(),
        validation: None,
        style: NumberInputStyle::default(),
        width: 160.0,
        height: 36.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── Autocomplete ─────────────────────────────────
    (
      "autocomplete",
      Widget::Autocomplete {
        id: "ac1".to_string(),
        value: String::new(),
        placeholder: "Search...".to_string(),
        suggestions: vec![
          "Apple".to_string(),
          "Banana".to_string(),
          "Cherry".to_string(),
        ],
        max_visible: 5,
        bounds: WidgetBounds::default(),
        style: AutocompleteStyle::default(),
        validation: None,
        width: 200.0,
        height: 36.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── DatePicker ───────────────────────────────────
    (
      "date_picker",
      Widget::DatePicker {
        id: "dp1".to_string(),
        value: None,
        placeholder: "Select date...".to_string(),
        min_date: None,
        max_date: None,
        format: "%Y-%m-%d".to_string(),
        bounds: WidgetBounds::default(),
        style: DatePickerStyle::default(),
        validation: None,
        width: 200.0,
        height: 36.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── DataGrid ─────────────────────────────────────
    (
      "data_grid",
      Widget::DataGrid {
        id: Some("dg1".to_string()),
        bounds: WidgetBounds {
          height: 150.0,
          ..Default::default()
        },
        columns: vec![
          col_def("Name"),
          col_def("Role"),
          col_def("Score"),
        ],
        data_source_id: Some("catalog_grid".to_string()),
        header_height: 30.0,
        row_height: 28.0,
        striped: true,
        selection_mode: Default::default(),
        show_vertical_lines: true,
        show_horizontal_lines: true,
        selected_rows: vec![],
        sort_column: None,
        sort_direction: None,
        style: DataGridStyle::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── Scrollbar ────────────────────────────────────
    (
      "scrollbar",
      Widget::Scrollbar {
        bounds: WidgetBounds::default(),
        content_size: 500.0,
        viewport_size: 200.0,
        scroll_offset: 100.0,
        orientation: Orientation::Vertical,
        style: ScrollbarStyle::default(),
        flex: 1.0,
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

  let provider = build_data_provider();
  let entries = catalog_entries();
  let count = entries.len();

  for (name, widget) in entries {
    let mut wrapped = wrap(widget);
    let path = format!("catalog/{}.png", name);
    renderer.save_screenshot(
      &mut wrapped,
      &path,
      None,
      Some(&provider),
    )?;
    println!("  {}", path);
  }

  println!("Catalog complete: {} widgets", count);
  Ok(())
}
