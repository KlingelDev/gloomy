//! GPU context and rendering orchestration.

use crate::image_renderer::ImageRenderer;
use crate::primitives::PrimitiveRenderer;
use crate::text::TextRenderer;
use crate::texture::Texture;
use std::collections::HashMap;
use glam::{Vec2, Vec4};

/// Default embedded font (Inter Regular).
const DEFAULT_FONT: &[u8] = include_bytes!("fonts/Inter-Regular.ttf");

// Inter font family (Variable fonts)
const INTER_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/google/InterVariable.ttf");
const INTER_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/google/InterVariable-Italic.ttf");

// Roboto font family variants for rich text support
const ROBOTO_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/google/Roboto-Regular.ttf");
const ROBOTO_BOLD: &[u8] = include_bytes!("../../../assets/fonts/google/Roboto-Bold.ttf");
const ROBOTO_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/google/Roboto-Italic.ttf");
const ROBOTO_BOLD_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/google/Roboto-BoldItalic.ttf");

// RobotoCondensed font family
const ROBOTO_CONDENSED_REGULAR: &[u8] = include_bytes!("../../../assets/fonts/google/RobotoCondensed-Regular.ttf");
const ROBOTO_CONDENSED_BOLD: &[u8] = include_bytes!("../../../assets/fonts/google/RobotoCondensed-Bold.ttf");
const ROBOTO_CONDENSED_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/google/RobotoCondensed-Italic.ttf");
const ROBOTO_CONDENSED_BOLD_ITALIC: &[u8] = include_bytes!("../../../assets/fonts/google/RobotoCondensed-BoldItalic.ttf");

/// Main renderer managing GPU resources and render passes.
pub struct GloomyRenderer {
  primitives: PrimitiveRenderer,
  text: TextRenderer,
  // Overlay renderers for deferred content (dropdowns, tooltips)
  overlay_primitives: PrimitiveRenderer,
  overlay_text: TextRenderer,
  images: ImageRenderer,
  textures: HashMap<String, Texture>,
  width: u32,
  height: u32,
  pub scale_factor: f32,
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
    scale_factor: f32,
  ) -> Self {
    let primitives = PrimitiveRenderer::new(device, format, width, height);
    let overlay_primitives = PrimitiveRenderer::new(device, format, width, height);
    
    // Load all available font families (10 fonts total)
    let text = TextRenderer::new_with_all_families(
      device,
      format,
      width,
      height,
      INTER_REGULAR,
      INTER_ITALIC,
      ROBOTO_REGULAR,
      ROBOTO_BOLD,
      ROBOTO_ITALIC,
      ROBOTO_BOLD_ITALIC,
      ROBOTO_CONDENSED_REGULAR,
      ROBOTO_CONDENSED_BOLD,
      ROBOTO_CONDENSED_ITALIC,
      ROBOTO_CONDENSED_BOLD_ITALIC,
    );

    // Initial output text renderer for overlay (lightweight, default font only)
    let overlay_text = TextRenderer::new(
      device,
      format,
      width,
      height,
      DEFAULT_FONT,
    );
    
    let images = ImageRenderer::new(device, format, width, height);

    Self {
      primitives,
      text,
      overlay_primitives,
      overlay_text,
      images,
      textures: HashMap::new(),
      width,
      height,
      scale_factor,
      clear_color: wgpu::Color { r: 0.1, g: 0.1, b: 0.12, a: 1.0 },
    }
  }

  /// Creates a renderer with a custom font.
  pub fn with_font(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    scale_factor: f32,
    font_bytes: &[u8],
  ) -> Self {
    let primitives = PrimitiveRenderer::new(device, format, width, height);
    let overlay_primitives = PrimitiveRenderer::new(device, format, width, height);
    let text = TextRenderer::new(device, format, width, height, font_bytes);
    let overlay_text = TextRenderer::new(device, format, width, height, font_bytes);
    let images = ImageRenderer::new(device, format, width, height);

    Self {
      primitives,
      text,
      overlay_primitives,
      overlay_text,
      images,
      textures: HashMap::new(),
      width,
      height,
      scale_factor,
      clear_color: wgpu::Color { r: 0.1, g: 0.1, b: 0.12, a: 1.0 },
    }
  }

  /// Registers a texture with a given name.
  pub fn register_texture(&mut self, name: String, texture: Texture) {
      self.textures.insert(name, texture);
  }

  /// Handles viewport resize.
  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32, scale_factor: f32) {
    log::info!("GloomyRenderer::resize: {}x{} @ {}", width, height, scale_factor);
    self.width = width;
    self.height = height;
    self.scale_factor = scale_factor;
    self.primitives.resize(queue, width, height, scale_factor);
    self.overlay_primitives.resize(queue, width, height, scale_factor);
    self.text.resize(queue, width, height, scale_factor);
    self.overlay_text.resize(queue, width, height, scale_factor);
    self.images.resize(queue, width, height);
  }

  /// Returns the current viewport size (Logical).
  pub fn size(&self) -> Vec2 {
    Vec2::new(
        self.width as f32 / self.scale_factor, 
        self.height as f32 / self.scale_factor
    )
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

  /// Adds a new font to the renderer.
  pub fn add_font(&mut self, name: &str, font_bytes: &[u8]) {
    self.text.add_font(name, font_bytes);
  }

  /// Draws text at the specified position with an optional font.
  pub fn draw_text_with_font(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text: &str,
    pos: Vec2,
    size: f32,
    color: Vec4,
    font_name: Option<&str>,
  ) {
    self.text.draw(device, queue, text, pos, size, color, wgpu_text::glyph_brush::HorizontalAlign::Left, font_name);
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
    self.text.draw(device, queue, text, pos, size, color, wgpu_text::glyph_brush::HorizontalAlign::Left, None);
  }

  /// Sets the clear color for the background.
  pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64, a: f64) {
    self.clear_color = wgpu::Color { r, g, b, a };
  }

  /// Prepares all draw commands for submission.
  pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
    self.primitives.prepare(device, queue);
    self.overlay_primitives.prepare(device, queue);
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
    // Text pass (now manages its own passes)
    self.text.render(encoder, view, device, queue);
    
    // --- Overlay Primitives Pass ---
    {
      let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("GloomyOverlayPrimitivesPass"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load, // Overlay on top of existing
            store: wgpu::StoreOp::Store,
          },
        })],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None,
      });
      self.overlay_primitives.render(&mut rp);
    }
    
    // --- Overlay Text Pass ---
    self.overlay_text.render(encoder, view, device, queue);
  }

  /// Splits renderer to access main layers mutably
  pub fn split_mut(&mut self) -> (&mut PrimitiveRenderer, &mut TextRenderer, &mut ImageRenderer, &mut HashMap<String, Texture>) {
      (&mut self.primitives, &mut self.text, &mut self.images, &mut self.textures)
  }

  /// Splits renderer to access overlay layers mutably
  pub fn split_overlay_mut(&mut self) -> (&mut PrimitiveRenderer, &mut TextRenderer) {
      (&mut self.overlay_primitives, &mut self.overlay_text)
  }
}
