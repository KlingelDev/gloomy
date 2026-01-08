//! Chart Demo - Example demonstrating the Chart widget.
//!
//! Press 'q' or Escape to quit.

use gloomy_app::{GloomyApp, Widget, WidgetBounds, render_ui};
use gloomy_core::data_source::{MapDataProvider, VecDataSource, CellValue, DataSource, DataProvider};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // Create a data source
    let mut ds = VecDataSource::with_columns(vec!["X".to_string(), "Y".to_string()]);
    // Initial data
    for i in 0..100 {
        let x = i as f64 / 10.0;
        let y = x.sin();
        ds.add_row(vec![CellValue::Number(x), CellValue::Number(y)]);
    }

    let ds_id = "sine_wave";
    let mut provider = MapDataProvider::new();
    provider.register(ds_id, ds);

    // Create a chart widget bound to data source
    let chart = Widget::Chart {
        id: Some("demo_chart".to_string()),
        chart_type: "line".to_string(),
        title: "Dynamic Sine Wave".to_string(),
        data_source_id: Some(ds_id.to_string()),
        bounds: WidgetBounds { 
            x: 50.0, 
            y: 50.0, 
            width: 700.0, 
            height: 500.0 
        },
        width: 700.0,
        height: 500.0,
        flex: 1.0,
        grid_col: None,
        grid_row: None,
        col_span: 1,
        row_span: 1,
        backend: Default::default(),
    };

    let mut frame = 0;

    GloomyApp::new()
        .with_title("Gloomy Chart Demo")
        .with_size(800, 600)
        .on_draw(move |window, ctx| {
             // Simulate data update
             frame += 1;
             
             // Access data source mutably to update it
             if let Some(ds) = provider.get_source_mut(ds_id) {
                 // Clear/Overwrite or just modify cells?
                 // Simple modification: shift phase
                 let phase = frame as f64 * 0.1;
                 
                 // We know the source is VecDataSource but through trait we can only set cells
                 // (unless we downcast, but set_cell is fine)
                 let rows = ds.row_count();
                 for r in 0..rows {
                     let x = r as f64 / 10.0;
                     let y = (x + phase).sin();
                     ds.set_cell(r, 1, CellValue::Number(y));
                 }
                 // set_cell updates version automatically
             }

             // We need to call render_ui.
             render_ui(
                 &chart,
                 &mut window.renderer,
                 ctx.device,
                 ctx.queue,
                 None, 
                 Some(&provider), // Pass provider
             );
        })
        .run()
}
