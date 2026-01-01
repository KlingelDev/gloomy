/// Simple DataGrid Example
///
/// Demonstrates:
/// - Basic DataGrid widget with column definitions
/// - VecDataSource with sample data
/// - Column headers
/// - Simple table layout
///
/// Run with: cargo run --example simple_datagrid

use gloomy_core::{
    layout::{Direction, Layout},
    layout_engine::compute_layout,
    ui::render_ui,
    widget::{Widget, WidgetBounds, TextAlign},
    datagrid::{ColumnDef, ColumnWidth},
    data_source::{VecDataSource, CellValue},
    Vec2,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    
    // Create sample data
    let data_source = create_sample_data();
    
    println!("DataGrid Example");
    println!("Rows: {}", data_source.row_count());
    println!("Columns: {}", data_source.column_count());
    
    gloomy_app::GloomyApp::new()
        .on_draw(move |win, ctx| {
            let mut ui_root = create_ui();
            
            let size = win.renderer.size();
            compute_layout(&mut ui_root, 0.0, 0.0, size.x, size.y);
            
            render_ui(
                &ui_root,
                &mut win.renderer,
                ctx.device,
                ctx.queue,
                None,
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
    
    let rows = vec![
        vec![
            CellValue::Integer(1),
            CellValue::Text("Alice Smith".to_string()),
            CellValue::Integer(28),
            CellValue::Text("New York".to_string()),
            CellValue::Text("Active".to_string()),
        ],
        vec![
            CellValue::Integer(2),
            CellValue::Text("Bob Johnson".to_string()),
            CellValue::Integer(34),
            CellValue::Text("Los Angeles".to_string()),
            CellValue::Text("Active".to_string()),
        ],
        vec![
            CellValue::Integer(3),
            CellValue::Text("Charlie Brown".to_string()),
            CellValue::Integer(25),
            CellValue::Text("Chicago".to_string()),
            CellValue::Text("Pending".to_string()),
        ],
        vec![
            CellValue::Integer(4),
            CellValue::Text("Diana Prince".to_string()),
            CellValue::Integer(31),
            CellValue::Text("San Francisco".to_string()),
            CellValue::Text("Active".to_string()),
        ],
        vec![
            CellValue::Integer(5),
            CellValue::Text("Eve Davis".to_string()),
            CellValue::Integer(29),
            CellValue::Text("Seattle".to_string()),
            CellValue::Text("Inactive".to_string()),
        ],
    ];
    
    VecDataSource::new(columns, rows)
}

fn create_ui() -> Widget {
    Widget::Container {
        id: Some("root".to_string()),
        scrollable: false,
        bounds: WidgetBounds::default(),
        width: Some(1200.0),
        height: Some(800.0),
        background: Some((0.1, 0.1, 0.1, 1.0)),
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
                text: "Simple DataGrid Example".to_string(),
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
            
            // DataGrid
            Widget::DataGrid {
                bounds: WidgetBounds::default(),
                columns: vec![
                    ColumnDef::new("ID", "id")
                        .width(ColumnWidth::Fixed(60.0))
                        .align(TextAlign::Right),
                    ColumnDef::new("Name", "name")
                        .width(ColumnWidth::Flex(2.0))
                        .align(TextAlign::Left),
                    ColumnDef::new("Age", "age")
                        .width(ColumnWidth::Fixed(80.0))
                        .align(TextAlign::Right),
                    ColumnDef::new("City", "city")
                        .width(ColumnWidth::Flex(1.5))
                        .align(TextAlign::Left),
                    ColumnDef::new("Status", "status")
                        .width(ColumnWidth::Fixed(100.0))
                        .align(TextAlign::Center),
                ],
                data_source_id: Some("users".to_string()),
                header_height: 40.0,
                row_height: 36.0,
                striped: true,
                selection_mode: gloomy_core::datagrid::SelectionMode::Single,
                show_vertical_lines: true,
                show_horizontal_lines: true,
                style: gloomy_core::datagrid::DataGridStyle::default(),
                flex: 1.0,
                grid_col: None,
                grid_row: None,
                col_span: 1,
                row_span: 1,
            },
            
            // Info text
            Widget::Label {
                text: "DataGrid with 5 rows, flexible column widths, and striped rows".to_string(),
                x: 0.0,
                y: 0.0,
                width: 1140.0,
                height: 25.0,
                size: 13.0,
                color: (0.6, 0.6, 0.6, 1.0),
                text_align: TextAlign::Left,
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
