use gloomy_app::GloomyApp;
use gloomy_core::{
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions, hit_test, load_ui},
    widget::Widget,
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use std::time::Instant;
use winit::event::ElementState;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Load widget tree from file
    let widget = load_ui("examples/ui/widgets_demo.ron")?;
    
    let state = Rc::new(RefCell::new(AppState {
        root: widget,
        interaction: InteractionState::default(),
        last_update: Instant::now(),
    }));

    let state_clone_move = state.clone();
    let state_clone_input = state.clone();
    let state_clone_draw = state.clone();

    GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state_clone_move.borrow_mut();
        s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
    })
    .on_mouse_input(move |_win, state_elem, _btn| {
        let mut s = state_clone_input.borrow_mut();
        if state_elem == ElementState::Pressed {
            s.interaction.set_pressed(true);
            
            let mouse_pos = s.interaction.mouse_pos;
            let scroll_offsets = s.interaction.scroll_offsets.clone();
            
            if let Some(res) = hit_test(&s.root, mouse_pos, Some(&scroll_offsets)) {
                let id = res.action.to_string();
                s.interaction.set_active(Some(id.clone()));
                s.interaction.set_clicked(Some(id));
            } else {
                 s.interaction.set_active(None);
                 s.interaction.set_clicked(None);
            }
        } else {
            s.interaction.set_pressed(false);
            s.interaction.set_active(None);
            s.interaction.set_clicked(None);
        }
    })
    .on_draw(move |win, ctx| {
        let mut s = state_clone_draw.borrow_mut();
        let now = Instant::now();
        let _dt = now.duration_since(s.last_update).as_secs_f32();
        s.last_update = now;
        
        let width = win.config.width as f32;
        let height = win.config.height as f32;

        compute_layout(&mut s.root, 0.0, 0.0, width, height);

        let iter_copy = s.interaction.clone();
        handle_interactions(&mut s.root, &iter_copy, Vec2::ZERO);
        
        render_ui(
            &s.root, 
            &mut win.renderer, 
            ctx.device, 
            ctx.queue, 
            Some(&s.interaction)
        );
        
        s.interaction.set_clicked(None);
    })
    .run()?;

    Ok(())
}

struct AppState {
    root: Widget,
    interaction: InteractionState,
    last_update: Instant,
}
