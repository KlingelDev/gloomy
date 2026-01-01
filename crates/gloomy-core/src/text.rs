use wgpu_text::glyph_brush::ab_glyph::{Font, FontArc, ScaleFont};
use glam::{Vec2, Vec4};
use wgpu_text::{
  glyph_brush::{HorizontalAlign, Section, Text, FontId},
  BrushBuilder,
};
use std::collections::HashMap;

/// Text renderer wrapping wgpu_text for TTF rendering.
pub struct TextRenderer {
  brush: wgpu_text::TextBrush<FontArc>,
  fonts: HashMap<String, FontId>,
  font_instances: Vec<FontArc>,
  width: u32,
  height: u32,
  pending: Vec<(String, Vec2, f32, Vec4, Option<(u32, u32, u32, u32)>, HorizontalAlign, Option<String>)>,
  current_scissor: Option<(u32, u32, u32, u32)>,
  screen_size: Vec2,
}

impl TextRenderer {
  /// Creates a new text renderer with the given default font.
  pub fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    font_bytes: &[u8],
  ) -> Self {
    let font = FontArc::try_from_vec(font_bytes.to_vec())
      .expect("Failed to load font");
      
    let mut brush =
      BrushBuilder::using_font(font.clone()).build(device, width, height, format);

    let mut fonts = HashMap::new();
    fonts.insert("default".to_string(), FontId(0));
    
    Self { 
        brush, 
        fonts,
        font_instances: vec![font],
        width, 
        height, 
        pending: Vec::new(), 
        current_scissor: None, 
        screen_size: Vec2::new(width as f32, height as f32) 
    }
  }

  /// Adds a new font to the renderer.
  /// NOTE: wgpu-text 0.8 doesn't support adding fonts after initialization.
  /// This is a placeholder for future implementation.
  pub fn add_font(&mut self, name: &str, font_bytes: &[u8]) {
    // TODO: wgpu-text 0.8 doesn't have add_font method
    // We would need to rebuild the brush with all fonts
    // For now, just track the font name
    let font = FontArc::try_from_vec(font_bytes.to_vec())
      .expect("Failed to load font");
    let next_id = FontId(self.font_instances.len());
    self.fonts.insert(name.to_string(), next_id);
    self.font_instances.push(font);
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
    align: HorizontalAlign,
    font_name: Option<&str>,
  ) {
    self.pending.push((
        text.to_string(),
        pos,
        size,
        color,
        self.current_scissor,
        align,
        font_name.map(|s| s.to_string()),
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
      
      let items: Vec<_> = self.pending.drain(..).collect();
      
      let sections: Vec<Section> = items.iter()
          .map(|(text, pos, size, color, _scissor, align, font_name)| {
              let font_id = font_name.as_deref()
                  .and_then(|name| self.fonts.get(name))
                  .copied()
                  .unwrap_or(FontId(0));

              Section::default()
                  .add_text(
                      Text::new(text.as_str())
                        .with_scale(*size)
                        .with_color([color.x, color.y, color.z, color.w])
                        .with_font_id(font_id),
                  )
                  .with_screen_position((pos.x, pos.y))
                  .with_layout(
                      wgpu_text::glyph_brush::Layout::default()
                          .h_align(*align)
                  )
          }).collect();
      
      self.brush.queue(device, queue, sections).unwrap();
      
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
  pub fn measure(&self, text: &str, size: f32, font_name: Option<&str>) -> Vec2 {
      if text.is_empty() {
          return Vec2::new(0.0, size);
      }
      
      let font_id = font_name
          .and_then(|name| self.fonts.get(name))
          .copied()
          .unwrap_or(FontId(0));

      let font = &self.font_instances[font_id.0];
      let scaled_font = font.as_scaled(size);
      let mut width = 0.0;
      
      for c in text.chars() {
          let glyph_id = scaled_font.glyph_id(c);
          let advance = scaled_font.h_advance(glyph_id);
          width += advance;
      }
      
      Vec2::new(width, size)
  }
}
