/// Simple starter example demonstrating basic Gloomy UI usage.
///
/// This example shows:
/// - Creating widgets programmatically
/// - Using the callback-based API
/// - Handling button clicks
/// - Basic interaction state management
///
/// Run with: cargo run --example simple_starter

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions, hit_test},
    widget::{Widget, WidgetBounds},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::ElementState;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        ui_root: create_ui(0),
        interaction: InteractionState::default(),
        counter: 0,
    }));
    
    let state_move = state.clone();
    let state_input = state.clone();
    let state_draw = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_cursor_move(move |win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x as f32, y as f32);
            s.interaction.update_mouse(pos);
            
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
                
                if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                    let action = res.action.to_string();
                    s.interaction.set_active(Some(action.clone()));
                }
            } else {
                if let Some(ref action) = s.interaction.active_action.clone() {
                    let mouse_pos = s.interaction.mouse_pos;
                    let scroll_offsets = s.interaction.scroll_offsets.clone();
                    
                    if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                        if res.action == action {
                            s.handle_click(action);
                            s.rebuild_ui();
                        }
                    }
                }
                
                s.interaction.set_pressed(false);
                s.interaction.set_active(None);
            }
            
            win.window.request_redraw();
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            // Compute layout
            let size = win.renderer.size();
            compute_layout(&mut s.ui_root, 0.0, 0.0, size.x, size.y);
            
            // Handle any widget interactions (updates widget internal state)
            let interaction_copy = s.interaction.clone();
            handle_interactions(&mut s.ui_root, &interaction_copy, Vec2::ZERO);
            
            // Render the UI
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
}

impl AppState {
    fn handle_click(&mut self, action: &str) {
        match action {
            "increment" => {
                self.counter += 1;
                println!("Counter: {}", self.counter);
            }
            _ => println!("Unknown action: {}", action),
        }
    }
    
    fn rebuild_ui(&mut self) {
        self.ui_root = create_ui(self.counter);
    }
}

fn create_ui(counter: i32) -> Widget {
    Widget::Container {
        id: Some("main".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(800.0),
        height: Some(600.0),
        background: Some((0.15, 0.15, 0.15, 1.0)),
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        padding: 30.0,
        layout: Layout {
            direction: Direction::Column,
            spacing: 20.0,
            ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        corner_radii: None,
        children: vec![
            // Title
            Widget::Label {
                text: "Gloomy UI - Simple Starter".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 32.0,
                color: (0.9, 0.9, 0.9, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Description
            Widget::Label {
                text: "Click the button to increment the counter".to_string(),
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
            
            // Counter display
            Widget::Label {
                text: format!("Counter: {}", counter),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 24.0,
                color: (1.0, 1.0, 1.0, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Increment button
            Widget::Button {
                text: "Click me!".to_string(),
                action: "increment".to_string(),
                bounds: WidgetBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 150.0,
                    height: 40.0,
                },
                background: (0.25, 0.25, 0.25, 1.0),
                hover_color: (0.35, 0.35, 0.35, 1.0),
                active_color: (0.45, 0.45, 0.45, 1.0),
                border: None,
                corner_radius: 4.0,
                shadow: None,
                gradient: None,
                corner_radii: None,
                layout: Layout::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
        ],
    }
}
