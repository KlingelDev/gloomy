//! Gloomy Designer - Visual editor for RON UI layouts.
//!
//! A three-panel designer tool:
//! - Left: Widget palette
//! - Center: Design canvas + widget tree
//! - Right: Property inspector

mod app;
mod model;
mod panels;
mod ron_export;

use app::DesignerApp;
use gloomy_app::GloomyApp;
use gloomy_core::interaction::InteractionState;
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::ui::{render_ui, handle_interactions, hit_test};
use gloomy_core::widget::Widget;
use gloomy_core::Vec2;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use winit::event::ElementState;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Starting Gloomy Designer");

    let state = Rc::new(RefCell::new(DesignerState::new()));
    
    let state_move = state.clone();
    let state_input = state.clone();
    let state_draw = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x as f32, y as f32);
            s.interaction.update_mouse(pos);
            
            // Perform hit test for hover state
            let scroll_offsets = s.interaction.scroll_offsets.clone();
            if let Some(res) = hit_test(
                &s.app.ui_root, pos, Some(&scroll_offsets)
            ) {
                s.interaction.hovered_action = Some(res.action.to_string());
            } else {
                s.interaction.hovered_action = None;
            }
            
            // Trigger redraw so potential hover changes are visible
            win.window.request_redraw();
        })
        .on_mouse_input(move |win, elem_state, _btn| {
            let mut s = state_input.borrow_mut();
            
            if elem_state == ElementState::Pressed {
                s.interaction.set_pressed(true);
                
                // Hit test to find what was clicked
                let mouse_pos = s.interaction.mouse_pos;
                let scroll_offsets = s.interaction.scroll_offsets.clone();
                
                if let Some(res) = hit_test(
                    &s.app.ui_root, mouse_pos, Some(&scroll_offsets)
                ) {
                    let action = res.action.to_string();
                    log::debug!("Hit: {}", action);
                    s.interaction.set_active(Some(action.clone()));
                    s.interaction.set_clicked(Some(action));
                } else {
                    s.interaction.set_active(None);
                    s.interaction.set_clicked(None);
                }
            } else {
                // Mouse released
                // Check if we should trigger the action (clicked and released on same)
                if let Some(ref action) = s.interaction.active_action.clone() {
                    // Verify still over the button
                    let mouse_pos = s.interaction.mouse_pos;
                    let scroll_offsets = s.interaction.scroll_offsets.clone();
                    
                    if let Some(res) = hit_test(
                        &s.app.ui_root, mouse_pos, Some(&scroll_offsets)
                    ) {
                        if res.action == action {
                            log::info!("Action triggered: {}", action);
                            s.app.handle_action(action);
                            s.app.refresh_ui();
                        }
                    }
                }
                
                s.interaction.set_pressed(false);
                s.interaction.set_active(None);
                s.interaction.set_clicked(None);
            }
            // Trigger redraw for click state changes
            win.window.request_redraw();
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            let width = win.config.width as f32;
            let height = win.config.height as f32;
            
            // Ensure root fills window
            if let Widget::Container { bounds, .. } = &mut s.app.ui_root {
                bounds.x = 0.0;
                bounds.y = 0.0;
                bounds.width = width;
                bounds.height = height;
            }
            
            compute_layout(&mut s.app.ui_root, 0.0, 0.0, width, height);
            
            // Handle interactions (for sliders, etc.)
            let interaction_copy = s.interaction.clone();
            handle_interactions(&mut s.app.ui_root, &interaction_copy, Vec2::ZERO);
            
            render_ui(
                &s.app.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
            );
        })
        .run()
}

/// Combined state for the designer.
struct DesignerState {
    app: DesignerApp,
    interaction: InteractionState,
}

impl DesignerState {
    fn new() -> Self {
        Self {
            app: DesignerApp::new(),
            interaction: InteractionState::default(),
        }
    }
}
