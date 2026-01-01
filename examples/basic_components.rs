use gloomy_app::GloomyApp;
use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions, handle_keyboard_event, hit_test},
    widget::{Widget, WidgetBounds, TextAlign, TextInputStyle, CheckboxStyle, SliderStyle, Border},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::ElementState;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create the UI tree manually
    let ui_root = create_ui();
    
    let state = Rc::new(RefCell::new(AppState {
        ui_root,
        interaction: InteractionState::default(),
        counter: 0,
        input_value: String::new(),
        checked: false,
        slider_val: 0.5,
        icon_loaded: false,
    }));
    
    let state_move = state.clone();
    let state_input = state.clone();
    let state_key = state.clone();
    let state_draw = state.clone();
    // ... (rest of main setup)
    
        GloomyApp::new()
        .on_cursor_move(move |win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x as f32, y as f32);
            s.interaction.update_mouse(pos);
            
            // Critical for interactivity: Hit test on hover
            let scroll_offsets = s.interaction.scroll_offsets.clone();
            if let Some(res) = hit_test(&s.ui_root, pos, Some(&scroll_offsets)) {
                s.interaction.hovered_action = Some(res.action.to_string());
            } else {
                s.interaction.hovered_action = None;
            }
            
            win.window.request_redraw();
        })
        .on_mouse_input(move |win, elem_state, _btn| {
            let mut s = state_input.borrow_mut();
            
            if elem_state == ElementState::Pressed {
                s.interaction.set_pressed(true);
                
                let mouse_pos = s.interaction.mouse_pos;
                let scroll_offsets = s.interaction.scroll_offsets.clone();
                
                // Active/Clicked state
                if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                    let action = res.action.to_string();
                    s.interaction.set_active(Some(action.clone()));
                    // Handle focus
                    s.interaction.focused_id = Some(action.clone());
                } else {
                    s.interaction.set_active(None);
                    s.interaction.focused_id = None;
                }
            } else {
                // Released - check for click trigger
                if let Some(ref action) = s.interaction.active_action.clone() {
                     let mouse_pos = s.interaction.mouse_pos;
                     let scroll_offsets = s.interaction.scroll_offsets.clone();
                     
                     if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                         if res.action == action {
                             s.handle_click(action);
                             // Re-create UI to reflect state changes
                             s.rebuild_ui();
                         }
                     }
                }
                
                s.interaction.set_pressed(false);
                s.interaction.set_active(None);
            }
            
            win.window.request_redraw();
        })
        .on_keyboard_input(move |win, event| {
            let mut s = state_key.borrow_mut();
            let AppState { ui_root, interaction, input_value, .. } = &mut *s;
            
             if handle_keyboard_event(ui_root, interaction, &event) {
                // Fetch updated values back to state if needed (two-way binding simulation)
                if let Widget::Container { children, .. } = ui_root {
                    // Extract text input value (this is hacky for basic example, normally use ID map)
                     if let Some(Widget::TextInput { value, .. }) = children.get(3) {
                         *input_value = value.clone();
                     }
                }
                win.window.request_redraw();
             }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            if !s.icon_loaded {
                if let Ok(tex) = gloomy_core::svg_loader::load_svg_texture(
                    ctx.device, 
                    ctx.queue, 
                    SVG_DATA.as_bytes(), 
                    64, 
                    64
                ) {
                    win.renderer.register_texture("test_icon".to_string(), tex);
                    s.icon_loaded = true;
                }
            }
            let width = win.config.width as f32;
            let height = win.config.height as f32;
            
            compute_layout(&mut s.ui_root, 0.0, 0.0, width, height);
            
            // Handle internal interactions (slider dragging)
            let interaction_copy = s.interaction.clone();
            handle_interactions(&mut s.ui_root, &interaction_copy, Vec2::ZERO);
            
            // Sync slider state back
            let mut new_val = 0.0;
            if let Widget::Container { children, .. } = &mut s.ui_root {
                if let Some(Widget::Slider { value, .. }) = children.get(5) {
                    new_val = *value;
                }
                
                // Update label in real-time
                if let Some(Widget::Label { text, .. }) = children.get_mut(6) {
                    *text = format!("Slider: {:.2}", new_val);
                }
            }
            s.slider_val = new_val;
            
            render_ui(
                &s.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
            );
        })
        .run()
}

