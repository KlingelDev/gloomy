use gloomy_app::GloomyApp;
use gloomy_core::{
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions, load_ui},
    widget::Widget,
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use std::time::Instant;
use std::path::Path;
use winit::event::ElementState;

fn generate_assets() -> anyhow::Result<()> {
    std::fs::create_dir_all("examples/assets")?;
    
    // User Icon (Blue circle)
    let path_user = Path::new("examples/assets/icon_user.png");
    if !path_user.exists() {
        let mut img = image::ImageBuffer::new(64, 64);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let dx = x as f32 - 32.0;
            let dy = y as f32 - 32.0;
            if dx*dx + dy*dy < 30.0*30.0 {
                *pixel = image::Rgba([50u8, 100u8, 200u8, 255u8]);
            } else {
                *pixel = image::Rgba([0u8, 0u8, 0u8, 0u8]);
            }
        }
        img.save(path_user)?;
    }
    
    // Settings Icon (Gray Box)
    let path_settings = Path::new("examples/assets/icon_settings.png");
    if !path_settings.exists() {
        let mut img = image::ImageBuffer::new(64, 64);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            if x > 10 && x < 54 && y > 10 && y < 54 {
                 *pixel = image::Rgba([150u8, 150u8, 150u8, 255u8]);
            } else {
                 *pixel = image::Rgba([0u8, 0u8, 0u8, 0u8]);
            }
        }
        img.save(path_settings)?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    generate_assets()?;

    // Load widget tree from file
    let widget = load_ui("examples/ui/table_layout.ron")?;
    
    let state = Rc::new(RefCell::new(AppState {
        root: widget,
        interaction: InteractionState::default(),
        last_update: Instant::now(),
    }));

    let state_clone_move = state.clone();
    let state_clone_input = state.clone();
    let state_clone_draw = state.clone();
    
    // Keyboard copy from widgets_ui for 'q' to quit
    // Not critical for layout demo but nice to have.
    
    GloomyApp::new()
    .on_cursor_move(move |_win, x, y| {
        let mut s = state_clone_move.borrow_mut();
        s.interaction.update_mouse(Vec2::new(x as f32, y as f32));
    })
    .on_mouse_input(move |_win, state_elem, _btn| {
        let mut s = state_clone_input.borrow_mut();
        if state_elem == ElementState::Pressed {
             s.interaction.set_pressed(true);
        } else {
             s.interaction.set_pressed(false);
        }
    })
    .on_draw(move |win, ctx| {
        let mut s = state_clone_draw.borrow_mut();
        let now = Instant::now();
        let _dt = now.duration_since(s.last_update).as_secs_f32();
        s.last_update = now;
        
        let width = win.config.width as f32;
        let height = win.config.height as f32;

        // Ensure root fills window
        if let Widget::Container { bounds, .. } = &mut s.root {
             bounds.x = 0.0;
             bounds.y = 0.0;
             bounds.width = width;
             bounds.height = height;
        }

        compute_layout(&mut s.root, 0.0, 0.0, width, height);

        let iter_copy = s.interaction.clone();
        handle_interactions(&mut s.root, &iter_copy, Vec2::ZERO);
        
        // Use new render_ui signature
        render_ui(
            &s.root, 
            &mut win.renderer, 
            ctx.device, 
            ctx.queue, 
            Some(&s.interaction),
            None
        );
    })
    .run()?;

    Ok(())
}

struct AppState {
    root: Widget,
    interaction: InteractionState,
    last_update: Instant,
}
