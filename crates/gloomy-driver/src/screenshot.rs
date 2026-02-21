//! Headless wgpu rendering to in-memory images.

use anyhow::{Context, Result};
use gloomy_core::data_source::DataProvider;
use gloomy_core::widget::Widget;
use gloomy_core::{
  InteractionState, GloomyRenderer, compute_layout, render_ui,
};
use image::RgbaImage;

/// Texture format used for headless rendering.
///
/// Matches the sRGB preference from gloomy-app's windowed surface
/// selection (`caps.formats.iter().find(|f| f.is_srgb())`).
const HEADLESS_FORMAT: wgpu::TextureFormat =
  wgpu::TextureFormat::Rgba8UnormSrgb;

/// GPU resources for headless rendering.
pub struct HeadlessRenderer {
  device: wgpu::Device,
  queue: wgpu::Queue,
  renderer: GloomyRenderer,
  width: u32,
  height: u32,
}

impl HeadlessRenderer {
  /// Creates a new headless renderer.
  ///
  /// Requests a wgpu adapter with no surface (headless), then
  /// builds a `GloomyRenderer` targeting `Rgba8UnormSrgb`.
  pub fn new(
    width: u32,
    height: u32,
    scale_factor: f32,
  ) -> Result<Self> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
      backends: wgpu::Backends::all(),
      ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(
      &wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
      },
    ))
    .context("no suitable wgpu adapter found")?;

    let (device, queue) = pollster::block_on(
      adapter.request_device(
        &wgpu::DeviceDescriptor {
          label: Some("HeadlessDevice"),
          required_features: wgpu::Features::empty(),
          required_limits: wgpu::Limits::default(),
        },
        None,
      ),
    )
    .context("failed to create wgpu device")?;

    let mut renderer = GloomyRenderer::new(
      &device,
      HEADLESS_FORMAT,
      width,
      height,
      scale_factor,
    );
    // Write initial screen_size to GPU uniform buffers.
    // GloomyRenderer::new sets struct fields but doesn't
    // write uniforms; the windowed path relies on resize()
    // being called on first frame.
    renderer.resize(&queue, width, height, scale_factor);

    Ok(Self { device, queue, renderer, width, height })
  }

  /// Renders the widget tree to an in-memory RGBA image.
  ///
  /// Performs layout, populates draw commands via `render_ui`,
  /// renders to an offscreen texture, and copies the result to
  /// a CPU-accessible buffer.
  pub fn render_to_image(
    &mut self,
    root: &mut Widget,
    interaction: Option<&InteractionState>,
    data_provider: Option<&dyn DataProvider>,
  ) -> Result<RgbaImage> {
    let w = self.width;
    let h = self.height;
    let logical_w = w as f32 / self.renderer.scale_factor;
    let logical_h = h as f32 / self.renderer.scale_factor;

    // Set root container dimensions and compute layout.
    if let Widget::Container { bounds, .. } = root {
      bounds.width = logical_w;
      bounds.height = logical_h;
    }
    compute_layout(root, 0.0, 0.0, logical_w, logical_h);

    // Populate draw commands.
    render_ui(
      root,
      &mut self.renderer,
      &self.device,
      &self.queue,
      interaction,
      data_provider,
    );

    // Create offscreen texture.
    let texture = self.device.create_texture(
      &wgpu::TextureDescriptor {
        label: Some("HeadlessTarget"),
        size: wgpu::Extent3d {
          width: w,
          height: h,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: HEADLESS_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
          | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
      },
    );
    let view = texture.create_view(
      &wgpu::TextureViewDescriptor::default(),
    );

    // Prepare GPU buffers and render.
    self.renderer.prepare(&self.device, &self.queue);
    let mut encoder = self.device.create_command_encoder(
      &wgpu::CommandEncoderDescriptor {
        label: Some("HeadlessEncoder"),
      },
    );
    self.renderer.render(
      &mut encoder, &view, &self.device, &self.queue,
    );

    // Copy texture to staging buffer (256-byte row alignment).
    let bytes_per_pixel = 4u32;
    let unpadded_row = w * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded_row =
      (unpadded_row + align - 1) / align * align;
    let buf_size = (padded_row * h) as u64;

    let staging = self.device.create_buffer(
      &wgpu::BufferDescriptor {
        label: Some("HeadlessStaging"),
        size: buf_size,
        usage: wgpu::BufferUsages::MAP_READ
          | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      },
    );

    encoder.copy_texture_to_buffer(
      wgpu::ImageCopyTexture {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::ImageCopyBuffer {
        buffer: &staging,
        layout: wgpu::ImageDataLayout {
          offset: 0,
          bytes_per_row: Some(padded_row),
          rows_per_image: Some(h),
        },
      },
      wgpu::Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
      },
    );

    self.queue.submit(std::iter::once(encoder.finish()));

    // Map and read back pixel data.
    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
      tx.send(result).ok();
    });
    self.device.poll(wgpu::Maintain::Wait);
    rx.recv()
      .context("buffer map channel closed")?
      .context("buffer map failed")?;

    let mapped = slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for row in 0..h {
      let start = (row * padded_row) as usize;
      let end = start + unpadded_row as usize;
      pixels.extend_from_slice(&mapped[start..end]);
    }
    drop(mapped);
    staging.unmap();

    RgbaImage::from_raw(w, h, pixels)
      .context("failed to build RgbaImage from pixel data")
  }

  /// Returns the pixel width.
  pub fn width(&self) -> u32 {
    self.width
  }

  /// Returns the pixel height.
  pub fn height(&self) -> u32 {
    self.height
  }

  /// Provides mutable access to the inner `GloomyRenderer`.
  pub fn renderer_mut(&mut self) -> &mut GloomyRenderer {
    &mut self.renderer
  }

  /// Provides a reference to the wgpu device.
  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }

  /// Provides a reference to the wgpu queue.
  pub fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use gloomy_core::widget::WidgetBounds;

  fn empty_container(w: f32, h: f32) -> Widget {
    let mut root = Widget::container();
    if let Widget::Container { bounds, .. } = &mut root {
      bounds.width = w;
      bounds.height = h;
    }
    root
  }

  #[test]
  fn render_empty_has_correct_dimensions() {
    let mut renderer =
      HeadlessRenderer::new(200, 100, 1.0).unwrap();
    let mut root = empty_container(200.0, 100.0);
    let img = renderer
      .render_to_image(&mut root, None, None)
      .unwrap();
    assert_eq!(img.width(), 200);
    assert_eq!(img.height(), 100);
  }

  #[test]
  fn render_is_deterministic() {
    let mut renderer =
      HeadlessRenderer::new(64, 64, 1.0).unwrap();
    let mut root = empty_container(64.0, 64.0);
    let a = renderer
      .render_to_image(&mut root, None, None)
      .unwrap();
    let b = renderer
      .render_to_image(&mut root, None, None)
      .unwrap();
    assert_eq!(a.as_raw(), b.as_raw());
  }
}
