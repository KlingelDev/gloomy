//! Hello Gloomy - Example demonstrating text and containers.
//!
//! Press 'q' or Escape to quit.

use gloomy_app::{Container, DrawContext, GloomyApp, Rect, Vec2, Vec4};

fn main() -> anyhow::Result<()> {
  env_logger::init();

  GloomyApp::new()
    .on_draw(|window, ctx| {
      let size = window.renderer.size();

      // Root container (entire window)
      let mut root = Container::root(size.x, size.y);

      // Add a header panel
      let header = Container::new(Rect::new(20.0, 20.0, size.x - 40.0, 60.0))
        .with_background(Vec4::new(0.2, 0.2, 0.25, 1.0))
        .with_corner_radius(8.0)
        .with_padding(10.0);
      root.add_child(header);

      // Add a sidebar
      let sidebar =
        Container::new(Rect::new(20.0, 100.0, 200.0, size.y - 120.0))
          .with_background(Vec4::new(0.15, 0.15, 0.18, 1.0))
          .with_corner_radius(8.0);
      root.add_child(sidebar);

      // Add main content area
      let content = Container::new(Rect::new(
        240.0,
        100.0,
        size.x - 260.0,
        size.y - 120.0,
      ))
      .with_background(Vec4::new(0.18, 0.18, 0.22, 1.0))
      .with_corner_radius(8.0);
      root.add_child(content);

      // Draw all containers
      root.draw(window.renderer.primitives());

      // Draw text labels
      let white = Vec4::new(1.0, 1.0, 1.0, 1.0);
      let gray = Vec4::new(0.7, 0.7, 0.7, 1.0);

      // Header title
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "Gloomy UI Demo",
        Vec2::new(40.0, 35.0),
        24.0,
        white,
      );

      // Sidebar items
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "Dashboard",
        Vec2::new(40.0, 120.0),
        16.0,
        white,
      );
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "Data View",
        Vec2::new(40.0, 150.0),
        16.0,
        gray,
      );
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "Settings",
        Vec2::new(40.0, 180.0),
        16.0,
        gray,
      );

      // Content area
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "Welcome to Gloomy",
        Vec2::new(260.0, 120.0),
        20.0,
        white,
      );
      window.renderer.draw_text(
        ctx.device,
        ctx.queue,
        "A keyboard-centric UI library for data-heavy applications.",
        Vec2::new(260.0, 160.0),
        14.0,
        gray,
      );
    })
    .run()
}
