//! Gloomy App - Window management and event loop for gloomy applications.
//!
//! Provides:
//! - Single and multi-window support
//! - Event loop management
//! - Keyboard-centric input handling

mod app;
mod window;

pub use app::{DrawContext, GloomyApp};
pub use gloomy_core::{
  compute_layout, hit_test, load_ui, parse_ui, render_ui, Align, Container,
  Direction, GloomyRenderer, Instance, InteractionState, Justify, Layout,
  PrimitiveRenderer, Rect, RenderContext, TextRenderer, Vec2, Vec4, Widget,
  WidgetBounds,
};
pub use window::GloomyWindow;
