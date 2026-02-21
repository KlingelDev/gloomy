//! Golden snapshot storage, comparison, and update workflow.

use crate::diff::{DiffConfig, DiffReport, compare_images};
use anyhow::{Context, Result};
use gloomy_core::widget::Widget;
use image::RgbaImage;
use std::path::{Path, PathBuf};

/// Manages golden reference images on disk.
///
/// Directory layout under `base_dir`:
/// ```text
/// snapshots/
///   golden/   — reference PNGs
///   actual/   — current render output
///   diff/     — visual diff overlay PNGs
/// ```
pub struct SnapshotManager {
  golden_dir: PathBuf,
  actual_dir: PathBuf,
  diff_dir: PathBuf,
}

impl SnapshotManager {
  /// Creates a new manager rooted at `base_dir`.
  ///
  /// Directories are created lazily on first write.
  pub fn new(base_dir: impl AsRef<Path>) -> Self {
    let base = base_dir.as_ref().to_path_buf();
    Self {
      golden_dir: base.join("golden"),
      actual_dir: base.join("actual"),
      diff_dir: base.join("diff"),
    }
  }

  /// Saves (or overwrites) the golden reference for `name`.
  pub fn update_golden(
    &self,
    name: &str,
    image: &RgbaImage,
  ) -> Result<PathBuf> {
    std::fs::create_dir_all(&self.golden_dir)
      .context("create golden dir")?;
    let path = self.golden_path(name);
    image.save(&path).with_context(|| {
      format!("save golden image {}", path.display())
    })?;
    Ok(path)
  }

  /// Compares `actual` against the stored golden for `name`.
  ///
  /// Also writes the actual render and a visual diff overlay to
  /// their respective directories.
  pub fn compare(
    &self,
    name: &str,
    actual: &RgbaImage,
    config: &DiffConfig,
    widget_tree: Option<&Widget>,
  ) -> Result<DiffReport> {
    let golden_path = self.golden_path(name);
    let expected = image::open(&golden_path)
      .with_context(|| {
        format!(
          "missing golden image: {}. Run with 'update' first.",
          golden_path.display()
        )
      })?
      .to_rgba8();

    // Persist actual render for manual inspection.
    std::fs::create_dir_all(&self.actual_dir)
      .context("create actual dir")?;
    let actual_path = self.actual_dir.join(format!("{name}.png"));
    actual
      .save(&actual_path)
      .context("save actual image")?;

    let report =
      compare_images(&expected, actual, config, widget_tree);

    // Generate visual diff overlay.
    if !report.passed {
      self.save_diff_image(
        name, &expected, actual, config,
      )?;
    }

    Ok(report)
  }

  /// Generates a visual diff image where unchanged pixels are
  /// dimmed 50% and changed pixels are highlighted in magenta.
  pub fn save_diff_image(
    &self,
    name: &str,
    expected: &RgbaImage,
    actual: &RgbaImage,
    config: &DiffConfig,
  ) -> Result<PathBuf> {
    std::fs::create_dir_all(&self.diff_dir)
      .context("create diff dir")?;

    let w = expected.width();
    let h = expected.height();
    let mut diff_img = RgbaImage::new(w, h);
    let exp_raw = expected.as_raw();
    let act_raw = actual.as_raw();

    for i in 0..((w * h) as usize) {
      let base = i * 4;
      let dr = exp_raw[base].abs_diff(act_raw[base]);
      let dg = exp_raw[base + 1].abs_diff(act_raw[base + 1]);
      let db = exp_raw[base + 2].abs_diff(act_raw[base + 2]);
      let da = exp_raw[base + 3].abs_diff(act_raw[base + 3]);
      let max_d = dr.max(dg).max(db).max(da);

      let px = if max_d > config.threshold {
        // Magenta overlay for changed pixels.
        image::Rgba([255, 0, 255, 255])
      } else {
        // Dim unchanged pixels by 50%.
        image::Rgba([
          act_raw[base] / 2,
          act_raw[base + 1] / 2,
          act_raw[base + 2] / 2,
          act_raw[base + 3],
        ])
      };

      let x = (i as u32) % w;
      let y = (i as u32) / w;
      diff_img.put_pixel(x, y, px);
    }

    let path = self.diff_dir.join(format!("{name}.png"));
    diff_img
      .save(&path)
      .context("save diff image")?;
    Ok(path)
  }

  /// Returns the golden image path for a given snapshot name.
  pub fn golden_path(&self, name: &str) -> PathBuf {
    self.golden_dir.join(format!("{name}.png"))
  }

  /// Lists all golden snapshot names (without extension).
  pub fn list_goldens(&self) -> Result<Vec<String>> {
    if !self.golden_dir.exists() {
      return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in
      std::fs::read_dir(&self.golden_dir).context("read golden dir")?
    {
      let entry = entry?;
      let path = entry.path();
      if path.extension().map_or(false, |e| e == "png") {
        if let Some(stem) = path.file_stem() {
          names.push(stem.to_string_lossy().into_owned());
        }
      }
    }
    names.sort();
    Ok(names)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn solid_image(
    w: u32,
    h: u32,
    rgba: [u8; 4],
  ) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for p in img.pixels_mut() {
      *p = image::Rgba(rgba);
    }
    img
  }

  #[test]
  fn update_and_compare_identical() {
    let dir = tempfile::tempdir().unwrap();
    let mgr = SnapshotManager::new(dir.path());
    let img = solid_image(32, 32, [100, 50, 25, 255]);

    let path = mgr.update_golden("test1", &img).unwrap();
    assert!(path.exists());

    let report = mgr
      .compare("test1", &img, &DiffConfig::default(), None)
      .unwrap();
    assert!(report.passed);
    assert_eq!(report.diff_pixels, 0);
  }

  #[test]
  fn missing_golden_returns_error() {
    let dir = tempfile::tempdir().unwrap();
    let mgr = SnapshotManager::new(dir.path());
    let img = solid_image(32, 32, [0, 0, 0, 255]);
    let result =
      mgr.compare("nonexistent", &img, &DiffConfig::default(), None);
    assert!(result.is_err());
    let msg = format!("{:#}", result.unwrap_err());
    assert!(msg.contains("missing golden image"));
  }
}
