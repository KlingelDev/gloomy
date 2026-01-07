use gloomy_app::GloomyApp;
use gloomy_core::{
    ui::{render_ui, hit_test},
    widget::{
        Widget, WidgetBounds, TabItem, TabStyle, Orientation, TextAlign,
    },
    style::{TextInputStyle, ButtonStyle},
    datagrid::{ColumnDef, ColumnWidth, SelectionMode, DataGridStyle},

    kpi::{KpiCardStyle, KpiTrend, TrendDirection},

    layout::{Layout, Direction, Align, Justify},
    layout_engine::compute_layout,
    interaction::InteractionState,
    Vec2,
};
use std::{rc::Rc, cell::RefCell};
use winit::event::{ElementState, MouseButton};

struct AppState {
    interaction: InteractionState,
    orientation: Orientation,
    selected_tab: usize,
    input_text: String,
    // DataGrid state
    selected_rows: std::collections::HashSet<usize>,
    // Persist UI for hit testing
    ui: Option<Widget>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        orientation: Orientation::Horizontal,
        selected_tab: 0,
        input_text: String::new(),
        selected_rows: std::collections::HashSet::new(),
        ui: None,
    }));

    let state_draw = state.clone();
    let state_move = state.clone();
    let state_click = state.clone();

    GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
             state_move.borrow_mut().interaction.update_mouse(Vec2::new(x as f32, y as f32));
        })
        .on_mouse_input(move |_win, element_state, button| {
            if element_state == ElementState::Pressed && button == MouseButton::Left {
                let mut s = state_click.borrow_mut();
                if let Some(ui) = &s.ui {
                    if let Some(res) = hit_test(ui, s.interaction.mouse_pos, Some(&s.interaction)) {
                         s.interaction.clicked_id = Some(res.action);
                    }
                }
            }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            let (w, h) = (win.config.width as f32, win.config.height as f32);
            
            // --- Interaction Handling ---
            let clicked = s.interaction.clicked_id.clone();
            if let Some(id) = &clicked {
                if id.starts_with("main_tabs:tab:") {
                    if let Ok(idx) = id["main_tabs:tab:".len()..].parse::<usize>() {
                        s.selected_tab = idx;
                    }
                }
                if id == "toggle_orient" {
                    s.orientation = match s.orientation {
                        Orientation::Horizontal => Orientation::Vertical,
                        Orientation::Vertical => Orientation::Horizontal,
                    };
                    println!("Toggled Orientation to: {:?}", s.orientation);
                }
                // Handle DataGrid selection
                if id.starts_with("dg1:cell:") {
                     let parts: Vec<&str> = id.split(':').collect();
                     if parts.len() >= 4 {
                         if let Ok(row) = parts[2].parse::<usize>() {
                             // Single selection logic
                             s.selected_rows.clear();
                             s.selected_rows.insert(row);
                             println!("Selected DataGrid Row: {}", row);
                         }
                     }
                }
            }
            s.interaction.clicked_id = None;

            // --- Page Content Construction ---
            
            // 1. DataGrid Page
            let page_datagrid = Widget::Container {
                id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, style: Default::default(), padding: 10.0, layout: Layout { direction: Direction::Column, align_items: Align::Stretch, spacing: 10.0, ..Default::default() }, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                    Widget::label("DataGrid Example"),
                    Widget::DataGrid {
                        id: Some("dg1".to_string()),
                        columns: vec![
                            ColumnDef::new("ID", "id").width(ColumnWidth::Fixed(50.0)),
                            ColumnDef::new("Name", "name"),
                            ColumnDef::new("Value", "value").width(ColumnWidth::Fixed(100.0)),
                        ],
                        data_source_id: None,
                        style: DataGridStyle::default(),
                        bounds: WidgetBounds::default(), flex: 1.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                        // Defaults for remaining fields
                        header_height: 40.0,
                        row_height: 32.0,
                        striped: true,

                        selection_mode: SelectionMode::Single,
                        show_vertical_lines: true,
                        show_horizontal_lines: true,
                        selected_rows: s.selected_rows.iter().cloned().collect(),
                        sort_column: None,
                        sort_direction: None,
                    }


                ], 
                layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };
            
            // 2. Form Page
            let page_form = Widget::Container {
                 id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, style: Default::default(), padding: 20.0, layout: Layout { direction: Direction::Column, align_items: Align::Stretch, spacing: 15.0, ..Default::default() }, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                     Widget::label("User Form"),
                     Widget::TextInput { id: "fname".into(), value: "John".into(), placeholder: "First Name".into(), validation: None, style: TextInputStyle::default(), bounds: WidgetBounds::default(), width: 2.5.into(), height: 0.0.into(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font_size: 14.0, text_align: TextAlign::Left },
                     Widget::TextInput { id: "lname".into(), value: "Doe".into(), placeholder: "Last Name".into(), validation: None, style: TextInputStyle::default(), bounds: WidgetBounds::default(), width: 250.0.into(), height: 0.0.into(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font_size: 14.0, text_align: TextAlign::Left },
                     Widget::Button { text: "Submit".into(), action: "submit".into(), bounds: WidgetBounds::default(), style: ButtonStyle::default(), width: Some(100.0), height: None, disabled: false, layout: Layout::default(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font: None },


                 ],
                 layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };

            // 3. TextInput Demo
            let page_text = Widget::Container {
                 id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, style: Default::default(), padding: 20.0, layout: Layout { direction: Direction::Column, align_items: Align::Stretch, spacing: 10.0, ..Default::default() }, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                     Widget::label("Text Input Demo"),
                     Widget::TextInput { id: "demo_input".into(), value: s.input_text.clone(), placeholder: "Type here...".into(), validation: None, style: TextInputStyle::default(), bounds: WidgetBounds::default(), width: 300.0.into(), height: 0.0.into(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font_size: 14.0, text_align: TextAlign::Left },
                     Widget::label(format!("You typed: {}", s.input_text)),
                 ],
                 layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };

            // 4. Chart Placeholder
            let page_chart = Widget::Container {
                 id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, style: Default::default(), padding: 20.0, layout: Layout { direction: Direction::Column, align_items: Align::Stretch, spacing: 10.0, ..Default::default() }, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                     Widget::label("Analytics Chart"),
                     Widget::KpiCard {
                         id: Some("kpi1".into()),
                         title: "Revenue".into(),
                         value: "$12,450".into(),
                         trend: Some(KpiTrend { value: "+12%".into(), direction: TrendDirection::Up }),
                         style: KpiCardStyle::default(),
                         bounds: WidgetBounds { width: 200.0, height: 120.0, ..Default::default() },
                         flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1
                     },
                     // Colored rect as chart placeholder
                     Widget::Container {
                         id: None, scrollable: false, bounds: WidgetBounds { width: 400.0, height: 200.0, ..Default::default() }, width: Some(400.0), height: Some(200.0), 
                         style: Default::default(), // Need to set background color here
                         // Check BoxStyle: background is Option<Color>
                         // Wait, in widget.rs style: BoxStyle.
                         padding: 0.0, layout: Layout::default(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![],
                         layout_cache: None, render_cache: std::cell::RefCell::new(None)
                     }
                 ],
                 layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };

            // 5. Static Label
            let page_label = Widget::Container {
                 id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: None, style: Default::default(), padding: 20.0, layout: Layout::default(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                     Widget::label("Just a simple label page."),
                 ],
                 layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };

            // --- Root UI ---
            let mut ui = Widget::Container {
                id: Some("root".into()),
                scrollable: false,
                bounds: WidgetBounds { x: 0.0, y: 0.0, width: w, height: h },
                width: Some(w), height: Some(h),
                style: Default::default(),
                padding: 10.0,
                layout: Layout { direction: Direction::Column, align_items: Align::Stretch, spacing: 10.0, ..Default::default() },
                flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                children: vec![
                    // Top Bar
                    Widget::Container {
                        id: None, scrollable: false, bounds: WidgetBounds::default(), width: None, height: Some(40.0), style: Default::default(), padding: 0.0, layout: Layout { direction: Direction::Row, align_items: Align::Center, justify_content: Justify::SpaceBetween, ..Default::default() }, flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, children: vec![
                             Widget::label("Tab Component Showcase"),
                             Widget::Button { text: "Toggle Orientation".into(), action: "toggle_orient".into(), bounds: WidgetBounds::default(), style: ButtonStyle::default(), width: None, height: None, disabled: false, layout: Layout::default(), flex: 0.0, grid_col: None, grid_row: None, col_span: 1, row_span: 1, font: None }
                        ],
                        layout_cache: None, render_cache: std::cell::RefCell::new(None)
                    },
                    // Tab Widget
                    Widget::tab(
                        "main_tabs",
                        vec![
                            TabItem { title: "DataGrid".into(), content: Box::new(page_datagrid) },
                            TabItem { title: "Form".into(), content: Box::new(page_form) },
                            TabItem { title: "Text".into(), content: Box::new(page_text) },
                            TabItem { title: "Chart".into(), content: Box::new(page_chart) },
                            TabItem { title: "Label".into(), content: Box::new(page_label) },
                        ],
                        s.orientation,
                        TabStyle::default()
                    ) // removed with_selected since it doesn't exist on enum
                ],
                layout_cache: None, render_cache: std::cell::RefCell::new(None)
            };



            // Post-construction modifications
            if let Widget::Container { children, .. } = &mut ui {
                 if let Some(child) = children.get_mut(1) { // 2nd child is Tab
                     if let Widget::Tab { selected, flex, .. } = child {
                         *selected = s.selected_tab;
                         *flex = 1.0; // Ensure it fills the screen!
                     }
                 }
            }
            
            // Layout & Render
            compute_layout(&mut ui, 0.0, 0.0, w, h);


            
            // Persist UI
            s.ui = Some(ui.clone());

            render_ui(
                s.ui.as_ref().unwrap(),
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
                None
            );
        })
        .run()
}


