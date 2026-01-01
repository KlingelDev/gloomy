/// Text Clipping Demo
///
/// Demonstrates:
/// - Text clipping within label bounds
/// - Overflow text being cut off at boundary
/// - Different sized containers
/// - Long text that would overflow without clipping
///
/// Run with: cargo run --example text_clipping_demo

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions},
    widget::{Widget, WidgetBounds},
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
    let long_text = "This is a very long text that would definitely overflow the container bounds if text clipping was not working properly. It just keeps going and going with no end in sight!";
    let medium_text = "This text is moderately long and should be clipped at the edge of its container.";
    
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
                text: "Text Clipping Demo".to_string(),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 40.0,
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
            
            // Example 1: Wide container, long text
            create_clip_example(
                "Example 1: 600px Wide Container",
                long_text,
                600.0,
                30.0,
                16.0,
                (0.15, 0.15, 0.15, 1.0)
            ),
            
            // Example 2: Narrow container, long text
            create_clip_example(
                "Example 2: 300px Wide Container",
                long_text,
                300.0,
                30.0,
                14.0,
                (0.15, 0.2, 0.15, 1.0)
            ),
            
            // Example 3: Very narrow container
            create_clip_example(
                "Example 3: 150px Wide Container",
                medium_text,
                150.0,
                30.0,
                12.0,
                (0.2, 0.15, 0.15, 1.0)
            ),
            
            // Example 4: Height clipping
            Widget::Container {
                id: None,
                scrollable: false,
                bounds: WidgetBounds::default(),
                width: Some(700.0),
                height: Some(120.0),
                background: Some((0.15, 0.15, 0.2, 1.0)),
                border: None,
                corner_radius: 8.0,
                shadow: None,
                gradient: None,
                padding: 15.0,
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
                        text: "Example 4: Height Clipping (Fixed 30px Height)".to_string(),
                        x: 0.0,
                        y: 0.0,
                        width: 670.0,
                        height: 25.0,
                        size: 14.0,
                        color: (0.6, 0.6, 0.7, 1.0),
                        text_align: Default::default(),
                        flex: 0.0,
                        grid_col: None,
                        grid_row: None,
                        col_span: 1,
                        row_span: 1,
                        font: None,
                    },
                    Widget::Label {
                        text: "This text has a very tall font size (32px) but is clipped to 30px height".to_string(),
                        x: 0.0,
                        y: 0.0,
                        width: 670.0,
                        height: 30.0,  // Smaller than text size!
                        size: 32.0,     // Will be clipped
                        color: (0.9, 0.7, 0.7, 1.0),
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
            
            // Info
            Widget::Label {
                text: "✓ All text is clipped to container/label bounds\n✓ No text overflows beyond boundaries".to_string(),
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 60.0,
                size: 14.0,
                color: (0.5, 0.7, 0.5, 1.0),
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

fn create_clip_example(
    title: &str,
    text: &str,
    width: f32,
    height: f32,
    font_size: f32,
    bg_color: (f32, f32, f32, f32)
) -> Widget {
    Widget::Container {
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(width + 30.0),
        height: Some(height + 50.0),
        background: Some(bg_color),
        border: None,
        corner_radius: 8.0,
        shadow: None,
        gradient: None,
        padding: 15.0,
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
                width: width,
                height: 20.0,
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
            Widget::Label {
                text: text.to_string(),
                x: 0.0,
                y: 0.0,
                width: width,
                height: height,
                size: font_size,
                color: (0.9, 0.9, 0.9, 1.0),
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
