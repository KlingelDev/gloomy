/// Scrollbar Widget Example
///
/// Demonstrates:
/// - Vertical scrollbar with scrollable content
/// - Horizontal scrollbar
/// - Dynamic thumb sizing based on content size
/// - Thumb position based on scroll offset
/// - ScrollbarStyle customization
///
/// Run with: cargo run --example scrollbar_demo

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions},
    widget::{Widget, WidgetBounds, Orientation, ScrollbarStyle},
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
            handle_interactions(
                &mut s.ui_root,
                &interaction_copy,
                Vec2::ZERO
            );
            
            render_ui(
                &s.ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
            );
        })
        .run()
}

struct AppState {
    ui_root: Widget,
    interaction: InteractionState,
    vertical_scroll: f32,
    horizontal_scroll: f32,
}

impl AppState {
    fn new() -> Self {
        Self {
            ui_root: create_ui(0.0, 0.0),
            interaction: InteractionState::default(),
            vertical_scroll: 0.0,
            horizontal_scroll: 0.0,
        }
    }
}

fn create_ui(vertical_scroll: f32, horizontal_scroll: f32) -> Widget {
    // Simulated content sizes
    let content_height = 1000.0;
    let content_width = 1500.0;
    let viewport_height = 400.0;
    let viewport_width = 600.0;
    
    Widget::Container {
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
            spacing: 20.0,
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
                text: "Scrollbar Widget Demo".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: 28.0,
                color: (0.9, 0.9, 0.9, 1.0),
                text_align: Default::default(),
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Vertical scrollbar section
            Widget::Container {
                id: Some("vertical_section".to_string()),
                scrollable: false,
                bounds: WidgetBounds::default(),
                width: None,
                height: Some(viewport_height + 40.0),
                background: Some((0.15, 0.15, 0.15, 1.0)),
                border: None,
                corner_radius: 8.0,
                shadow: None,
                gradient: None,
                padding: 20.0,
                layout: Layout {
                    direction: Direction::Column,
                    spacing: 10.0,
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
                        text: "Vertical Scrollbar".to_string(),
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                        size: 18.0,
                        color: (0.8, 0.8, 0.8, 1.0),
                        text_align: Default::default(),
                        flex: 0.0,
                        grid_col: None,
                        grid_row: None,
                        col_span: 1,
                        row_span: 1,
                        font: None,
                    },
                    
                    // Scrollable area container
                    Widget::Container {
                        id: Some("scrollable".to_string()),
                        scrollable: false,
                        bounds: WidgetBounds::default(),
                        width: None,
                        height: Some(viewport_height),
                        background: Some((0.1, 0.1, 0.1, 1.0)),
                        border: None,
                        corner_radius: 4.0,
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
                            // Content area
                            Widget::Container {
                                id: Some("content".to_string()),
                                scrollable: false,
                                bounds: WidgetBounds::default(),
                                width: None,
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
                                flex: 1.0,
                                grid_col: None,
                                grid_row: None,
                                col_span: 1,
                                row_span: 1,
                                corner_radii: None,
                                children: vec![
                                    Widget::Label {
                                        text: format!(
                                            "Scrollable Content Area\n\n\
                                            Content Height: {:.0}px\n\
                                            Viewport Height: {:.0}px\n\
                                            Scroll Offset: {:.0}px\n\n\
                                            The scrollbar thumb size represents\n\
                                            the viewport-to-content ratio.\n\n\
                                            Thumb position shows current scroll.",
                                            content_height,
                                            viewport_height,
                                            vertical_scroll
                                        ),
                                        x: 0.0,
                                        y: 0.0,
                                        width: 0.0,
                                        height: 0.0,
                                        size: 14.0,
                                        color: (0.7, 0.7, 0.7, 1.0),
                                        text_align: Default::default(),
                                        flex: 0.0,
                                        grid_col: None,
                                        grid_row: None,
                                        col_span: 1,
                                        row_span: 1,
                                        font: None,
                                    },
                                ],
                            },
                            
                            // Vertical scrollbar
                            Widget::Scrollbar {
                                bounds: WidgetBounds::default(),
                                content_size: content_height,
                                viewport_size: viewport_height,
                                scroll_offset: vertical_scroll,
                                orientation: Orientation::Vertical,
                                style: ScrollbarStyle {
                                    track_color: (0.08, 0.08, 0.08, 1.0),
                                    thumb_color: (0.35, 0.35, 0.35, 1.0),
                                    thumb_hover_color: (0.45, 0.45, 0.45, 1.0),
                                    width: 14.0,
                                    corner_radius: 7.0,
                                },
                                flex: 0.0,
                                grid_col: None,
                                grid_row: None,
                                col_span: 1,
                                row_span: 1,
                            },
                        ],
                    },
                ],
            },
            
            // Horizontal scrollbar section
            Widget::Container {
                id: Some("horizontal_section".to_string()),
                scrollable: false,
                bounds: WidgetBounds::default(),
                width: None,
                height: Some(150.0),
                background: Some((0.15, 0.15, 0.15, 1.0)),
                border: None,
                corner_radius: 8.0,
                shadow: None,
                gradient: None,
                padding: 20.0,
                layout: Layout {
                    direction: Direction::Column,
                    spacing: 10.0,
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
                        text: "Horizontal Scrollbar".to_string(),
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                        size: 18.0,
                        color: (0.8, 0.8, 0.8, 1.0),
                        text_align: Default::default(),
                        flex: 0.0,
                        grid_col: None,
                        grid_row: None,
                        col_span: 1,
                        row_span: 1,
                        font: None,
                    },
                    
                    // Horizontal scrollbar container
                    Widget::Container {
                        id: Some("horizontal_area".to_string()),
                        scrollable: false,
                        bounds: WidgetBounds::default(),
                        width: Some(viewport_width),
                        height: Some(60.0),
                        background: Some((0.1, 0.1, 0.1, 1.0)),
                        border: None,
                        corner_radius: 4.0,
                        shadow: None,
                        gradient: None,
                        padding: 10.0,
                        layout: Layout {
                            direction: Direction::Column,
                            spacing: 5.0,
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
                                text: format!(
                                    "Content Width: {:.0}px | Viewport: {:.0}px | Scroll: {:.0}px",
                                    content_width,
                                    viewport_width,
                                    horizontal_scroll
                                ),
                                x: 0.0,
                                y: 0.0,
                                width: 0.0,
                                height: 0.0,
                                size: 12.0,
                                color: (0.7, 0.7, 0.7, 1.0),
                                text_align: Default::default(),
                                flex: 1.0,
                                grid_col: None,
                                grid_row: None,
                                col_span: 1,
                                row_span: 1,
                                font: None,
                            },
                            
                            // Horizontal scrollbar
                            Widget::Scrollbar {
                                bounds: WidgetBounds::default(),
                                content_size: content_width,
                                viewport_size: viewport_width,
                                scroll_offset: horizontal_scroll,
                                orientation: Orientation::Horizontal,
                                style: ScrollbarStyle::default(),
                                flex: 0.0,
                                grid_col: None,
                                grid_row: None,
                                col_span: 1,
                                row_span: 1,
                            },
                        ],
                    },
                ],
            },
        ],
    }
}
