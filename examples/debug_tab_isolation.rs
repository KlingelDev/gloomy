use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::render_ui,
    widget::{Widget, WidgetBounds, TabItem, TabStyle, Orientation, ButtonStyle},
    ui::{hit_test},
    layout::{Layout, Direction, Align},
    interaction::InteractionState,
    layout_engine::compute_layout,
    Vec2,
};
use std::{rc::Rc, cell::RefCell};
use winit::event::{ElementState, MouseButton};

struct AppState {
    interaction: InteractionState,
    orientation: Orientation,
    // Store the UI widget tree to allow hit testing against it
    ui: Option<Widget>, 
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        orientation: Orientation::Horizontal,
        ui: None,
    }));

    let state_draw = state.clone();
    let state_move = state.clone();
    let state_click = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
             state_move.borrow_mut().interaction.update_mouse(Vec2::new(x as f32, y as f32));
        })
        .on_mouse_input(move |_win, element_state, button| {
            if element_state == ElementState::Pressed && button == MouseButton::Left {
                let mut s = state_click.borrow_mut();
                if let Some(ui) = &s.ui {
                    if let Some(res) = hit_test(ui, s.interaction.mouse_pos, Some(&s.interaction)) {
                        println!("Hit: {:?}", res);
                        // Store the clicked ID so the draw loop can process it
                        s.interaction.clicked_id = Some(res.action);
                    }
                }
            }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            let (w, h) = (win.config.width as f32, win.config.height as f32);
            
            // Interaction Logic
            let clicked = s.interaction.clicked_id.clone();
            if let Some(id) = &clicked {
                if id == "btn_toggle" {
                    println!("Toggle clicked! Current: {:?}", s.orientation);
                    s.orientation = match s.orientation {
                        Orientation::Horizontal => Orientation::Vertical,
                        Orientation::Vertical => Orientation::Horizontal,
                    };
                    println!("New: {:?}", s.orientation);
                }
            }
            s.interaction.clicked_id = None;

            // UI Construction
            let mut ui = Widget::Container {
                id: Some("root".into()),
                scrollable: false,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: h },
                width: Some(w), height: Some(h),
                style: Default::default(),
                padding: 10.0,
                layout: Layout { direction: Direction::Column, spacing: 10.0, ..Default::default() },
                flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                children: vec![
                    Widget::Button { 
                        text: format!("Toggle Orientation (Current: {:?})", s.orientation), 
                        action: "btn_toggle".into(), 
                        bounds: WidgetBounds::default(), 
                        style: ButtonStyle::default(), 
                        width: None, height: Some(40.0), 
                        disabled: false, layout: Layout::default(), flex: 0.0, 
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1, font: None 
                    },
                    Widget::tab(
                        "debug_tabs",
                        vec![
                            TabItem { 
                                title: "Tab A".into(), 
                                content: Box::new(Widget::Container {
                                    id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, 
                                    style: Default::default(), padding: 20.0, layout: Layout::default(), flex: 0.0, 
                                    grid_col: None, grid_row: None, col_span: 1, row_span: 1, 
                                    children: vec![Widget::label("Content A")],
                                    layout_cache: None, render_cache: std::cell::RefCell::new(None)
                                })
                            },
                            TabItem { 
                                title: "Tab B".into(), 
                                content: Box::new(Widget::Container {
                                    id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, 
                                    style: Default::default(), padding: 20.0, layout: Layout::default(), flex: 0.0, 
                                    grid_col: None, grid_row: None, col_span: 1, row_span: 1, 
                                    children: vec![Widget::label("Content B")],
                                    layout_cache: None, render_cache: std::cell::RefCell::new(None)
                                })
                            },
                        ],
                        s.orientation,
                        TabStyle::default()
                    )
                ],
                layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };

            compute_layout(&mut ui, 0.0, 0.0, w, h);

            // Persist UI for hit testing next frame
            // Note: In a real app we might optimize this to avoid cloning if possible, 
            // or separate the model from the view widget tree.
            s.ui = Some(ui.clone());
            
            render_ui(
                s.ui.as_ref().unwrap(),
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
                None
            );
        })
        .run()
}
