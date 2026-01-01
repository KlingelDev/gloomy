//! RON UI Example - Load UI from a RON file.
//!
//! Press 'q' or Escape to quit.

use gloomy_app::{
    compute_layout, hit_test, load_ui, render_ui, GloomyApp, InteractionState, Vec2,
};

fn main() -> anyhow::Result<()> {
  env_logger::init();

  // Interaction state
  let state = std::sync::Arc::new(std::sync::Mutex::new(InteractionState::new()));
  let state_clone = state.clone();
  let state_clone2 = state.clone();

  // Load UI definition from RON file
  let mut ui = load_ui("examples/ui/dashboard.ron")?;

  GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state.lock().unwrap();
        s.update_mouse(Vec2::new(x, y));
    })
    .on_mouse_input(move |_win, el_state, button| {
        use winit::event::{ElementState, MouseButton};
        if button == MouseButton::Left {
            let mut s = state_clone.lock().unwrap();
            s.set_pressed(el_state == ElementState::Pressed);
            if el_state == ElementState::Released {
                 if let Some(action) = &s.active_action {
                     // Check if still hovering same widget
                     if s.is_hovered(action) {
                         println!("ACTION TRIGGERED: {}", action);
                         s.triggered_action = Some(action.clone());
                     }
                 }
            }
        }
    })
    .on_draw(move |window, ctx| {
      let mut s = state_clone2.lock().unwrap();
      
      let size = window.renderer.size();
      let (w, h) = (size.x, size.y);

      // Compute layout
      compute_layout(&mut ui, 0.0, 0.0, w, h);

      // Hit testing
      let hit = hit_test(&ui, s.mouse_pos, Some(&s.scroll_offsets));
      s.hovered_action = hit.map(|res| res.action.to_string());

      if s.is_pressed && s.active_action.is_none() {
          s.active_action = s.hovered_action.clone();
      }

      // Split renderer to get both primitives and text
      let (primitives, text) = window.renderer.split_mut();

      // Render the loaded UI
      render_ui(&ui, primitives, text, ctx.device, ctx.queue, Some(&s));
    })
    .run()
}
