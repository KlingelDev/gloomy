/// Divider Widget Example
///
/// Demonstrates:
/// - Horizontal dividers for separating sections
/// - Vertical dividers for side-by-side content
/// - Configurable thickness, color, and margin
/// - Integration with layouts
///
/// Run with: cargo run --example divider_demo

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions},
    widget::{Widget, WidgetBounds, Orientation},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState::new()));
    let state_draw = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            let size = win.renderer.size();
            compute_layout(&mut s.ui_root, 0.0, 0.0, size.x, size.y);
            
            let interaction_copy = s.interaction.clone();
            handle_interactions(&mut s.ui_root, &interaction_copy, Vec2::ZERO);
            
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
    ui_root: Widget,
    interaction: InteractionState,
}

impl AppState {
    fn new() -> Self {
        Self {
            ui_root: create_ui(),
            interaction: InteractionState::default(),
        }
    }
}

fn create_ui() -> Widget {
    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
        id: Some("root".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(900.0),
        height: Some(700.0),
        background: Some((0.12, 0.12, 0.12, 1.0)),
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        padding: 30.0,
        layout: Layout {
            direction: Direction::Column,
            spacing: 0.0,
            ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        corner_radii: None,
        children: vec![
            // Title
            Widget::Label {
                text: "Divider Widget Demo".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 32.0,
                color: (0.9, 0.9, 0.9, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Horizontal divider
            Widget::Divider {
                bounds: WidgetBounds::default(),
                orientation: Orientation::Horizontal,
                thickness: 2.0,
                color: (0.3, 0.3, 0.3, 1.0),
                margin: 16.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Section 1
            Widget::Label {
                text: "Section 1: Horizontal Dividers".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 20.0,
                color: (0.8, 0.8, 0.8, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            Widget::Label {
                text: "Horizontal dividers are great for separating content vertically.".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 14.0,
                color: (0.6, 0.6, 0.6, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Thin divider
            Widget::Divider {
                bounds: WidgetBounds::default(),
                orientation: Orientation::Horizontal,
                thickness: 1.0,
                color: (0.25, 0.25, 0.25, 1.0),
                margin: 12.0,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Section 2
            Widget::Label {
                text: "Section 2: Vertical Dividers".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 20.0,
                color: (0.8, 0.8, 0.8, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Row container with vertical dividers
            Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                id: Some("row".to_string()),
                scrollable: false,
                bounds: WidgetBounds::default(),
                width: None,
                height: Some(150.0),
                background: Some((0.18, 0.18, 0.18, 1.0)),
                border: None,
                corner_radius: 8.0,
                shadow: None,
                gradient: None,
                padding: 15.0,
                layout: Layout {
                    direction: Direction::Row,
                    spacing: 0.0,
                    ..Default::default()
                },
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                corner_radii: None,
                children: vec![
                    create_text_box("Column 1", "First column of content"),
                    
                    // Vertical divider
                    Widget::Divider {
                        bounds: WidgetBounds::default(),
                        orientation: Orientation::Vertical,
                        thickness: 2.0,
                        color: (0.4, 0.4, 0.4, 1.0),
                        margin: 10.0,
                        flex: 0.0,
                        grid_col: None,
                        grid_row: None,
                        col_span: 1,
                        row_span: 1,
                    },
                    
                    create_text_box("Column 2", "Second column of content"),
                    
                    // Another vertical divider
                    Widget::Divider {
                        bounds: WidgetBounds::default(),
                        orientation: Orientation::Vertical,
                        thickness: 2.0,
                        color: (0.4, 0.4, 0.4, 1.0),
                        margin: 10.0,
                        flex: 0.0,
                        grid_col: None,
                        grid_row: None,
                        col_span: 1,
                        row_span: 1,
                    },
                    
                    create_text_box("Column 3", "Third column of content"),
                ],
            },
        ],
    }
}

fn create_text_box(title: &str, description: &str) -> Widget {
    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(200.0),
        height: None,
        background: None,
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        padding: 10.0,
        layout: Layout {
            direction: Direction::Column,
            spacing: 8.0,
            ..Default::default()
        },
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        corner_radii: None,
        children: vec![
            Widget::Label {
                text: title.to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 16.0,
                color: (0.9, 0.9, 0.9, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            Widget::Label {
                text: description.to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 12.0,
                color: (0.6, 0.6, 0.6, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
        ],
    }
}
