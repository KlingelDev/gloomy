use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::render_ui,
    widget::{
        Widget, WidgetBounds, ToggleSwitchStyle, ProgressBarStyle, RadioButtonStyle, DropdownStyle,
    },
    layout::{Layout, Direction, Align},
    layout_engine::compute_layout,
    interaction::InteractionState,
    Vec2,
};
use std::{rc::Rc, cell::RefCell};

struct AppState {
    interaction: InteractionState,
    toggle_1: bool,
    toggle_2: bool,
    progress: f32,
    radio_sel: String,
    dropdown_idx: Option<usize>,
    dropdown_expanded: bool,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        toggle_1: false,
        toggle_2: true,
        progress: 0.3,
        radio_sel: "opt1".to_string(),
        dropdown_idx: Some(0),
        dropdown_expanded: false,
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
            
            // --- Update Logic ---
            if s.interaction.clicked_id.as_deref() == Some("toggle_1") {
                s.toggle_1 = !s.toggle_1;
            }
            if s.interaction.clicked_id.as_deref() == Some("toggle_2") {
                s.toggle_2 = !s.toggle_2;
            }
            if s.interaction.clicked_id.as_deref() == Some("opt1") {
                s.radio_sel = "opt1".to_string();
            }
            if s.interaction.clicked_id.as_deref() == Some("opt2") {
                s.radio_sel = "opt2".to_string();
            }
            if s.interaction.clicked_id.as_deref() == Some("dd1") {
                s.dropdown_expanded = !s.dropdown_expanded;
            }
            let clicked_id = s.interaction.clicked_id.clone(); 
            if let Some(clicked) = &clicked_id {
                let clicked_str: &str = clicked; 
                if clicked_str.starts_with("select_dd1_") {
                    if let Ok(idx) = clicked_str["select_dd1_".len()..].parse::<usize>() {
                        s.dropdown_idx = Some(idx);
                        s.dropdown_expanded = false; 
                    }
                }
            }
            s.progress = (s.progress + 0.005) % 1.0;
            s.interaction.clicked_id = None;

            // --- UI Definition ---
            let header_lbl = {
                let mut l = Widget::label("Widgets Showcase");
                if let Widget::Label { size, .. } = &mut l { *size = 24.0; }
                l
            };

            let mut ui = Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                id: Some("root".to_string()),
                scrollable: true,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: h }, 
                width: Some(w), height: Some(h),
                background: Some((0.1, 0.1, 0.12, 1.0)),
                border: None, corner_radius: 0.0, shadow: None, gradient: None, padding: 20.0,
                corner_radii: None,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1, flex: 0.0,
                layout: Layout {
                    direction: Direction::Column,
                    spacing: 20.0,
                    align_items: Align::Start,
                    ..Default::default()
                },
                children: vec![
                    header_lbl,
                    
                    // --- Toggles ---
                    Widget::label("1. Toggle Switches"),
                    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                        layout: Layout { direction: Direction::Row, spacing: 20.0, align_items: Align::Center, ..Default::default() },
                        children: vec![
                            Widget::ToggleSwitch {
                                id: "toggle_1".to_string(),
                                checked: s.toggle_1,
                                style: ToggleSwitchStyle { width: 50.0, ..Default::default() },
                                bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                            },
                            Widget::label(if s.toggle_1 { "On" } else { "Off" }),
                            
                            Widget::ToggleSwitch {
                                id: "toggle_2".to_string(),
                                checked: s.toggle_2,
                                style: ToggleSwitchStyle { 
                                    width: 60.0, 
                                    thumb_color: Some((1.0, 0.5, 0.5, 1.0)),
                                    track_color_on: Some((0.8, 0.2, 0.2, 1.0)),
                                    ..Default::default() 
                                },
                                bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                            },
                        ],
                         id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, background: None, border: None, corner_radius: 0.0, shadow: None, gradient: None, padding: 0.0, corner_radii: None, grid_col: None, grid_row: None, col_span: 1, row_span: 1, flex: 0.0,
                    },
                    
                    // --- Progress Bar ---
                    Widget::label("2. Progress Bar"),
                    Widget::ProgressBar {
                        value: s.progress,
                        min: 0.0, max: 1.0,
                        style: ProgressBarStyle {
                            fill_color: Some((0.3, 0.6, 1.0, 1.0)),
                            background_color: Some((0.2, 0.2, 0.2, 1.0)),
                            corner_radius: 4.0,
                            ..Default::default()
                        },
                        width: Some(400.0),
                        height: Some(10.0),
                        bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                    
                    // --- Radio Buttons ---
                    Widget::label("3. Radio Buttons"),
                    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
                        layout: Layout { direction: Direction::Row, spacing: 10.0, align_items: Align::Center, ..Default::default() },
                        children: vec![
                             Widget::RadioButton {
                                 style: RadioButtonStyle { size: 20.0, ..Default::default() },
                                 group_id: "g1".to_string(),
                                 value: "opt1".to_string(),
                                 selected: s.radio_sel == "opt1",
                                 label: "Option 1".to_string(),
                                 bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                                 grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                             },
                             Widget::label("Option 1"),
                             
                             Widget::RadioButton {
                                 style: RadioButtonStyle { size: 20.0, ..Default::default() },
                                 group_id: "g1".to_string(),
                                 value: "opt2".to_string(),
                                 selected: s.radio_sel == "opt2",
                                 label: "Option 2".to_string(),
                                 bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                                 grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                             },
                             Widget::label("Option 2"),
                        ],
                         id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, background: None, border: None, corner_radius: 0.0, shadow: None, gradient: None, padding: 0.0, corner_radii: None, grid_col: None, grid_row: None, col_span: 1, row_span: 1, flex: 0.0,
                    },
                    
                    // --- Dropdown ---
                    Widget::label("4. Dropdown"),
                    Widget::Dropdown {
                        id: "dd1".to_string(),
                        options: vec!["Item One".to_string(), "Item Two".to_string(), "Item Three".to_string()],
                        selected_index: s.dropdown_idx,
                        expanded: s.dropdown_expanded,
                        style: DropdownStyle { ..Default::default() },
                        width: Some(200.0),
                        height: Some(36.0),
                        bounds: WidgetBounds::default(), layout: Layout::default(), flex: 0.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                    },
                    
                ],
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
