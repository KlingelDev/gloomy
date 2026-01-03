/// DataGrid Editing Demo
///
/// Demonstrates:
/// - Inline Editing (Double-click)
/// - Row Operations (Add/Delete)
/// - Dirty State Tracking
/// - Selection
///
/// Run with: cargo run --example datagrid_edit_demo

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::{render_ui, hit_test},
    widget::{Widget, WidgetBounds, TextAlign},
    datagrid::{ColumnDef, ColumnWidth, DataGridStyle, SelectionMode},
    data_source::{VecDataSource, CellValue, MapDataProvider, DataProvider},
    interaction::InteractionState,
    Vec2,
};
use std::{cell::RefCell, rc::Rc};
use std::time::{SystemTime, UNIX_EPOCH};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{Key, NamedKey};

struct AppState {
    interaction: InteractionState,
    provider: MapDataProvider,
    ui_root: Widget,
    selected_row: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create sample data
    let data_source = create_sample_data();
    let mut provider = MapDataProvider::new();
    provider.register("data", data_source);
    
    let columns = vec![
        ColumnDef::new("ID", "ID").width(ColumnWidth::Fixed(60.0)).align(TextAlign::Right),
        ColumnDef::new("Name", "Name").width(ColumnWidth::Flex(2.0)).align(TextAlign::Left),
        ColumnDef::new("Value", "Value").width(ColumnWidth::Fixed(100.0)).align(TextAlign::Right),
    ];
    
