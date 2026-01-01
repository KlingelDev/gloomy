use gloomy_app::{GloomyApp, GloomyWindow};
use gloomy_core::ui::{load_ui, render_ui, hit_test};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::Widget;
use winit::event::{ElementState, MouseButton, MouseScrollDelta};
use gloomy_core::Vec2;
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    ui: Widget,
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
  env_logger::init();
  
  let ui = load_ui("examples/ui/scroll_demo.ron")?;
  
  let state = Rc::new(RefCell::new(AppState {
      ui,
      interaction: InteractionState::new(),
  }));

  let state_clone = state.clone();
  let state_clone2 = state.clone();
  let state_clone3 = state.clone();
  let state_clone_scroll = state.clone();

  GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state_clone.borrow_mut();
        s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
    })
    .on_mouse_input(move |_win, state, _button| {
        let mut s = state_clone2.borrow_mut();
        s.interaction.set_pressed(state == ElementState::Pressed);
        
        if state == ElementState::Pressed {
             let hit_action = hit_test(&s.ui, s.interaction.mouse_pos, Some(&s.interaction.scroll_offsets))
                .map(|h| h.action.to_string());

             if let Some(action) = hit_action {
                 if action == "reset" {
                     s.interaction.scroll_offsets.insert("scroll_view".to_string(), Vec2::ZERO);
                 }
             }
        }
    })
    .on_scroll(move |_win, delta, _phase| {
        let mut s = state_clone_scroll.borrow_mut();
        
        // Simple logic: scroll "scroll_view" if it exists
        // In real app, check hit test or hovered element
        if let Some(offset) = s.interaction.scroll_offsets.get_mut("scroll_view") {
             // Treat delta y
             let d = match delta {
                 MouseScrollDelta::LineDelta(_x, y) => y * 30.0,
                 MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
             };
             
             offset.y -= d;
             // Ensure not negative
             if offset.y < 0.0 { offset.y = 0.0; }
             // Max scroll? We need content height. 
             // Ideally we get content bounds from layout result stored in widgets.
             // But layout engine modifies widget bounds.
             // We can traverse to find "scroll_view" and check children total height?
             // For now, let it scroll infinitely.
        } else {
             // Init
             s.interaction.scroll_offsets.insert("scroll_view".to_string(), Vec2::ZERO);
        }
    })
    .on_draw(move |win, ctx| {
      let mut s = state_clone3.borrow_mut();
      
      let width = win.config.width as f32;
      let height = win.config.height as f32;
      
      // Layout
      compute_layout(&mut s.ui, 0.0, 0.0, width, height);
      
      let (prims, text) = win.renderer.split_mut();
      
      // Clear before drawing? win.render does clear.
      // We assume render_ui just adds to primitives.

      render_ui(&s.ui, &mut win.renderer, ctx.device, ctx.queue, Some(&s.interaction));
    })
    .run();

  Ok(())
}
