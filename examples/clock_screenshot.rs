//! Headless screenshot of the clock+calendar layout.

use chrono::{Datelike, Local, NaiveDate};
use gloomy_core::interaction::InteractionState;
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::ui::{find_widget_mut, load_ui};
use gloomy_core::widget::Widget;
use gloomy_driver::screenshot::HeadlessRenderer;

const WIDTH: u32 = 900;
const HEIGHT: u32 = 520;

fn days_in_month(y: i32, m: u32) -> u32 {
  let (ny, nm) =
    if m == 12 { (y + 1, 1) } else { (y, m + 1) };
  NaiveDate::from_ymd_opt(ny, nm, 1)
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

fn month_name(m: u32) -> &'static str {
  match m {
    1 => "January",
    2 => "February",
    3 => "March",
    4 => "April",
    5 => "May",
    6 => "June",
    7 => "July",
    8 => "August",
    9 => "September",
    10 => "October",
    11 => "November",
    12 => "December",
    _ => "???",
  }
}

fn cal_label(
  text: &str,
  size: f32,
  height: f32,
  color: (f32, f32, f32, f32),
) -> Widget {
  Widget::Label {
    text: text.to_string(),
    x: 0.0,
    y: 0.0,
    width: 36.0,
    height,
    size,
    color,
    text_align: gloomy_core::widget::TextAlign::Center,
    flex: 0.0,
    grid_col: None,
    grid_row: None,
    col_span: 1,
    row_span: 1,
    font: Some("FiraMono".to_string()),
  }
}

fn build_calendar_grid(
  grid: &mut Widget,
  year: i32,
  month: u32,
  today: NaiveDate,
) {
  let children = match grid {
    Widget::Container { children, .. } => children,
    _ => return,
  };
  children.clear();

  let turquoise = (0.498, 0.827, 0.647, 1.0);
  let fg = (0.965, 0.980, 0.980, 1.0);
  let blue = (0.129, 0.533, 0.937, 1.0);
  let white = (1.0, 1.0, 1.0, 1.0);

  for hdr in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
    children.push(cal_label(hdr, 14.0, 24.0, turquoise));
  }

  let first =
    NaiveDate::from_ymd_opt(year, month, 1).unwrap();
  let offset =
    first.weekday().num_days_from_monday() as usize;
  for _ in 0..offset {
    children.push(cal_label("", 16.0, 28.0, fg));
  }

  let num_days = days_in_month(year, month);
  for day in 1..=num_days {
    let is_today = year == today.year()
      && month == today.month()
      && day == today.day();

    if is_today {
      children.push(Widget::Container {
        id: None,
        scrollable: false,
        bounds: Default::default(),
        width: Some(36.0),
        height: Some(28.0),
        style: gloomy_core::style::BoxStyle {
          background: Some(blue),
          corner_radii: [6.0; 4],
          ..Default::default()
        },
        padding: 0.0,
        layout: gloomy_core::layout::Layout {
          direction:
            gloomy_core::layout::Direction::Column,
          align_items:
            gloomy_core::layout::Align::Stretch,
          justify_content:
            gloomy_core::layout::Justify::Center,
          ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        children: vec![cal_label(
          &day.to_string(),
          16.0,
          20.0,
          white,
        )],
        layout_cache: None,
        render_cache: Default::default(),
      });
    } else {
      children.push(cal_label(
        &day.to_string(),
        16.0,
        28.0,
        fg,
      ));
    }
  }
}

