use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::{load_ui, render_ui, hit_test},
    layout_engine::compute_layout,
    interaction::InteractionState,
    Vec2, 
    widget::{Widget, WidgetBounds},
    style::{BoxStyle, ButtonStyle, Border, ListViewStyle},
    layout::{Layout, Direction, Align},
    data_source::{VecDataSource, CellValue, MapDataProvider, DataProvider},
};
use std::{rc::Rc, cell::RefCell};

// Navigation States
#[derive(Debug, Clone, Copy, PartialEq)]
enum Page {
    Dashboard,
    Inputs,
    AllWidgets,
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
        Page::AllWidgets => "examples/ui/showcase_all.ron",
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
                items: vec!["Dashboard".to_string(), "Inputs".to_string(), "All Widgets".to_string()],
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

fn create_activity_log() -> VecDataSource {
    let columns = vec![
        "Time".to_string(),
        "Event".to_string(),
        "Status".to_string(),
        "User".to_string(),
    ];
    
    let rows = vec![
        vec![CellValue::Text("09:45 AM".to_string()), CellValue::Text("User Login".to_string()), CellValue::Text("Success".to_string()), CellValue::Text("karl_d".to_string())],
        vec![CellValue::Text("10:12 AM".to_string()), CellValue::Text("Update Profile".to_string()), CellValue::Text("Completed".to_string()), CellValue::Text("karl_d".to_string())],
        vec![CellValue::Text("10:30 AM".to_string()), CellValue::Text("Purchase Item #123".to_string()), CellValue::Text("Processing".to_string()), CellValue::Text("alice_99".to_string())],
        vec![CellValue::Text("11:05 AM".to_string()), CellValue::Text("Logout".to_string()), CellValue::Text("Success".to_string()), CellValue::Text("bob_builder".to_string())],
        vec![CellValue::Text("12:15 PM".to_string()), CellValue::Text("System Alert".to_string()), CellValue::Text("Warning".to_string()), CellValue::Text("SYSTEM".to_string())],
        vec![CellValue::Text("01:20 PM".to_string()), CellValue::Text("New Registration".to_string()), CellValue::Text("Pending".to_string()), CellValue::Text("new_user_1".to_string())],
        vec![CellValue::Text("02:00 PM".to_string()), CellValue::Text("Database Backup".to_string()), CellValue::Text("Success".to_string()), CellValue::Text("SYSTEM".to_string())],
    ];
    
    VecDataSource::new(columns, rows)
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Initial State
    let root = Widget::Container {
        id: Some("root".to_string()),
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
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
            create_sidebar(),
            load_page(Page::Dashboard),
        ],
        scrollable: false,
        padding: 0.0,
        flex: 0.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        layout_cache: None,
        render_cache: RefCell::new(None),
    };
    
    let root_widget = Rc::new(RefCell::new(root));
    let current_page = Rc::new(RefCell::new(Page::Dashboard));

    // Interaction state
    let interaction = Rc::new(RefCell::new(InteractionState::new()));

    // Data Provider
    let mut provider = MapDataProvider::new();
    provider.register("activity_log", create_activity_log());
    let provider = Rc::new(RefCell::new(provider));
    
    let interact_move = interaction.clone();
    let interact_click = interaction.clone();
    let interact_draw = interaction.clone();
    
    let root_click = root_widget.clone();
    let root_draw = root_widget.clone();
    let page_state = current_page.clone();

    println!("Starting Gloomy Showcase...");

    GloomyApp::new()
        .with_title("Gloomy UI Showcase")
        .with_size(1280, 800)
        .on_cursor_move(move |_win, x, y| {
            interact_move.borrow_mut().update_mouse(Vec2::new(x, y));
        })
        .on_mouse_input(move |_win, state, btn| {
             if btn == winit::event::MouseButton::Left {
                let pressed = state == winit::event::ElementState::Pressed;
                interact_click.borrow_mut().set_pressed(pressed);
                
                if pressed {
                    let pos = interact_click.borrow().mouse_pos;
                    let action = {
                        let root = root_click.borrow();
                        let interact = interact_click.borrow();
                        hit_test(&*root, pos, Some(&*interact)).map(|h| h.action.clone())
                    };
                    
                    if let Some(act) = action {
                        interact_click.borrow_mut().clicked_id = Some(act);
                    }
                }
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
                    "menu:2" => (Some(Page::AllWidgets), 2),
                    _ => (None, 0),
                };
                
                if let Some(np) = new_page {
                    if *p != np {
                        *p = np;
                        println!("Switched to {:?}", np);
                        
                        let mut root = root_draw.borrow_mut();
                        if let Widget::Container { children, .. } = &mut *root {
                            // Swap Content
                            if children.len() >= 2 {
                                children[1] = load_page(np);
                            }
                            
                            // Update sidebar selection
                            if let Some(sidebar) = children.get_mut(0) {
                                if let Widget::Container { children: sb_children, .. } = sidebar {
                                     // Assuming ListView is at index 2 (Label, Spacer, ListView)
                                     if let Some(Widget::ListView { selected_index, .. }) = sb_children.get_mut(2) {
                                         *selected_index = Some(idx);
                                     }
                                }
                            }
                        }
                    }
                }
            }
            
            // Update Layout
            {
                let mut root = root_draw.borrow_mut();
                if let Widget::Container { bounds, width: w, height: h, .. } = &mut *root {
                    *w = Some(width);
                    *h = Some(height);
                    bounds.width = width;
                    bounds.height = height;
                }
                compute_layout(&mut *root, 0.0, 0.0, width, height);
            }

            // Render
            render_ui(
                &*root_draw.borrow(),
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&interact),
                Some(&*provider.borrow() as &dyn DataProvider)
            );
            
            // Reset click
            interact.clicked_id = None;
        })
        .run()
}
