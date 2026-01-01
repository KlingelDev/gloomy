//! Rectangle bounds for layout calculations.

use glam::Vec2;

/// A rectangle defined by position and size.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
  /// X position (left edge)
  pub x: f32,
  /// Y position (top edge)
  pub y: f32,
  /// Width
  pub width: f32,
  /// Height
  pub height: f32,
}

impl Rect {
  /// Creates a new rectangle.
  pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
    Self { x, y, width, height }
  }

  /// Creates a rectangle at origin with given size.
  pub fn from_size(width: f32, height: f32) -> Self {
    Self { x: 0.0, y: 0.0, width, height }
  }

  /// Creates a rectangle from a Vec2 size.
  pub fn from_vec2(size: Vec2) -> Self {
    Self::from_size(size.x, size.y)
  }

  /// Returns the center position.
  pub fn center(&self) -> Vec2 {
    Vec2::new(self.x + self.width * 0.5, self.y + self.height * 0.5)
  }

  /// Returns the top-left position.
  pub fn top_left(&self) -> Vec2 {
    Vec2::new(self.x, self.y)
  }

  /// Returns the size as Vec2.
  pub fn size(&self) -> Vec2 {
    Vec2::new(self.width, self.height)
  }

  /// Returns a rect inset by the given amount on all sides.
  pub fn inset(&self, amount: f32) -> Self {
    Self {
      x: self.x + amount,
      y: self.y + amount,
      width: (self.width - amount * 2.0).max(0.0),
      height: (self.height - amount * 2.0).max(0.0),
    }
  }

  /// Checks if a point is inside the rectangle.
  pub fn contains(&self, point: Vec2) -> bool {
    point.x >= self.x
      && point.x <= self.x + self.width
      && point.y >= self.y
      && point.y <= self.y + self.height
  }
}
