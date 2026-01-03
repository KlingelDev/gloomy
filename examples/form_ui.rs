use gloomy_app::{GloomyApp, GloomyWindow};
use gloomy_core::ui::{load_ui, render_ui, find_widget_mut};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::Widget;
use winit::keyboard::{Key, NamedKey};
use winit::event::ElementState;
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    ui: Widget,
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
  env_logger::init();
  
  // Load UI definition
  let ui = load_ui("examples/ui/form_demo.ron")?;
  
  let state = Rc::new(RefCell::new(AppState {
      ui,
      interaction: InteractionState::new(),
  }));

  let state_clone = state.clone();
  let state_clone2 = state.clone();
  let state_clone3 = state.clone();
  let state_clone_key = state.clone();

  GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state_clone.borrow_mut();
        s.interaction.update_mouse(gloomy_core::Vec2::new(x as f32, y as f32));
    })
    .on_mouse_input(move |_win, state, _button| {
        let mut s = state_clone2.borrow_mut();
        s.interaction.set_pressed(state == ElementState::Pressed);
        
        if state == ElementState::Pressed {
             // Hit test logic
             // We need to re-run hit test here because ui.hit_test is decoupled from interaction state update?
             // Actually, usually we rely on current mouse pos.
             // But simpler: just use generic hit_test again if we exposed it, or rely on interaction updates.
             // Wait, GloomyApp doesn't auto-update interaction state hit test logic. 
             // We have to do it manually in example or move logic to library.
             
             // For this example, let's implement hit test handling here.
             use gloomy_core::ui::hit_test;
             let hit_action = hit_test(&s.ui, s.interaction.mouse_pos, Some(&s.interaction)).map(|h| h.action.to_string());

             if let Some(action) = hit_action {
                 println!("Clicked: {}", action);
                 s.interaction.active_action = Some(action.clone());
                 s.interaction.focused_id = Some(action);
             } else {
                 s.interaction.focused_id = None;
             }
        }
    })
    .on_keyboard_input(move |_win, event| {
         if event.state == ElementState::Pressed {
             let mut s = state_clone_key.borrow_mut();
             if let Some(focused) = &s.interaction.focused_id {
                 // Clone ID to avoid borrow issues
                 let focused_id = focused.clone(); 
                 
                 // Handle text editing
                 if let Some(widget) = find_widget_mut(&mut s.ui, &focused_id) {
                     if let Widget::TextInput { value, .. } = widget {
                        match &event.logical_key {
                            Key::Named(NamedKey::Backspace) => {
                                value.pop();
                            }
                            Key::Character(c) => {
                                // Filter control chars etc?
                                // winit usually sends even for enter etc.
                                if !c.chars().any(|ch| ch.is_control()) {
                                    value.push_str(c);
                                }
                            }
                            Key::Named(NamedKey::Space) => {
                                value.push(' ');
                            }
                             _ => {}
                        }
                     }
                 }
             }
         }
    })
    .on_draw(move |win, ctx| {
      let mut s = state_clone3.borrow_mut();
      
      let width = win.config.width as f32;
      let height = win.config.height as f32;
      
      // Layout
      compute_layout(&mut s.ui, 0.0, 0.0, width, height);
      
      // Hit test update for hover
      use gloomy_core::ui::hit_test;
      if let Some(hit) = hit_test(&s.ui, s.interaction.mouse_pos, Some(&s.interaction)) {
          s.interaction.hovered_action = Some(hit.action.to_string());
      } else {
          s.interaction.hovered_action = None;
      }

      let (prims, text_renderer) = win.renderer.split_mut();

      render_ui(
        &s.ui, 
        prims, 
        text_renderer, 
        ctx.device, 
        ctx.queue, 
        Some(&s.interaction)
      );
    })
    .run();


  Ok(())
}
