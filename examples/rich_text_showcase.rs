/// Rich Text Showcase Example
///
/// Demonstrates the universal rich text system with HTML-like markup
/// working across all widgets: Labels, Buttons, Trees, and DataGrids.
///
/// Supported tags:
/// - <color="#RRGGBB"> or <color="#RRGGBBAA">
/// - <size="N">
/// - <font="FontName">
/// - <bold>, <b>
/// - <italic>, <i>
/// - <underline>, <u>
/// - <span color="#..." size="N" bold italic>
///
/// Run with: cargo run --example rich_text_showcase

use gloomy_core::{
    layout::Layout,
    layout_engine::compute_layout,
    ui::render_ui,
    widget::{Widget, WidgetBounds, TextAlign},
    tree::{TreeNode, TreeStyle},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let initial_ui = create_ui();
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        ui_root: initial_ui,
    }));
    
    println!("Rich Text Showcase Example");
    println!("=========================");
    println!("All text supports HTML-like markup!");
    
    let state_draw = state.clone();
    let state_move = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x, y);
            s.interaction.update_mouse(pos);
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            let window_size = win.window.inner_size();
            let width = window_size.width as f32;
            let height = window_size.height as f32;
            
            if let Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None), bounds, .. } = &mut s.ui_root {
                bounds.width = width;
                bounds.height = height;
            }
            
            compute_layout(&mut s.ui_root, 0.0, 0.0, width, height);
            
            render_ui(
                &s.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
                None,
            );
        })
        .run()
}

struct AppState {
    interaction: InteractionState,
    ui_root: Widget,
}

fn create_ui() -> Widget {
    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
        bounds: WidgetBounds::default(),
        background: Some((0.10, 0.10, 0.12, 1.0)),
        border: None,
        shadow: None,
        gradient: None,
        corner_radius: 0.0,
        corner_radii: None,
        padding: 30.0,
        children: vec![
            // Title
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 40.0,
                text: "<size=\"28\"><bold>Rich Text \
                       Showcase</bold></size>".to_string(),
                size: 24.0,
                color: (0.95, 0.95, 0.97, 1.0),
                font: None,
                text_align: TextAlign::Center,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Color examples
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 30.0,
                text: "<bold>Colors:</bold> \
                       <color=\"#FF0000\">Red</color> \
                       <color=\"#00FF00\">Green</color> \
                       <color=\"#0000FF\">Blue</color> \
                       <color=\"#FF00FF\">Magenta</color>".to_string(),
                size: 16.0,
                color: (0.9, 0.9, 0.92, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Size examples
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 35.0,
                text: "<bold>Sizes:</bold> \
                       <size=\"12\">Small</size> \
                       <size=\"16\">Medium</size> \
                       <size=\"24\">Large</size> \
                       <size=\"32\">Huge</size>".to_string(),
                size: 16.0,
                color: (0.9, 0.9, 0.92, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Style examples
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 30.0,
                text: "<bold>Styles:</bold> \
                       <bold>Bold</bold> \
                       <italic>Italic</italic> \
                       <underline>Underline</underline>".to_string(),
                size: 16.0,
                color: (0.9, 0.9, 0.92, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Nested examples
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 30.0,
                text: "<bold>Nested:</bold> \
                       <bold><color=\"#FF6600\">Bold \
                       Orange</color></bold> \
                       <italic><size=\"18\">Big \
                       Italic</size></italic>".to_string(),
                size: 16.0,
                color: (0.9, 0.9, 0.92, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Span tag examples
            Widget::Label {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 35.0,
                text: "<bold>Combined:</bold> \
                       <span color=\"#00FFFF\" size=\"20\" \
                       bold>Cyan Bold 20px</span>".to_string(),
                size: 16.0,
                color: (0.9, 0.9, 0.92, 1.0),
                font: None,
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Button with rich text
            Widget::Button {
                text: "<size=\"18\"><bold>🚀 \
                       <color=\"#00FF00\">Launch</color> \
                       </bold></size>".to_string(),
                action: "launch".to_string(),
                bounds: WidgetBounds {
                    x: 0.0,
                    y: 0.0,
                    width: 200.0,
                    height: 45.0,
                },
                background: (0.2, 0.3, 0.8, 1.0),
                hover_color: (0.3, 0.4, 0.9, 1.0),
                active_color: (0.1, 0.2, 0.7, 1.0),
                border: None,
                corner_radius: 8.0,
                corner_radii: None,
                shadow: None,
                gradient: None,
                layout: Layout {
                    direction: gloomy_core::layout::Direction::Column,
                    spacing: 0.0,
                    align_items: gloomy_core::layout::Align::Center,
                    justify_content: gloomy_core::layout::Justify::Center,
                    template_columns: vec![],
                },
                font: None,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Tree with rich text
            Widget::Tree {
                id: Some("tree".to_string()),
                bounds: WidgetBounds::default(),
                root_nodes: vec![
                    TreeNode::new(
                        "root1",
                        "<bold>📁</bold> \
                         <color=\"#FFD700\">Important</color>"
                    )
                    .child(
                        TreeNode::new(
                            "file1",
                            "📄 <italic>document.txt</italic>"
                        )
                        .leaf()
                    )
                    .child(
                        TreeNode::new(
                            "file2",
                            "📄 <color=\"#FF0000\"><bold>\
                            urgent.pdf</bold></color>"
                        )
                        .leaf()
                    ),
                ],
                selected_id: None,
                expanded_ids: std::collections::HashSet::new(),
                style: TreeStyle::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
        ],
        layout: Layout {
            direction: gloomy_core::layout::Direction::Column,
            spacing: 15.0,
            align_items: gloomy_core::layout::Align::Start,
            justify_content: gloomy_core::layout::Justify::Start,
            template_columns: vec![],
        },
        width: None,
        height: None,
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        id: None,
        scrollable: false,
    }
}
