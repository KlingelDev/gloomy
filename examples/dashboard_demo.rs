use gloomy_app::GloomyApp;
use gloomy_core::{
    Align, Container, Direction, GloomyRenderer, InteractionState, Justify, Layout,
    RenderContext, Widget, WidgetBounds, KpiCard, KpiCardStyle, KpiTrend, TrendDirection, BoxStyle,
};
use winit::keyboard::{Key, NamedKey};
use std::cell::RefCell;

struct AppState {
    interaction: InteractionState,
}

impl AppState {
    fn new() -> Self {
        Self {
            interaction: InteractionState::default(),
        }
    }
}

fn create_kpi_style(positive: bool) -> KpiCardStyle {
    let trend_color = if positive {
         (0.3, 0.8, 0.4, 1.0)
    } else {
         (0.9, 0.3, 0.3, 1.0)
    };
    
    KpiCardStyle {
        background: (0.18, 0.18, 0.20, 1.0),
        border_color: (0.3, 0.3, 0.3, 1.0),
        border_width: 1.0,
        corner_radius: 8.0,
        label_color: (0.7, 0.7, 0.75, 1.0),
        label_size: 14.0,
        value_color: (1.0, 1.0, 1.0, 1.0),
        value_size: 28.0,
        trend_up_color: trend_color,
        trend_down_color: trend_color,
        trend_neutral_color: (0.6, 0.6, 0.6, 1.0),
    }
}

// ... create_ui ...

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let mut state = AppState::new();
    let mut ui_root = create_ui();
    
    // Shared state for callbacks
    let state = std::rc::Rc::new(std::cell::RefCell::new(state));
    let state_draw = state.clone();
    let state_mouse = state.clone();
    let state_cursor = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            state_cursor.borrow_mut().interaction.mouse_pos = gloomy_core::Vec2::new(x, y);
        })
        .on_mouse_input(move |_win, state, _btn| {
             state_mouse.borrow_mut().interaction.is_pressed = state == winit::event::ElementState::Pressed;
        })
        .on_draw(move |window, ctx| {
            let width = window.window.inner_size().width as f32 / window.renderer.scale_factor;
            let height = window.window.inner_size().height as f32 / window.renderer.scale_factor;
            
            if let Widget::Container { bounds, .. } = &mut ui_root {
                bounds.width = width;
                bounds.height = height;
            }

            gloomy_core::compute_layout(&mut ui_root, 0.0, 0.0, width, height);
            
            let interaction = state_draw.borrow().interaction.clone();
            
            gloomy_core::render_ui(
                &ui_root,
                &mut window.renderer,
                &ctx.device,
                &ctx.queue,
                Some(&interaction),
                None, 
            );
        })
        .on_keyboard_input(move |win, event| {
            if event.state == winit::event::ElementState::Pressed {
                if let Key::Named(NamedKey::Escape) = event.logical_key {
                     // app exit handling usually done by closing window or return loop
                }
            }
        })
        .run()
}

fn create_ui() -> Widget {
    Widget::Container {
        id: Some("root".to_string()),
        scrollable: true,
        bounds: WidgetBounds::default(),
        style: BoxStyle::fill((0.12, 0.12, 0.14, 1.0)),
        padding: 30.0,
        layout: Layout {
            direction: Direction::Column,
            justify_content: Justify::Start,
            align_items: Align::Stretch,
            spacing: 20.0,
            ..Default::default()
        },
        flex: 1.0,
        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
        corner_radii: None, layout_cache: None, render_cache: RefCell::new(None),
        children: vec![
            Widget::Label {
                text: "Analytics Dashboard".to_string(),
                x: 0.0, y: 0.0,
                width: 300.0, height: 40.0, size: 24.0,
                color: (1.0, 1.0, 1.0, 1.0),
                text_align: gloomy_core::widget::TextAlign::Left,
                flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font: None,
            },
            
            // KPI Grid
            Widget::Container {
                id: None, scrollable: false,
                bounds: WidgetBounds::default(),
                width: None, height: None,
                width: None, height: None,
                style: BoxStyle::default(),
                padding: 0.0,
                layout: Layout {
                    direction: Direction::Grid { columns: 3 },
                    justify_content: Justify::Start,
                    align_items: Align::Start,
                    spacing: 20.0,
                    template_columns: vec![
                        gloomy_core::layout::TrackSize::Fr(1.0),
                        gloomy_core::layout::TrackSize::Fr(1.0),
                        gloomy_core::layout::TrackSize::Fr(1.0)
                    ],
                    ..Default::default()
                },
                flex: 0.0,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                corner_radii: None, layout_cache: None, render_cache: RefCell::new(None),
                children: vec![
                    Widget::KpiCard {
                        id: Some("kpi1".to_string()),
                        title: "Total Revenue".to_string(),
                        value: "$124,592".to_string(),
                        trend: Some(KpiTrend { direction: TrendDirection::Up, value: "+12.5%".to_string() }),
                        style: create_kpi_style(true),
                        bounds: WidgetBounds { width: 0.0, height: 120.0, ..Default::default() },
                        flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                    Widget::KpiCard {
                        id: Some("kpi2".to_string()),
                        title: "Active Users".to_string(),
                        value: "8,942".to_string(),
                        trend: Some(KpiTrend { direction: TrendDirection::Down, value: "-2.1%".to_string() }),
                        style: create_kpi_style(false),
                        bounds: WidgetBounds { width: 0.0, height: 120.0, ..Default::default() },
                        flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                    Widget::KpiCard {
                        id: Some("kpi3".to_string()),
                        title: "Avg Session".to_string(),
                        value: "4m 32s".to_string(),
                        trend: Some(KpiTrend { direction: TrendDirection::Neutral, value: "0.0%".to_string() }),
                        style: create_kpi_style(true),
                        bounds: WidgetBounds { width: 0.0, height: 120.0, ..Default::default() },
                        flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                ],
            },
        ],
    }
}

