//! Container system for hierarchical UI layout.

use crate::primitives::PrimitiveRenderer;
use crate::rect::Rect;
use glam::Vec4;

/// A container that can hold child containers.
///
/// The root container typically represents the entire window.
/// Child containers are positioned relative to their parent.
pub struct Container {
  /// Bounds of this container
  bounds: Rect,
  /// Child containers
  children: Vec<Container>,
  /// Padding inside the container
  padding: f32,
  /// Background color (None for transparent)
  background: Option<Vec4>,
  /// Corner radius for background
  corner_radius: f32,
}

impl Container {
  /// Creates a new container with the given bounds.
  pub fn new(bounds: Rect) -> Self {
    Self {
      bounds,
      children: Vec::new(),
      padding: 0.0,
      background: None,
      corner_radius: 0.0,
    }
  }

  /// Creates a root container for the entire window.
  pub fn root(width: f32, height: f32) -> Self {
    Self::new(Rect::from_size(width, height))
  }

  /// Sets the padding inside the container.
  pub fn with_padding(mut self, padding: f32) -> Self {
    self.padding = padding;
    self
  }

  /// Sets the background color.
  pub fn with_background(mut self, color: Vec4) -> Self {
    self.background = Some(color);
    self
  }

  /// Sets the corner radius for the background.
  pub fn with_corner_radius(mut self, radius: f32) -> Self {
    self.corner_radius = radius;
    self
  }

  /// Adds a child container.
  pub fn add_child(&mut self, child: Container) {
    self.children.push(child);
  }

  /// Returns the content bounds (bounds minus padding).
  pub fn content_bounds(&self) -> Rect {
    self.bounds.inset(self.padding)
  }

  /// Returns the bounds of this container.
  pub fn bounds(&self) -> Rect {
    self.bounds
  }

  /// Sets new bounds for this container.
  pub fn set_bounds(&mut self, bounds: Rect) {
    self.bounds = bounds;
  }

  /// Returns a mutable reference to children.
  pub fn children_mut(&mut self) -> &mut Vec<Container> {
    &mut self.children
  }

  /// Draws the container and all children.
  pub fn draw(&self, primitives: &mut PrimitiveRenderer) {
    // Draw background if set
    if let Some(bg) = self.background {
      let half_size = self.bounds.size() * 0.5;
      primitives.draw_rect(
        self.bounds.center(),
        half_size,
        bg,
        [self.corner_radius; 4],
        0.0,
      );
    }

    // Recursively draw children
    for child in &self.children {
      child.draw(primitives);
    }
  }
}