    // Initial UI
    let ui_root = create_ui(&columns, None);
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        provider,
        ui_root,
        selected_row: None,
    }));
    
    println!("DataGrid Editing Demo");
    println!("Double-click a cell to edit. Enter to commit.");
    println!("Select row to delete. Add Row appends to end.");
    
    let state_draw = state.clone();
    let state_move = state.clone();
    let state_click = state.clone();
    let state_key = state.clone();
    
    gloomy_app::GloomyApp::new()
        .on_cursor_move(move |_win, x, y| {
            let mut s = state_move.borrow_mut();
            s.interaction.update_mouse(Vec2::new(x, y));
        })
        .on_mouse_input(move |win, element_state, button| {
            if button != MouseButton::Left { return; }
            
            let mut s = state_click.borrow_mut();
            let pressed = element_state == ElementState::Pressed;
            s.interaction.set_pressed(pressed);
            
            if pressed {
                let pos = s.interaction.mouse_pos;
                let hit = hit_test(&s.ui_root, pos, Some(&s.interaction))
                    .map(|h| h.action.clone());
                
                if let Some(action) = &hit {
                    // 1. Button Actions
                    if action == "add_row" {
                         if let Some(source) = s.provider.get_source_mut("data") {
                             if let Some(new_idx) = source.add_row_default() {
                                 println!("Added row {}", new_idx);
                                 // Select new row
                                 s.selected_row = Some(new_idx);
                                 s.interaction.cancel_grid_edit();
                             }
                         }
                    } else if action == "delete_row" {
                        if let Some(selected) = s.selected_row {
                            if let Some(source) = s.provider.get_source_mut("data") {
                                if source.delete_row(selected) {
                                    println!("Deleted row {}", selected);
                                    s.selected_row = None;
                                    s.interaction.cancel_grid_edit();
                                    // Clear dirty state for this row? 
                                    // ideally we'd rebuild dirty state but for now just clear all to be safe or ignore
                                    s.interaction.clear_dirty(None);
                                }
                            }
                        } else {
                            println!("No row selected to delete");
                        }
                    } 
                    // 2. Cell Clicks
                    else if action.contains(":cell:") {
                        let parts: Vec<&str> = action.split(':').collect();
                        if parts.len() >= 4 {
                            let grid_id = parts[0];
                            if let (Ok(row), Ok(col)) = (
                                parts[2].parse::<usize>(), 
                                parts[3].parse::<usize>()
                            ) {
                                // Select Row
                                s.selected_row = Some(row);
                            
                                // Check Double Click for Edit
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .map(|d| d.as_millis() as u64)
                                    .unwrap_or(0);
                                
                                if s.interaction.check_double_click(action, now) {
                                    // Start Editing
                                    let initial = s.provider.get_source("data")
                                        .map(|ds| ds.cell_text(row, col))
                                        .unwrap_or_default();
                                    s.interaction.start_grid_edit(grid_id, row, col, &initial);
                                    println!("Editing cell ({}, {}): {}", row, col, initial);
                                }
                            }
                        }
                    }
                } else {
                    // Clicked background -> deselect
                    s.selected_row = None;
                    s.interaction.cancel_grid_edit();
                }
                
                s.interaction.set_clicked(hit);
                win.window.request_redraw();
            }
        })
        .on_keyboard_input(move |win, event| {
            if event.state != ElementState::Pressed { return; }
            
            let mut s = state_key.borrow_mut();
            
            // Handle editing keys
            if s.interaction.editing_grid_cell.is_some() {
                match &event.logical_key {
                    Key::Named(NamedKey::Enter) => {
                        // Commit
                        if let Some((grid_id, row, col, value_str)) = s.interaction.commit_grid_edit() {
                            println!("Committed: ({}, {}) = {}", row, col, value_str);
                            if let Some(source) = s.provider.get_source_mut("data") {
                                // Try to preserve original type
                                let current_val = source.cell_value(row, col);
                                let new_val = match current_val {
                                    CellValue::Integer(_) => {
                                        value_str.parse::<i64>().map(CellValue::Integer).unwrap_or(CellValue::Text(value_str.clone()))
                                    },
                                    CellValue::Number(_) => {
                                        value_str.parse::<f64>().map(CellValue::Number).unwrap_or(CellValue::Text(value_str.clone()))
                                    },
                                    CellValue::Boolean(_) => {
                                        value_str.parse::<bool>().map(CellValue::Boolean).unwrap_or(CellValue::Text(value_str.clone()))
                                    },
                                    _ => CellValue::Text(value_str.clone()),
                                };
                                
                                if source.set_cell(row, col, new_val) {
                                    s.interaction.mark_dirty(&grid_id, row, col);
                                }
                            }
                        }
                        win.window.request_redraw();
                    }
                    Key::Named(NamedKey::Escape) => {
                        s.interaction.cancel_grid_edit();
                        win.window.request_redraw();
                    }
                    Key::Named(NamedKey::Backspace) => {
                        s.interaction.grid_edit_buffer.pop();
                        win.window.request_redraw();
                    }
                    Key::Character(ch) => {
                        if !ch.chars().any(|c| c.is_control()) {
                            s.interaction.grid_edit_buffer.push_str(ch);
                            win.window.request_redraw();
                        }
                    }
                    _ => {}
                }
            } else {
                // Navigation / Shortcuts (outside edit mode)
                // e.g. Delete key to delete row?
            }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            
            let scale = win.renderer.scale_factor;
            let width = win.config.width as f32 / scale;
            let height = win.config.height as f32 / scale;
            
            // Re-create UI to update selection visualization
            // In a real app we might update just the field, but here we rebuild declarative tree
            let columns = vec![
                ColumnDef::new("ID", "ID").width(ColumnWidth::Fixed(60.0)).align(TextAlign::Right),
                ColumnDef::new("Name", "Name").width(ColumnWidth::Flex(2.0)).align(TextAlign::Left),
                ColumnDef::new("Value", "Value").width(ColumnWidth::Fixed(100.0)).align(TextAlign::Right),
            ];
            s.ui_root = create_ui(&columns, s.selected_row);
            
            if let Widget::Container { bounds, .. } = &mut s.ui_root {
                bounds.width = width;
                bounds.height = height;
            }
            
            compute_layout(&mut s.ui_root, 0.0, 0.0, width, height);
            
            // Hover logic
            if let Some(hit) = hit_test(&s.ui_root, s.interaction.mouse_pos, Some(&s.interaction)) {
                s.interaction.hovered_action = Some(hit.action.to_string());
            } else {
                s.interaction.hovered_action = None;
            }
            
            render_ui(
                &s.ui_root,
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
        "Value".to_string(),
    ];
    
    let rows = vec![
        vec![CellValue::Integer(1), CellValue::Text("Alpha".to_string()), CellValue::Number(100.5)],
        vec![CellValue::Integer(2), CellValue::Text("Beta".to_string()), CellValue::Number(200.0)],
        vec![CellValue::Integer(3), CellValue::Text("Gamma".to_string()), CellValue::Number(350.75)],
        vec![CellValue::Integer(4), CellValue::Text("Delta".to_string()), CellValue::Number(425.0)],
        vec![CellValue::Integer(5), CellValue::Text("Epsilon".to_string()), CellValue::Number(555.55)],
    ];
    
    VecDataSource::new(columns, rows)
}

fn create_ui(columns: &[ColumnDef], selected_row: Option<usize>) -> Widget {
    Widget::Container {
        id: Some("root".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        background: Some((0.12, 0.12, 0.14, 1.0)),
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
        flex: 1.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        corner_radii: None,
        layout_cache: None,
        render_cache: RefCell::new(None),
        children: vec![
            Widget::Label {
                text: "DataGrid Editing Demo".to_string(),
                x: 0.0, y: 0.0,
                width: 500.0,
                height: 40.0,
                size: 24.0,
                color: (1.0, 1.0, 1.0, 1.0), // Pure white
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                font: None,
            },
            
            // Toolbar
            Widget::Container {
                id: None,
                scrollable: false,
                bounds: WidgetBounds::default(),
                width: None,
                height: Some(40.0),
                background: None,
                border: None,
                corner_radius: 0.0,
                shadow: None,
                gradient: None,
                padding: 0.0,
                layout: Layout {
                    direction: Direction::Row,
                    justify_content: gloomy_core::Justify::Start,
                    align_items: gloomy_core::Align::Center,
                    spacing: 10.0,
                    ..Default::default()
                },
                flex: 0.0,
                grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                corner_radii: None,
                layout_cache: None,
                render_cache: RefCell::new(None),
                children: vec![
                    Widget::Button {
                        text: "Add Row".to_string(),
                        action: "add_row".to_string(),
                        bounds: WidgetBounds { width: 100.0, height: 32.0, ..Default::default() },
                        background: (0.2, 0.6, 0.8, 1.0),
                        hover_color: (0.3, 0.7, 0.9, 1.0),
                        active_color: (0.1, 0.5, 0.7, 1.0),
                        border: None,
                        corner_radius: 4.0,
                        shadow: None,
                        gradient: None,
                        corner_radii: None,
                        layout: Default::default(),
                        flex: 0.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                        font: None,
                    },
                    Widget::Button {
                        text: "Delete Row".to_string(),
                        action: "delete_row".to_string(),
                        bounds: WidgetBounds { width: 110.0, height: 32.0, ..Default::default() },
                        background: (0.8, 0.3, 0.3, 1.0),
                        hover_color: (0.9, 0.4, 0.4, 1.0),
                        active_color: (0.7, 0.2, 0.2, 1.0),
                        border: None,
                        corner_radius: 4.0,
                        shadow: None,
                        gradient: None,
                        corner_radii: None,
                        layout: Default::default(),
                        flex: 0.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                        font: None,
                    },
                    Widget::Label {
                        text: "Select a row to delete.".to_string(),
                        x: 0.0, y: 0.0,
                        width: 200.0,
                        height: 25.0,
                        size: 14.0,
                        color: (0.7, 0.7, 0.75, 1.0),
                        text_align: TextAlign::Left,
                        flex: 0.0,
                        grid_col: None, grid_row: None, col_span: 1, row_span: 1,
                        font: None,
                    },
                ],
            },
            
            Widget::DataGrid {
                id: Some("grid".to_string()),
                bounds: WidgetBounds::default(),
                columns: columns.to_vec(),
                data_source_id: Some("data".to_string()),
                header_height: 40.0,
                row_height: 36.0,
                striped: true,
                selection_mode: SelectionMode::Single,
                selected_rows: selected_row.into_iter().collect(),
                sort_column: None,
                sort_direction: None,
                show_vertical_lines: true,
                show_horizontal_lines: true,
                style: DataGridStyle::default(),
                flex: 1.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
        ],
    }
}
