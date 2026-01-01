use gloomy_app::{GloomyApp, GloomyWindow};
use gloomy_core::ui::{load_ui, render_ui, hit_test};
use gloomy_core::layout_engine::compute_layout;
use gloomy_core::interaction::InteractionState;
use gloomy_core::widget::Widget;
use winit::event::{ElementState, MouseButton};
use gloomy_core::Vec2;
use std::rc::Rc;
use std::cell::RefCell;

struct AppState {
    ui: Widget,
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
  env_logger::init();
  
  let ui = load_ui("examples/ui/style_demo.ron")?;
  
  let state = Rc::new(RefCell::new(AppState {
      ui,
      interaction: InteractionState::new(),
  }));

  let state_clone = state.clone();
  let state_clone2 = state.clone();
  let state_clone3 = state.clone();

  GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state_clone.borrow_mut();
        s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
    })
    .on_mouse_input(move |_win, state, _button| {
        let mut s = state_clone2.borrow_mut();
        s.interaction.set_pressed(state == ElementState::Pressed);
        
        let mut focused = None;
        if state == ElementState::Pressed {
             let hit = hit_test(&s.ui, s.interaction.mouse_pos, Some(&s.interaction.scroll_offsets));
             if let Some(h) = hit {
                 focused = Some(h.action.to_string());
             }
        }
        
        if let Some(id) = focused {
            s.interaction.focused_id = Some(id);
        }
    })
    .on_draw(move |win, ctx| {
      let mut s = state_clone3.borrow_mut();
      
      let width = win.config.width as f32;
      let height = win.config.height as f32;
      
      // Layout
      compute_layout(&mut s.ui, 0.0, 0.0, width, height);
      
      let (prims, text) = win.renderer.split_mut();

      render_ui(&s.ui, &mut win.renderer, ctx.device, ctx.queue, Some(&s.interaction));
    })
    .run();

  Ok(())
}
