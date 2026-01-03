use gloomy_app::GloomyApp;
use gloomy_core::ui::{render_ui, hit_test, find_widget_mut};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::{Widget, WidgetBounds, NumberInputStyle, AutocompleteStyle};
use gloomy_core::validation::ValidationRule;
use gloomy_core::Vec2;
use winit::event::ElementState;
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    ui: Widget,
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create UI programmatically
    let number_input = Widget::NumberInput {
        id: "num_input".to_string(),
        value: 10.0,
        min: Some(0.0),
        max: Some(100.0),
        step: 0.5,
        precision: 1,
        show_spinner: true,
        bounds: WidgetBounds { width: 200.0, height: 40.0, ..Default::default() },
        style: NumberInputStyle::default(),
        width: 200.0,
        height: 40.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        validation: None,
    };

    let autocomplete = Widget::Autocomplete {
        id: "auto".to_string(),
        value: "".to_string(),
        placeholder: "Search...".to_string(),
        suggestions: vec![
            "Apple".to_string(), 
            "Banana".to_string(), 
            "Cherry".to_string(), 
            "Date".to_string(), 
            "Elderberry".to_string(),
            "Fig".to_string(),
            "Grape".to_string()
        ],
        max_visible: 5,
        bounds: WidgetBounds { width: 200.0, height: 40.0, ..Default::default() },
        style: AutocompleteStyle::default(),
        validation: Some(vec![ValidationRule::Required]),
        width: 200.0,
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
        children: vec![number_input, autocomplete],
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        background: Some((0.1, 0.1, 0.1, 1.0)),
        layout: gloomy_core::layout::Layout { 
            direction: gloomy_core::layout::Direction::Column, 
            justify_content: gloomy_core::layout::Justify::Center, 
            align_items: gloomy_core::layout::Align::Center, 
            spacing: 20.0,
            ..Default::default()
        },
        padding: 20.0,
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
                 // Context-aware hit test
                 let hit_action = hit_test(ui, interaction.mouse_pos, Some(interaction)).map(|h| h.action.to_string());
                 
                 if let Some(action) = hit_action {
                     println!("Clicked: {}", action);
                     interaction.active_action = Some(action.clone());
                     
                     // Focus handling
                     if action == "num_input" || action == "auto" {
                         interaction.focused_id = Some(action.clone());
                     } else if !action.contains(":opt:") && !action.contains(":up") && !action.contains(":down") {
                         interaction.focused_id = None;
                     }

                     // Autocomplete Selection
                     if action.starts_with("auto:opt:") {
                         let parts: Vec<&str> = action.split(":opt:").collect();
                         if let Ok(idx) = parts[1].parse::<usize>() {
                             if let Some(Widget::Autocomplete { value, suggestions, .. }) = find_widget_mut(ui, "auto") {
                                 if idx < suggestions.len() {
                                     *value = suggestions[idx].clone();
                                     interaction.focused_id = None; // Close dropdown
                                 }
                             }
                         }
                     }

                     // Handle NumberInput actions
                     if action == "num_input:up" {
                         if let Some(Widget::NumberInput { value, step, max, .. }) = find_widget_mut(ui, "num_input") {
                             *value += *step;
                             if let Some(m) = max { *value = value.min(*m); }
                         }
                     } else if action == "num_input:down" {
                         if let Some(Widget::NumberInput { value, step, min, .. }) = find_widget_mut(ui, "num_input") {
                             *value -= *step;
                             if let Some(m) = min { *value = value.max(*m); }
                         }
                     }
                 } else {
                     interaction.focused_id = None;
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
