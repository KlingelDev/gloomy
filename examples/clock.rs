use chrono::{Datelike, Local, NaiveDate};
use gloomy_app::{GloomyApp, GloomyWindow};
use gloomy_core::interaction::InteractionState;
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::ui::{find_widget_mut, hit_test, load_ui, render_ui};
use gloomy_core::widget::Widget;
use std::cell::RefCell;
use std::rc::Rc;
use winit::event::ElementState;

struct AppState {
  ui: Widget,
  interaction: InteractionState,
  display_month: u32,
  display_year: i32,
}

/// Returns the number of days in the given month/year.
fn days_in_month(y: i32, m: u32) -> u32 {
  let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
  NaiveDate::from_ymd_opt(ny, nm, 1)
    .unwrap()
    .pred_opt()
    .unwrap()
    .day()
}

/// Month name for display.
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

/// Creates a Label cell for the calendar grid.
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

/// Rebuilds the calendar grid children for the given month/year.
/// Uses auto-flow — cells are pushed in order and the 7-column
/// grid places them automatically.
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

  // Kitty theme palette.
  let dim = (0.620, 0.620, 0.620, 1.0);
  let fg = (0.992, 0.988, 0.988, 1.0);
  let tomato = (1.0, 0.376, 0.353, 1.0);
  let white = (1.0, 1.0, 1.0, 1.0);

  // Day-of-week headers.
  for hdr in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
    children.push(cal_label(hdr, 14.0, 24.0, dim));
  }

  // Blank cells before day 1.
  let first =
    NaiveDate::from_ymd_opt(year, month, 1).unwrap();
  let offset =
    first.weekday().num_days_from_monday() as usize;
  for _ in 0..offset {
    children.push(cal_label("", 16.0, 28.0, fg));
  }

  // Day cells.
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
          background: Some(tomato),
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
  env_logger::init();

  let ui = load_ui("examples/ui/clock.ron")?;
  let now = Local::now();

  let state = Rc::new(RefCell::new(AppState {
    ui,
    interaction: InteractionState::new(),
    display_month: now.month(),
    display_year: now.year(),
  }));

  let draw_state = state.clone();
  let mouse_state = state.clone();
  let cursor_state = state.clone();

  GloomyApp::new()
    .with_title("Clock + Calendar")
    .with_size(900, 520)
    .on_cursor_move(move |_win, x, y| {
      let mut s = cursor_state.borrow_mut();
      s.interaction.update_mouse(
        gloomy_core::Vec2::new(x as f32, y as f32),
      );
    })
    .on_mouse_input(move |_win, btn_state, _button| {
      let mut s = mouse_state.borrow_mut();
      s.interaction.set_pressed(
        btn_state == ElementState::Pressed,
      );
      if btn_state == ElementState::Pressed {
        let hit = hit_test(
          &s.ui,
          s.interaction.mouse_pos,
          Some(&s.interaction),
        )
        .map(|h| h.action.to_string());

        if let Some(action) = hit {
          s.interaction.active_action =
            Some(action.clone());
          s.interaction.focused_id = Some(action.clone());
          match action.as_str() {
            "prev_month" => {
              if s.display_month == 1 {
                s.display_month = 12;
                s.display_year -= 1;
              } else {
                s.display_month -= 1;
              }
            }
            "next_month" => {
              if s.display_month == 12 {
                s.display_month = 1;
                s.display_year += 1;
              } else {
                s.display_month += 1;
              }
            }
            _ => {}
          }
        } else {
          s.interaction.focused_id = None;
        }
      }
    })
    .on_draw(move |win: &mut GloomyWindow, ctx| {
      let mut s = draw_state.borrow_mut();

      // #101010 background via clear color (sRGB → linear).
      win.renderer.set_clear_color(
        0.00486, 0.00486, 0.00486, 1.0,
      );
      let now = Local::now();

      // Update clock labels.
      if let Some(Widget::Container {
        children, ..
      }) = find_widget_mut(&mut s.ui, "clock_section")
      {
        // children[0] = time label
        if let Some(Widget::Label { text, .. }) =
          children.get_mut(0)
        {
          *text = now.format("%H:%M").to_string();
        }
        // children[2] = date label (after Spacer)
        if let Some(Widget::Label { text, .. }) =
          children.get_mut(2)
        {
          *text = now.format("%A, %B %-d").to_string();
        }
      }

      // Update month header label.
      let dm = s.display_month;
      let dy = s.display_year;
      if let Some(Widget::Container {
        children, ..
      }) = find_widget_mut(&mut s.ui, "calendar_section")
      {
        // children[0] = header row container
        if let Some(Widget::Container {
          children: hdr, ..
        }) = children.get_mut(0)
        {
          // hdr[1] = month/year label
          if let Some(Widget::Label { text, .. }) =
            hdr.get_mut(1)
          {
            *text = format!("{} {}", month_name(dm), dy);
          }
        }
      }

      // Rebuild calendar grid.
      let today = now.date_naive();
      if let Some(grid) =
        find_widget_mut(&mut s.ui, "calendar_grid")
      {
        build_calendar_grid(grid, dy, dm, today);
      }

      // Hover tracking.
      if let Some(hit) = hit_test(
        &s.ui,
        s.interaction.mouse_pos,
        Some(&s.interaction),
      ) {
        s.interaction.hovered_action =
          Some(hit.action.to_string());
      } else {
        s.interaction.hovered_action = None;
      }

      let scale = win.renderer.scale_factor;
      let width = win.config.width as f32 / scale;
      let height = win.config.height as f32 / scale;

      // Invalidate layout caches — children change each frame.
      s.ui.mark_dirty();

      // Root container must have explicit bounds for flex.
      if let Widget::Container { bounds, .. } = &mut s.ui {
        bounds.width = width;
        bounds.height = height;
      }
      compute_layout(&mut s.ui, 0.0, 0.0, width, height);

      let interaction = s.interaction.clone();
      render_ui(
        &s.ui,
        &mut win.renderer,
        ctx.device,
        ctx.queue,
        Some(&interaction),
        None,
      );
    })
    .run()
}
