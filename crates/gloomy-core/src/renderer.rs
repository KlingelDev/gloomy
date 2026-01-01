//! GPU context and rendering orchestration.

use crate::image_renderer::ImageRenderer;
use crate::primitives::PrimitiveRenderer;
use crate::text::TextRenderer;
use crate::texture::Texture;
use std::collections::HashMap;
use glam::{Vec2, Vec4};

/// Default embedded font (Inter Regular).
const DEFAULT_FONT: &[u8] = include_bytes!("fonts/Inter-Regular.ttf");

/// Main renderer managing GPU resources and render passes.
pub struct GloomyRenderer {
  primitives: PrimitiveRenderer,
  text: TextRenderer,
  images: ImageRenderer,
  textures: HashMap<String, Texture>,
  width: u32,
  height: u32,
  clear_color: wgpu::Color,
}

impl GloomyRenderer {
  /// Creates a new renderer with the given device and surface format.
  ///
  /// # Arguments
  /// * `device` - The wgpu device
  /// * `format` - The surface texture format
  /// * `width` - Initial viewport width
  /// * `height` - Initial viewport height
  pub fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
  ) -> Self {
    let primitives = PrimitiveRenderer::new(device, format, width, height);
    let text = TextRenderer::new(device, format, width, height, DEFAULT_FONT);
    let images = ImageRenderer::new(device, format, width, height);

    Self {
      primitives,
      text,
      images,
      textures: HashMap::new(),
      width,
      height,
      clear_color: wgpu::Color { r: 0.1, g: 0.1, b: 0.12, a: 1.0 },
    }
  }

  /// Creates a renderer with a custom font.
  pub fn with_font(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    font_bytes: &[u8],
  ) -> Self {
    let primitives = PrimitiveRenderer::new(device, format, width, height);
    let text = TextRenderer::new(device, format, width, height, font_bytes);
    let images = ImageRenderer::new(device, format, width, height);

    Self {
      primitives,
      text,
      images,
      textures: HashMap::new(),
      width,
      height,
      clear_color: wgpu::Color { r: 0.1, g: 0.1, b: 0.12, a: 1.0 },
    }
  }

  /// Registers a texture with a given name.
  pub fn register_texture(&mut self, name: String, texture: Texture) {
      self.textures.insert(name, texture);
  }

  /// Handles viewport resize.
  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
    self.width = width;
    self.height = height;
    self.primitives.resize(queue, width, height);
    self.text.resize(queue, width, height);
    self.images.resize(queue, width, height);
  }

  /// Returns the current viewport size.
  pub fn size(&self) -> Vec2 {
    Vec2::new(self.width as f32, self.height as f32)
  }

  /// Access the primitive renderer for drawing shapes.
  pub fn primitives(&mut self) -> &mut PrimitiveRenderer {
    &mut self.primitives
  }

  /// Access the text renderer.
  pub fn text(&mut self) -> &mut TextRenderer {
    &mut self.text
  }

  /// Access the image renderer.
  pub fn images(&mut self) -> &mut ImageRenderer {
    &mut self.images
  }

  /// Access the texture cache.
  pub fn textures(&mut self) -> &mut HashMap<String, Texture> {
    &mut self.textures
  }

  /// Splits the renderer into mutable references components.
  pub fn split_mut(
    &mut self,
  ) -> (&mut PrimitiveRenderer, &mut TextRenderer, &mut ImageRenderer, &mut HashMap<String, Texture>) {
    (&mut self.primitives, &mut self.text, &mut self.images, &mut self.textures)
  }

  /// Draws text at the specified position.
  pub fn draw_text(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text: &str,
    pos: Vec2,
    size: f32,
    color: Vec4,
  ) {
    self.text.draw(device, queue, text, pos, size, color);
  }

  /// Sets the clear color for the background.
  pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
    self.clear_color = wgpu::Color { r, g, b, a };
  }

  /// Prepares all draw commands for submission.
  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    self.primitives.prepare(device, queue);
    self.images.prepare(device, queue);
  }

  /// Renders a frame to the given texture view.
  pub fn render(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) {
    // Primitives pass
    {
      let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("GloomyPrimitivesPass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(self.clear_color),
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
      self.primitives.render(&mut rp);
    }

    // Images pass (before text)
    {
        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("GloomyImagePass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        self.images.render(&mut rp);
    }
    self.images.clear(); // Clear instance list after render

    // Text pass (now manages its own passes)
    self.text.render(encoder, view, device, queue);
  }
}
