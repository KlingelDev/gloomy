use gloomy_app::GloomyApp;
use gloomy_core::ui::{render_ui, hit_test, find_widget_mut};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::{Widget, WidgetBounds, DatePickerStyle};
use gloomy_core::Vec2;
use winit::event::ElementState;
use std::rc::Rc;
use std::cell::RefCell;
use chrono::NaiveDate;

struct AppState {
    ui: Widget,
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Date Picker Input
    let date_input = Widget::DatePicker {
        id: "date_input".to_string(),
        value: None,
        placeholder: "Select Date".to_string(),
        min_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
        max_date: Some(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()),
        format: "%Y-%m-%d".to_string(),
        bounds: WidgetBounds { width: 300.0, height: 40.0, ..Default::default() },
        style: DatePickerStyle::default(),
        validation: None,
        width: 300.0,
        height: 40.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    let container = Widget::Container {
        id: Some("root".to_string()),
        scrollable: false,
        children: vec![
            Widget::Label { 
                text: "DatePicker Demo".to_string(), 
                size: 24.0, 
                width: 300.0, 
                height: 40.0, 
                color: (1.0, 1.0, 1.0, 1.0),
                text_align: gloomy_core::widget::TextAlign::Center,
                font: None,
                flex: 0.0,
                grid_col: None, grid_row: None, col_span:1, row_span:1,
                x:0.0, y:0.0 
            },
            date_input,
        ],
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        background: Some((0.12, 0.12, 0.14, 1.0)),
        layout: gloomy_core::layout::Layout { 
            direction: gloomy_core::layout::Direction::Column, 
            justify_content: gloomy_core::layout::Justify::Center, 
            align_items: gloomy_core::layout::Align::Center, 
            spacing: 15.0,
            ..Default::default()
        },
        padding: 40.0,
        flex: 1.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        corner_radii: None,
        layout_cache: None,
        render_cache: RefCell::new(None),
    };
  
    let state = Rc::new(RefCell::new(AppState {
        ui: container,
        interaction: InteractionState::new(),
    }));

    let state_click = state.clone();
    let state_draw = state.clone();
    let state_cursor = state.clone();

    // Run App
    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_cursor.borrow_mut();
            s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
        })
        .on_mouse_input(move |_win, element_state, _button| {
            let mut s = state_click.borrow_mut();
            let app_state = &mut *s;
            let ui = &mut app_state.ui;
            let interaction = &mut app_state.interaction;
            
            interaction.set_pressed(element_state == ElementState::Pressed);
            
            if element_state == ElementState::Pressed {
                 let mouse_pos = interaction.mouse_pos;
                 // Interaction pass first for clone issues
                 let hit = hit_test(ui, mouse_pos, Some(interaction)).map(|h| h.action.to_string());
                 
                 interaction.set_clicked(hit.clone());
                 if let Some(id) = &hit {
                     // Keep focus if clicking on same widget or its components
                     if id.starts_with("date_input") {
                         interaction.focused_id = Some("date_input".to_string());
                     } else {
                         interaction.focused_id = Some(id.clone());
                     }
                 } else {
                     interaction.focused_id = None;
                 }
                 
                 interaction.triggered_action = hit.clone();
                 
                 // Handle DatePicker interactions
                 if let Some(act) = &hit {
                     // 1. Navigation (Prev/Next)
                     if interaction.handle_datepicker_action(act) {
                         // State updated internally
                         return;
                     } 
                     // 2. Day Selection
                     if let Some(stripped) = act.strip_prefix("date_input:day:") {
                         if let Ok(date) = NaiveDate::parse_from_str(stripped, "%Y-%m-%d") {
                             if let Some(Widget::DatePicker { value, .. }) = find_widget_mut(ui, "date_input") {
                                 *value = Some(date);
                                 interaction.focused_id = None; // Close picker
                             }
                         }
                     }
                 }
            }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            let scale = win.renderer.scale_factor;
            let width = win.config.width as f32 / scale;
            let height = win.config.height as f32 / scale;
            
            if let Widget::Container { bounds, .. } = &mut s.ui {
                bounds.width = width;
                bounds.height = height;
            }
            compute_layout(&mut s.ui, 0.0, 0.0, width, height);
            
            if let Some(hit) = hit_test(&s.ui, s.interaction.mouse_pos, Some(&s.interaction)) {
                s.interaction.hovered_action = Some(hit.action.to_string());
            } else {
                s.interaction.hovered_action = None;
            }

            render_ui(
                &s.ui, 
                &mut win.renderer,
                ctx.device, 
                ctx.queue, 
                Some(&s.interaction),
                None
            );
        })
        .run();
        
    Ok(())
}
