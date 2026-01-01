//! Text rendering using wgpu_text for TTF fonts.

use glam::{Vec2, Vec4};
use wgpu_text::glyph_brush::ab_glyph::FontArc;
use wgpu_text::glyph_brush::{Section, Text};
use wgpu_text::BrushBuilder;

use wgpu_text::glyph_brush::ab_glyph::{Font, ScaleFont};

/// Text renderer wrapping wgpu_text for TTF rendering.
pub struct TextRenderer {
  brush: wgpu_text::TextBrush<FontArc>,
  font: FontArc,
  width: u32,
  height: u32,
  pending: Vec<(String, Vec2, f32, Vec4, Option<(u32, u32, u32, u32)>)>,
  current_scissor: Option<(u32, u32, u32, u32)>,
  screen_size: Vec2,
}

impl TextRenderer {
  /// Creates a new text renderer with the given font.
  ///
  /// # Arguments
  /// * `device` - wgpu device
  /// * `format` - Surface texture format
  /// * `width` - Viewport width
  /// * `height` - Viewport height
  /// * `font_bytes` - TTF font data
  pub fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    font_bytes: &[u8],
  ) -> Self {
    let font = FontArc::try_from_vec(font_bytes.to_vec())
      .expect("Failed to load font");
      
    let brush =
      BrushBuilder::using_font(font.clone()).build(device, width, height, format);

    Self { 
        brush, 
        font, 
        width, 
        height, 
        pending: Vec::new(), 
        current_scissor: None, 
        screen_size: Vec2::new(width as f32, height as f32) 
    }
  }

  /// Handles viewport resize.
  pub fn resize(&mut self, queue: &wgpu::Queue, width: u32, height: u32) {
    self.width = width;
    self.height = height;
    self.screen_size = Vec2::new(width as f32, height as f32);
    self.brush.resize_view(width as f32, height as f32, queue);
  }
  
  /// Sets the current scissor rect.
  pub fn set_scissor(&mut self, rect: Option<(u32, u32, u32, u32)>) -> Option<(u32, u32, u32, u32)> {
      let old = self.current_scissor;
      self.current_scissor = rect;
      old
  }

  /// Queues text for rendering.
  pub fn draw(
    &mut self,
    _device: &wgpu::Device,
    _queue: &wgpu::Queue,
    text: &str,
    pos: Vec2,
    size: f32,
    color: Vec4,
  ) {
    // Store text for batched rendering with current scissor state
    self.pending.push((
        text.to_string(),
        pos,
        size,
        color,
        self.current_scissor,
    ));
  }

  /// Renders queued text.
  pub fn render(
    &mut self, 
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) {
      if self.pending.is_empty() { return; }
      
      // Drain pending items into a local Vec (takes ownership of strings)
      let items: Vec<_> = self.pending.drain(..).collect();
      
      // Build sections referencing the stored strings
      let sections: Vec<Section> = items.iter()
          .map(|(text, pos, size, color, _scissor)| {
              Section::default()
                  .add_text(
                      Text::new(text.as_str())
                        .with_scale(*size)
                        .with_color([color.x, color.y, color.z, color.w]),
                  )
                  .with_screen_position((pos.x, pos.y))
          }).collect();
      
      // Queue ALL sections at once
      self.brush.queue(device, queue, sections).unwrap();
      
      // Single render pass for all text
      {
          let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
              label: Some("GloomyTextPass"),
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

          // Full screen scissor (no clipping)
          rpass.set_scissor_rect(
              0, 0, 
              self.screen_size.x as u32, 
              self.screen_size.y as u32
          );
          
          self.brush.draw(&mut rpass);
      }
      
      self.current_scissor = None;
  }

  /// Measures the bounds of the given text.
  /// Returns (width, height).
  pub fn measure(&self, text: &str, size: f32) -> Vec2 {
      if text.is_empty() {
          return Vec2::new(0.0, size);
      }
      
      let scaled_font = self.font.as_scaled(size);
      let mut width = 0.0;
      
      // Simple cumulative advance measurement
      // Does not handle kerning or complex shaping perfectly, but better than estimation
      for c in text.chars() {
          let glyph_id = scaled_font.glyph_id(c);
          let advance = scaled_font.h_advance(glyph_id);
          width += advance;
      }
      
      Vec2::new(width, size)
  }
}
