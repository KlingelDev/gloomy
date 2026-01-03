use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::{load_ui, render_ui},
    layout_engine::compute_layout,
    interaction::InteractionState,
    Vec2, 
    widget::{Widget, WidgetBounds},
    style::{BoxStyle, ButtonStyle, Border, ListViewStyle},
    layout::{Layout, Direction, Align},
};
use std::{rc::Rc, cell::RefCell};

// Navigation States
#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Dashboard,
    Inputs,
}

// Helpers for widget construction since struct update syntax from enum doesn't work
fn make_spacer(size: f32) -> Widget {
    Widget::Spacer {
        size,
        flex: 0.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
    }
}

fn make_label(text: &str, size: f32) -> Widget {
    Widget::Label {
        text: text.to_string(),
        size,
        color: (1.0, 1.0, 1.0, 1.0),
        text_align: glob::widget::TextAlign::Center,
        x: 0.0, y: 0.0, width: 0.0, height: 0.0,
        flex: 0.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        font: None,
    }
}

fn make_button(text: &str, action: &str) -> Widget {
    Widget::Button {
        text: text.to_string(),
        action: action.to_string(),
        height: Some(40.0),
        width: None,
        style: ButtonStyle::default(),
        bounds: WidgetBounds::default(),
        disabled: false,
        layout: Layout::default(),
        flex: 0.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        font: None,
    }
}

fn load_page(page: Page) -> Widget {
    let path = match page {
        Page::Dashboard => "examples/ui/showcase_dashboard.ron",
        Page::Inputs => "examples/ui/showcase_inputs.ron",
    };
    println!("Loading page: {}", path);
    load_ui(path).expect("Failed to load page")
}

fn create_sidebar() -> Widget {
    // Manually create sidebar widget
    Widget::Container {
        id: Some("sidebar".to_string()),
        width: Some(250.0),
        height: None, 
        layout: Layout {
            direction: Direction::Column,
            spacing: 10.0,
            align_items: Align::Stretch,
            ..Default::default()
        },
        style: BoxStyle {
            background: Some((0.12, 0.12, 0.14, 1.0)),
            border: Some(Border { width: 1.0, color: (0.2, 0.2, 0.25, 1.0), radius: [0.0; 4] }),
            ..BoxStyle::default()
        },
        children: vec![
            make_label("Showcase", 24.0),
            make_spacer(30.0),
            Widget::ListView {
                id: "menu".to_string(),
                items: vec!["Dashboard".to_string(), "Inputs".to_string()],
                selected_index: Some(0), // Will be updated by state
                style: ListViewStyle::default(),
                width: None, // Auto width (fill parent due to Align::Stretch)
                height: None, // Auto height
                bounds: WidgetBounds::default(),
                layout: Layout::default(),
                flex: 0.0,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
            }
        ],
        bounds: WidgetBounds::default(),
        scrollable: false,
        padding: 20.0,
        flex: 0.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        layout_cache: None,
        render_cache: RefCell::new(None),
    }
}


mod glob {
    pub use gloomy_core::widget;
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Initial State
    let current_page = Rc::new(RefCell::new(Page::Dashboard));
    let content_widget = Rc::new(RefCell::new(load_page(Page::Dashboard)));
    
    let sidebar_widget = Rc::new(RefCell::new(create_sidebar())); // Sidebar is now stateful (selection)

    // Interaction state
    let interaction = Rc::new(RefCell::new(InteractionState::new()));
    
    let interact_move = interaction.clone();
    let interact_click = interaction.clone();
    let interact_draw = interaction.clone();
    
    let page_state = current_page.clone();
    let content_state = content_widget.clone();

    println!("Starting Gloomy Showcase...");

    GloomyApp::new()
        .with_title("Gloomy UI Showcase")
        .with_size(1280, 800)
        .on_cursor_move(move |_win, x, y| {
            interact_move.borrow_mut().update_mouse(Vec2::new(x, y));
        })
        .on_mouse_input(move |_win, state, btn| {
             if btn == winit::event::MouseButton::Left {
                interact_click.borrow_mut().set_pressed(state == winit::event::ElementState::Pressed);
            }
        })
        .on_draw(move |win, ctx| {
            let width = win.config.width as f32;
            let height = win.config.height as f32;
            
            let mut interact = interact_draw.borrow_mut();
            
            // Handle Navigation Logic
            if let Some(clicked) = &interact.clicked_id {
                let mut p = page_state.borrow_mut();
                let (new_page, idx) = match clicked.as_str() {
                    "menu:0" => (Some(Page::Dashboard), 0),
                    "menu:1" => (Some(Page::Inputs), 1),
                    _ => (None, 0),
                };
                
                if let Some(np) = new_page {
                    if *p != np {
                        *p = np;
                        *content_state.borrow_mut() = load_page(np);
                        println!("Switched to {:?}", np);
                        
                        // Update sidebar selection
                        let mut sb = sidebar_widget.borrow_mut();
                        if let Widget::Container { children, .. } = &mut *sb {
                            if let Some(Widget::ListView { selected_index, .. }) = children.get_mut(2) {
                                *selected_index = Some(idx);
                            }
                        }
                    }
                }
            }
            
            let mut content = content_state.borrow_mut();
            
            let mut root = Widget::Container {
                id: Some("root".to_string()),
                bounds: WidgetBounds { x: 0.0, y: 0.0, width, height },
                width: Some(width),
                height: Some(height),
                layout: Layout {
                    direction: Direction::Row,
                    align_items: Align::Stretch,
                    ..Default::default()
                },
                style: BoxStyle {
                    background: Some((0.05, 0.05, 0.08, 1.0)),
                    ..Default::default()
                },
                children: vec![
                    sidebar_widget.borrow().clone(),
                    content.clone(), 
                ],
                scrollable: false,
                padding: 0.0,
                flex: 0.0,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                layout_cache: None,
                render_cache: RefCell::new(None),
            };
            
            // 1. Compute Layout
            compute_layout(&mut root, 0.0, 0.0, width, height);

            // 2. Render
            render_ui(
                &root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&interact),
                None
            );
            
            // 3. Reset click
            interact.clicked_id = None;
        })
        .run()
}
