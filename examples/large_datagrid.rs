use gloomy_app::GloomyApp;
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
    row_count: usize,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create 100,000 rows
    let row_count = 100_000;
    let provider = create_large_provider(row_count);
    
    let columns = vec![
        ColumnDef::new("ID", "idx").width(ColumnWidth::Fixed(80.0)).align(TextAlign::Right),
        ColumnDef::new("Value", "val").width(ColumnWidth::Fixed(100.0)).align(TextAlign::Right),
        ColumnDef::new("Label", "label").width(ColumnWidth::Flex(1.0)).align(TextAlign::Left),
        ColumnDef::new("Status", "status").width(ColumnWidth::Fixed(100.0)).align(TextAlign::Center),
    ];
    
    let ui_root = create_ui(&columns);
    
    let state = Rc::new(RefCell::new(AppState {
        interaction: InteractionState::new(),
        provider,
        ui_root,
        row_count,
    }));
    
    let state_move = state.clone();
    let state_click = state.clone();
    let state_draw = state.clone();
    let state_scroll = state.clone();
    
    GloomyApp::new()
        .on_scroll(move |win, delta, _phase| {
            let mut s = state_scroll.borrow_mut();
            
            // Scroll speed: 200px per line for faster navigation in large lists
            let d = match delta {
                winit::event::MouseScrollDelta::LineDelta(x, y) => Vec2::new(x * 200.0, y * 200.0),
                winit::event::MouseScrollDelta::PixelDelta(p) => Vec2::new(p.x as f32 * 10.0, p.y as f32 * 10.0),
            };
            
            if d != Vec2::ZERO {
                 // Only scroll if hovering over a scrollable widget
                 let target = s.interaction.hovered_action.as_ref()
                    .and_then(|h| h.split(":").next())
                    .map(|s| s.to_string());
                 
                 if let Some(id) = target {
                      s.interaction.handle_scroll(&id, d);
                      win.window.request_redraw();
                 }
            }
        })
        .on_cursor_move(move |win, x, y| {
            let mut s = state_move.borrow_mut();
            let pos = Vec2::new(x as f32, y as f32);
            s.interaction.update_mouse(pos);
            
            // Hit Test
            let hit_action = hit_test(&s.ui_root, pos, Some(&s.interaction.scroll_offsets))
                .map(|h| h.action.clone());
                
            let cursor = if let Some(ref action) = hit_action {
                 if action.contains("header_resize") {
                     CursorIcon::EwResize
                 } else {
                     CursorIcon::Default
                 }
            } else {
                CursorIcon::Default
            };
            
            win.window.set_cursor_icon(cursor);
            s.interaction.handle_hit(hit_action);
            win.window.request_redraw();
        })
        .on_mouse_input(move |win, state, button| {
             if button == MouseButton::Left {
                let mut s = state_click.borrow_mut();
                let pressed = state == ElementState::Pressed;
                s.interaction.set_pressed(pressed);
                win.window.request_redraw();
             }
        })
        .on_draw(move |win, ctx| {
            let mut s = state_draw.borrow_mut();
            let size = win.renderer.size();
            
            
            // Layout
            if let Widget::Container { width, height, bounds, .. } = &mut s.ui_root {
                 *width = Some(size.x);
                 *height = Some(size.y);
                 // bounds are output of layout, but good to init check
                 bounds.width = size.x;
                 bounds.height = size.y;
            }
            compute_layout(&mut s.ui_root, 0.0, 0.0, size.x, size.y);

            // Update FPS in title
            // Note: In real app use a proper FPS counter
            
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

fn create_large_provider(count: usize) -> MapDataProvider {
    let mut rows = Vec::with_capacity(count);
    for i in 0..count {
        rows.push(vec![
            CellValue::Integer(i as i64),
            CellValue::Integer((i * 17 % 997) as i64),
            CellValue::Text(format!("Row Item #{}", i)),
            CellValue::Text(if i % 2 == 0 { "OK".to_string() } else { "Err".to_string() }),
        ]);
    }
    
    let columns = vec![
        ColumnDef::new("ID", "idx"),
        ColumnDef::new("Value", "val"),
        ColumnDef::new("Label", "label"),
        ColumnDef::new("Status", "status"),
    ];
    
    let col_ids = columns.iter().map(|c| c.field.clone()).collect();
    let source = VecDataSource::new(col_ids, rows);
    let mut provider = MapDataProvider::new();
    provider.register("large_data", source);
    provider
}

fn create_ui(columns: &[ColumnDef]) -> Widget {
    Widget::Container { layout_cache: None, render_cache: std::cell::RefCell::new(None),
        id: Some("root".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: None,
        height: None,
        background: Some((0.05, 0.05, 0.05, 1.0)),
        border: None,
        corner_radius: 0.0,
        shadow: None,
        gradient: None,
        padding: 20.0,
        layout: Layout {
            direction: Direction::Column,
            align_items: gloomy_core::Align::Stretch,
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
                text: "Large DataGrid Virtualization Benchmark (100k rows)".to_string(),
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 30.0,
                size: 24.0,
                color: (1.0, 1.0, 1.0, 1.0),
                text_align: TextAlign::Left,
                flex: 0.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
                font: None,
            },
            
            Widget::DataGrid {
                id: Some("datagrid".to_string()),
                bounds: WidgetBounds::default(),
                columns: columns.to_vec(),
                data_source_id: Some("large_data".to_string()),
                header_height: 32.0,
                row_height: 28.0,
                striped: true,
                selection_mode: gloomy_core::datagrid::SelectionMode::Single,
                selected_rows: Vec::new(),
                sort_column: None,
                sort_direction: Some(SortDirection::Ascending),
                show_vertical_lines: true,
                show_horizontal_lines: false,
                style: gloomy_core::datagrid::DataGridStyle::default(),
                flex: 1.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            }
        ],
    }
}
