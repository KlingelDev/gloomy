//! RON→PNG CLI tool.
//!
//! Renders a RON UI definition file to a PNG image using
//! headless GPU rendering.
//!
//! Usage:
//!   cargo run --example ron_to_png -- input.ron output.png
//!   cargo run --example ron_to_png -- input.ron output.png \
//!     --width 1920 --height 1080 --theme dark --layout
//!   cargo run --example ron_to_png -- input.ron output.png \
//!     --data sources.json

use gloomy_core::headless::{HeadlessConfig, HeadlessRenderer};
use gloomy_core::load_ui;
use gloomy_driver::dump_layout;
use std::collections::HashMap;

struct Args {
  input: String,
  output: String,
  width: u32,
  height: u32,
  theme: String,
  show_layout: bool,
  data_path: Option<String>,
}

fn parse_args() -> anyhow::Result<Args> {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 3 {
    anyhow::bail!(
      "Usage: ron_to_png <input.ron> <output.png> \
       [--width N] [--height N] [--theme dark|light] \
       [--layout] [--data sources.json]"
    );
  }

  let mut result = Args {
    input: args[1].clone(),
    output: args[2].clone(),
    width: 1280,
    height: 720,
    theme: "dark".to_string(),
    show_layout: false,
    data_path: None,
  };

  let mut i = 3;
  while i < args.len() {
    match args[i].as_str() {
      "--width" => {
        i += 1;
        result.width = args[i].parse()?;
      }
      "--height" => {
        i += 1;
        result.height = args[i].parse()?;
      }
      "--theme" => {
        i += 1;
        result.theme = args[i].clone();
      }
      "--layout" => {
        result.show_layout = true;
      }
      "--data" => {
        i += 1;
        result.data_path = Some(args[i].clone());
      }
      other => {
        anyhow::bail!("Unknown argument: {}", other);
      }
    }
    i += 1;
  }

  Ok(result)
}

/// Loads data sources from a JSON file.
///
/// Expected format:
/// ```json
/// {
///   "source_id": [
///     ["col1", "col2"],
///     ["val1", "val2"]
///   ]
/// }
/// ```
fn load_data_sources(
  path: &str,
) -> anyhow::Result<JsonDataProvider> {
  let content = std::fs::read_to_string(path)?;
  let raw: HashMap<String, Vec<Vec<String>>> =
    serde_json::from_str(&content)?;

  let mut provider = JsonDataProvider {
    sources: HashMap::new(),
  };

  for (id, mut all_rows) in raw {
    // First row is headers, rest is data.
    if all_rows.is_empty() {
      continue;
    }
    let headers = all_rows.remove(0);
    let rows: Vec<Vec<gloomy_core::data_source::CellValue>> =
      all_rows
        .into_iter()
        .map(|row| {
          row
            .into_iter()
            .map(|s| {
              gloomy_core::data_source::CellValue::Text(s)
            })
            .collect()
        })
        .collect();
    let source =
      gloomy_core::data_source::VecDataSource::new(
        headers, rows,
      );
    provider.sources.insert(id, Box::new(source));
  }

  Ok(provider)
}

struct JsonDataProvider {
  sources: HashMap<
    String,
    Box<dyn gloomy_core::data_source::DataSource>,
  >,
}

impl gloomy_core::data_source::DataProvider
  for JsonDataProvider
{
  fn get_source(
    &self,
    id: &str,
  ) -> Option<&dyn gloomy_core::data_source::DataSource> {
    self.sources.get(id).map(|s| s.as_ref())
  }

  fn get_source_mut(
    &mut self,
    id: &str,
  ) -> Option<
    &mut (dyn gloomy_core::data_source::DataSource + 'static),
  > {
    self.sources.get_mut(id).map(|s| s.as_mut())
  }
}

fn main() -> anyhow::Result<()> {
  let args = parse_args()?;

  let mut ui = load_ui(&args.input)?;

  // Apply theme background color.
  let theme = match args.theme.as_str() {
    "light" => gloomy_core::theme::Theme::light(),
    "high_contrast" => {
      gloomy_core::theme::Theme::high_contrast()
    }
    _ => gloomy_core::theme::Theme::dark(),
  };

  let mut renderer =
    HeadlessRenderer::new(HeadlessConfig {
      width: args.width,
      height: args.height,
      force_fallback_adapter: true,
      ..Default::default()
    })?;

  let bg = theme.colors.background;
  renderer
    .renderer_mut()
    .set_clear_color(
      bg.0 as f64,
      bg.1 as f64,
      bg.2 as f64,
      bg.3 as f64,
    );

  let data_provider = args
    .data_path
    .as_deref()
    .map(load_data_sources)
    .transpose()?;

  renderer.save_screenshot(
    &mut ui,
    &args.output,
    None,
    data_provider
      .as_ref()
      .map(|p| {
        p as &dyn gloomy_core::data_source::DataProvider
      }),
  )?;

  if args.show_layout {
    println!("{}", dump_layout(&ui));
  }

  println!("Saved {}", args.output);
  Ok(())
}
