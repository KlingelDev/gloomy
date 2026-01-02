/// Theme Switcher Example
///
/// Demonstrates:
/// - Loading themes from RON files
/// - Live theme switching at runtime
/// - Using StyleContext in the render loop
/// - Applying theme colors to widgets
///
/// Run with: cargo run --example theme_switcher

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, handle_interactions, hit_test},
    widget::{Widget, WidgetBounds},
    interaction::InteractionState,
    theme::Theme,
    style::GlobalStyle,
    style_context::StyleContext,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::ElementState;

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        ui_root: Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
            id: Some("root".to_string()),
            scrollable: false,
            bounds: WidgetBounds::default(),
            width: Some(800.0),
            height: Some(600.0),
            background: None,
            border: None,
            corner_radius: 0.0,
            shadow: None,
            gradient: None,
            padding: 0.0,
            layout: Layout::default(),
            flex: 0.0,
            grid_col: None,
            grid_row: None,
            col_span: 1,
            row_span: 1,
            corner_radii: None,
            children: vec![],
        },
        interaction: InteractionState::default(),
        style_ctx: StyleContext::default(),
        current_theme_index: 0,
    }));
    
    // Build initial UI
    state.borrow_mut().rebuild_ui();
    
    let state_move = state.clone();
    let state_input = state.clone();
    let state_draw = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_cursor_move(move |win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x as f32, y as f32);
            s.interaction.update_mouse(pos);
            
            let scroll_offsets = s.interaction.scroll_offsets.clone();
            if let Some(res) = hit_test(&s.ui_root, pos, Some(&scroll_offsets)) {
                s.interaction.hovered_action = Some(res.action.to_string());
            } else {
                s.interaction.hovered_action = None;
            }
            
            win.window.request_redraw();
        })
        .on_mouse_input(move |win, elem_state, _btn| {
            let mut s = state_input.borrow_mut();
            
            if elem_state == ElementState::Pressed {
                s.interaction.set_pressed(true);
                
                let mouse_pos = s.interaction.mouse_pos;
                let scroll_offsets = s.interaction.scroll_offsets.clone();
                
                if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                    let action = res.action.to_string();
                    s.interaction.set_active(Some(action.clone()));
                }
            } else {
                if let Some(action) = s.interaction.active_action.clone() {
                    let mouse_pos = s.interaction.mouse_pos;
                    let scroll_offsets = s.interaction.scroll_offsets.clone();
                    
                    if let Some(res) = hit_test(&s.ui_root, mouse_pos, Some(&scroll_offsets)) {
                        if res.action == action {
                            s.handle_click(&action);
                            s.rebuild_ui();
                        }
                    }
                }
                
                s.interaction.set_pressed(false);
                s.interaction.set_active(None);
            }
            
            win.window.request_redraw();
        })
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
    style_ctx: StyleContext,
    current_theme_index: usize,
}

impl AppState {
    fn handle_click(&mut self, action: &str) {
        match action {
            "switch_theme" => {
                self.switch_theme();
            }
            _ => println!("Unknown action: {}", action),
        }
    }
    
    fn switch_theme(&mut self) {
        let themes = vec![
            ("Dark", Theme::dark()),
            ("Light", Theme::light()),
            ("High Contrast", Theme::high_contrast()),
        ];
        
        self.current_theme_index = (self.current_theme_index + 1) % themes.len();
        let (name, theme) = &themes[self.current_theme_index];
        
        self.style_ctx.set_theme(theme.clone());
        println!("Switched to {} theme", name);
    }
    
    fn rebuild_ui(&mut self) {
        let style = &self.style_ctx.global_style;
        let theme = &self.style_ctx.theme;
        
        // Get theme colors
        let bg = theme.colors.background;
        let surface = theme.colors.surface;
        let text = theme.colors.text;
        let text_secondary = theme.colors.text_secondary;
        let primary = theme.colors.primary;
        let success = theme.colors.success;
        let warning = theme.colors.warning;
        let error = theme.colors.error;
        
        self.ui_root = Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
            id: Some("root".to_string()),
            scrollable: false,
            bounds: WidgetBounds::default(),
            width: Some(800.0),
            height: Some(600.0),
            background: Some(bg),
            border: None,
            corner_radius: 0.0,
            shadow: None,
            gradient: None,
            padding: style.spacing_large,
            layout: Layout {
                direction: Direction::Column,
                spacing: style.spacing_medium,
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
                    text: format!("Theme Switcher - Current: {}", theme.name),
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    size: style.font_size_heading,
                    color: text,
                    text_align: Default::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    font: None,
                },
                
                // Description
                Widget::Label {
                    text: "Click the button below to cycle through themes".to_string(),
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                    size: style.font_size_normal,
                    color: text_secondary,
                    text_align: Default::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    font: None,
                },
                
                // Switch button
                Widget::Button {
                    text: "Switch Theme".to_string(),
                    action: "switch_theme".to_string(),
                    bounds: WidgetBounds {
                        x: 0.0,
                        y: 0.0,
                        width: 200.0,
                        height: 50.0,
                    },
                    background: primary,
                    hover_color: (primary.0 * 1.2, primary.1 * 1.2, primary.2 * 1.2, primary.3),
                    active_color: (primary.0 * 0.8, primary.1 * 0.8, primary.2 * 0.8, primary.3),
                    border: None,
                    corner_radius: style.corner_radius_medium,
                    shadow: style.shadow_small.clone(),
                    gradient: None,
                    corner_radii: None,
                    layout: Layout::default(),
                    flex: 0.0,
                    grid_col: None,
                    grid_row: None,
                    col_span: 1,
                    row_span: 1,
                    font: None,
                },
                
                // Color palette display
                Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                    id: Some("palette".to_string()),
                    scrollable: false,
                    bounds: WidgetBounds::default(),
                    width: None,
                    height: Some(150.0),
                    background: Some(surface),
                    border: None,
                    corner_radius: style.corner_radius_medium,
                    shadow: style.shadow_small.clone(),
                    gradient: None,
                    padding: style.spacing_medium,
                    layout: Layout {
                        direction: Direction::Column,
                        spacing: style.spacing_small,
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
                            text: "Semantic Colors:".to_string(),
                            x: 0.0,
                            y: 0.0,
                            width: 0.0,
                            height: 0.0,
                            size: style.font_size_large,
                            color: text,
                            text_align: Default::default(),
                            flex: 0.0,
                            grid_col: None,
                            grid_row: None,
                            col_span: 1,
                            row_span: 1,
                            font: None,
                        },
                        create_color_box("Success", success, style),
                        create_color_box("Warning", warning, style),
                        create_color_box("Error", error, style),
                    ],
                },
            ],
        };
    }
}

fn create_color_box(label: &str, color: (f32, f32, f32, f32), style: &GlobalStyle) -> Widget {
    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
        id: None,
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: Some(30.0),
        background: Some(color),
        border: None,
        corner_radius: style.corner_radius_small,
        shadow: None,
        gradient: None,
        padding: style.spacing_small,
        layout: Layout::default(),
        flex: 0.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        corner_radii: None,
        children: vec![
            Widget::Label {
                text: label.to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
                size: style.font_size_normal,
                color: (1.0, 1.0, 1.0, 1.0),
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
