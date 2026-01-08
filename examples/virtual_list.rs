use std::rc::Rc;
use std::cell::RefCell;
use gloomy_app::GloomyApp;
use gloomy_core::{
    widget::{Widget, WidgetBounds},
    style::{ListViewStyle, BoxStyle},
    layout::{Layout, Direction, Align, Justify},
    ui::render_ui,
    compute_layout,
    InteractionState,
    hit_test,
    Vec2,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Generate 10,000 items
    let items: Vec<String> = (0..10_000)
        .map(|i| format!("Item #{}", i))
        .collect();

    let ui = Widget::Container {
        id: Some("root".to_string()),
        bounds: WidgetBounds::default(),
        scrollable: false,
        style: BoxStyle {
            background: Some((0.1, 0.1, 0.12, 1.0)),
            ..Default::default()
        },
        padding: 20.0,
        layout: Layout {
            direction: Direction::Column,
            align_items: Align::Stretch,
            justify_content: Justify::Start,
            spacing: 10.0,
            ..Default::default()
        },
        width: None, height: None, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        children: vec![
            Widget::label("Virtual ListView Demo (10,000 items)"),
            
            // ListView with explicit height
            Widget::ListView {
                id: "my_list".to_string(),
                items,
                selected_index: None,
                style: ListViewStyle {
                    item_height: 30.0,
                    idle: BoxStyle {
                        background: Some((0.15, 0.15, 0.18, 1.0)),
                        ..Default::default()
                    },
                    hover: BoxStyle {
                        background: Some((0.2, 0.2, 0.25, 1.0)),
                        ..Default::default()
                    },
                    selected: BoxStyle {
                        background: Some((0.3, 0.4, 0.6, 1.0)),
                        ..Default::default()
                    },
                    text_color_idle: (0.9, 0.9, 0.9, 1.0),
                    text_color_selected: (1.0, 1.0, 1.0, 1.0),
                },
                bounds: WidgetBounds::default(),
                // Fix height to force scrolling
                width: None,
                height: Some(400.0), 
                layout: Layout::default(),
                flex: 1.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                scroll_offset: 0.0,
            },
            
            Widget::label("Try scrolling the list!"),
        ],
        layout_cache: None,
        render_cache: std::cell::RefCell::new(None),
    };

    // Shared State
    let interaction = Rc::new(RefCell::new(InteractionState::default()));
    let ui_root = Rc::new(RefCell::new(ui));

    let ui_draw = ui_root.clone();
    let int_draw = interaction.clone();
    
    let int_cursor = interaction.clone();
    
    let ui_scroll = ui_root.clone();
    let int_scroll = interaction.clone();

    GloomyApp::new()
        .with_title("Virtual List Demo")
        .with_size(600, 600)
        .on_cursor_move(move |_win, x, y| {
            let mut int = int_cursor.borrow_mut();
            int.update_mouse(Vec2::new(x, y));
        })
        .on_scroll(move |_win, delta, _phase| {
             let mut int = int_scroll.borrow_mut();
             let ui = ui_scroll.borrow();
             
             // Hit test to identify scroll target
             if let Some(hit) = hit_test(&ui, int.mouse_pos, Some(&int)) {
                 let action = hit.action;
                 // Extract ID from action (e.g. "my_list:5" -> "my_list")
                 let id = if let Some(idx) = action.find(':') {
                     &action[0..idx]
                 } else {
                     &action
                 };
                 
                 let (dx, dy) = match delta {
                     winit::event::MouseScrollDelta::LineDelta(x, y) => (x * 30.0, y * 30.0),
                     winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                 };
                 
                 // Pass Vec2 to handle_scroll
                 int.handle_scroll(id, Vec2::new(dx, dy));
             }
        })
        .on_draw(move |win, ctx| {
            let size = win.renderer.size();
            let mut ui = ui_draw.borrow_mut();
            let mut int = int_draw.borrow_mut();
            
            compute_layout(&mut ui, 0.0, 0.0, size.x, size.y);
            
            // Update hit test for hover state
            if let Some(hit) = hit_test(&ui, int.mouse_pos, Some(&int)) {
                int.handle_hit(Some(hit.action));
            } else {
                int.handle_hit(None);
            }

            render_ui(&ui, &mut win.renderer, ctx.device, ctx.queue, Some(&int), None);
        })
        .run()
}