use gloomy_app::GloomyApp;
use gloomy_core::{Widget, InteractionState};
use winit::keyboard::{Key, NamedKey};
use std::fs;
use std::path::Path;

struct AppState {
    interaction: InteractionState,
}

impl AppState {
    fn new() -> Self {
        Self {
            interaction: InteractionState::default(),
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Load UI from file
    let run_path = std::env::current_dir()?;
    let asset_path = run_path.join("examples/assets/dashboard.ron");
    println!("Loading UI from: {:?}", asset_path);
    
    let ron_content = fs::read_to_string(&asset_path)
        .or_else(|_| fs::read_to_string("assets/dashboard.ron")) // Try local assets if running from examples
        .expect("Failed to read dashboard.ron");
        
    let mut ui_root = gloomy_core::parse_ui(&ron_content)
        .expect("Failed to parse dashboard.ron");
        
    let mut state = AppState::new();
    
    // Shared state for callbacks
    let state = std::rc::Rc::new(std::cell::RefCell::new(state));
    let state_draw = state.clone();
    let state_mouse = state.clone();
    let state_cursor = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            state_cursor.borrow_mut().interaction.mouse_pos = gloomy_core::Vec2::new(x, y);
        })
        .on_mouse_input(move |_win, state, _btn| {
             state_mouse.borrow_mut().interaction.is_pressed = state == winit::event::ElementState::Pressed;
        })
        .on_draw(move |window, ctx| {
            let width = window.window.inner_size().width as f32 / window.renderer.scale_factor;
            let height = window.window.inner_size().height as f32 / window.renderer.scale_factor;
            
            if let Widget::Container { bounds, .. } = &mut ui_root {
                bounds.width = width;
                bounds.height = height;
            }

            // Re-layout (could be optimized to only run on resize/content change)
            gloomy_core::compute_layout(&mut ui_root, 0.0, 0.0, width, height);
            
            let interaction = state_draw.borrow().interaction.clone();
            
            gloomy_core::render_ui(
                &ui_root,
                &mut window.renderer,
                &ctx.device,
                &ctx.queue,
                Some(&interaction),
                None, 
            );
        })
        .on_keyboard_input(move |win, event| {
            if event.state == winit::event::ElementState::Pressed {
                if let Key::Named(NamedKey::Escape) = event.logical_key {
                     // println!("Escape"); 
                }
            }
        })
        .run()
}
