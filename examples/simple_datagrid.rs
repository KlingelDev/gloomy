/// Simple DataGrid Example
///
/// Demonstrates:
/// - Basic DataGrid widget with column definitions
/// - VecDataSource with sample data
/// - Scrolling and Selection interaction
/// - Hit testing and state management
///
/// Run with: cargo run --example simple_datagrid

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, hit_test},
    widget::{Widget, WidgetBounds, TextAlign},
    datagrid::{ColumnDef, ColumnWidth},
    data_source::{VecDataSource, CellValue, MapDataProvider, DataProvider, SortDirection},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use winit::event::{ElementState, MouseButton};
use winit::window::CursorIcon;

struct AppState {
    interaction: InteractionState,
    provider: MapDataProvider,
    ui_root: Widget,
    selected_rows: Vec<usize>,
    sort_col: Option<usize>,
    sort_dir: Option<SortDirection>,
    col_specs: Vec<ColumnDef>,
    resizing_col: Option<usize>,
    drag_start_x: f32,
    start_width: f32,
    modifiers: winit::event::Modifiers,
    last_selected_row: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create sample data
    let data_source = create_sample_data();
    let mut provider = MapDataProvider::new();
    provider.register("users", data_source);
    
    // Initial Columns
    let columns = vec![
        ColumnDef::new("ID", "ID").width(ColumnWidth::Fixed(60.0)).align(TextAlign::Right),
        ColumnDef::new("Name", "Name").width(ColumnWidth::Flex(2.0)).align(TextAlign::Left),
        ColumnDef::new("Age", "Age").width(ColumnWidth::Fixed(80.0)).align(TextAlign::Right),
        ColumnDef::new("City", "City").width(ColumnWidth::Flex(1.5)).align(TextAlign::Left),
        ColumnDef::new("Status", "Status").width(ColumnWidth::Fixed(100.0)).align(TextAlign::Center),
    ];
    
