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
use gloomy_core::layout::{
  Align, Direction, Justify, Layout,
};
use gloomy_core::style::{
  Border, BoxStyle, ButtonStyle, Gradient, ListViewStyle,
  Shadow, TextInputStyle,
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

fn make_setting_row(
  label: &str,
  control: Widget,
) -> Widget {
  Widget::Container {
    id: None,
    scrollable: false,
    bounds: WidgetBounds::default(),
    width: None,
    height: None,
    style: BoxStyle::default(),
    padding: 0.0,
    layout: Layout {
      direction: Direction::Row,
      spacing: 8.0,
      align_items: Align::Center,
      justify_content: Justify::SpaceBetween,
      ..Default::default()
    },
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    children: vec![Widget::label(label), control],
    layout_cache: None,
    render_cache: Default::default(),
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
    // ── Scrollbar horizontal ──────────────────────────
    (
      "scrollbar_h",
      Widget::Scrollbar {
        bounds: WidgetBounds::default(),
        content_size: 800.0,
        viewport_size: 300.0,
        scroll_offset: 200.0,
        orientation: Orientation::Horizontal,
        style: ScrollbarStyle::default(),
        flex: 1.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
      },
    ),
    // ── Tab vertical ─────────────────────────────────
    (
      "tab_vertical",
      Widget::Tab {
        id: Some("tab_v".to_string()),
        tabs: vec![
          TabItem {
            title: "General".to_string(),
            content: Box::new(Widget::label("Settings")),
          },
          TabItem {
            title: "Advanced".to_string(),
            content: Box::new(Widget::label("Advanced")),
          },
          TabItem {
            title: "About".to_string(),
            content: Box::new(Widget::label("Info")),
          },
        ],
        selected: 0,
        orientation: Orientation::Vertical,
        style: TabStyle::default(),
        bounds: WidgetBounds {
          height: 140.0,
          ..Default::default()
        },
        width: Some(350.0),
        height: Some(140.0),
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
    // ── Tab second selected ──────────────────────────
    (
      "tab_selected_2",
      Widget::Tab {
        id: Some("tab_s2".to_string()),
        tabs: vec![
          TabItem {
            title: "Overview".to_string(),
            content: Box::new(Widget::label("Tab 1")),
          },
          TabItem {
            title: "Details".to_string(),
            content: Box::new(Widget::label(
              "Detail content",
            )),
          },
        ],
        selected: 1,
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
    // ── KPI down trend ───────────────────────────────
    (
      "kpi_card_down",
      Widget::KpiCard {
        id: Some("kpi_down".to_string()),
        title: "Churn Rate".to_string(),
        value: "4.2%".to_string(),
        trend: Some(KpiTrend {
          direction: TrendDirection::Down,
          value: "-0.8%".to_string(),
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
    // ── KPI neutral ──────────────────────────────────
    (
      "kpi_card_neutral",
      Widget::KpiCard {
        id: Some("kpi_neut".to_string()),
        title: "Active Users".to_string(),
        value: "12,340".to_string(),
        trend: Some(KpiTrend {
          direction: TrendDirection::Neutral,
          value: "0%".to_string(),
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
    // ── Tree with selection ──────────────────────────
    (
      "tree_selected",
      Widget::Tree {
        id: Some("tree_sel".to_string()),
        bounds: WidgetBounds::default(),
        root_nodes: vec![TreeNode {
          id: "src".to_string(),
          label: "src".to_string(),
          icon: None,
          children: vec![
            TreeNode {
              id: "main_rs".to_string(),
              label: "main.rs".to_string(),
              icon: None,
              children: vec![],
              leaf: true,
            },
            TreeNode {
              id: "lib_rs".to_string(),
              label: "lib.rs".to_string(),
              icon: None,
              children: vec![],
              leaf: true,
            },
            TreeNode {
              id: "ui_rs".to_string(),
              label: "ui.rs".to_string(),
              icon: None,
              children: vec![],
              leaf: true,
            },
          ],
          leaf: false,
        }],
        selected_id: Some("lib_rs".to_string()),
        expanded_ids: {
          let mut s = HashSet::new();
          s.insert("src".to_string());
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
    // ── DatePicker with value ────────────────────────
    (
      "date_picker_value",
      Widget::DatePicker {
        id: "dp2".to_string(),
        value: chrono::NaiveDate::from_ymd_opt(
          2026, 2, 22,
        ),
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
    // ── NumberInput decimal ──────────────────────────
    (
      "number_input_decimal",
      Widget::NumberInput {
        id: "nid".to_string(),
        value: 3.14159,
        min: Some(0.0),
        max: Some(100.0),
        step: 0.01,
        precision: 2,
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
    // ── Container: styled card ──────────────────────
    (
      "container_card",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(280.0),
        height: Some(100.0),
        style: BoxStyle {
          background: Some((0.18, 0.18, 0.22, 1.0)),
          border: Some(Border {
            width: 1.0,
            color: (0.35, 0.35, 0.4, 1.0),
            radius: [0.0; 4],
          }),
          corner_radii: [8.0; 4],
          shadow: Some(Shadow {
            offset: (2.0, 3.0),
            blur: 6.0,
            color: (0.0, 0.0, 0.0, 0.4),
          }),
          ..Default::default()
        },
        padding: 16.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 4.0,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::label("Card Title"),
          Widget::label("Body text inside a card."),
        ],
        layout_cache: None,
        render_cache: Default::default(),
      },
    ),
    // ── Container: gradient ─────────────────────────
    (
      "container_gradient",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(280.0),
        height: Some(80.0),
        style: BoxStyle {
          gradient: Some(Gradient {
            start: (0.15, 0.25, 0.55, 1.0),
            end: (0.35, 0.15, 0.45, 1.0),
          }),
          corner_radii: [12.0; 4],
          ..Default::default()
        },
        padding: 16.0,
        layout: Layout {
          direction: Direction::Column,
          justify_content: Justify::Center,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![Widget::label("Gradient")],
        layout_cache: None,
        render_cache: Default::default(),
      },
    ),
    // ── Composite: radio group ──────────────────────
    (
      "radio_group",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle::default(),
        padding: 0.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 8.0,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::RadioButton {
            group_id: "size".to_string(),
            value: "sm".to_string(),
            selected: false,
            label: "Small".to_string(),
            style: RadioButtonStyle::default(),
            bounds: WidgetBounds::default(),
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::RadioButton {
            group_id: "size".to_string(),
            value: "md".to_string(),
            selected: true,
            label: "Medium".to_string(),
            style: RadioButtonStyle::default(),
            bounds: WidgetBounds::default(),
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::RadioButton {
            group_id: "size".to_string(),
            value: "lg".to_string(),
            selected: false,
            label: "Large".to_string(),
            style: RadioButtonStyle::default(),
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
      },
    ),
    // ── Composite: toolbar ──────────────────────────
    (
      "toolbar",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle {
          background: Some((0.16, 0.16, 0.2, 1.0)),
          border: Some(Border {
            width: 1.0,
            color: (0.25, 0.25, 0.3, 1.0),
            radius: [0.0; 4],
          }),
          corner_radii: [4.0; 4],
          ..Default::default()
        },
        padding: 8.0,
        layout: Layout {
          direction: Direction::Row,
          spacing: 6.0,
          align_items: Align::Center,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::Button {
            text: "New".to_string(),
            action: "new".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(60.0),
            height: Some(28.0),
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
            text: "Open".to_string(),
            action: "open".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(60.0),
            height: Some(28.0),
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
            text: "Save".to_string(),
            action: "save".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(60.0),
            height: Some(28.0),
            disabled: false,
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            font: None,
          },
          Widget::Divider {
            bounds: WidgetBounds::default(),
            orientation: Orientation::Vertical,
            thickness: 1.0,
            color: (0.3, 0.3, 0.35, 1.0),
            margin: 4.0,
            flex: 1.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::Button {
            text: "Delete".to_string(),
            action: "delete".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle::default(),
            width: Some(60.0),
            height: Some(28.0),
            disabled: true,
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
    ),
    // ── Composite: login form ───────────────────────
    (
      "form_login",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(300.0),
        height: Some(160.0),
        style: BoxStyle {
          background: Some((0.15, 0.15, 0.19, 1.0)),
          border: Some(Border {
            width: 1.0,
            color: (0.3, 0.3, 0.35, 1.0),
            radius: [0.0; 4],
          }),
          corner_radii: [6.0; 4],
          ..Default::default()
        },
        padding: 14.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 8.0,
          align_items: Align::Stretch,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::label("Sign In"),
          Widget::TextInput {
            value: String::new(),
            placeholder: "Username".to_string(),
            id: "user".to_string(),
            font_size: 14.0,
            text_align: Default::default(),
            bounds: WidgetBounds::default(),
            validation: None,
            style: TextInputStyle::default(),
            width: 200.0,
            height: 32.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::TextInput {
            value: String::new(),
            placeholder: "Password".to_string(),
            id: "pass".to_string(),
            font_size: 14.0,
            text_align: Default::default(),
            bounds: WidgetBounds::default(),
            validation: None,
            style: TextInputStyle::default(),
            width: 200.0,
            height: 32.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::Button {
            text: "Log In".to_string(),
            action: "login".to_string(),
            bounds: WidgetBounds::default(),
            style: ButtonStyle {
              idle: BoxStyle::fill(
                (0.25, 0.47, 0.85, 1.0),
              )
              .with_radius(4.0),
              hover: BoxStyle::fill(
                (0.3, 0.52, 0.9, 1.0),
              )
              .with_radius(4.0),
              active: BoxStyle::fill(
                (0.2, 0.4, 0.75, 1.0),
              )
              .with_radius(4.0),
              disabled: ButtonStyle::default().disabled,
              text_color: (1.0, 1.0, 1.0, 1.0),
            },
            width: None,
            height: Some(34.0),
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
    ),
    // ── Composite: settings row ─────────────────────
    (
      "settings_row",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle::default(),
        padding: 0.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 12.0,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          make_setting_row(
            "Dark Mode",
            Widget::ToggleSwitch {
              id: "dark".to_string(),
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
          make_setting_row(
            "Notifications",
            Widget::ToggleSwitch {
              id: "notif".to_string(),
              checked: false,
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
          make_setting_row(
            "Auto-save",
            Widget::Checkbox {
              id: "autosave".to_string(),
              checked: true,
              size: 18.0,
              style: CheckboxStyle::default(),
              bounds: WidgetBounds::default(),
              flex: 0.0,
              grid_col: None,
              grid_row: None,
              col_span: 1,
              row_span: 1,
            },
          ),
        ],
        layout_cache: None,
        render_cache: Default::default(),
      },
    ),
    // ── Composite: KPI dashboard row ────────────────
    (
      "kpi_row",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle::default(),
        padding: 0.0,
        layout: Layout {
          direction: Direction::Row,
          spacing: 10.0,
          align_items: Align::Stretch,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::KpiCard {
            id: Some("k1".to_string()),
            title: "Revenue".to_string(),
            value: "$1.2M".to_string(),
            trend: Some(KpiTrend {
              direction: TrendDirection::Up,
              value: "+12%".to_string(),
            }),
            style: KpiCardStyle::default(),
            bounds: WidgetBounds {
              width: 110.0,
              height: 80.0,
              ..Default::default()
            },
            flex: 1.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::KpiCard {
            id: Some("k2".to_string()),
            title: "Users".to_string(),
            value: "8,421".to_string(),
            trend: Some(KpiTrend {
              direction: TrendDirection::Up,
              value: "+5%".to_string(),
            }),
            style: KpiCardStyle::default(),
            bounds: WidgetBounds {
              width: 110.0,
              height: 80.0,
              ..Default::default()
            },
            flex: 1.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::KpiCard {
            id: Some("k3".to_string()),
            title: "Errors".to_string(),
            value: "23".to_string(),
            trend: Some(KpiTrend {
              direction: TrendDirection::Down,
              value: "-8%".to_string(),
            }),
            style: KpiCardStyle::default(),
            bounds: WidgetBounds {
              width: 110.0,
              height: 80.0,
              ..Default::default()
            },
            flex: 1.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
        ],
        layout_cache: None,
        render_cache: Default::default(),
      },
    ),
    // ── Composite: slider with labels ───────────────
    (
      "slider_labeled",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle::default(),
        padding: 0.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 4.0,
          align_items: Align::Stretch,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::label("Volume"),
          Widget::Slider {
            id: "vol".to_string(),
            value: 0.7,
            min: 0.0,
            max: 1.0,
            style: SliderStyle::default(),
            bounds: WidgetBounds::default(),
            width: 300.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::label("Brightness"),
          Widget::Slider {
            id: "bri".to_string(),
            value: 0.4,
            min: 0.0,
            max: 1.0,
            style: SliderStyle::default(),
            bounds: WidgetBounds::default(),
            width: 300.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
        ],
        layout_cache: None,
        render_cache: Default::default(),
      },
    ),
    // ── Composite: progress steps ───────────────────
    (
      "progress_steps",
      Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        style: BoxStyle::default(),
        padding: 0.0,
        layout: Layout {
          direction: Direction::Column,
          spacing: 6.0,
          align_items: Align::Stretch,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![
          Widget::label("Upload"),
          Widget::ProgressBar {
            value: 1.0,
            min: 0.0,
            max: 1.0,
            style: ProgressBarStyle::default(),
            width: Some(300.0),
            height: Some(16.0),
            bounds: WidgetBounds::default(),
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::label("Processing"),
          Widget::ProgressBar {
            value: 0.6,
            min: 0.0,
            max: 1.0,
            style: ProgressBarStyle::default(),
            width: Some(300.0),
            height: Some(16.0),
            bounds: WidgetBounds::default(),
            layout: Default::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
          },
          Widget::label("Deploy"),
          Widget::ProgressBar {
            value: 0.0,
            min: 0.0,
            max: 1.0,
            style: ProgressBarStyle::default(),
            width: Some(300.0),
            height: Some(16.0),
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