fn main() -> anyhow::Result<()> {
  let mut ui = load_ui("examples/ui/clock.ron")?;
  let now = Local::now();
  let today = now.date_naive();
  let dm = now.month();
  let dy = now.year();

  // Update clock labels.
  if let Some(Widget::Container { children, .. }) =
    find_widget_mut(&mut ui, "clock_section")
  {
    if let Some(Widget::Label { text, .. }) =
      children.get_mut(0)
    {
      *text = now.format("%H:%M").to_string();
    }
    if let Some(Widget::Label { text, .. }) =
      children.get_mut(2)
    {
      *text = now.format("%A, %B %-d").to_string();
    }
  }

  // Update month header.
  if let Some(Widget::Container { children, .. }) =
    find_widget_mut(&mut ui, "calendar_section")
  {
    if let Some(Widget::Container {
      children: hdr, ..
    }) = children.get_mut(0)
    {
      if let Some(Widget::Label { text, .. }) =
        hdr.get_mut(1)
      {
        *text = format!("{} {}", month_name(dm), dy);
      }
    }
  }

  // Build calendar grid.
  if let Some(grid) =
    find_widget_mut(&mut ui, "calendar_grid")
  {
    build_calendar_grid(grid, dy, dm, today);
  }

  // Debug: print widget tree structure after grid build.
  print_tree(&ui, 0);

  // Frame 1: fresh render (populates layout cache).
  let interaction = InteractionState::new();
  {
    let mut hr =
      HeadlessRenderer::new(WIDTH, HEIGHT, 1.0)?;
    let img = hr.render_to_image(
      &mut ui, Some(&interaction), None,
    )?;
    img.save("snapshots/clock_frame1.png")?;
    eprintln!("Saved frame 1 (first render — correct)");
  }

  // Simulate frame 2: rebuild grid, NO mark_dirty.
  if let Some(grid) =
    find_widget_mut(&mut ui, "calendar_grid")
  {
    build_calendar_grid(grid, dy, dm, today);
  }
  {
    let mut hr =
      HeadlessRenderer::new(WIDTH, HEIGHT, 1.0)?;
    let img = hr.render_to_image(
      &mut ui, Some(&interaction), None,
    )?;
    img.save("snapshots/clock_frame2_no_dirty.png")?;
    eprintln!(
      "Saved frame 2 (no mark_dirty — cached/broken)"
    );
  }

  // Frame 3: rebuild grid, WITH mark_dirty.
  if let Some(grid) =
    find_widget_mut(&mut ui, "calendar_grid")
  {
    build_calendar_grid(grid, dy, dm, today);
  }
  ui.mark_dirty();
  {
    let mut hr =
      HeadlessRenderer::new(WIDTH, HEIGHT, 1.0)?;
    let img = hr.render_to_image(
      &mut ui, Some(&interaction), None,
    )?;
    img.save("snapshots/clock_frame3_mark_dirty.png")?;
    eprintln!(
      "Saved frame 3 (with mark_dirty — fixed)"
    );
  }

  // Frame 4: same scene at 2.0x scale (simulates HiDPI).
  if let Some(grid) =
    find_widget_mut(&mut ui, "calendar_grid")
  {
    build_calendar_grid(grid, dy, dm, today);
  }
  ui.mark_dirty();
  {
    let mut hr = HeadlessRenderer::new(
      WIDTH * 2, HEIGHT * 2, 2.0,
    )?;
    let img = hr.render_to_image(
      &mut ui, Some(&interaction), None,
    )?;
    img.save("snapshots/clock_frame4_hidpi.png")?;
    eprintln!(
      "Saved frame 4 (2.0x HiDPI scale)"
    );
  }

  Ok(())
}

fn print_tree(w: &Widget, depth: usize) {
  let indent = "  ".repeat(depth);
  match w {
    Widget::Container {
      id,
      bounds,
      width,
      height,
      flex,
      layout,
      children,
      ..
    } => {
      eprintln!(
        "{}Container id={:?} w={:?} h={:?} flex={} \
         dir={:?} bounds=({:.0},{:.0},{:.0},{:.0}) \
         children={}",
        indent,
        id,
        width,
        height,
        flex,
        layout.direction,
        bounds.x,
        bounds.y,
        bounds.width,
        bounds.height,
        children.len(),
      );
      for c in children {
        print_tree(c, depth + 1);
      }
    }
    Widget::Label {
      text,
      x,
      y,
      width,
      height,
      size,
      ..
    } => {
      eprintln!(
        "{}Label '{}' size={} ({:.0},{:.0},{:.0},{:.0})",
        indent, text, size, x, y, width, height,
      );
    }
    Widget::Button {
      text, action, ..
    } => {
      eprintln!(
        "{}Button '{}' action='{}'",
        indent, text, action,
      );
    }
    Widget::Spacer { size, .. } => {
      eprintln!("{}Spacer({})", indent, size);
    }
    _ => {
      eprintln!("{}(other widget)", indent);
    }
  }
}
