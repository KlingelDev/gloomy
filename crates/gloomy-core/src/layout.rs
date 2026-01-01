//! Layout primitives for Flexbox-style positioning.

use serde::{Deserialize, Serialize};

/// Layout direction for containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Direction {
  /// Widgets are arranged horizontally.
  Row,
  /// Widgets are arranged vertically.
  Column,
  /// Grid layout with fixed number of columns.
  Grid { columns: usize },
  /// Absolute positioning (no auto-layout).
  None,
}

impl Default for Direction {
  fn default() -> Self {
    Self::Column
  }
}

/// Alignment of items along the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Align {
  /// Align to start of cross axis.
  Start,
  /// Align to center of cross axis.
  Center,
  /// Align to end of cross axis.
  End,
  /// Stretch to fill cross axis.
  Stretch,
}

impl Default for Align {
  fn default() -> Self {
    Self::Start
  }
}

/// Justification of content along the main axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum Justify {
  /// Pack items to start.
  Start,
  /// Pack items to center.
  Center,
  /// Pack items to end.
  End,
  /// Distribute items with space between them.
  SpaceBetween,
  /// Distribute items with space around them.
  SpaceAround,
}

impl Default for Justify {
  fn default() -> Self {
    Self::Start
  }
}

/// Size definition for a grid track (column/row).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub enum TrackSize {
  /// User-defined fixed size in pixels.
  Px(f32),
  /// Flexible fraction of remaining space.
  Fr(f32),
  /// Auto-sizing based on content (currently mimics Fr(1.0) or min-content in future).
  Auto,
}

impl Default for TrackSize {
    fn default() -> Self {
        Self::Auto
    }
}

/// complete Layout configuration for a container.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Layout {
  #[serde(default)]
  pub direction: Direction,
  #[serde(default)]
  pub spacing: f32,
  #[serde(default)]
  pub align_items: Align,
  #[serde(default)]
  pub justify_content: Justify,
  #[serde(default)]
  pub template_columns: Vec<TrackSize>,
}