    let initial_ui = create_ui(&[], None, None, &columns);
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        provider,
        ui_root: initial_ui,
        selected_rows: Vec::new(),
        sort_col: None,
        sort_dir: None,
        col_specs: columns,
        resizing_col: None,
        drag_start_x: 0.0,
        start_width: 0.0,
        modifiers: Default::default(),
        last_selected_row: None,
    }));
    
    println!("DataGrid Example with Interaction");
    
    // Clones for callbacks
    let state_draw = state.clone();
    let state_move = state.clone();
    let state_click = state.clone();
    let state_scroll = state.clone();
    let state_mods = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_modifiers_changed(move |_win, mods| {
             state_mods.borrow_mut().modifiers = mods;
        })
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x, y);
            s.interaction.update_mouse(pos);
            
            // Handle Resizing
            if let Some(col_idx) = s.resizing_col {
                let delta = x - s.drag_start_x;
                let new_width = (s.start_width + delta).max(20.0);
                
                // Update Column Definition
                if let Some(col) = s.col_specs.get_mut(col_idx) {
                     col.width = ColumnWidth::Fixed(new_width);
                }
                
                // Request redraw
                _win.window.request_redraw();
            }

            // Hit test to update hovered state
            let hit_action = hit_test(&s.ui_root, pos, Some(&s.interaction.scroll_offsets))
                .map(|h| h.action.clone());
            
            // Update Cursor
            let cursor = if s.resizing_col.is_some() {
                CursorIcon::EwResize
            } else if let Some(action) = hit_action {
                if action.contains("header_resize") {
                    CursorIcon::EwResize
                } else if action.contains("header") || action.contains("row") {
                    CursorIcon::Default
                } else {
                    CursorIcon::Default
                }
            } else {
                CursorIcon::Default
            };
            _win.window.set_cursor_icon(cursor);
            s.interaction.handle_hit(hit_action);
        })
        .on_mouse_input(move |win, state, button| {
            if button == MouseButton::Left {
                let mut s = state_click.borrow_mut();
                let pressed = state == ElementState::Pressed;
                s.interaction.set_pressed(pressed);
                
                if pressed {
                    let hovered = s.interaction.hovered_action.clone();
                    
                    if let Some(action) = &hovered {
                        if action.starts_with("main_grid:row:") {
                            if let Ok(row_idx) = action.split(":").nth(2).unwrap_or("0").parse::<usize>() {
                                println!("Selected row: {}", row_idx);
                                let state = s.modifiers.state();
                                let ctrl = state.control_key();
                                let shift = state.shift_key();
                                
                                if shift {
                                    // Range Selection
                                    if let Some(last) = s.last_selected_row {
                                        let start = last.min(row_idx);
                                        let end = last.max(row_idx);
                                        // Clear if not ctrl? Standard behavior usually keeps existing if ctrl held too, but lets simplify:
                                        // Shift usually extends from anchor.
                                        if !ctrl {
                                            s.selected_rows.clear();
                                        }
                                        for r in start..=end {
                                            if !s.selected_rows.contains(&r) {
                                                s.selected_rows.push(r);
                                            }
                                        }
                                    } else {
                                        // Treat as click
                                        s.selected_rows.clear();
                                        s.selected_rows.push(row_idx);
                                        s.last_selected_row = Some(row_idx);
                                    }
                                } else if ctrl {
                                    // Toggle
                                    if let Some(pos) = s.selected_rows.iter().position(|&r| r == row_idx) {
                                        s.selected_rows.remove(pos);
                                        // Update anchor?
                                        s.last_selected_row = Some(row_idx);
                                    } else {
                                        s.selected_rows.push(row_idx);
                                        s.last_selected_row = Some(row_idx);
                                    }
                                } else {
                                    // Single Select
                                    s.selected_rows.clear();
                                    s.selected_rows.push(row_idx);
                                    s.last_selected_row = Some(row_idx);
                                }
                                
                                win.window.request_redraw();
                            }
                        } else if action.starts_with("main_grid:header:") {
                            if let Ok(col_idx) = action.split(":").nth(2).unwrap_or("0").parse::<usize>() {
                                println!("Sort column: {}", col_idx);
                                
                                // Update Sort State
                                let (new_col, new_dir) = if s.sort_col == Some(col_idx) {
                                    (Some(col_idx), match s.sort_dir {
                                        Some(SortDirection::Ascending) => Some(SortDirection::Descending),
                                        _ => Some(SortDirection::Ascending),
                                    })
                                } else {
                                    (Some(col_idx), Some(SortDirection::Ascending))
                                };
                                
                                s.sort_col = new_col;
                                s.sort_dir = new_dir;
                                
                                // Perform Sort
                                if let Some(source) = s.provider.get_source_mut("users") { 
                                     if let Some(dir) = new_dir {
                                          source.sort(col_idx, dir);
                                     }
                                }
                                
                                win.window.request_redraw();
                            }
                        } else if action.starts_with("main_grid:header_resize:") {
                            if let Ok(col_idx) = action.split(":").nth(2).unwrap_or("0").parse::<usize>() {
                                println!("Resize start: {}", col_idx);
                                s.resizing_col = Some(col_idx);
                                s.drag_start_x = s.interaction.mouse_pos.x;
                                
                                // Get current width
                                if let Some(col) = s.col_specs.get(col_idx) {
                                    s.start_width = match col.width {
                                        ColumnWidth::Fixed(w) => w,
                                        ColumnWidth::Flex(_f) => 100.0, // Default
                                        _ => 0.0,
                                    };
                                }
                            }
                        }
                    }
                    s.interaction.set_clicked(hovered);
                } else {
                    s.interaction.set_clicked(None);
                    s.resizing_col = None; // Stop resizing
                }
            }
        })
        .on_scroll(move |win, delta, _phase| {
            let mut s = state_scroll.borrow_mut();
            // Try to find what we are scrolling
             let action_opt = s.interaction.hovered_action.clone();
             if let Some(action) = action_opt {
                 let target_id = if action.starts_with("main_grid") {
                     "main_grid"
                 } else {
                     action.as_str()
                 };
                 
                let dy = match delta {
                     winit::event::MouseScrollDelta::LineDelta(_, y) => y * 20.0,
                     winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                 };

                 // Calculate limits first (Immutable Borrow)
                 let mut max_scroll = f32::MAX;
                 if target_id == "main_grid" {
                     let row_height = 36.0;
                     let header_height = 40.0;
                         if let Some(gloomy_core::widget::Widget::DataGrid { bounds, .. }) = children.first() {
                            let visible_height = (bounds.height - header_height).max(0.0);
                            let total_height = s.provider.get_source("data").map(|ds| ds.row_count() as f32 * row_height).unwrap_or(0.0);
                            max_scroll = (total_height - visible_height).max(0.0);
                         }
                     }
                 }
                 
                 // Apply Scroll (Mutable Borrow)
                 let current = s.interaction.scroll_offsets.entry(target_id.to_string()).or_insert(Vec2::ZERO);
                 current.y -= dy;
                 current.y = current.y.clamp(0.0, max_scroll);
                 
                 win.window.request_redraw();
             }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            // Rebuild UI with current selection
            let ui_root_new = create_ui(&s.selected_rows, s.sort_col, s.sort_dir, &s.col_specs);
            s.ui_root = ui_root_new; // Update the stored root for layout and rendering
            
            let window_size = win.window.inner_size();
            let width = window_size.width as f32;
            let height = window_size.height as f32;
        
            // Manually update root bounds since compute_layout expects them to be set
                 bounds.width = width;
                 bounds.height = height;
            }

            gloomy_core::layout_engine::compute_layout(&mut s.ui_root, 0.0, 0.0, width, height);

            // Render
            render_ui(
                &s.ui_root, // Render the updated and laid-out s.ui_root
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                Some(&s.interaction),
                Some(&s.provider as &dyn DataProvider),
            );
            
        })
        .run()
}

