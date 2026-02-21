//! CLI for visual regression testing of Gloomy UIs.
//!
//! Usage:
//!   cargo run --example visual_test -- compare <name>
//!   cargo run --example visual_test -- update <name>
//!   cargo run --example visual_test -- compare-all

use gloomy_driver::diff::{DiffConfig, collect_widgets};
use gloomy_driver::screenshot::HeadlessRenderer;
use gloomy_driver::snapshot::SnapshotManager;
use std::path::PathBuf;
use std::process;

const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
const SCALE: f32 = 1.0;

fn ui_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/ui")
}

fn snapshot_dir() -> PathBuf {
  PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("snapshots")
}

fn load_and_render(
  name: &str,
  renderer: &mut HeadlessRenderer,
) -> anyhow::Result<image::RgbaImage> {
  let ron_path = ui_dir().join(format!("{name}.ron"));
  let mut root = gloomy_core::load_ui(&ron_path)?;
  renderer.render_to_image(&mut root, None, None)
}

fn cmd_update(name: &str) -> anyhow::Result<()> {
  let mut renderer =
    HeadlessRenderer::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, SCALE)?;
  let image = load_and_render(name, &mut renderer)?;
  let mgr = SnapshotManager::new(snapshot_dir());
  let path = mgr.update_golden(name, &image)?;
  let report = serde_json::json!({
    "action": "update",
    "name": name,
    "path": path.display().to_string(),
    "width": image.width(),
    "height": image.height(),
  });
  println!("{}", serde_json::to_string_pretty(&report)?);
  Ok(())
}

fn cmd_compare(name: &str) -> anyhow::Result<bool> {
  let mut renderer =
    HeadlessRenderer::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, SCALE)?;
  let ron_path = ui_dir().join(format!("{name}.ron"));
  let mut root = gloomy_core::load_ui(&ron_path)?;
  let image =
    renderer.render_to_image(&mut root, None, None)?;
  let mgr = SnapshotManager::new(snapshot_dir());
  let config = DiffConfig::default();
  let report =
    mgr.compare(name, &image, &config, Some(&root))?;
  println!(
    "{}",
    serde_json::to_string_pretty(&report)?
  );
  Ok(report.passed)
}

fn cmd_compare_all() -> anyhow::Result<bool> {
  let mgr = SnapshotManager::new(snapshot_dir());
  let names = mgr.list_goldens()?;
  if names.is_empty() {
    eprintln!("No golden snapshots found. Run 'update' first.");
    return Ok(false);
  }

  let mut renderer =
    HeadlessRenderer::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, SCALE)?;
  let config = DiffConfig::default();
  let mut all_passed = true;
  let mut results = Vec::new();

  for name in &names {
    let ron_path = ui_dir().join(format!("{name}.ron"));
    if !ron_path.exists() {
      results.push(serde_json::json!({
        "name": name,
        "error": format!(
          "RON file not found: {}", ron_path.display()
        ),
      }));
      all_passed = false;
      continue;
    }
    let mut root = gloomy_core::load_ui(&ron_path)?;
    let image =
      renderer.render_to_image(&mut root, None, None)?;
    let report =
      mgr.compare(name, &image, &config, Some(&root))?;
    if !report.passed {
      all_passed = false;
    }
    results.push(serde_json::to_value(&report)?);
  }

  let summary = serde_json::json!({
    "all_passed": all_passed,
    "count": results.len(),
    "results": results,
  });
  println!("{}", serde_json::to_string_pretty(&summary)?);
  Ok(all_passed)
}

fn cmd_render(ron_path_str: &str) -> anyhow::Result<bool> {
  let path = PathBuf::from(ron_path_str);
  if !path.exists() {
    anyhow::bail!(
      "RON file not found: {}", path.display()
    );
  }
  let stem = path
    .file_stem()
    .map(|s| s.to_string_lossy().to_string())
    .unwrap_or_else(|| "output".to_string());

  let mut renderer =
    HeadlessRenderer::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, SCALE)?;
  let mut root = gloomy_core::load_ui(&path)?;
  let image =
    renderer.render_to_image(&mut root, None, None)?;

  let render_dir = snapshot_dir().join("render");
  std::fs::create_dir_all(&render_dir)?;
  let out_path = render_dir.join(format!("{stem}.png"));
  image.save(&out_path)?;

  let widgets = collect_widgets(&root);
  let abs = out_path
    .canonicalize()
    .unwrap_or(out_path);
  let report = serde_json::json!({
    "image": abs.display().to_string(),
    "width": image.width(),
    "height": image.height(),
    "widgets": widgets,
  });
  println!(
    "{}",
    serde_json::to_string_pretty(&report)?
  );
  Ok(true)
}

fn usage() {
  eprintln!("Usage:");
  eprintln!(
    "  cargo run --example visual_test -- compare <name>"
  );
  eprintln!(
    "  cargo run --example visual_test -- update <name>"
  );
  eprintln!(
    "  cargo run --example visual_test -- compare-all"
  );
  eprintln!(
    "  cargo run --example visual_test -- render <path.ron>"
  );
  eprintln!();
  eprintln!(
    "<name> matches a .ron file in examples/ui/ \
     (e.g. 'form_demo')."
  );
  eprintln!(
    "<path.ron> is a direct path to any RON UI file."
  );
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  if args.len() < 2 {
    usage();
    process::exit(2);
  }

  let result = match args[1].as_str() {
    "update" => {
      if args.len() < 3 {
        usage();
        process::exit(2);
      }
      cmd_update(&args[2]).map(|_| true)
    }
    "compare" => {
      if args.len() < 3 {
        usage();
        process::exit(2);
      }
      cmd_compare(&args[2])
    }
    "compare-all" => cmd_compare_all(),
    "render" => {
      if args.len() < 3 {
        usage();
        process::exit(2);
      }
      cmd_render(&args[2])
    }
    other => {
      eprintln!("Unknown command: {other}");
      usage();
      process::exit(2);
    }
  };

  match result {
    Ok(true) => process::exit(0),
    Ok(false) => process::exit(1),
    Err(e) => {
      eprintln!("Error: {e:#}");
      process::exit(2);
    }
  }
}
