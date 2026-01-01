use gloomy_app::GloomyApp;
use gloomy_core::widget::{Widget, Shadow, Gradient, TextAlign}; // Added imports for struct usage if needed, though they are inside RON string mostly.
use gloomy_core::ui::load_ui;

struct AppState {
    counter: i32,
    ui_root: Widget,
}

impl AppState {
    fn new() -> Self {
        // We will load from RON, but for simplicity let's define programmatically if RON fails or just use RON.
        // Let's use RON to test deserialization of new fields.
        let ui_root = load_ui("examples/ui/visuals_demo.ron").unwrap_or_else(|e| {
            eprintln!("Failed to load UI: {}", e);
            Widget::label("Failed to load details")
        });
        
        Self {
            counter: 0,
            ui_root,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let app = GloomyApp::new();
    
    // Create assets dir if not exists (checked in previous steps)
    // Write RON file first
    std::fs::create_dir_all("examples/ui")?;
    std::fs::write("examples/ui/visuals_demo.ron", r#"
        Container(
            layout: (direction: Column, spacing: 10.0),
            background: Some((0.15, 0.15, 0.18, 1.0)),
            padding: 20.0,
            children: [
                Label(text: "Visual Polish Demo", size: 24.0, flex: 0.0, padding: 10.0),
                Spacer(size: 20.0),
                
                // Shadow Demo
                Container(
                    width: Some(300.0),
                    height: Some(100.0),
                    background: Some((0.2, 0.2, 0.25, 1.0)),
                    corner_radius: 8.0,
                    shadow: Some(Shadow(
                        offset: (0.0, 10.0),
                        blur: 20.0,
                        color: (0.0, 0.0, 0.0, 0.5)
                    )),
                    children: [
                        Label(text: "Container with Drop Shadow", size: 18.0, text_align: Center)
                    ]
                ),
                
                Spacer(size: 40.0),
                
                // Gradient Button
                Button(
                    text: "Gradient Button",
                    action: "grad_btn",
                    bounds: (width: 200.0, height: 50.0),
                    corner_radius: 25.0,
                    gradient: Some(Gradient(
                        start: (0.0, 0.4, 0.8, 1.0), // Blue
                        end: (0.0, 0.2, 0.5, 1.0)    // Dark Blue
                    )),
                    shadow: Some(Shadow(
                        offset: (0.0, 5.0),
                        blur: 10.0,
                        color: (0.0, 0.0, 0.0, 0.4)
                    ))
                ),
                
                Spacer(size: 40.0),
                
                 // Soft Border / Glow
                Container(
                    width: Some(300.0),
                    height: Some(100.0),
                    background: Some((0.1, 0.1, 0.1, 1.0)),
                    border: Some(Border(
                        width: 2.0,
                        color: (1.0, 0.5, 0.0, 1.0),
                        style: Solid
                    )),
                    corner_radius: 12.0,
                    // Simulate glow with shadow?
                    shadow: Some(Shadow(
                        offset: (0.0, 0.0),
                        blur: 15.0,
                        color: (1.0, 0.5, 0.0, 0.3)
                    )),
                    children: [
                         Label(text: "Glow Effect (Shadow + Border)", size: 16.0, text_align: Center)
                    ]
                ),
            ]
        )
    "#)?;

    let app_state = std::rc::Rc::new(std::cell::RefCell::new(AppState::new()));
    let state_draw = app_state.clone();

    app.on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            // Recompute layout
            let width = win.config.width as f32;
            let height = win.config.height as f32;
            
            gloomy_core::layout_engine::compute_layout(
                &mut s.ui_root,
                0.0,
                0.0,
                width,
                height
            );
            
            // Render
            gloomy_core::ui::render_ui(
                &s.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                None,
            );
        })
        .run()
}