fn create_sample_data() -> VecDataSource {
    let columns = vec![
        "ID".to_string(),
        "Name".to_string(),
        "Age".to_string(),
        "City".to_string(),
        "Status".to_string(),
    ];
    
    let mut rows = Vec::new();
    for i in 1..=100 {
        rows.push(vec![
            CellValue::Integer(i),
            CellValue::Text(format!("User Name {}", i)),
            CellValue::Integer(20 + (i % 45)),
            CellValue::Text(match i % 4 {
                0 => "New York".to_string(),
                1 => "London".to_string(),
                2 => "Tokyo".to_string(),
                _ => "Paris".to_string(),
            }),
            CellValue::Text(if i % 3 == 0 { "Inactive".to_string() } else { "Active".to_string() }),
        ]);
    }
    
    VecDataSource::new(columns, rows)
}

fn create_ui(selected_rows: &[usize], sort_col: Option<usize>, sort_dir: Option<SortDirection>, columns: &[ColumnDef]) -> Widget {
        id: Some("root".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(1200.0),
        height: Some(800.0),
        background: Some((0.15, 0.15, 0.15, 1.0)),
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        padding: 30.0,
        layout: Layout {
            direction: Direction::Column,
            justify_content: gloomy_core::Justify::Start,
            align_items: gloomy_core::Align::Stretch,
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
                text: "Interactive DataGrid Example".to_string(),
                x: 0.0,
                y: 0.0,
                width: 1140.0,
                height: 40.0,
                size: 28.0,
                color: (0.9, 0.9, 0.9, 1.0),
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // Instruction Label
            Widget::Label {
                text: "Scroll with mouse wheel. Click rows to select.".to_string(),
                x: 0.0,
                y: 0.0,
                width: 1140.0,
                height: 25.0,
                size: 14.0,
                color: (0.7, 0.7, 0.7, 1.0),
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            // DataGrid
            Widget::DataGrid {
                id: Some("main_grid".to_string()),
                bounds: WidgetBounds::default(),
                columns: columns.to_vec(),
                data_source_id: Some("users".to_string()),
                header_height: 40.0,
                row_height: 36.0,
                striped: true,
                selection_mode: gloomy_core::datagrid::SelectionMode::Multiple,
                selected_rows: selected_rows.to_vec(),
                sort_column: sort_col,
                sort_direction: sort_dir,
                show_vertical_lines: true,
                show_horizontal_lines: true,
                style: gloomy_core::datagrid::DataGridStyle {
                    header_background: (0.8, 0.2, 0.2, 1.0), // Red Header
                    header_text_color: (1.0, 1.0, 1.0, 1.0),
                    row_background: (1.0, 1.0, 1.0, 1.0),    // White Rows
                    alt_row_background: (0.9, 0.9, 0.9, 1.0), // Light Gray Striping
                    row_text_color: (0.0, 0.0, 0.0, 1.0),    // Black Text
                    grid_line_color: (0.0, 0.0, 0.0, 1.0),   // Black Lines
                    grid_line_width: 1.0,
                    hover_background: (0.5, 0.5, 1.0, 0.5),  // Blue Hover
                    selected_background: (0.2, 0.8, 0.2, 1.0), // Green Selection
                    cell_padding: 8.0,
                },
                flex: 1.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
        ],
    }
}