struct AppState {
    ui_root: Widget,
    interaction: InteractionState,
    counter: i32,
    input_value: String,
    checked: bool,
    slider_val: f32,
    icon_loaded: bool,
}

const SVG_DATA: &str = "<svg width='64' height='64' xmlns='http://www.w3.org/2000/svg'><circle cx='32' cy='32' r='30' fill='red' /><rect x='16' y='16' width='32' height='32' fill='none' stroke='white' stroke-width='4' /></svg>";

impl AppState {
    fn handle_click(&mut self, action: &str) {
        if action == "btn_click" {
            self.counter += 1;
        } else if action == "chk_toggle" {
            self.checked = !self.checked;
        }
    }
    
    fn rebuild_ui(&mut self) {
        // Simple immediate mode style: recreate UI on state change
        self.ui_root = Widget::Container {
            id: None,
            scrollable: false,
            bounds: WidgetBounds::default(),
            gradient: None,
            shadow: None,
            width: None,
            height: None,
            background: Some((0.15, 0.15, 0.15, 1.0)),
            border: None,
            corner_radius: 0.0,
            corner_radii: None,
            padding: 20.0,
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            layout: Layout {
                direction: Direction::Column,
                spacing: 12.0,
                ..Default::default()
            },
            children: vec![
                Widget::label("Basic Widget Test"),
                Widget::Spacer { size: 10.0, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1 },
                
                Widget::Button {
                    text: format!("Clicked: {}", self.counter),
                    action: "btn_click".to_string(),
                    bounds: WidgetBounds { x: 0.0, y: 0.0, width: 200.0, height: 40.0 },
                    background: (0.3, 0.3, 0.3, 1.0),
                    hover_color: (0.4, 0.4, 0.4, 1.0),
                    active_color: (0.2, 0.2, 0.2, 1.0),
                    border: None,
                    gradient: None,
                    shadow: None,
                    corner_radius: 4.0,
                    corner_radii: None,
                    layout: Layout::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    font: None,
                },
                
                Widget::TextInput {
                    id: "text_input".to_string(),
                    value: self.input_value.clone(),
                    placeholder: "Type something...".to_string(),
                    font_size: 16.0,
                    text_align: TextAlign::Left,
                    bounds: WidgetBounds { x: 0.0, y: 0.0, width: 200.0, height: 40.0 },
                    style: TextInputStyle {
                        background: Some((0.1, 0.1, 0.1, 1.0).into()),
                        border: Some(Border {
                            width: 1.0, 
                            color: (0.3, 0.3, 0.3, 1.0).into(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    width: 200.0,
                    height: 40.0,
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                },
                
                Widget::Checkbox {
                    id: "chk_toggle".to_string(),
                    checked: self.checked,
                    size: 24.0,
                    style: CheckboxStyle {
                        background: (0.2, 0.2, 0.2, 1.0).into(),
                        checkmark_color: (0.9, 0.9, 0.9, 1.0).into(),
                        ..Default::default()
                    },
                    bounds: WidgetBounds::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                },
                
                Widget::Slider {
                    id: "slider".to_string(),
                    value: self.slider_val,
                    min: 0.0,
                    max: 1.0,
                    style: SliderStyle {
                        track_height: 6.0,
                        thumb_radius: 10.0,
                        active_track_color: (0.6, 0.6, 0.6, 1.0).into(),
                        track_color: (0.2, 0.2, 0.2, 1.0).into(),
                        ..Default::default()
                    },
                    bounds: WidgetBounds::default(),
                    width: 200.0,
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                },
                
                Widget::label(format!("Slider: {:.2}", self.slider_val)),
                
                Widget::Spacer { size: 20.0, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1 },
                
                Widget::Icon {
                    id: "my_icon".to_string(),
                    icon_name: "test_icon".to_string(),
                    size: 64.0,
                    color: None,
                    bounds: WidgetBounds::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                },
            ],
        }
    }
}

fn create_ui() -> Widget {
    let mut state = AppState {
        ui_root: Widget::Spacer { size: 0.0, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1 },
        interaction: InteractionState::default(),
        counter: 0,
        input_value: String::new(),
        checked: false,
        slider_val: 0.5,
        icon_loaded: false,
    };
    state.rebuild_ui();
    state.ui_root
}

