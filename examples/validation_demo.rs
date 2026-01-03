use gloomy_app::GloomyApp;
use gloomy_core::ui::{render_ui, hit_test, find_widget_mut};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::{Widget, WidgetBounds, TextInputStyle, NumberInputStyle, AutocompleteStyle, Border};
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
    
    // 1. Define Widgets
    
    // Name Input (Required, MinLength 3)
    let name_input = Widget::TextInput {
        id: "name_input".to_string(),
        value: "".to_string(),
        placeholder: "Enter Name (Min 3 chars)".to_string(),
        font_size: 16.0,
        text_align: gloomy_core::widget::TextAlign::Left,
        bounds: WidgetBounds { width: 300.0, height: 40.0, ..Default::default() },
        style: TextInputStyle::default(),
        validation: Some(vec![
            ValidationRule::Required,
            ValidationRule::MinLength(3)
        ]),
        width: 300.0,
        height: 40.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    // Age Input (Min 18, Max 120)
    let age_input = Widget::NumberInput {
        id: "age_input".to_string(),
        value: 18.0,
        min: Some(0.0),
        max: Some(120.0),
        step: 1.0,
        precision: 0,
        show_spinner: true,
        bounds: WidgetBounds { width: 300.0, height: 40.0, ..Default::default() },
        style: NumberInputStyle::default(),
        validation: Some(vec![
            ValidationRule::Min(18.0),
            ValidationRule::Max(120.0)
        ]),
        width: 300.0,
        height: 40.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };
    
    // Country Autocomplete (Required, Must be in list)
    let countries = vec![
        "Argentina".to_string(), "Australia".to_string(), "Brazil".to_string(), 
        "Canada".to_string(), "China".to_string(), "Denmark".to_string(), 
        "Egypt".to_string(), "France".to_string(), "Germany".to_string(), 
        "India".to_string(), "Japan".to_string(), "Mexico".to_string(), 
        "Norway".to_string(), "Poland".to_string(), "Russia".to_string(), 
        "South Africa".to_string(), "Spain".to_string(), "Sweden".to_string(), 
        "United Kingdom".to_string(), "United States".to_string()
    ];
    
    let country_input = Widget::Autocomplete {
        id: "country_input".to_string(),
        value: "".to_string(),
        placeholder: "Select Country".to_string(),
        suggestions: countries.clone(),
        max_visible: 6,
        bounds: WidgetBounds { width: 300.0, height: 40.0, ..Default::default() },
        style: AutocompleteStyle::default(),
        validation: Some(vec![ValidationRule::Required]),
        width: 300.0,
        height: 40.0,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
    };

    // Submit Button
    let submit_btn = Widget::Button {
        // Button uses `action` as ID effectively
        text: "Submit".to_string(),
        action: "submit".to_string(),
        bounds: WidgetBounds { width: 150.0, height: 40.0, ..Default::default() },
        background: (0.2, 0.6, 0.3, 1.0),
        hover_color: (0.25, 0.65, 0.35, 1.0),
        active_color: (0.15, 0.55, 0.25, 1.0),
        border: Some(Border { 
            color: (0.3, 0.7, 0.4, 1.0), 
            width: 1.0, 
            ..Default::default()
        }),
        corner_radius: 4.0,
        shadow: None,
        gradient: None,
        corner_radii: None,
        layout: Default::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        font: None,
    };
    
    let container = Widget::Container {
        id: Some("root".to_string()),
        scrollable: false,
        children: vec![
            Widget::Label { 
                text: "Validation Demo".to_string(), 
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
            name_input, 
            age_input, 
            country_input,
            Widget::Spacer { size: 20.0, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1 },
            submit_btn
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
    let state_key = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_cursor.borrow_mut();
            s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
        })
        .on_mouse_input(move |_win, element_state, _button| {
            let mut s = state_click.borrow_mut();
            s.interaction.set_pressed(element_state == ElementState::Pressed);
            
            if element_state == ElementState::Pressed {
                 // Clone needed for lifetime issues in hit_test vs check_validations
                 let mouse_pos = s.interaction.mouse_pos;
                 
                 // Use generic hit test
                 let hit = hit_test(&s.ui, mouse_pos, Some(&s.interaction)).map(|h| h.action.to_string());
                 
                 if let Some(action) = hit {
                     println!("Clicked: {}", action);
                     
                     // Focus handling
                     if action == "name_input" || action == "age_input" || action == "country_input" {
                         s.interaction.focused_id = Some(action.clone());
                     } else if !action.contains(":opt:") && !action.contains(":up") && !action.contains(":down") {
                         // Click outside -> Blur
                         s.interaction.focused_id = None;
                     }

                     // Autocomplete Selection
                     if action.starts_with("country_input:opt:") {
                         let parts: Vec<&str> = action.split(":opt:").collect();
                         if let Ok(idx) = parts[1].parse::<usize>() {
                             if let Some(Widget::Autocomplete { value, suggestions, .. }) = find_widget_mut(&mut s.ui, "country_input") {
                                 if idx < suggestions.len() {
                                     *value = suggestions[idx].clone();
                                     s.interaction.focused_id = None; // Close dropdown
                                 }
                             }
                         }
                     }
                     
                     // NumberInput Spinner
                     if action == "age_input:up" {
                         if let Some(Widget::NumberInput { value, step, max, .. }) = find_widget_mut(&mut s.ui, "age_input") {
                             *value += *step;
                             if let Some(m) = max { *value = value.min(*m); }
                         }
                     } else if action == "age_input:down" {
                         if let Some(Widget::NumberInput { value, step, min, .. }) = find_widget_mut(&mut s.ui, "age_input") {
                             *value -= *step;
                             if let Some(m) = min { *value = value.max(*m); }
                         }
                     }

                     // Submit -> Validate All
                     if action == "submit" {
                         // Clear previous errors first? Or they get overwritten.
                         let mut all_errors = std::collections::HashMap::new();
                         
                         // Helper to validate a specific widget
                         let mut check_widget = |id: &str| {
                             if let Some(w) = find_widget_mut(&mut s.ui, id) {
                                 let errors = w.validate();
                                 if !errors.is_empty() {
                                     all_errors.insert(id.to_string(), errors);
                                 }
                             }
                         };
                         
                         check_widget("name_input");
                         check_widget("age_input");
                         check_widget("country_input");
                         
                         s.interaction.validation_errors = all_errors;
                         if s.interaction.validation_errors.is_empty() {
                             println!("Form Submitted Successfully!");
                         } else {
                             println!("Form Validation Failed: {:?}", s.interaction.validation_errors);
                         }
                     }
                 } else {
                     // Clicked nothing interactive -> Blur
                     s.interaction.focused_id = None;
                 }
            }
        })
        .on_keyboard_input(move |_win, event| {
             if event.state == ElementState::Pressed {
                 let mut s = state_key.borrow_mut();
                 if let Some(focus_id) = s.interaction.focused_id.clone() {
                     // Simple text interaction for Name and Country
                     if let Some(w) = find_widget_mut(&mut s.ui, &focus_id) {
                         match w {
                             Widget::TextInput { value, .. } | Widget::Autocomplete { value, .. } => {
                                 if let winit::keyboard::Key::Character(ch) = &event.logical_key {
                                     if !ch.chars().any(|c| c.is_control()) {
                                         value.push_str(ch);
                                     }
                                 } else if let winit::keyboard::Key::Named(winit::keyboard::NamedKey::Backspace) = &event.logical_key {
                                     value.pop();
                                 }
                             }
                             _ => {}
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
