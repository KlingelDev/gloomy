use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::render_ui,
    widget::{
        Border, BorderStyle, Gradient, Shadow, TextInputStyle,
        Widget, WidgetBounds, TextAlign,
    },
    layout::{Layout, Direction, Align, Justify},
    layout_engine::compute_layout,
    interaction::InteractionState,
    Vec2,
};
use std::{rc::Rc, cell::RefCell};

struct AppState {
    interaction: InteractionState,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
    }));

    let state_draw = state.clone();
    let state_move = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
             state_move.borrow_mut().interaction.update_mouse(Vec2::new(x as f32, y as f32));
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            let (w, h) = (win.config.width as f32, win.config.height as f32);
            
            // Define UI
            let mut ui = Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                id: Some("root".to_string()),
                scrollable: true,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: h },
                width: Some(w),
                height: Some(h),
                background: Some((0.1, 0.1, 0.12, 1.0)),
                border: None,
                gradient: None,
                shadow: None,
                corner_radius: 0.0,
                corner_radii: None,
                padding: 20.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                layout: Layout {
                    direction: Direction::Column,
                    spacing: 30.0,
                    align_items: Align::Start, // Start to prevent left-cutoff on small screens
                    ..Default::default()
                },
                children: vec![
                    Widget::label("Advanced Visuals Showcase"),
                    
                    // --- 1. Gradient Borders ---
                    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                         id: None,
                         scrollable: false,
                         bounds: WidgetBounds::default(),
                         width: Some(400.0),
                         height: Some(80.0),
                         background: Some((0.15, 0.15, 0.2, 1.0)),
                         border: Some(Border {
                             width: 4.0,
                             color: (1.0, 1.0, 1.0, 1.0), // Fallback
                             gradient: Some(Gradient {
                                 start: (1.0, 0.0, 0.5, 1.0), // Pink
                                 end: (0.0, 1.0, 1.0, 1.0),   // Cyan
                             }),
                             style: BorderStyle::Solid,
                             ..Default::default()
                         }),
                         gradient: None,
                         shadow: None,
                         corner_radius: 16.0,
                         corner_radii: None,
                         padding: 0.0,
                         flex: 0.0, 
                         grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                         layout: Layout { justify_content: Justify::Center, align_items: Align::Center, ..Default::default() },
                         children: vec![{
                             let mut w = Widget::label("Gradient Border (4px)");
                             if let Widget::Label { text_align, .. } = &mut w {
                                 *text_align = TextAlign::Center;
                             }
                             w
                         }],
                    },

                    // --- 2. Neon Glow Effect ---
                    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                         id: None,
                         scrollable: false,
                         bounds: WidgetBounds::default(),
                         width: Some(400.0),
                         height: Some(80.0),
                         background: Some((0.1, 0.1, 0.1, 1.0)),
                         border: Some(Border {
                             width: 2.0,
                             color: (1.0, 0.6, 0.0, 1.0), // Orange
                             gradient: None,
                             style: BorderStyle::Solid,
                             ..Default::default()
                         }),
                         gradient: None,
                         shadow: Some(Shadow {
                             offset: (0.0, 0.0),
                             blur: 20.0,
                             color: (1.0, 0.6, 0.0, 0.6).into(),
                         }),
                         corner_radius: 8.0,
                         corner_radii: None,
                         padding: 0.0,
                         flex: 0.0, 
                         grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                         layout: Layout { justify_content: Justify::Center, align_items: Align::Center, ..Default::default() },
                         children: vec![{
                             let mut w = Widget::label("Neon Glow (Border + Shadow)");
                             if let Widget::Label { text_align, .. } = &mut w {
                                 *text_align = TextAlign::Center;
                             }
                             w
                         }],
                    },

                     // --- 3. Custom Styled Input ---
                    Widget::TextInput {
                        id: "styled_input".to_string(),
                        value: "Custom Input Style".to_string(),
                        placeholder: "Type here...".to_string(),
                        font_size: 16.0,
                        text_align: TextAlign::Left,
                        bounds: WidgetBounds { x: 0.0, y: 0.0, width: 400.0, height: 40.0 },
                        width: 400.0,
                        height: 40.0,
                        flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                        style: TextInputStyle {
                            background: Some((0.05, 0.05, 0.05, 1.0).into()),
                            border: Some(Border {
                                width: 1.0,
                                color: (0.4, 0.4, 0.4, 1.0).into(),
                                ..Default::default()
                            }),
                            border_focused: Some(Border {
                                width: 2.0,
                                color: (0.2, 0.8, 0.2, 1.0).into(), // Green focus
                                gradient: None,
                                style: BorderStyle::Solid,
                                ..Default::default()
                            }),
                            // selection_color: (0.2, 0.8, 0.2, 0.3).into(),
                            ..Default::default()
                        }
                    },
                    
                    // --- 4. Gradient Button with Shadow ---
                     Widget::Button {
                        text: "Gradient Button".to_string(),
                        action: "btn".to_string(),
                        bounds: WidgetBounds { x: 0.0, y: 0.0, width: 250.0, height: 50.0 }, 
                        background: (0.0, 0.0, 0.0, 0.0), // Transparent base
                        hover_color: (1.0, 1.0, 1.0, 0.1),
                        active_color: (1.0, 1.0, 1.0, 0.2),
                        border: Some(Border {
                            width: 2.0,
                            color: (1.0, 1.0, 1.0, 0.3).into(),
                            ..Default::default()
                        }),
                        gradient: Some(Gradient {
                             start: (0.4, 0.0, 0.8, 1.0), // Purple
                             end: (0.8, 0.0, 0.4, 1.0),   // Pink
                        }),
                        shadow: Some(Shadow { 
                            offset: (0.0, 4.0), 
                            blur: 12.0, 
                            color: (0.0,0.0,0.0,0.6).into() 
                        }),
                        corner_radius: 25.0,
                        corner_radii: None,
                        layout: Layout::default(),
                        flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                ]
            };
            
            compute_layout(&mut ui, 0.0, 0.0, w, h);
            
            render_ui(
                &ui,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction)
            );
        })
        .run()
}
