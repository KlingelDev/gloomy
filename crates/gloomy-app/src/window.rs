//! Window wrapper managing wgpu surface and rendering.

use gloomy_core::GloomyRenderer;
use std::sync::Arc;
use winit::window::Window;

/// A gloomy window with its own rendering context.
pub struct GloomyWindow {
  pub window: Arc<Window>,
  surface: wgpu::Surface<'static>,
  pub config: wgpu::SurfaceConfiguration,
  pub renderer: GloomyRenderer,
}

impl GloomyWindow {
  /// Creates a new gloomy window.
  pub fn new(
    window: Arc<Window>,
    instance: &wgpu::Instance,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
  ) -> anyhow::Result<Self> {
    let size = window.inner_size();
    let surface = instance.create_surface(window.clone())?;

    let caps = surface.get_capabilities(adapter);
    let format = caps
      .formats
      .iter()
      .find(|f| f.is_srgb())
      .copied()
      .unwrap_or(caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      format,
      width: size.width.max(1),
      height: size.height.max(1),
      present_mode: wgpu::PresentMode::AutoVsync,
      alpha_mode: caps.alpha_modes[0],
      view_formats: vec![],
      desired_maximum_frame_latency: 2,
    };
    surface.configure(device, &config);

    let renderer =
      GloomyRenderer::new(device, format, config.width, config.height, window.scale_factor() as f32);

    Ok(Self { window, surface, config, renderer })
  }

  /// Handles window resize.
  pub fn resize(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
  ) {
    if width == 0 || height == 0 {
      return;
    }
    self.config.width = width;
    self.config.height = height;
    self.surface.configure(device, &self.config);
    self.renderer.resize(queue, width, height, self.window.scale_factor() as f32);
  }

  /// Renders a frame.
  pub fn render(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) -> anyhow::Result<()> {
    let output = self.surface.get_current_texture()?;
    let view =
      output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Check for size mismatch (e.g. if window system gave us different size than configured)
    let width = output.texture.width();
    let height = output.texture.height();
    if width != self.config.width || height != self.config.height {
        log::warn!("Surface size mismatch! Config: {}x{}, Texture: {}x{}. Resizing renderer.", 
                   self.config.width, self.config.height, width, height);
        self.config.width = width;
        self.config.height = height;
        self.renderer.resize(queue, width, height, self.window.scale_factor() as f32);
    }

    self.renderer.prepare(device, queue);

    let mut encoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("GloomyEncoder"),
      });

    self.renderer.render(&mut encoder, &view, device, queue);

    queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
  }

  /// Returns the window ID.
  pub fn id(&self) -> winit::window::WindowId {
    self.window.id()
  }
}
