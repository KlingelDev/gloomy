//! Headless rendering support for offscreen GPU rendering.
//!
//! Provides `HeadlessRenderer` which creates a wgpu device without
//! a window or display server, renders widget trees to offscreen
//! textures, and reads back pixel data as `image::RgbaImage`.

use crate::renderer::GloomyRenderer;
use crate::widget::Widget;
use crate::{compute_layout, render_ui};
use crate::data_source::DataProvider;
use crate::interaction::InteractionState;

/// Configuration for headless rendering.
pub struct HeadlessConfig {
  /// Viewport width in pixels.
  pub width: u32,
  /// Viewport height in pixels.
  pub height: u32,
  /// Force software rendering (lavapipe/swiftshader).
  pub force_fallback_adapter: bool,
  /// Scale factor for high-DPI rendering.
  pub scale_factor: f32,
}

impl Default for HeadlessConfig {
  fn default() -> Self {
    Self {
      width: 1280,
      height: 720,
      force_fallback_adapter: false,
      scale_factor: 1.0,
    }
  }
}

/// Offscreen renderer that works without a window or display.
pub struct HeadlessRenderer {
  device: wgpu::Device,
  queue: wgpu::Queue,
  texture: wgpu::Texture,
  renderer: GloomyRenderer,
  width: u32,
  height: u32,
}

impl HeadlessRenderer {
  /// Creates a new headless renderer.
  ///
  /// Attempts hardware GPU first, then falls back to software
  /// rendering if `force_fallback_adapter` is set or no hardware
  /// GPU is available.
  pub fn new(config: HeadlessConfig) -> anyhow::Result<Self> {
    Self::new_async(config)
  }

  fn new_async(config: HeadlessConfig) -> anyhow::Result<Self> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let adapter = pollster::block_on(async {
      // Try with requested fallback setting first.
      let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::default(),
          compatible_surface: None,
          force_fallback_adapter: config.force_fallback_adapter,
        },
      ).await;
      if adapter.is_some() || config.force_fallback_adapter {
        return adapter;
      }
      // If hardware failed and we didn't force fallback, try
      // software as a last resort.
      instance.request_adapter(
        &wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::default(),
          compatible_surface: None,
          force_fallback_adapter: true,
        },
      ).await
    }).ok_or_else(|| {
      anyhow::anyhow!("No suitable GPU adapter found")
    })?;

    let (device, queue) = pollster::block_on(
      adapter.request_device(
        &wgpu::DeviceDescriptor {
          label: Some("headless"),
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::downlevel_defaults(),
        },
        None,
      ),
    )?;

    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("headless_target"),
      size: wgpu::Extent3d {
        width: config.width,
        height: config.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::COPY_SRC,
      view_formats: &[],
    });

    let mut renderer = GloomyRenderer::new(
      &device,
      format,
      config.width,
      config.height,
      config.scale_factor,
    );

    // Initialize uniform buffers (screen_size etc.) that the
    // windowed path sets via the first resize event.
    renderer.resize(
      &queue,
      config.width,
      config.height,
      config.scale_factor,
    );

    Ok(Self {
      device,
      queue,
      texture,
      renderer,
      width: config.width,
      height: config.height,
    })
  }

  /// Returns a mutable reference to the underlying renderer.
  pub fn renderer_mut(&mut self) -> &mut GloomyRenderer {
    &mut self.renderer
  }

  /// Returns a reference to the wgpu device.
  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }

  /// Returns a reference to the wgpu queue.
  pub fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }

  /// Renders a widget tree and returns the pixel data.
  ///
  /// Runs layout, renders all widgets, then reads back the
  /// texture contents as an RGBA image.
  pub fn render_to_image(
    &mut self,
    widget: &mut Widget,
    interaction: Option<&InteractionState>,
    data_provider: Option<&dyn DataProvider>,
  ) -> anyhow::Result<image::RgbaImage> {
    let w = self.width as f32 / self.renderer.scale_factor;
    let h = self.height as f32 / self.renderer.scale_factor;

    // Set root bounds if container.
    if let Widget::Container { bounds, .. } = widget {
      bounds.width = w;
      bounds.height = h;
    }

    compute_layout(widget, 0.0, 0.0, w, h);

    render_ui(
      widget,
      &mut self.renderer,
      &self.device,
      &self.queue,
      interaction,
      data_provider,
    );

    self.renderer.prepare(&self.device, &self.queue);

    let view = self.texture.create_view(
      &wgpu::TextureViewDescriptor::default(),
    );
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor {
        label: Some("headless_render"),
      },
    );

    self.renderer.render(
      &mut encoder,
      &view,
      &self.device,
      &self.queue,
    );

    self.readback(encoder)
  }

  /// Renders a widget tree and saves the result as a PNG.
  pub fn save_screenshot(
    &mut self,
    widget: &mut Widget,
    path: impl AsRef<std::path::Path>,
    interaction: Option<&InteractionState>,
    data_provider: Option<&dyn DataProvider>,
  ) -> anyhow::Result<()> {
    let img = self.render_to_image(
      widget, interaction, data_provider,
    )?;
    img.save(path)?;
    Ok(())
  }

  /// Reads back the current texture contents as an RGBA image.
  fn readback(
    &self,
    mut encoder: wgpu::CommandEncoder,
  ) -> anyhow::Result<image::RgbaImage> {
    let bytes_per_pixel = 4u32;
    // wgpu requires rows aligned to 256 bytes.
    let unpadded_bytes_per_row = self.width * bytes_per_pixel;
    let padded_bytes_per_row = (unpadded_bytes_per_row + 255) & !255;

    let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("readback"),
      size: (padded_bytes_per_row * self.height) as u64,
      usage: wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::MAP_READ,
      mapped_at_creation: false,
    });

    encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: &self.texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &buffer,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(padded_bytes_per_row),
          rows_per_image: Some(self.height),
        },
      },
      wgpu::Extent3d {
        width: self.width,
        height: self.height,
        depth_or_array_layers: 1,
      },
    );

    self.queue.submit(std::iter::once(encoder.finish()));

    let slice = buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
      let _ = tx.send(result);
    });
    self.device.poll(wgpu::Maintain::Wait);
    rx.recv()??;

    let data = slice.get_mapped_range();
    let mut pixels =
      Vec::with_capacity((self.width * self.height * 4) as usize);
    for row in 0..self.height {
      let start = (row * padded_bytes_per_row) as usize;
      let end = start + (self.width * bytes_per_pixel) as usize;
      pixels.extend_from_slice(&data[start..end]);
    }
    drop(data);
    buffer.unmap();

    image::RgbaImage::from_raw(self.width, self.height, pixels)
      .ok_or_else(|| anyhow::anyhow!("Failed to create image"))
  }

  /// Resizes the offscreen render target.
  pub fn resize(&mut self, width: u32, height: u32) {
    self.width = width;
    self.height = height;

    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    self.texture =
      self.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("headless_target"),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      });

    self.renderer.resize(
      &self.queue,
      width,
      height,
      self.renderer.scale_factor,
    );
  }
}
